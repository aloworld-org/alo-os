# Native window control strip painting

Date: 2026-09-08. Workstream: native desktop interaction.
Responsible contributor: single desktop worker and progress integration owner in
`C:\dev\alo-os`. Status: ready for integration after the checks below.

## Change, decisions and acceptance

The native shell now has a complete immutable view component for minimise,
maximise/restore and close. Three 32x32 buttons separated by transparent four-pixel
gaps share the exact rectangles used for clipped painting and hit testing.
Malformed viewport/origin values refuse; non-finite hits miss. Disabled buttons
retain their hit areas, preventing a future router from treating unavailability
as permission to click through. They have a separate non-color mark as well as
a changed ground. Maximize and restore use distinct original glyphs.

`crates/alo-shell/src/window_controls.rs` owns geometry and action metadata;
`window_control_paint.rs` owns clipped solid drawing and glyphs. The shell adds
an existing workspace dependency on `alo-appearance`, reflected in Cargo.lock,
to reuse light/dark tokens. ADR 0010's terracotta is never used. `Action::said`
retains the existing externalized label path, including the combined maximize/
restore wording; no user-visible English or second vocabulary is introduced.
The host must still present native text/accessibility labels. Original simple
glyphs, fixed scale-one geometry and supplied availability are explicit view
choices, not a new decoration policy or a claim that operations are available.

The additive public contract is `docs/contracts/native-window-controls.md`.
The view has no server, client/mapping identity, callbacks or input authority.
It cannot execute an action. The host owns placement, a matching active normal-
transform frame, publication and error recovery. Painter errors propagate and a
failed partial frame must not be submitted. No production scene/input path changes.
Ordinary keyboard input and all existing component tests remain intact.

Read constitution, delivery/ownership/report instructions, current queue and
STATE tail, v0.01 feature/roadmap sections, ADRs 0002/0010 and the native command
contract. Recorded this bounded view component and acceptance in QUEUE before
implementation. This completes layout and painting, not usable native controls.

## Executed verification

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). Prerequisites checked with:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
```

Versions respectively: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1,
26.0.8-1ubuntu0.3, 0.9.2. WSLg socket present; no installation required.
An upstream-source search found rg absent inside Ubuntu; used grep for that
read-only search. Repository searches used Windows rg.

Before every build/test/lint/format command, PowerShell read `(Get-PSDrive C).Free`
and refused below `12GB`. Recorded preflights ranged from 15,109,337,088 bytes
(14.0717 GiB) to 13,969,088,512 bytes (13.0097 GiB). No automatic cleanup or build
handoff; only the desktop workstream ran. The reserve is not a running-build quota.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo build -p alo-shell --examples --locked
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
```

The first focused command failed compilation: redundant `.into()` made Smithay's
generic `Rectangle::contains` argument ambiguous in production and test callers.
Removed those conversions; all six focused tests passed. The first affected
clippy command rejected constant `chunks_exact(4)` in the new graphics fixture.
Changed it to `as_chunks::<4>()`, explicitly asserted an empty remainder and
retained exact pixel comparisons. The second clippy command passed. No test
assertion, lint or gate was weakened; there was no repeated failing test run.

Final full shell suite passed 150 unit, 171 real-client lifecycle and three socket
tests plus three compile-fail doctests, none failed or ignored. This includes
existing ordinary keyboard, focus isolation and operation refusal coverage;
there is no new control-to-client input integration claim. New tests cover every
paint/hit pixel and gaps, fractional and half-open boundaries, disabled ownership,
malformed values, all-edge/extreme clipping, exact distinct glyph counts and
disabled non-color marks in both schemes. Rustdoc with warnings denied and the
example build passed. Windows/Linux formatting passed.

After the example build, with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

Exited zero. The new fixture checks 128 complete 120x48 frames (5,760 pixels each,
737,280 total): both schemes, maximize/restore, all eight enabled combinations
and four origin/clipping cases. Expected glyphs are independent literal masks in
`examples/support/window_controls_pixels.rs`, with fixed palette bytes; expected
pixels do not call the production painter or its geometry. Every readback pixel
also verifies hit ownership, including gaps and disabled controls. Expected Mesa
fallback diagnostics appeared; assertions were not skipped. This is offscreen
GLES on a real WSLg Wayland connection, not nested/direct display submission or
window-control interaction. Existing nested/offscreen scene examples were rebuilt
but not rerun; their historical evidence is not credited to this iteration.

Windows affected commands from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Passed; Windows executes zero Linux-only shell tests. Format was run twice during
development; final Windows/Linux format checks are clean. Source, tests, fixture,
lockfile and documentation diff inspected; final whitespace check passed.

## Integration and remaining work

Initial tree clean; every published report filename was already referenced in
STATE.md. Nothing awaited reconciliation; publication arrivals reconcile next
iteration. All four progress documents record this component and keep window
management unchecked. No report from another contributor was edited. No stage,
commit, push, identity/credential access, tools/dev-loop change, other checkout
edit, worker/loop launch, unrelated host change or cleanup occurred.

Next executable component: live mapped-window snapshots and truthful availability
for this view, then native label presentation and pointer press/release dispatch,
cancel/leave and stale-target isolation, ordinary typing and graphical interaction
acceptance. Presentation state is not authority: operations must validate the
current mapping and conditions at release. The host still needs hover/pressed
feedback and nested/direct scene/input integration. Launcher/dock, raw configurable
keyboard/settings wiring, clipboard and later delivery requirements remain open.

Supervisor full independent Windows/Linux workspace test/lint/rustdoc and pinned
BPF publication gates have not been run by this worker. Physical keyboard/libinput,
direct DRM/session recovery, integrated VM boot/update recovery and certified
laptop/GPU workstation records remain owed in their delivery phases. No WSLg
result certifies hardware, usable controls, complete window management or release.
