# Libinput seat-event routing

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; event-routing component complete, compositor unfinished.

## Change and decisions

`crates/alo-shell/src/libinput_routing.rs` connects borrowed SeatInput updates
to the existing Server keyboard/pointer validation. It extracts Linux evdev codes
without adding XKB's offset twice, accelerated relative motion, normalized
absolute positions and button transitions. No event or device reference escapes
the synchronous callback. Reset calls clear_input even without capabilities or
a valid output extent. Handler errors must be propagated into SeatInput dispatch
so it retires the context; the trusted caller still polls and flushes.

`direct_seat.rs` handles first press/last release across multiple devices using
libinput's per-key/per-button seat counts. Zero-count presses and invalid codes
refuse. Any device removal conservatively clears the entire seat: held keys,
modifiers, buttons, focus, drags and popup initiating serials. This deliberately
cancels interactions on surviving devices too. It avoids retaining device refs or
inventing a second per-device ledger; explicit keyboard focus and fresh pointer
motion are required afterwards. The trusted DirectSeatEvent API is not exposed
over agent or client IPC. Existing lower-level input APIs remain available.

`libinput_scroll.rs` accepts modern wheel, finger and continuous scroll events.
Wheel v120 defines a 15-pixel step per 120 units, independent of hardware angle;
finger/continuous values retain upstream signs and natural-scroll metadata.
Only reported axes are emitted; zero finger/continuous values end that axis.
Nonfinite/unrepresentable values refuse before casting. Legacy axis events are
ignored to avoid duplicate delivery alongside modern events. Device additions
do not select focus; unsupported touch/tablet/gesture/switch events are ignored.
This implements the existing v0.01 keyboard/pointer scope, not later touch support.

ADR 0002 authorizes native Rust/Smithay input. No engine patch, dependency change,
unsafe exception, user-facing string or daemon-contract change was needed. Public
Rust API behavior and caller obligations are documented in rustdoc. Seat-count
semantics were checked against pinned upstream source and the official
[libinput keyboard documentation](https://wayland.freedesktop.org/libinput/doc/1.27.1/api/group__event__keyboard.html).
The component and acceptance criteria were entered in QUEUE before coding.

## Verification

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). Checked
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat`:
1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present; `/dev/dri` absent. No package installation or shared
kernel/BPF/service change was needed.

Windows commands passed (zero runtime tests because this crate is Linux-only):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux commands, from `/mnt/c/dev/alo-os` with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and checkout-specific
`CARGO_TARGET_DIR=/root/alo-os-target`, passed:

```sh
cargo test -p alo-shell --locked libinput
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Final shell totals: 135 unit, 80 client-lifecycle and three socket tests, plus
three compile-fail doctests. Nine new tests: three scroll conversion/refusal
tests and six real Wayland client tests for multi-device counts, removal,
modifier/button cleanup, invalid data/activity, motion/scroll stop/disconnect,
popup dismissal/stale serial refusal, and real empty-seat libinput pause/reset.
The wire tests inject extracted event data, not kernel evdev events. The real
libinput test dispatches an empty udev seat and routes its pause reset into the
Server, observing matched release/leave/modifier reset on a client. Raw populated
libinput event extraction and physical hotplug are compiled, not runtime-proven.

Initial test compilation needed a Session error type implementing AsErrno. One
initial wire assertion expected motion where the first pointer target correctly
received enter; the assertion now checks entry coordinates. Clippy found test
module placement, indexing, expect calls and an explicit test panic; corrected
without lint exemptions or weakening checks. No repeated runtime test failure.
Early read commands used a wrong journal path and shell quoting; corrected reads
made no environment changes.

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`, passed:

```sh
test -S /mnt/wslg/runtime-dir/wayland-0
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

WSLg golden pixels, truncated-SHM refusal and nested popup/cursor regression
passed, 115 client surfaces, with expected Mesa/invalid-client diagnostics.
Source/test/documentation diff review and `git diff --check` passed.

## Reconciliation and remaining work

At iteration start, two published reports were absent from STATE:
`network-boundary-decisions-proposed.md` and `one-kernel-two-checkouts.md`.
Reviewed their evidence and relevant provider-refusal/Waited source. Consolidated
the corrected Answers::Service relay entry point, implemented shared kernel-test
lock, reported cross-process/unit/kernel/full gates and their limits. The later
report revises ADR 0021's recommendation from B to C1; it remains proposed, not
approved. The proposal-only publication limitation was carried with the lock's
code publication, not treated as authorization to bypass a gate. No security
policy changed and no kernel gates were rerun here. Claude retains that workstream.

CHANGELOG, ROADMAP, QUEUE and STATE record this component and the reconciled
reports. Next: wire SeatInput into DirectSession's latched seat polling and direct
frame loop, guaranteeing error/reset flush before output retirement. Populated
device acquisition/removal, safe standalone GLES construction, real DRM/seat
behavior, GPU context-loss/failed-disable recovery and certified laptop/GPU
workstation physical records remain owed. No WSL test certifies hardware.

The supervisor owns independent full Windows/Linux workspace/rustdoc/BPF gates
and publication; those were not run by this worker. No staging, commit, push,
other checkout edit, worker/loop launch, tools/dev-loop modification or physical
installation. Reports arriving during publication are reconciled next iteration.
