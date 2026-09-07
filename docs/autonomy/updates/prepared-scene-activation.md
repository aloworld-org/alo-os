# Prepared scene activation

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker, integration owner.
Status: ready for integration; supervisor publication gates pending.

## Change and decisions

`crates/alo-shell/src/scene_scanout.rs` adds consuming
`PreparedScanout::activate` and session-borrowed `ActiveScene`. Full-mode extent
and schema validation precede allocation; shared production allocation/upload/
TEST_ONLY/blocking-enable code retains the exact drawn identities with immutable
active resources. Explicit disable preserves cleanup errors and quarantine;
no callback, membership update or client dispatch is performed by this API.
The caller exclusively owns an inactive output and keeps the session active.
`ScanoutPixels::size` is additive. COMPOSITOR and public rustdoc describe the API.

ADR 0002's native pinned-engine path remains: reuse the existing safe blocking
transport for initial activation, which needs no asynchronous event cookie.
ADR 0001 and adapter contracts are unchanged; this exposes no agent capture or
verb. No engine patch, unsafe exemption, new scope or owner decision needed.
Queue acceptance was recorded before implementation. All published reports were
already referenced in STATE at iteration start; no reports needed reconciliation.
Claude's security workstream and other checkout were untouched.

## Verification executed

Windows PowerShell, repository root:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Final checks pass; shell Linux implementation/tests are cfg-excluded on Windows.
Ubuntu WSL2, Rust 1.98.0, repository `/mnt/c/dev/alo-os`, environment
`PATH=/root/.cargo/bin:/usr/bin:/bin`, `CARGO_TARGET_DIR=/root/alo-os-target`:

```text
cargo test -p alo-shell --lib prepared_scene --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

All final commands exit 0: 87 unit + 65 client lifecycle + 3 socket + 2 doctests
= 157 checks, no failures/ignored. Four new tests cover every pixel of a padded
1280x720 upload, zero padding/tail, ordered retirement, no-I/O size refusal,
allocation/upload/TEST_ONLY/enable failures with all cleanup errors, and failed
disable quarantine without drop retry. Initial compile found allocator visibility;
initial clippy found test slicing/constant chunks. Corrected both without lowering
the gate. Final command group uses bash `set -e`; log `.git/scene-activation-linux.log`.

WSLg prerequisites verified: socket `/run/user/0/wayland-0`; pkg-config
wayland-server 1.24.0, EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3,
libseat 0.9.2, libinput 1.31.1. No installation needed; `/dev/dri` absent.
With `XDG_RUNTIME_DIR=/run/user/0 WAYLAND_DISPLAY=wayland-0`:

```text
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-scene-activation-egl.json timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

First two exit 0. Real SHM window/child/popup/cursor scene verifies 1,056 pixels
and existing lifecycle checks. New public activation against `/dev/null` preserves
ENOTTY (25), returns no owner and leaves callbacks pending; mismatched mode also
refuses. Nested regression submits 134 client surfaces. Invalid EGL exits 1 with
invalid display and no success. Logs `.git/scene-activation-{gles,nested,refusal}.log`.
Known Mesa startup diagnostics and deliberate truncated-SHM protocol error remain
expected fixture evidence, not graphics acceleration measurements.

## Limits and remaining work

Successful DRM enable/disable is fault-injected, not kernel or physical evidence.
This completes an initial static scene transaction, not the native compositor.
Next: synchronous active-scene replacement, retaining old storage on commit
refusal and retiring it only after success; then continuous direct FrameTarget,
default cursor and pause/input/session integration. Safe cookie-bearing atomic
transport, asynchronous retirement, actual GPU context-loss/draw/readback faults,
parent-leave/libseat limits and pinned unmap-panic limitation remain.
Successful scanout requires a DRM-equipped development login/VM. Physical business
laptop and >=24 GB GPU workstation display/input, session switching, suspend/resume
and hardware checklist records remain owed. No WSL hardware certification.

All four shared progress documents updated with these exact limits. Source/new
files and documentation diff reviewed; `git diff --check` passes. No staging,
commit/push, worker launch, dev-loop edits, shared kernel/BPF/cgroup/service changes
or physical install. Supervisor full Windows/Linux workspace/BPF test/lint/rustdoc
gates have not run for this change. Reports arriving during publication reconcile
next iteration; full v0.01 remains the target and release stays unchecked.
