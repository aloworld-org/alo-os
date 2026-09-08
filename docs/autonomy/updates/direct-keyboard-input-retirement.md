# Direct keyboard and input retirement

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; input retirement component complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/direct_keyboard.rs` adds the trusted `DirectKeyEvent`,
`Server::direct_keyboard` and `Server::clear_input` interfaces. Activity-checked
evdev transitions use the existing XKB/focus/serial validation path. Active
invalid keys preserve state; inactive calls discard even malformed queued input,
release held keys, dismiss popup grabs and clear focus. Active calls do not
select a recipient. Trusted shell policy must explicitly restore keyboard focus.
Duplicates and unmatched releases cannot deliver keys or authorize popups.

Whole-seat cleanup also clears pointer focus and releases held buttons. Repeated
cleanup does not repeat releases or leaves, and missing capabilities are harmless.
Cleanup queues events; it does not acquire/revoke device authority or poll a seat.
The caller must stop feeding input while inactive. Pointer reentry requires fresh
motion, and keyboard reentry cannot inherit held keys or depressed modifiers.

`src/direct_loop.rs` clears input before session acquisition/discovery and before
output retirement on all returned loop exits, including stop, pause and runtime
errors. Retirement failure cannot prevent cleanup; the existing final flush sends
queued events without dispatching requests after pause. On acquisition/discovery
refusal before target creation, cleanup remains queued for the caller to flush
or dispatch. Existing runtime/retirement/flush/descriptor errors remain separate.

This is the keyboard cleanup component of delivery step 2, selected in QUEUE
before implementation. Libinput acquisition and event dispatch are still next;
this report does not claim direct input is complete. Reusing the existing seat
avoids separate modifier/grab state. ADR 0002 and the v0.01 one-display input
feature authorize this native Rust work. No engine patch, lint exemption, new
scope, user-facing strings, agent capability or daemon contract change. Public
Rust interfaces have rustdoc; daemon-protocol's invocation-only boundary remains.

## Verification actually run

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). Verified with `rustc --version`
and `pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput
gbm libseat`: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3,
0.9.2. WSLg socket present at `/mnt/wslg/runtime-dir/wayland-0`; `/dev/dri`
absent. No dependency installation or shared kernel/service changes.

Windows, passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows runs zero tests for this Linux-only crate. Linux, from `/mnt/c/dev/alo-os`
with `PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`, passed:

```sh
cargo test -p alo-shell --locked direct_keyboard
cargo test -p alo-shell --locked direct_input_retirement
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
test -S /mnt/wslg/runtime-dir/wayland-0
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Five new tests: four real Wayland socket tests, and a real Smithay seat/XKB-state
ordering test across four stop/pause/disable-refusal combinations. Additional
integration evidence includes exact matched key/button releases and leaves,
depressed modifier reset, empty reentry keys, no delivery without restored focus,
no inherited keys after disconnect, popup dismissal and refusal of a fresh,
unconsumed root serial after pause. The ordering test uses an identity-only
Wayland surface and injected target: it asserts cleared focus/pressed keys at
retirement, exactly one retirement, zero submissions and retained disable failure.
It does not simulate a mapped client or real DRM; wire delivery is checked by the
separate client tests. Existing direct-loop production-target tests also pass.

Linux shell totals: 118 unit tests, 74 client-lifecycle tests, three socket tests
and three compile-fail doctests. The popup test was strengthened during final
review to exercise an unconsumed serial; all four direct-keyboard tests and
affected Linux clippy were rerun afterward. Initial test compilation caught an
untyped integer keycode; initial clippy rejected a test `panic!`. Both were fixed,
the latter with counted refusal plus a zero-submissions assertion, without lint
exceptions or weakened tests. No runtime test failed.

WSLg golden-pixel/offscreen refusal and nested popup/cursor regression pass;
nested submitted 115 client surfaces. Expected Mesa and deliberately invalid
client diagnostics remain. This is graphics regression evidence, not physical
libinput or DRM evidence. Source/test diff review and `git diff --check` pass.

## Progress integration and remaining work

Read constitution, delivery/ownership/report instructions, current queue and
journal tail, relevant feature/roadmap, ADR 0002 and daemon input boundary.
Working tree was clean. At iteration start, every published report filename
was already referenced in STATE; none needed reconciliation. Claude's network
request-boundary work stays with Claude. Reports arriving during publication
are reconciled next iteration. CHANGELOG, ROADMAP, QUEUE and STATE record this
component and its limits in the same change.

Next: seat-owned libinput acquisition and event routing, tested open/dispatch
failure and device removal feeding these cleanup boundaries. Safe standalone
GLES construction is still blocked by pinned upstream unsafe constructors under
workspace policy; no exception or engine patch introduced. Live input/session/
renderer integration, real DRM and libseat behavior, GPU context loss,
failed-disable recovery, and certified laptop/GPU-workstation physical records
remain owed. Neither compositor nor release is complete.

Full independent Windows/Linux workspace, rustdoc and BPF publication gates have
not been run by this worker; they belong to the supervisor. No staging, commit,
push, other checkout/repository edits, worker/loop launch, physical installation,
or modification of tools/dev-loop.
