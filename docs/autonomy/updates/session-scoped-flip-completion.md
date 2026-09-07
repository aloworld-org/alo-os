# Session-scoped page-flip completion gate

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker / integration owner.
Status: ready for integration; completion bookkeeping component complete,
nonblocking scanout and the compositor feature remain unfinished.

## Change and decisions

`crates/alo-shell/src/flip_gate.rs` adds exported `FlipGate`, with focused tests
in `flip_gate_tests.rs`. It reserves a process-unique nonzero cookie before one
submission callback, burns refused cookies, preserves transport errno and refuses
another submission while pending. An exact cookie/CRTC match completes once.
Foreign, old-session, duplicate and non-flip events cannot complete a pending flip.
Sequence/timestamps are not identities. Atomic allocation is safe across parallel
gates; exhaustion is terminal instead of wrapping into reused identities.

The integrated bounded descriptor reader decodes the whole batch before matching.
Malformed batches, WouldBlock, EOF and blocking-descriptor refusal leave pending
state intact. Unrelated events are consumed: one gate/CRTC and one exclusive reader
per session is the supported scope. The callback must report actual kernel
acceptance truthfully and use the supplied cookie. This library is not an agent
surface or an authentication boundary against its own caller.

ADR 0002's native Rust boundary and ADR 0001/application-adapter contract remain
unchanged. No upstream patch, unsafe exemption, dependency change or new release
scope. Inspection found pinned drm-ffi 0.9.1 `mode::atomic_commit` initializes
user_data to zero; its raw ioctl requires unsafe, which the workspace forbids.
Therefore the independent completion gate was selected in QUEUE before coding;
this does not claim that a cookie-bearing kernel transport has been implemented.
Next investigate an unpatched upstream safe cookie-bearing API. Any engine patch
requires an ADR; do not weaken the unsafe gate as a shortcut.

This component owns identity only. It is deliberately not connected to blocking
ActiveScanout yet. Dropping a gate never authorizes resource release. A future
scanout owner must retain both buffers until matching completion or synchronous
disable, keep the newly displayed buffer alive, and quarantine on failed disable.
Fresh gates are required after reacquisition; process-restarted/inherited event
streams cannot be reused. Successful completion cannot certify rendered pixels.

## Executed verification

Windows PowerShell, repository root, all exit 0:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows excludes Linux shell tests by cfg; this is not Windows DRM evidence.
Ubuntu WSL2, repository `/mnt/c/dev/alo-os`, PATH=/root/.cargo/bin:/usr/bin:/bin,
CARGO_TARGET_DIR=/root/alo-os-target, all exit 0:

```text
cargo test -p alo-shell --lib flip_gate --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Eight new happy/refusal tests passed on the first run. Full shell suite: 70 unit,
64 client lifecycle, 3 socket, 2 lifetime doctests = 139 passed, zero failed/ignored.
Tests cover acceptance versus completion, exactly once matching, pending submission
refusal without transport invocation, wrong CRTC, zero/stale cookie, submission errno,
failed-cookie burning, same-CRTC session recreation, terminal u64 exhaustion and
512 concurrent unique identities. Additional integration sends synthetic native DRM
ABI bytes over real nonblocking Linux UnixStream descriptors through the production
event reader and gate. Mixed/duplicate events complete once, malformed trailing
bytes prevent partial completion, and descriptor flags/lifetime survive.
This is synthetic descriptor integration, not a successful DRM ioctl or kernel flip.

WSLg regression: 135 client-surface GLES submissions; popup/cursor callbacks,
unmap/remap, protocol refusal and disconnect passed. Known Mesa ZINK diagnostic
precedes successful rendering; no pixel readback or physical input measurement.
Local logs under `.git/`: flip-gate-focused.log, flip-gate-clippy.log,
flip-gate-linux-tests.log, flip-gate-docs.log, flip-gate-examples.log,
flip-gate-wslg.log. Bash scripts used fail-fast execution and propagated exit status.

Prerequisites checked: Rust 1.98.0, WSLg socket present, wayland-server 1.24.0,
EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2, libinput 1.31.1.
No packages needed. No /dev/dri. Initial root-level queue/state paths and WSL source
inspection quoting were corrected; no test failure or gate weakening occurred.

## Integration and remaining work

All published task reports were already referenced in STATE at iteration start;
none required reconciliation. No Claude-owned security task was taken. Reports
arriving during publication will be reconciled next iteration. CHANGELOG, ROADMAP,
QUEUE, STATE and COMPOSITOR are updated with this component and its limits.

Next: safe cookie-bearing atomic transport and pending-buffer ownership wired to
this gate, commit/refusal/teardown tests, rendered frames, session pause ordering,
direct input and production entry. Existing parent-leave/libseat limits remain.
A DRM-equipped development login/VM is needed for successful atomic flip evidence.
Certified business laptop and >=24 GB GPU workstation display/input, session
switching, suspend/resume and all physical hardware checklist records remain owed.
No compositor or release checkbox is ticked.

Source/tests and tracked/new-file diff inspected; git diff --check passes.
No staging, commit, push, dev-loop edit, new worker, other checkout access, shared
kernel/BPF/cgroup/service change or physical installation. Supervisor full
Windows/Linux/BPF test/lint/rustdoc publication gates have not run for this change.
