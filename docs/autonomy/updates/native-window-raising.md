# Native window raising

Date: 2026-09-08. Workstream: native desktop window management.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; stacking component complete, feature unfinished.

## Change and decisions

`crates/alo-shell/src/window_raise.rs` exposes trusted `Server::raise_window`.
`surfaces.rs` validates a live mapped root in this display before rotating it to
the front, preserving other roots' relative order. Existing renderer and pointer
scene traversal carry popup subtrees with their parents. Repeated raising is
idempotent. Foreign, dead, unmapped and popup targets refuse without reordering;
a fresh configured remap is eligible. New roles retain their existing initial
order. No buffer, client role or process is destroyed.

`pointer.rs` exposes the last validated location/time only while focus exists.
Raising refreshes that focus through ordinary routing, preserving implicit held
buttons and explicit popup grabs. It cannot synthesize entry after pointer leave.
WindowRaiseError distinguishes invalid targets from a pointer refresh error after
the order changed. That latter defensive propagation is not failure-injected:
the stored coordinates and existing pointer satisfy current routing validation.

ADR 0002 and existing v0.01 window management justify this native Rust component.
COMPOSITOR.md and public rustdoc state the boundary. There is no agent endpoint,
context capture, engine patch, lint exception, new UI string or scope expansion.
Keyboard activation/XDG activated state remain separate policy; the nested
backend retains its existing front-root keyboard-event selection. This avoids
claiming complete activation while its lifecycle/grab policy remains unbuilt.
Native controls/shortcuts and application adapter approval/grants remain owed.

Delivery step 2's direct standalone GLES constructor restriction remains open
under the pinned engine and workspace unsafe prohibition; no exception was made.
The queue expressly permits independent delivery-step-3 work. Selected this
complete stacking component and acceptance criteria in QUEUE before coding.

## Verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18). Checked prerequisites with
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat`:
1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
`test -S /mnt/wslg/runtime-dir/wayland-0` passed; `/dev/dri` absent.
No dependency install, kernel/BPF pin, service or other checkout changes.

Windows commands from this checkout passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

The shell is Linux-only; Windows runs zero runtime tests. Linux commands from
`/mnt/c/dev/alo-os`, with `PATH=/root/.cargo/bin:/usr/bin:/bin` and isolated
`CARGO_TARGET_DIR=/root/alo-os-target`, passed:

```sh
cargo test -p alo-shell --locked window_raise
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Four new tests in `tests/window_raise/mod.rs` use actual Wayland sockets/SHM
clients: order passed to Server::render, stationary-pointer enter/leave, stable
other-root order and idempotence, preserved keyboard recipient, held-button
press/release isolation, popup-grab keyboard/pointer isolation, no entry after
pointer leave, foreign/popup/unmapped/dead refusal and configured remapping.
The render-order sink is controlled, not a physical output. Final Linux totals:
138 unit, 87 client-lifecycle, three socket and three compile-fail doctests pass;
no ignored cases. Formatting, all-target clippy and rustdoc are warnings-clean.

After rebuilding examples, with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`, both passed:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

New `examples/support/window_raise_check.rs` uses real GLES readback: an opaque
second client occludes the first root and its popup, then raising the first
restores both root and popup pixels at independent sample positions. The expanded
offscreen fixture completed eight stages, including existing golden pixels and
truncated-SHM refusal. Nested regression submitted 115 client surfaces. Expected
Mesa and deliberately invalid-client diagnostics appeared; both exited zero.

Development corrections: the first order test hit the default FrameTarget cursor
refusal; the test sink now explicitly models cursor support as existing fixtures
do. Subsequent runtime tests passed. Clippy exposed unchecked slicing/indexing in
production, example and test files on successive checks, then panic/unused/borrow
lint findings in test cleanup. Replaced those with checked access and fallible
test returns; no suppression or gate change. An earlier graphics run preceded
the successful example rebuild and was only old-binary regression evidence;
the final run above executed and printed the new raising pixel check. Initial
document paths/globs and narrow registry grep attempts were unsuccessful; corrected
document reads, with no claim based on the empty registry search.

Source/test/documentation diff reviewed; `git diff --check` passed. Full independent
Windows/Linux workspace/rustdoc/BPF publication gates remain supervisor work and
are not claimed here.

## Reconciliation and remaining work

Read report instructions and compared every published report filename against
STATE at iteration start: none were unreferenced. No contributor evidence needed
consolidation. Claude retains the security/model-choice workstream and ADR 0021
remains proposed. Reports arriving during publication are reconciled next iteration.

All four shared progress documents updated without checking a release feature.
Next: explicit keyboard activation with XDG state and lifecycle/grab policy, then
native switching/close controls and configurable shortcuts. Move/resize/minimise/
maximise/tile, launcher/dock, clipboard and later delivery steps remain unfinished.
Safe standalone GLES, real DRM/seat entry, populated input/hotplug and GPU/disable
recovery remain owed. Certified laptop and GPU workstation physical acceptance
requires real hardware; WSLg pixels are development evidence only.

No staging, commit, push, worker/loop launch, tools/dev-loop modification, other
repository edit or physical installation. Supervisor owns integration/publication.
