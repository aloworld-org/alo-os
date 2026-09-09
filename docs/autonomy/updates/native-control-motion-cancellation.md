# Native control motion and input-loss cancellation

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Completed the motion and input-loss cancellation component of the queued native
strip router. `window_control_input.rs` adds `Server::window_control_motion`:
every observed motion checks the original action, current painted geometry,
visibility identity and maximize/restore intent. An excursion permanently disarms
the held press, including when the pointer returns before release. Duplicates
cannot rearm it. Motion and release share one read-only matching predicate;
existing live release policy and typed errors are unchanged.

`pointer.rs` cancels native execution before pointer-leave cleanup, even when
pointer capability is absent and cleanup refuses. Existing nested/direct pointer
deactivation inherits that hook. `direct_keyboard.rs::clear_input` disarms native
presses on whole-seat reset. Cancellation retains primary release ownership.
The host still must intercept primary press/release, observe each motion with
current painted geometry, withhold owned events and cancel removed UI. Motion
observation does not move ordinary seat position, focus or windows.

The selected component was recorded in QUEUE before implementation. Completing
the cancellation boundary first keeps primary interception and presentation
integration reviewable; this is not a completed strip router or usable controls.
ADRs 0002/0010 and existing v0.01 scope are unchanged. No vocabulary, dependencies,
protocol or stored formats changed. Additive public rustdoc and
`docs/contracts/native-window-controls.md` explain the host contract.

## Acceptance and verification

Four new private-display tests in `tests/window_controls/motion.rs` exercise
in-hit execution and ordinary typing/pointer isolation; other buttons' hit areas,
gaps, output/half-open bounds and non-finite excursions; out-and-back and duplicate
refusal; changed geometry and transient maximize/restore intent; pointer leave,
nested/direct deactivation and whole-seat reset with and without pointer capability.
Matching release ownership survives cancellation and cleanup errors; a fresh
gesture works after the consumed release. Existing mapping-lifetime, live-policy
and client-grab refusal tests continue to run. No shared kernel state is mutated.

Ubuntu prerequisites: Rust 1.98.0; `pkg-config --modversion wayland-client egl
glesv2 xkbcommon libudev libinput gbm libseat` returned 1.24.0, 1.5, 3.2, 1.13.1,
259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2. WSLg socket
`/mnt/wslg/runtime-dir/wayland-0` exists. No install or shared maintenance needed.

Each build/test/lint/format command uses a PowerShell `(Get-PSDrive C).Free`
preflight, refusing below `12GB`. No cleanup or outer kernel-fixture lock used.
Lowest measured preflight: 71,270,281,216 bytes (66.38 GiB). The reserve is
operational headroom, not a running disk quota.

Linux commands, from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
```

All passed. Focused run: six view units and 17 control lifecycle tests. Full run:
150 units, 188 lifecycle tests, three socket tests and three compile-fail doctests;
none ignored. Existing malformed-client/keymap diagnostics are refusal fixtures.
No failed tests, assertion corrections, lint exemptions or test weakening.

Windows commands, from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed; formatting ran twice as Rust edits progressed. Windows executes zero
Linux-only shell tests. One initial read-only PowerShell search had invalid brace
syntax and was corrected before running; no build/test command failed.

WSLg commands, with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

Both passed. `examples/support/window_minimize_check.rs` adds one complete
6,400-pixel visible-scene comparison after out-and-back cancellation, then uses
in-hit motion before the existing native minimize transaction and hidden/restored
scene checks. All 28 offscreen stages and the twelve snapshot-derived 5,760-pixel
frames remain passing. The unchanged control painter passed all 128 complete
5,760-pixel frames. Mesa fallback and deliberate malformed SHM diagnostics skipped
no assertions. These are offscreen development checks; the on-screen nested
example was rebuilt but not rerun. `git diff --check` passed; tracked diff and new
test/report files were inspected.

## Contributor reconciliation and remaining evidence

The iteration began clean. The only published report not referenced in STATE was
`docs/autonomy/updates/the-daemon-asks-for-the-key.md`; it is reconciled into all
four progress documents in this change. Production lookup and four distinct
externalized refusal branches are present in `alo-agentd`; hermetic default
keyring selection prevents unit tests from reaching the process store by default.
The contributor corrects earlier claims about the build environment: a session
bus and activated keyring exist. No real credentials were inspected here.

That report gives no exact verification commands/results. Its code/test narrative
is retained as contributor evidence, not a claimed rerun or new end-to-end gate.
Daemon retrieval with a real fixture, authenticated HTTPS, connection lifetime,
concurrent retrieval and logout remain unproved by that report. Claude retains
that workstream. Reports arriving during publication reconcile next iteration.

Next desktop component: native primary-event interception with current painted
target/geometry and ordinary client routing isolation, then native labels,
hover/pressed feedback and production nested/direct composition. No feature or
release checkbox is promoted. WSLg offscreen checks do not certify on-screen
interaction, physical input or direct scanout. Populated direct input/session
recovery, integrated VM boot/update recovery and certified laptop/GPU workstation
records remain owed at their scheduled phases. Full independent Windows/Linux
workspace test/lint/rustdoc and pinned BPF publication gates belong to the
supervisor. No staging, commit, push, loop/worker launch, other-checkout edit,
tools/dev-loop change, identity/credential access or shared-system maintenance.
