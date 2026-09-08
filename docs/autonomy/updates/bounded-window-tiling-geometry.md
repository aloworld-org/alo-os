# Bounded window tiling geometry

Date: 2026-09-08. Workstream: native desktop window operations, delivery phases
2/3. Responsible contributor: single desktop development worker in
`C:\dev\alo-os`, also the shared progress integration owner.

Status: ready for integration. Geometry component complete; tile/restore
transactions and the full window-management feature remain unfinished.

## Change and decisions

Native controls can calculate exact left/right half-output targets without
disturbing the current scene or ordinary typing. The additive trusted API lives
in `crates/alo-shell/src/window_tiling.rs`, with public rustdoc and
`docs/contracts/native-window-tiling.md`. It reuses the successful-output extent
and retirement rules of maximize and the effective-geometry validation of resize.
No new agent or application-adapter endpoint, engine patch, strings or colors.
Accepted ADRs 0002/0010 remain unchanged.

Odd output widths give their extra pixel to the right, keeping full coverage.
Widths below two pixels and dimensions above one million refuse. Unlike maximize,
exact half tiles respect committed min/max hints; incompatible hints refuse
instead of silently clamping into overlap. Pending hints cannot affect a plan.
Actual-size anchoring preserves top and the selected outside edge, including
bounded nonconforming client sizes. Pixels are never scaled or fabricated.
Snapshots are immutable data without role/serial authority; later changes require
recapture and validation. Output bounds are the full submitted scale-one extent,
without an invented dock reservation. These are routine v0.01 policy choices,
not v0.5 linked-neighbour layouts, corner snapping or saved arrangements.

## Acceptance and verification

Three unit tests cover even/odd and extreme output coverage, unsupported outputs,
exact hint boundaries/conflicts and actual-size anchoring/refusal. Four real
Wayland tests in `tests/window_tiling/mod.rs` cover committed versus pending
limits, immutable snapshots, no extra configures, preserved placement/order,
normal keyboard/pointer isolation, output failure/retirement/replacement,
foreign/child/popup/hidden/unmapped/dead refusal, remapping and excessive effective
tree geometry followed by valid client recovery. All seven focused tests pass.

The complete Linux shell suite passes 144 unit, 153 lifecycle, three socket tests
and three compile-fail doctests, zero failed/ignored. Linux affected all-target
clippy passes with warnings denied. No test failure, retry workaround or weakened
assertion occurred. Review caught and corrected a missing Linux cfg guard on the
new graphical example module before Windows verification.

Ubuntu WSL2 prerequisites verified: Rust 1.98.0, WSLg Wayland socket,
wayland-client/server 1.24.0, EGL 1.5, GBM 26.0.8-1ubuntu0.3, libudev 259,
libinput 1.31.1, libseat 0.9.2 and xkbcommon 1.13.1. No dependency installation,
shared service/kernel changes or cleanup. The desktop alone held the build slot.
Windows C: was checked before each build/test/lint command and stayed above the
12 GiB reserve; readings and final verification are recorded below.

Linux commands run from `/mnt/c/dev/alo-os` with
`PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
and checkout-specific `CARGO_TARGET_DIR=/root/alo-os-target`:

```text
rustc --version
test -S /mnt/wslg/runtime-dir/wayland-0
pkg-config --modversion wayland-client wayland-server egl gbm libudev libinput libseat xkbcommon
cargo test -p alo-shell --locked window_tiling
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Additional Linux commands, all passed:

```text
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Graphical commands, both exit 0, with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir`
and `WAYLAND_DISPLAY=wayland-0`:

```text
/root/alo-os-target/debug/examples/nested_check --offscreen
/root/alo-os-target/debug/examples/nested_check --popups --cursor
```

All 22 offscreen stages pass, including the new `tile_geometry_check.rs` check
of all 6,400 pixels after each half's accepted plan and refused zero-width
actual-size calculation. Nested popup/cursor regression submits 115 client
surfaces. Mesa fallback messages and deliberate malformed-client diagnostics
did not skip assertions. The submitted output used for plans is the fixture's
33x32 extent; the preservation readback is 80x80. This is scene preservation,
not evidence of applying tiles or direct scanout.

Windows commands from `C:\dev\alo-os`, all passed:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows runs zero Linux-only shell cases; all 300 runtime cases and three
doctests were executed on Linux. Final Linux fmt check passed. Initial C: reading
was 15,550,222,336 bytes (about 14.483 GiB); command preflights were at least
14.491 GiB. Last verification preflight: 15,559,065,600 bytes (14.491 GiB).
The reserve is operational headroom, not a continuous quota. No cleanup.

Changed source, tests, examples, contract and shared progress diff reviewed;
`git diff --check` passed. All four shared progress documents updated.
Independent supervisor Windows/Linux workspace, rustdoc and BPF publication
gates have not run for this task.

## Integration and remaining work

Initial tree clean; supervisor owns the pull and publication. Read constitution,
delivery/shared-main/report rules, current queue/STATE tail, relevant feature/
roadmap, accepted ADRs 0002/0010 and native-window/application-adapter contracts.
All published task report filenames were already referenced in STATE at iteration
start; none required reconciliation. Publication arrivals reconcile next iteration.
Claude's filesystem/credential work and other checkout were untouched.

QUEUE recorded this exact component and acceptance before implementation. Next:
trusted tile/restore transactions sharing normal geometry with maximize, tiled
XDG states, configure/ack/commit ordering, actual-size placement, stale-response
refusal, output-change and unmap/disconnect cancellation, then native controls.
This geometry API does not configure a tile or restore a window, and no feature
or release checkbox is promoted. Configurable shortcut integration follows
underlying operations in phase 3.

Real-client and WSLg GLES fixtures do not certify direct DRM/seat entry, populated
input/hotplug, GPU/recovery or the certified laptop/workstation. Those machine
records remain owed in their delivery phases; physical acceptance follows phase
7 VM image validation. Planning pixel preservation is not tiled-window rendering
or tile transaction evidence. No staging, commit, push, tools/dev-loop change,
worker/loop launch, private credential/identity access or unrelated host changes.
