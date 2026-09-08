# Seat-checked libinput context lifetime

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; context lifetime complete, direct input unfinished.

## Change and decisions

`crates/alo-shell/src/seat_input.rs` adds `SeatInput`, an owned udev/libinput
context using the existing restricted SessionInput bridge. It obtains the seat
name from the manager and rejects inactivity, empty names and embedded NUL before
calling upstream assignment. Callback failures during assignment retire the
context even though libinput can report successful assignment after failed opens.
Smithay's existing `backend_udev` feature is enabled; no package versions or
lockfile changed, no engine patch or unsafe/lint exception was added.

Dispatch polls trusted seat authority before reading and before each delivery,
including a post-dispatch poll on an empty batch. The callback contract requires
pause to remain latched across same-batch reactivation. The failure latch is
checked before reading and delivering. Pause, dispatch errors and handler errors
suspend and destroy the context before emitting one Reset notification; subsequent
dispatch refuses without polling or delivering. Explicit shutdown resets once;
Drop suspends as an unwind backstop but cannot reset a Server it does not own.
The caller must map Reset to Server::clear_input and flush queued protocol events.

Terminal retirement deliberately matches the existing active-device scope: a
fresh lifetime is required after pause. Explicit suspension closes devices even
if an upstream event reference survives. The original failure and callback
cleanup evidence remain separately inspectable; a reset-handler failure is also
retained. The restricted callback latch still preserves only its first error,
so a later close error cannot replace an earlier open error. This does not claim
an exhaustive list of device errors. Udev allocation behavior remains that of
the pinned unmodified upstream wrapper.

This complete context-lifetime component was selected in QUEUE before coding.
ADR 0002 and the v0.01 keyboard/pointer requirement authorize it. There is no
agent API, background context capture, daemon-contract change or user-facing UI
string. Event translation, per-device removal cleanup and connection to the live
DirectSession notifier remain the next components, not completed by this API.

## Verification

Ubuntu WSL2 prerequisites: Rust 1.98.0 (88d9e12ae 2026-08-18);
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm
libseat`: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg `/mnt/wslg/runtime-dir/wayland-0` present; `/dev/dri` absent. No package
installation or shared kernel/service/BPF change was needed.

Windows commands passed (the shell is Linux-only, so tests execute zero cases):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux commands run from `/mnt/c/dev/alo-os` with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and checkout-specific
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked seat_input
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Eight new tests pass. Deterministic lifecycle tests verify delivery ordering,
queued-event discard after pause, no resurrection, dispatch/handler refusal,
reset ordering and reset-error preservation. The restricted callback test uses
a real Unix descriptor and peer EOF, retaining open/close failure evidence after
context destruction. Two tests use real libinput udev contexts on unique empty
seats: successful dispatch/shutdown and post-dispatch pause/reset. Neither grants
physical input devices; populated-seat acquisition and kernel device hotplug
remain unverified. Initial compilation required enabling backend_udev; initial
clippy found missing private rustdoc and a collapsible conditional. These were
fixed without weakening checks. No runtime test failed.

All listed Linux commands passed: 132 unit, 74 client-lifecycle and three socket
tests, plus three compile-fail doctests. Warnings-denied rustdoc and examples
passed. Additional integration commands passed with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
test -S /mnt/wslg/runtime-dir/wayland-0
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

WSLg golden pixels, SHM refusal and nested popup/cursor regression passed with
115 client surfaces. Expected Mesa and deliberately invalid-client diagnostics
remain. Source/test/documentation diff review and `git diff --check` passed.
The four shared progress documents record these results and remaining work.

## Remaining work and proposed progress updates

Record this component in CHANGELOG, ROADMAP and QUEUE while leaving compositor
and release unchecked. Next: translate libinput keyboard/pointer events and
handle device removal, then connect SeatInput to DirectSession seat polling and
the direct frame loop. The poll/Reset callback contracts are not evidence of
that wiring. Safe standalone GLES construction, real DRM/seat input acquisition,
GPU context-loss/failed-disable recovery, and physical laptop/GPU-workstation
acceptance remain owed. WSL/empty-seat checks certify no hardware.

The supervisor still owns independent full Windows/Linux workspace, rustdoc and
BPF publication gates. No staging, commit, push, other checkout edits, worker/loop
launch, tools/dev-loop modification or physical installation. At iteration start
all published report filenames were already referenced in STATE; no published
report needed reconciliation. Reports arriving during publication belong to the
next iteration.
