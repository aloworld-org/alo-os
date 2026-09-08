# Session-owned input descriptors

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; acquisition bridge complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/session_input.rs` implements the pinned libinput restricted
open/close interface through Smithay `Session`. Construction opens nothing.
Inactive sessions refuse acquisition without calling the manager; every granted
descriptor returns through the manager, including after pause or earlier failure.
Flags retain libinput's access mode and add CLOEXEC, NOCTTY and NONBLOCK. There is
no direct filesystem-open fallback. `SessionInputStatus` retains the first open
or close stage and errno in its diagnostic across callback/context destruction.
An open/close failure prevents subsequent acquisition through that bridge.
Ordinary inactivity is recoverable and does not itself poison the bridge.

Smithay 0.7.0's existing `LibinputSessionInterface` ignores close errors and does
not test activity before acquisition (`backend/libinput/mod.rs`, lines 687-696).
The local safe Rust adapter preserves those errors without patching an engine
or changing lint policy. A failed open is conservatively terminal for this
context; callers must retire it and clear input rather than silently continue
with a partially acquired seat. The caller must check the shared latch before
routing input. Existing descriptors are still owned by libinput: this bridge
cannot suspend them itself. Seat polling, suspend/drop on pause and clearing
server input are explicitly caller obligations, not claimed implementation.

This is the acquisition bridge portion of the next delivery component, selected
in QUEUE before implementation. Enabled Smithay's existing `backend_libinput`
feature explicitly in alo-shell, without changing dependency versions or the
lockfile. ADR 0002 and the v0.01 keyboard/pointer requirement authorize the work.
No agent capability, context capture, daemon contract change, UI string, engine
patch or additional release scope. Public interfaces have rustdoc.

## Verification

Ubuntu WSL2 prerequisites checked: Rust 1.98.0 (88d9e12ae 2026-08-18),
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm
libseat`: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket `/mnt/wslg/runtime-dir/wayland-0` present; `/dev/dri` absent.
No dependencies installed and no shared kernel, service or BPF changes.

Windows commands passed (the Linux-only shell runs zero Windows tests):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux commands from `/mnt/c/dev/alo-os`, with `PATH=/root/.cargo/bin:/usr/bin:/bin`
and checkout-specific `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked session_input
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
```

Both passed. Six new tests exercise multiple descriptors, no open during pause,
fresh acquisition after activity returns, terminal open/close errors, first-error
retention, cleanup despite failure and no close retry. Manager operations use
real Unix socket descriptors; peer EOF proves closure, not just a counter.
Two tests instantiate real pinned libinput path contexts: rejection of a
non-evdev descriptor closes through the manager exactly once and produces no
event; manager permission refusal is observable after libinput returns no device.
The manager is injected and the descriptor is a socket, not a physical evdev
device. This demonstrates the libinput callback boundary and descriptor cleanup,
not successful real-device acquisition or seat ownership on a workstation.

Additional Linux commands passed:

```sh
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
test -S /mnt/wslg/runtime-dir/wayland-0
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Linux shell totals: 124 unit, 74 client-lifecycle, three socket tests and three
compile-fail doctests. No runtime test, lint or compilation failures. WSLg
regression passed golden pixels, SHM refusal and nested popup/cursor routing,
with 115 client surfaces; expected Mesa and invalid-client diagnostics remain.
Source/diff review and `git diff --check` passed. The four shared progress
documents record this component, its checks and remaining work. No published
report was unreferenced in STATE at iteration start; no reconciliation was owed.

## Remaining work

Next: libinput context ownership and event dispatch tied to seat notifications,
open/dispatch failure retirement, and device-removal cleanup through existing
keyboard/pointer boundaries. Connect that lifecycle to DirectSession; this bridge
alone is not live direct input. Standalone safe GLES initialization, real DRM/seat
integration, GPU context-loss/failed-disable recovery and certified laptop/GPU
workstation physical acceptance remain owed. Compositor and release stay unchecked.

Full independent Windows/Linux workspace, rustdoc and BPF publication gates are
pending with the supervisor. No staging, commit, push, other checkout edits,
worker/loop launch, tools/dev-loop modification or physical installation.
