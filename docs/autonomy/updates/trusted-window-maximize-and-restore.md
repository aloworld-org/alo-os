# Trusted window maximization and restoration

Date: 2026-09-08. Workstream: native desktop/window operations (delivery phases
2/3). Responsible contributor: single desktop development worker in
`C:\dev\alo-os`, also the shared progress integration owner.

Status: ready for integration. Trusted transaction component complete; full
window-management feature and independent supervisor publication gates pending.

## Change and decisions

Native shell controls can request maximization and restore the original normal
window geometry. The new `crates/alo-shell/src/window_maximize.rs` retains one
snapshot per mapping, sends XDG maximized state and places only the newest
acknowledged committed response. Requests, acknowledgements, output submission
and actual client pixels remain distinct facts. Repeated requests are no-ops;
rapid toggles and output changes retain the first normal geometry.

`server.rs` publishes maximize dimensions only after successful submission;
`output_retirement.rs` suspends pending maximize placement only after successful
retirement. New output submission reconfigures surviving maximized windows;
restore works without an output. `surfaces.rs` handles commit and lifetime
cleanup. Unmap/disconnect forget state. `window_press.rs`, `window_size.rs` and
`window_placement.rs` refuse competing operations until restoration commits.

Accepted ADR 0002 keeps this entirely Rust/Smithay; no web runtime or upstream
patch. ADR 0010 remains unchanged; there are no new rendered strings or colors.
The application-adapter/agent contracts are unchanged. Public rustdoc and
`docs/contracts/native-window-maximize.md` describe the additive trusted API and
new refusal variants.

Policy choices: use the entire last submitted scale-one output (no invented dock
work area); maximize may ignore normal size hints as permitted by pinned XDG
`set_min_size`/`set_max_size`; restore clamps saved dimensions to current committed
limits. A million-pixel bound reuses native placement/resize arithmetic limits.
No configure implies allocation, forced scaling or proof that a client complied.
Normal geometry is remembered until restore commits, avoiding rapid-toggle drift.
No decision conflicts with accepted ADRs and no new release scope is introduced.

## Acceptance and verification

Seven real Wayland tests in `tests/window_maximize/mod.rs` cover conforming and
nonconforming client dimensions, request/ack/commit ordering, duplicate requests,
rapid toggles and stale response rejection, committed versus pending limits,
impossible restore limits, effective geometry versus buffer origin, excessive
tree bounds, foreign/child/popup/unmapped/dead refusal, remapping, move/resize
ownership, ordinary keyboard isolation, failed output submission/retirement,
successful retirement/replacement and unavailable/excessive output dimensions.
The test render sink proves coordinator behavior, not graphics or scanout.

Prerequisites verified in Ubuntu WSL2: Rust 1.98.0, WSLg Wayland socket,
wayland-client/server 1.24.0, EGL 1.5, GBM 26.0.8-1ubuntu0.3, libudev 259,
libinput 1.31.1, libseat 0.9.2 and xkbcommon 1.13.1. The initial unquoted inherited
PATH produced export diagnostics; the explicit fixed Linux PATH below corrected
the prerequisite command. No packages, system services or shared kernel state
were changed. Pinned registry XDG protocol XML was read locally for size policy.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
and checkout-specific `CARGO_TARGET_DIR=/root/alo-os-target`:

```text
rustc --version
test -S /mnt/wslg/runtime-dir/wayland-0
pkg-config --modversion wayland-client wayland-server egl gbm libudev libinput libseat xkbcommon
cargo test -p alo-shell --locked window_maximize
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Initial focused four-test run had two fixture assertion failures: reading
Smithay thread-local scene data from the test thread returned an empty origin.
Moved observations to serialized display-thread callbacks; all six then-existing
tests passed. Added effective-geometry/refusal coverage; affected full suite
passed 141 unit + 133 lifecycle + three socket = 277 tests and three compile-fail
doctests, zero failed/ignored. Warnings-denied rustdoc, example build and fmt
passed. First clippy found five potentially panicking indexing sites; replaced
them with iterator access without allowances. Clippy passed. Adding the complete
output-sized graphics buffer then exposed an unused helper in the protocol test
target; used it in the conforming protocol happy path rather than suppressing
the warning. Final focused run passed all seven tests; final affected Linux
all-target clippy and example build passed, without warning suppressions.

Windows commands from `C:\dev\alo-os`:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

All passed; zero Linux-only runtime cases execute on Windows. Final changes after
that check are fixture-only buffer evidence and documentation, covered by final
Linux affected checks. Full independent supervisor Windows/Linux workspace,
rustdoc and pinned BPF publication gates have not run for this task.

Graphical commands with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```text
/root/alo-os-target/debug/examples/nested_check --offscreen
/root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both graphical commands returned exit 0. All 21 offscreen stages passed; nested
popup/cursor regression submitted 115 client surfaces. Expected Mesa fallback
diagnostics and deliberately malformed-client protocol errors did not skip any
checks. The new
`examples/support/window_maximize_check.rs` checks all 6,400 pixels in each of
five frames: unchanged placed 32x24 buffer after request and acknowledgement,
actual 33x32 maximized buffer after commit, unchanged maximized pixels after
restore acknowledgement, and restored 32x24 buffer at (20,20) after commit.

Changed source, new files, fixtures, tests and documentation inspected. Final
Windows/Linux fmt and `git diff --check` passed. All four shared progress
documents updated with the component evidence and remaining work. No full
supervisor gate execution or release certification is claimed.

## Reconciliation and remaining work

Initial working tree clean. Read constitution, current delivery/ownership/report
rules, queue, STATE tail, relevant feature/roadmap sections, ADRs 0002/0010,
native resize and application-adapter contracts. Compared all published report
filenames with STATE: none awaited reconciliation at iteration start. Reports
arriving during publication reconcile next iteration. Claude's credential and
filesystem-security workstream was not taken or modified.

Next component: client XDG maximize/unmaximize request policy, including initial
pre-map requests, mapped requests and refusal/response/lifetime tests using these
transactions. Then minimise/tile and native controls, dock work areas, launcher
and remaining desktop integration. This task completes the trusted transaction
component, not the complete maximise or window-management feature. Configurable
shortcut integration remains in phase 3 after underlying operations.

WSLg is protocol/GLES fixture evidence, not physical DRM/seat display entry,
populated input/hotplug, GPU/recovery or certified laptop/workstation acceptance.
Those exact machine records remain owed in their delivery phases; physical
acceptance follows phase 7 VM image integration. No release checkbox is ticked.
No staging, commit, push, tools/dev-loop changes, worker/loop launch, other
checkout changes, credential reads, identity changes or unrelated host changes.
Supervisor owns integration/retesting and publication.
