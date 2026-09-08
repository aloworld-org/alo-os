# Session-integrated direct input

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; input-loop component complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/direct_input_loop.rs` adds
`DirectSession::run_compositor_with_input`. It clones the existing libseat manager
connection, never the display descriptor, and creates SeatInput inside the active
display scope after fresh atomic discovery and an authority poll. Dimensions come
from that lifetime's mode. Input uses the existing borrowed event translator and
the scope's latched poll before dispatch and every delivery. Idle iterations also
dispatch input. Pause followed by activation cannot revive this lifetime.

`direct_loop.rs` shares the frame loop with the existing display-only API. Stop,
seat failure, input/routing refusal, client failure and render failure all consume
input before output retirement. Context shutdown suspends/closes devices and
clears input; an explicit input flush is attempted before output disable. Output
retirement and a final output flush still run if input cleanup or its flush fails.
DirectLoopResult retains runtime, input cleanup, input flush, output retirement
and output flush outcomes; ActiveSessionResult retains display close separately.
Input acquisition errors retain their callback-cleanup evidence. Early display
acquisition refusal attempts an input-reset flush; if that also fails, the outer
SessionError diagnostic preserves both failures. No retry or automatic resume.
A flush does not guarantee a blocked client has consumed its events.

ADR 0002 and the existing v0.01 keyboard/pointer scope authorize this native Rust
component. Caller-owned GLES construction stays separate because the pinned
standalone constructors require unsafe forbidden by current workspace policy.
No engine patch, dependency change, unsafe or lint exemption, agent IPC, context
capture or user-facing string was introduced. The invocation-only contract is
unchanged; trusted session APIs are documented in rustdoc. The existing
run_compositor API remains display-only. A caller configures the Server's input
capabilities and supplies a current renderer. The new entry method does not
constitute a bootable direct desktop or physical acceptance.

## Verification

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). Checked:
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat`
reported 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present, `/dev/dri` absent. No packages, shared kernel, BPF pins or
services changed. An initial probe omitted the documented Rust PATH and included
an unnecessary libdrm pkg-config lookup; the corrected required-prerequisite
check above passed. Early read commands used incorrect root journal paths and
PowerShell wildcard arguments to rg; corrected reads made no changes.

Windows commands passed (this Linux-only crate runs zero Windows runtime tests):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and checkout-specific
`CARGO_TARGET_DIR=/root/alo-os-target`, all passed:

```sh
cargo test -p alo-shell --locked direct_input_loop
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Three new tests cover idle/render/stop ordering, simultaneous dispatch/callback/
close/disable refusals without rendering or retry, real empty-seat libinput under
an active scope with calloop pause-plus-activation, and pause during empty-batch
input dispatch preventing the scheduled frame even when output retirement fails.
The scope test observes target destruction before manager close and real Unix
socket EOF. Graphics/DRM/manager operations in those tests are injected; libinput,
udev context, calloop and descriptor lifetime are real. Existing client-wire
input, popup/grab cleanup and held-key retirement tests also pass. Final shell
totals: 138 unit, 80 client-lifecycle, three socket and three compile-fail doctests.
An initial compile used an incompatible test Session error type; corrected to
Smithay's supported unit error. Initial fmt ran before its test module existed;
it passed once that file was written. No runtime test failure or lint exemption.

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
test -S /mnt/wslg/runtime-dir/wayland-0
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both exited 0: offscreen golden pixels/truncated-SHM refusal and nested
popup/cursor regression with 115 client surfaces. Expected Mesa and deliberately
invalid-client diagnostics appeared. WSLg is regression evidence, not evidence
that the new direct entry method acquired physical input or scanned out to DRM.
Source/test/documentation diff reviewed; `git diff --check` passed.

## Reconciliation and remaining work

One published report was unreferenced at iteration start:
`docs/autonomy/updates/owner-model-choice-direction.md`. Reviewed it and the
corresponding feature clarification. Consolidated it into all four shared
progress documents: model ownership, processing location and verified privacy
are distinct; existing local, paired, provider and no-agent choices remain.
The report claims documentation review and diff checks only, no runtime tests.
ADR 0021 remains proposed; Claude owns its revision and policy-decision
presentation. No privacy policy or release tier was changed by reconciliation.

Next: safe standalone GLES construction under current lint policy, followed by
full direct entry integration on a DRM/seat fixture. If that dependency remains
blocked, delivery step 3's independent window-management work is eligible.
The new entry method is compiled, while the real empty-seat integration exercises
its shared loop/owner and active-scope boundaries. Populated evdev extraction,
hotplug, actual libseat/DRM acquisition, failed-disable/GPU-context-loss recovery
and certified laptop/GPU-workstation physical records remain owed. Pinned
libseat's early disable acknowledgement limitation remains unchanged.

All four shared documents updated without ticking compositor or release. Full
independent Windows/Linux workspace, rustdoc and BPF publication gates belong
to the supervisor and were not run here. No staging, commit, push, other checkout
edit, worker/loop launch, tools/dev-loop change or physical installation. Reports
arriving during publication are reconciled next iteration.
