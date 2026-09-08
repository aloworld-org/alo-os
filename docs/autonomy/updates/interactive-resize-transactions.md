# Interactive resize transactions

- Date: 2026-09-08
- Workstream: native desktop compositor
- Contributor: single desktop worker and shared progress integration owner,
  `C:\dev\alo-os`
- Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Applications can resize a native window from a held pointer press. The shell
respects the application's current size limits and keeps the opposite edge at
the size actually committed by the application. Invalid requests do not consume
input. Typing remains with its existing recipient. This completes the XDG resize
transaction component, not the full window-management feature or desktop UI.

`crates/alo-shell/src/resize_transaction.rs` owns negotiation, input lifetime and
committed-response anchoring. `window_press.rs` shares the existing move authority
check with resizing; geometry stays in `window_resize.rs`. `surfaces.rs` connects
the protocol and commit callbacks; `pointer.rs` connects the same trusted input
path used by nested and direct backends. Public input rustdoc and
`docs/contracts/native-window-resize.md` document the behavior. There is no agent
verb, adapter-contract change, engine patch, user-visible string or colour change.

Decisions follow ADR 0002's native Rust boundary and preserve ADR 0010:

- Validate this seat's held press on the exact mapped root or subsurface tree
  before configuring or consuming input. None/unknown edges, forged/stale/released
  serials, foreign targets and conflicting grabs refuse. Share move authority
  rather than maintaining a second security predicate.
- Capture one geometry/pointer anchor, refresh only committed constraints on
  each motion and button release, and suppress duplicate size configures.
  Invalid motion leaves the transaction unchanged. Impossible live limits cancel.
- Use Smithay's committed configure serial, not merely the last acknowledgement,
  to authorize actual-size anchoring. A client may keep its buffer, commit only
  geometry or choose dimensions different from the suggestion. Suggestions never
  allocate client storage or move old pixels.
- Balance the initial client press/leave and consume drag buttons. The last
  release clears XDG resizing state and restores pointer hit testing; retain the
  geometry anchor through the final acknowledged commit. Older resize responses
  may still commit while that final response is pending. Later spontaneous
  changes cannot reuse a completed anchor.
- A new move/resize waits while the final response is outstanding; leave cancels
  it without forcing a client response or inventing a timeout. Unmap/disconnect
  remove authority from that mapping. Out-of-range committed geometry cancels
  anchoring rather than overflowing scene coordinates.

## Acceptance and verification

Seven new real-client tests in `tests/interactive_resize/mod.rs` cover all eight
edges, actual-size choice, unchanged placement on acknowledgement alone,
unacknowledged buffer commits, geometry-only response commits, final response
retirement, stale/forged/foreign/released/None/unknown-edge refusal, no-pointer
and unmapped targets, current versus pending limits, initial impossible limits,
release-time limit refresh, invalid motion, duplicate requests/configures,
subsurface presses, multiple buttons, move exclusion, unrelated unmap,
leave/unmap/remap/disconnect and impossible-limit cancellation. Ordinary keyboard
delivery stays with the focused client and does not reach another client.

Ubuntu WSL2 prerequisites checked:

```text
export PATH=/root/.cargo/bin:/usr/bin:/bin
rustc --version
test -S /mnt/wslg/runtime-dir/wayland-0
pkg-config --modversion wayland-server wayland-client egl gbm libinput libudev xkbcommon libseat
```

Rust 1.98.0; socket present; versions respectively 1.24.0, 1.24.0, 1.5,
26.0.8-1ubuntu0.3, 1.31.1, 259, 1.13.1 and 0.9.2. No dependency installation,
service change, shared kernel/BPF mutation or other checkout access. Linux rg
was unavailable for an upstream-source read; grep/sed read the pinned Smithay
resize dispatch and committed-serial implementation instead.

Linux commands from `/mnt/c/dev/alo-os`, using that PATH and isolated
`CARGO_TARGET_DIR=/root/alo-os-target`:

```text
cargo check -p alo-shell --locked
cargo test -p alo-shell --locked interactive_resize
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Check passed. Initial focused run: four passed, one fixture failure because its
remap helper already acknowledged the configure and the test acknowledged it
again. Corrected only that duplicate acknowledgement; focused rerun passed all
five then-existing tests. First all-target clippy found the new graphical helper
accepted u32 while existing stages are u8; corrected its parameter. No lint
allowance or production/test weakening. Added two further lifecycle tests and
release-time limit refresh before the final checks.

Final Linux affected clippy, tests, rustdoc, example build and fmt all returned
exit 0. Tests: 141 unit + 126 lifecycle + three socket = 270, plus three
compile-fail doctests; zero failed/ignored. This includes all seven new resize
tests and the shared authority's existing move/popup/input regressions.

Windows commands from `C:\dev\alo-os`:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

All passed. Windows runs zero Linux-only shell runtime tests; this is not a
second platform's execution of the resize transaction.

Graphical commands with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```text
/root/alo-os-target/debug/examples/nested_check --offscreen
/root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both returned exit 0. `examples/support/interactive_resize_check.rs` checks all
6,400 pixels of each of four frames: initial placed 32x24 window, unchanged old
pixels after a 40x32 request and invalid-motion refusal, actual committed 16x16
choice anchored at (36,28), and final committed 32x24 choice anchored at (20,20).
The complete offscreen suite passes all 16 stages. The nested popup/cursor
regression passes with 115 submitted client surfaces. Expected Mesa fallback
diagnostics and deliberately malformed-client protocol errors are not skips or
failed checks. These fixtures do not measure physical DRM scanout.

Changed source, new files, tests, example and documentation inspected;
`git diff --check` passed. All four shared progress documents updated. Full
independent Windows/Linux workspace, rustdoc and BPF publication gates have not
run for this task; those remain the supervisor's work. No staging, commit or push.

## Reconciliation and remaining work

Initial working tree clean. Read constitution, delivery/shared-main/report rules,
current queue/STATE tail, relevant feature/roadmap sections, accepted ADRs 0002
and 0010, native resize and application-adapter contracts. Compared every
published report filename with STATE: none awaited reconciliation at iteration
start. Reports arriving during publication reconcile next iteration. Claude's
credential/security ownership is unchanged.

Next executable component: trusted maximise/restore transactions with remembered
normal geometry, output-bound sizing and committed-response placement, invalid
target and lifecycle refusal, normal keyboard tests and GLES frames. Minimise,
tile, launcher/dock/controls and remaining v0.01 scope remain unfinished. Raw
configurable shortcuts remain after underlying operations in phase 3.

WSLg proves the exercised protocol/GLES fixture, not real pointer devices,
DRM/seat display entry, hotplug, GPU/recovery or certified laptop/workstation
acceptance. Those machine records remain owed at their delivery phases; physical
acceptance follows phase 7 VM image integration. No feature/release tick or
hardware certification. No tools/dev-loop edits, worker/loop launch, other
repository changes, identity changes, physical installation or unrelated host
changes. The supervisor owns final integration, retesting, commit and push.
