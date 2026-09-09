# Mapping-bound window control transactions

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

The native control transaction component binds a primary press to one explicit
visible mapping and consumes its matching release exactly once. Disabled hits
retain ownership without gaining authority if availability later changes.
Duplicate presses cannot replace or rearm an owner. Explicit cancellation keeps
the release consumed. Release checks original visibility identity, current paint
geometry, hit action and captured maximize/restore intent, then calls the existing
live operation. Refusals consume the release and preserve typed operation errors.
No focused-window fallback, implicit activation, client pointer injection or
agent endpoint was introduced. Close remains cooperative.

`crates/alo-shell/src/window_control_input.rs` owns the transaction. Server owns
one pending primary press. `surfaces.rs` owns per-window visibility identity and
changes it inside unmap and hide/reveal transitions, even between dispatch
boundaries. A retained `Arc<()>` cannot wrap or reuse the allocation while a press
holds it. Ordinary commits preserve identity. Existing client held buttons,
popup grabs and interactive move/resize refuse a new native hit without stealing
their ownership. The prior snapshot API remains presentation data only.

This follows ADR 0002's native Rust shell and the existing v0.01 window-management
scope. ADR 0010's palette and the existing externalized action vocabulary are
unchanged. No dependencies, upstream patches, stored formats or public protocol
changes. The additive Rust API is documented in public rustdoc and
`docs/contracts/native-window-controls.md`. No new ADR or owner decision needed.

## Acceptance and verification

Seven real-client tests in `tests/window_controls/input.rs` cover exactly-once
close, disabled-to-enabled changes, minimize/maximize/restore, live output and
popup-busy refusal, duplicate presses, cancellation, mismatched/invalid geometry
and hits, hide/reveal in one backend call, unmap/remap, disconnect/replacement,
foreign roots, existing client drags and ordinary keyboard/pointer delivery.
These fixtures use private displays and do not mutate shared kernel state.

The first focused run passed the initial six new transaction tests, the six
existing snapshot tests and six view tests. The first full shell run passed all
150 units and 183/184 lifecycle tests. The added popup test incorrectly expected
delivery of the initiating key's later release; existing popup-grab semantics
consume it (also asserted by `tests/popups/keyboard_grabs.rs`). Corrected that
assertion to require consumption and added a subsequent ordinary key pair. No
production behavior, existing test or lint was weakened.

Prerequisites were verified using Ubuntu `rustc --version`,
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm
libseat`, and `test -S /mnt/wslg/runtime-dir/wayland-0`. Rust 1.98.0; package versions
1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2; WSLg available.
No package installation or shared-system maintenance was needed.

Every build/test/lint/format command had a PowerShell `(Get-PSDrive C).Free`
preflight refusing below `12GB`. Lowest measured preflight was 73,956,642,816
bytes (68.88 GiB). This is operational headroom, not a running disk quota. No
cleanup, shared-kernel mutation or outer suite lock was used.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo test -p alo-shell --locked window_controls_input_popup_started_after_press
cargo test -p alo-shell --locked --quiet
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

The non-quiet full test invocation is the single failed attempt described above.
After correction, the focused popup test passed, then the full suite passed:
150 units, 184 client-lifecycle tests, three socket tests and three compile-fail
doctests; none ignored. Final all-target clippy, warnings-denied rustdoc, examples
and formatting passed. No repeated failure, retries without changes or weakened
gate. Expected malformed-client/keymap diagnostics are refusal fixtures.

Windows commands from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed. Formatting was repeated after Rust edits. Windows checks execute
zero Linux-only shell tests; the popup assertion correction was Linux-only and
was verified by final Linux clippy/tests. `git diff --check` passed; tracked diff
and new source/test/report files were reviewed.

WSLg commands with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

Both passed. All 28 offscreen stages, existing 12 snapshot-derived 5,760-pixel
frames and prior client scene assertions remain intact. The minimize fixture
adds a cancelled native press, verifies it leaves the mapping visible, then uses
a fresh press/release to minimize, checks that a duplicate release is unowned,
and compares every pixel of the hidden scene. Trusted restoration checks every
pixel of the preserved scene: **two additional complete 6,400-pixel frames**.
Cancellation's no-effect check is mapping-count evidence, not a separate rendered
frame. The unchanged control painter passed all 128 complete 5,760-pixel frames.
Mesa fallback and the deliberately malformed SHM diagnostics skipped no checks.
No on-screen pointer or direct DRM claim follows; the existing nested on-screen
regression was rebuilt but not rerun.

## Integration and limits

The iteration began clean. All published task reports were already referenced in
STATE.md, so no new contributor evidence required consolidation. Reports arriving
during publication reconcile next iteration. Claude's workstream was not taken.

This completes the transaction boundary only. The production nested/direct host
still needs primary-event interception, current painted geometry, consumption of
owned events and cancellation hooks on pointer leave, input reset, removed UI and
seat/session loss. The standalone component does not automatically intercept
backend events. Native labels, hover/pressed feedback and composed production
controls remain unfinished. Next executable component: connect the transaction
to a native strip pointer router with motion/leave/reset lifecycle and client
routing isolation, then integrate rendered labels/feedback and production scenes.
No feature or release checkbox is promoted.

WSLg checks are offscreen development evidence, not on-screen interactions,
direct scanout or hardware certification. Populated physical input, direct
DRM/session recovery, integrated VM boot/update recovery and certified laptop/GPU
workstation records remain owed at their scheduled phases. The supervisor owns
full independent Windows/Linux workspace test/lint/rustdoc and pinned BPF gates,
staging, commits and publication. No staging, commit, push, tools/dev-loop edit,
other checkout change, cleanup, shared service/session/mount change, credentials,
git identity change or additional worker/loop launch was performed.
