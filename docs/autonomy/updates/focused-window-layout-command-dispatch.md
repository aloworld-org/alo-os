# Focused window layout command dispatch

Date: 2026-09-08. Workstream: native desktop interaction.
Responsible contributor: single desktop worker and progress integration owner in
`C:\dev\alo-os`. Status: ready for integration after recorded checks below.

## Change and decisions

The native shell can resolve the person's configured minimise, maximise/restore
and left/right snap commands against the window actually owning keyboard focus.
It refuses absent focus instead of choosing another window. Minimise retires
held input and popups without focusing a replacement. Maximise toggles the latest
requested mode, so rapid commands supersede pending replies correctly; tiled
windows maximize before a subsequent toggle restores original normal geometry.
Snap always requests its named half. Existing transaction checks and committed
client buffers remain authoritative.

`crates/alo-shell/src/window_command.rs` adds `dispatch_window_command` and a
non-exhaustive `WindowCommandError`. Close and cycling delegate to the existing
dispatcher. Review caught that adding variants to the original exhaustive error
enum would break source compatibility: the original method, behavior and enum
are unchanged, and a new regression compiles an exhaustive external match.
`window_mode.rs` supplies only an internal latest-intent query. No agent endpoint,
context reader, arbitrary execution, persisted setting or new vocabulary is added.
Labels use existing `Action::said`; diagnostic errors are not user-facing labels.
The additive contract is `docs/contracts/native-window-commands.md`, with public
rustdoc and the configured-command section in `docs/autonomy/COMPOSITOR.md`.

Read the constitution, delivery/ownership/report instructions, current queue,
STATE tail, v0.01 feature/roadmap sections, ADRs 0002/0010 and relevant native
window/application contracts. Native Rust, existing label vocabulary and existing
independent-half layout policy implement accepted decisions without scope changes.
Recorded the component and acceptance in QUEUE before implementation. This is
the complete command bridge component, not completed rendered controls or shortcuts.

## Executed verification

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). Verified using:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
```

Package versions respectively: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1,
26.0.8-1ubuntu0.3, 0.9.2. Socket available; no installation needed. No shared
kernel, service, BPF state or other checkout was modified.

Before each build/test/lint invocation, PowerShell read `(Get-PSDrive C).Free`
and refused below `12GB`. All readings exceeded 14.075 GiB; the lowest recorded
preflight was 15,113,056,256 bytes. No cleanup, parallel worker or build handoff.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked shortcut_dispatch
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

The first focused run passed eight and failed two newly written fixtures: one
expected a 32x24 normal buffer although `Application::attach` supplies 16x16;
one omitted popup protocol enablement. Corrected those fixture errors, retaining
exact size assertions, and the second run passed all ten. Added conflict/clearing
and legacy-compatibility/delegation coverage, then moved the bridge to its additive
API. Final affected all-target clippy passed; the final full shell suite passed
144 unit, 171 lifecycle and three socket tests plus three compile-fail doctests,
none failed or ignored. Seven new real-client tests cover current custom bindings,
all four conflicts/clearing, seat/focus refusal, output/limits, pending toggles,
actual-size tile placement, original restoration, typing, popup busy refusal,
minimise input retirement, unmap/remap/disconnect, and unchanged legacy behavior.
Rustdoc with warnings denied and the example build passed. No tests or lints were
weakened and no repeated failing run was retried.

WSLg commands after rebuilding, with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both exited zero. All 28 offscreen stages passed. The modified minimize fixture
dispatches configured minimise and checks all 6,400 pixels for each of four
hidden/restored frames. The tile fixture dispatches configured right/left snap;
all six 6,400-pixel request/ack/commit/restore boundaries pass unchanged. Existing
maximize client-request pixels remain regression evidence, not new command pixel
evidence. The nested regression passed 115 submitted client surfaces. Expected
Mesa fallback diagnostics and deliberate malformed-client protocol errors did
not skip assertions. These are Wayland/GLES development checks, not DRM scanout,
physical keyboard input or certified-machine evidence.

Windows affected checks from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

All Windows affected commands passed, executing zero Linux-only tests. Final
Linux formatting also passed. Source, new tests, examples and documentation diff
were inspected; `git diff --check` passed.

## Integration and remaining evidence

At iteration start the tree was clean and every published task report filename
was already referenced in STATE; there was nothing new to consolidate. Reports
arriving during publication reconcile next iteration. Claude's assigned work and
separate checkout remain untouched. CHANGELOG, ROADMAP, QUEUE and STATE record
this component without promoting a feature or release checkbox.

Next: rendered native window controls, with externalized labels and pointer
press/release ownership, disabled/refusal states, stale-target isolation and
graphical/input acceptance. Raw layout matching, consumed-key/repeat isolation,
nested/direct shortcut routing, settings persistence, launcher/dock, clipboard
and later delivery requirements remain unfinished. Physical keyboard/libinput,
direct DRM display/session recovery, integrated VM boot/update recovery, and
certified laptop/GPU workstation records remain owed in their delivery phases.
The supervisor's full independent Windows/Linux workspace test/lint/rustdoc and
pinned BPF publication gates have not been run by this worker. No stage, commit,
push, Git identity change, tools/dev-loop edit or host cleanup was performed.
