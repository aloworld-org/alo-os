# Bounded direct-pointer routing

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; pointer translation component complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/direct_pointer.rs` adds the trusted `DirectPointerEvent`
and `Server::direct_pointer` API. Relative mouse deltas accumulate from the last
accepted seat position. Normalized absolute coordinates map to scale-one output
pixels. Both clamp/map to the last pixel origin; fractional interior movement is
preserved. This keeps the pointer inside the one-display extent without changing
existing hit testing, popup grabs, cursor placement or protocol coordinates.
One-pixel outputs are valid. Dimensions beyond the existing signed Wayland fixed
coordinate boundary, nonfinite deltas, invalid normalized coordinates, buttons
and scroll are refused without changing position or held-button state.

Inactive input ignores any queued event and cancels the existing pointer focus
and drag. Reactivation alone cannot deliver buttons or scroll: fresh motion must
establish a recipient. Existing pointer cancellation supplies matched synthetic
releases. The server owns the position, avoiding a second cursor state that can
drift from the rendered cursor. This internal API is never an agent verb; the
trusted caller must supply seat activity and the current output dimensions.
It does not acquire devices or independently observe seat changes.

ADR 0002's native Rust compositor and the v0.01 one-display keyboard/pointer
feature authorize this component. No engine patch, new release scope, agent or
daemon contract change, or user-facing strings. Public Rust API has rustdoc.
`tests/direct_pointer/mod.rs` adds real socket-client acceptance and refusal
coverage; two unit tests cover numeric edges and malformed coordinates.

## Renderer dependency and task selection

The preceding task named standalone native GLES initialization as next. Inspected
pinned Smithay 0.7.0 registry sources: `backend/egl/display.rs:201` exposes
`EGLDisplay::new` as unsafe, and `backend/renderer/gles/mod.rs:468` exposes
`GlesRenderer::new` as unsafe (fresh exclusive context required). Workspace
`Cargo.toml:71` forbids unsafe code; its dependency policy explicitly uses safe
upstream APIs instead. No exception, excluded helper crate or engine patch was
introduced. A safe upstream construction path or an owner-approved design change
is still needed for standalone initialization. A nested Winit window is not a
native direct-display solution. Recorded this limit and selected independent
pointer translation in QUEUE before implementation, as DELIVERY step 2 permits.

## Verification actually run

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18). `pkg-config --modversion
wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat` reports 1.24.0,
1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2. WSLg socket exists;
`/dev/dri` is absent. No dependencies installed or shared kernel state changed.
An initial combined shell discovery command had quoting errors; direct commands
above provided the prerequisite evidence. WSL lacks rg; grep inspected the exact
known upstream source files. Repository searches used rg.

Windows commands, all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows runs zero tests for this Linux-only crate. Linux commands from
`/mnt/c/dev/alo-os`, with `PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`, all passed:

```sh
cargo test -p alo-shell --locked direct_pointer
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
test -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY"
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Five focused tests pass. Full shell: 117 unit tests, 70 client-lifecycle tests,
three socket-ownership tests and three compile-fail doctests. No test failure or
lint exemption. Additional integration evidence is real Wayland wire delivery:
fractional relative motion, unchanged wire frame count and position after invalid
input, exact matched release after refusal, drag motion outside the client clamped
to (31,31), pause leave/releases, no click/scroll on reactivation, absolute reentry
at (7.75,7.75), scroll delivery, and no inherited click after disconnect/replacement.
Missing pointer capability refuses both active and inactive calls.

WSLg offscreen golden pixels/refusal checks and nested regression pass, with 115
client surfaces submitted. Existing Mesa diagnostics and deliberate invalid-client
protocol errors are expected. These regressions verify existing graphics behavior;
the new direct API's events are exercised by socket tests, not physical libinput.
Local check script: `.git/direct-pointer-checks.sh`. Final source/diff inspection
and `git diff --check` pass. An intermediate PowerShell text read changed queue
encoding; restored original UTF-8 bytes and retained only the intended addition.

## Remaining work and integration

Next independently executable component: seat-owned libinput acquisition/event
routing and keyboard pause cleanup, with device removal and open/dispatch refusal
tests. Standalone GLES safe construction, direct renderer/input/session wiring,
real DRM and seat behavior, GPU context loss, failed-disable recovery and certified
laptop/GPU-workstation physical records remain owed. The translation layer must
be wired to live input before direct input is complete. WSL is not certification.

At iteration start every published report filename was already referenced in
STATE; none needed reconciliation. Claude retains the network request-boundary
workstream. All four shared progress documents include this change and limits.
Reports arriving during publication are reconciled next iteration. No staging,
commit, push, worker launch, other checkout change or tools/dev-loop modification.
Full independent Windows/Linux workspace, rustdoc and BPF publication gates are
still the supervisor's responsibility and were not run by this worker.
