# Unbound scanout frame upload

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop worker and progress integration owner.
Status: ready for integration; CPU upload component complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/scanout_frame.rs` adds validated borrowed `XrgbFrame` and
bounded full-frame copying. `DisplayResources::with_frame` uploads before activation
and returns the candidate only after mapping/copy/unmap success. Any upload refusal
consumes the candidate and attempts all cleanup, retaining original and cleanup
errors. ActiveScanout has no writable-buffer API. No scanout ioctl is issued by upload.

The source is top-to-bottom DRM XRGB8888 (B,G,R,X bytes), with explicit aligned byte
stride and exactly stride * height bytes. Checked arithmetic, nonzero dimensions,
exact destination size, format, pitch and mapping length prevent truncation, scaling,
partial-frame interpretation or out-of-bounds writes. Source row padding is ignored;
destination padding and the allocation tail are cleared. No colour conversion,
alpha blending or renderer readback is implied. These are diagnostic library errors,
not new hardcoded person-facing UI strings.

Selected in QUEUE before implementation as an independently executable prerequisite
inside delivery step 2. Pinned drm-ffi 0.9.1 and the upstream development
[`atomic_commit` helper](https://github.com/Smithay/drm-rs/blob/develop/drm-ffi/src/mode.rs)
were inspected: neither accepts a cookie and both leave user_data at its zero default.
The current upstream source was fetched read-only with PowerShell Invoke-WebRequest
-UseBasicParsing; it was not installed or patched. That specific helper cannot supply
the existing FlipGate's identities. No claim that every alternative crate was audited.
Unsafe code remains forbidden; no upstream patch, dependency change or ADR exception.
ADRs 0001/0002 and agent/application-adapter contracts remain unchanged. The new
internal shell library API is documented in rustdoc and COMPOSITOR.md.

## Executed verification

Windows PowerShell, repository root, all final commands exit 0:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Windows shell tests are cfg-excluded, not DRM evidence. Ubuntu WSL2, repository
`/mnt/c/dev/alo-os`, PATH=/root/.cargo/bin:/usr/bin:/bin and
CARGO_TARGET_DIR=/root/alo-os-target, final commands exit 0:

```text
cargo test -p alo-shell --lib scanout_frame --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Six new tests pass. Full Linux shell: 76 unit + 64 client lifecycle + 3 socket +
2 lifetime doctests = 145 passing, zero failed/ignored. Happy/refusal cases cover
independent strides, exact pixel bytes, padding/tail clearing, dimensions, format,
unaligned/short stride, arithmetic overflow, short/excess source length, short mapping
without partial writes, mapping errno and multiple cleanup errors without retries.
Additional component integration uses the production allocation/upload/scanout owners
with the existing fault-injection transport: a 1280x720 patterned frame survives
TEST_ONLY/enable unchanged, then disable precedes exactly-once reverse-order cleanup.
This is memory/transport integration, not a successful kernel DRM upload or flip.

WSLg: 128 client-surface GLES submissions, popup/reposition/reactive placement and
cursor callbacks, unmap/remap, refusal and disconnect pass. Known Mesa ZINK diagnostic
precedes successful GLES rendering. This regression does not use the new direct
upload path and includes no pixel readback or physical input measurement.
Local logs: `.git/scanout-upload-{tests,docs,examples,wslg}.log`; fail-fast Linux
command script: `.git/scanout-upload-checks.sh`.

First Linux clippy found two constant chunks_exact calls in tests; corrected to
as_chunks without exemptions, then clippy and full tests passed. An initial WSL
inspection quoting error, PowerShell web-fetch mode error and CRLF in the local
WSLg command script were corrected. No repeated test failure or lowered gate.
Checked Rust 1.98.0, WSLg socket and pkg-config: wayland-server 1.24.0, EGL 1.5,
GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2, libinput 1.31.1. No packages needed.
No /dev/dri exists here.

## Integration and remaining work

All published task reports were already referenced in STATE at iteration start;
none needed reconciliation. Claude's security workstream was not taken. Reports
arriving during publication reconcile next iteration. Updated CHANGELOG, ROADMAP,
QUEUE and STATE in this change, preserving all release gates and evidence limits.

Next independently executable component: renderer output conversion/readback into
XrgbFrame, with patterned pixel/orientation verification, before first-frame direct
presentation. Cookie-bearing nonblocking transport, pending-buffer retirement,
session pause ordering, direct input and production entry remain unfinished.
The pinned mapping destructor's munmap panic boundary and parent-leave/libseat
limits remain. Successful DRM upload/scanout needs a DRM-equipped development
login/VM. Physical business laptop and >=24 GB GPU workstation display/input,
session switching, suspend/resume and all hardware checklist records remain owed.
No compositor or release checkbox is ticked.

Source/tests and tracked/new-file diff reviewed; no staging, commit, push, dev-loop
changes, worker launch, other checkout access, shared kernel/BPF/cgroup/service
changes or physical installation. Full independent supervisor Windows/Linux/BPF
publication gates have not run for this change and remain pending.
