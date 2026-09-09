# Live window control snapshots

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Native controls can now show whether their explicit window can maximize or
restore, including while a previous request waits for acknowledgment/commit.
`Server::window_control_snapshot` returns its exact visible root, immutable layout
and a typed maximize/restore refusal. It never selects a focused or stacked
replacement. Hidden, child, popup, foreign, unmapped and dead roots refuse.
Minimise and close need neither output nor a keyboard/pointer seat and remain
available during a popup grab. Maximize requires a submitted supported output;
restore uses original geometry and committed limits and can survive retirement.

`src/window_control_snapshot.rs` owns capture; `src/window_mode_plan.rs` owns the
side-effect-free request planner extracted from `window_mode.rs`. Both execution
and presentation consume that planner, avoiding a second policy implementation.
The extraction preserves validation order, no-op semantics and configure/commit
boundaries, including tiling. Existing exhaustive `WindowMaximizeError` variants
and their translation remain unchanged. `WindowControlLayout::restoring()` is
additive. Public rustdoc and `docs/contracts/native-window-controls.md` describe
the presentation/input boundary; no stored format, protocol or agent API changed.

This is v0.01 window management under ADR 0002, using the existing native view,
appearance tokens, disabled non-color mark and `Action::said` vocabulary under
ADR 0010. No new dependencies, upstream patches, palette or UI strings. Snapshot
diagnostics are not user-facing labels. Routine choices require no new ADR.

Capture uses `&self`: it prunes nothing, sends no configure/close, remembers no
geometry, changes no focus and consumes no input. The selected complete component
is live presentation capture, not usable controls. A snapshot is not a
mapping-lifetime token: unmap/remap can reuse a protocol handle; stored snapshots
remain frozen. Future pointer routing must bind presses to mapping lifetime,
cancel stale ownership and revalidate at release. There is no snapshot dispatch
method that could accidentally treat captured availability as authorization.

## Acceptance and executed checks

All six new private-display integration tests in
`crates/alo-shell/tests/window_controls/mod.rs` passed on their first focused run.
They cover capture without wire/focus/geometry/typing effects, invalid view bounds,
failed/successful/unsupported/retired output, pending toggles and tiled intent,
pending versus committed restore hints, excessive geometry, popup busy refusal,
actual minimize/close availability during the grab, foreign/hidden/unmapped/dead
targets, fresh remapping and immutable old snapshots. Child-root refusal was
added before final clippy and passed in the full suite. Existing input and layout
transaction tests remain intact. No test or lint was weakened; no failing test
run occurred. Fixtures use private resources and do not mutate shared kernel state.

Ubuntu WSL2 prerequisites were checked with:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
```

Rust 1.98.0 (88d9e12ae 2026-08-18); respective package versions 1.24.0, 1.5,
3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3 and 0.9.2. Socket available.
No dependency installation, service/session change, mount or WSL restart.

Every build/test/lint/format invocation had a PowerShell `(Get-PSDrive C).Free`
preflight refusing below `12GB`. Lowest measured preflight:
74,726,813,696 bytes (69.59 GiB). Measurements are operational preflights, not
a continuous quota or attribution of other workers' disk usage. No cleanup.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

All exited zero. Focused run: six existing view unit tests and six new client
tests. Final full shell suite: 150 unit, 177 client-lifecycle and three socket
tests, plus three compile-fail doctests; none failed or ignored. Affected
all-target clippy and rustdoc with warnings denied passed; examples built.

Windows commands from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed; Windows executes zero Linux-only shell tests. Formatting ran after
the implementation and again after the final fixture additions. Final Linux fmt
also passed. `git diff --check` passed; source, tests, fixture and docs inspected.

WSLg commands after rebuilding, with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

Both exited zero. All 28 offscreen stages passed. New
`examples/support/window_control_snapshot_check.rs` paints live-derived views
and compares every pixel/hit boundary with independent existing literal masks
and palette bytes. Five maximize/restore boundaries plus the restored boundary
reused after minimization each check two schemes: **12 complete 5,760-pixel
frames**. Requested intent, including pre-commit glyphs, is asserted independently
by stage. Existing client scene comparisons remain unchanged (all 6,400 pixels at
each maximize/restore boundary). The separate view regression passes all 128
frames, covering every availability combination and clipping position. Mesa
fallback diagnostics and the deliberately truncated SHM client's protocol error
did not skip assertions. The live snapshot frames cover enabled states; disabled
state derivation is real-client evidence, while disabled pixels are the separate
view regression. No on-screen/native interaction or DRM claim follows.

## Integration and remaining work

The iteration began with a clean tree; every published report was already
referenced in STATE.md, so there was no new contributor report to reconcile.
Reports arriving during publication reconcile next iteration. Claude's assigned
workstream and checkout remain untouched. CHANGELOG, ROADMAP, QUEUE and STATE
record this complete component without promoting a feature or release checkbox.

Next: mapping-lifetime-bound press/release ownership, live operation revalidation,
cancellation and disabled hit isolation; then native labels, hover/pressed feedback
and production nested/direct composition. Launcher/dock, raw configurable
shortcut/settings integration, clipboard and later delivery work remain open.
Normal client keyboard component coverage ran in this task. WSLg evidence is
offscreen development evidence; physical keyboard/libinput, direct DRM/session
recovery, integrated VM boot/update recovery and certified laptop/GPU workstation
records remain owed in their scheduled phases. The unchanged nested on-screen
popup/cursor regression was rebuilt but not rerun in this task.

The supervisor's full independent Windows/Linux workspace test/lint/rustdoc and
pinned BPF publication gates have not been run by this worker. No staging,
commit, push, Git identity/credential access, tools/dev-loop edit, other-checkout
change, host cleanup or additional worker/loop launch was performed.
