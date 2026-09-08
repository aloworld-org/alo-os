# Interactive window movement

- Date: 2026-09-08
- Workstream: native desktop compositor
- Contributor: desktop development worker, integration owner in `C:\dev\alo-os`
- Status: ready for integration; independent supervisor gates remain pending.

## Change and decisions

Applications can request a drag using `xdg_toplevel.move`. The compositor checks
the mapped root, this display's seat, and Smithay's active implicit press serial
on that root or a subsurface. Another window, fabricated/stale/release serial,
unmapped target, missing pointer, existing movement or active popup grab cannot
start one. Invalid requests are ignored, as permitted by XDG; no agent verb or
capability contract changes. ADRs 0002 and 0010 remain intact.

`crates/alo-shell/src/window_move.rs` owns authorization and drag lifetime.
`surfaces.rs` connects protocol dispatch and unmap cleanup; `pointer.rs` connects
the common nested/direct motion, button and cancellation paths. Placement uses
the existing committed-geometry scene origin, so rendering, popup positioning
and subsequent hit tests agree. No configure, stacking or keyboard-focus change
is caused by movement. Integer placement rounds the initial-anchor delta, never
accumulating rounding error. The existing million-pixel placement bound refuses
out-of-range motion without losing the drag; no silent clamping.

At takeover, balance the client's existing button presses and clear pointer
focus. Then consume motion/buttons until all held buttons are released, just as
an implicit pointer grab ends on the last button. The final release re-hits the
scene without replaying a button or granting popup release authority. Leave,
deactivation through the common backend path, root unmap or disconnect cancels;
an unrelated root's unmap does not. Unmap cancellation happens during commit,
so a remapped protocol object cannot inherit the old drag.

The six new real-client tests in `tests/window_move/mod.rs` cover movement,
fractional/negative deltas, committed geometry, unchanged configuration/order,
ordinary keyboard isolation, no drag motion/button delivery to another client,
forged/foreign/released/reused serials, invalid coordinates, multiple buttons,
child initiation, unrelated unmap, no pointer, leave/unmap/remap and disconnect.
The offscreen example adds a real client move request after acknowledged resize,
then compares every pixel of a 40x40 GLES frame at positive and negative clipped
positions and proves motion after release cannot continue the drag.

## Verification

Prerequisites: Ubuntu WSL2, Rust 1.98.0, WSLg Wayland socket present. Checked
`pkg-config --modversion wayland-client egl glesv2 libinput libudev gbm xkbcommon
libseat`: 1.24.0, 1.5, 3.2, 1.31.1, 259, 26.0.8-1ubuntu0.3, 1.13.1, 0.9.2.
No dependencies, system services or shared kernel/BPF state changed.
Linux commands run from `/mnt/c/dev/alo-os` with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and isolated
`CARGO_TARGET_DIR=/root/alo-os-target`.

Initial `cargo check -p alo-shell --locked` passed. Focused
`cargo test -p alo-shell --locked --test client_lifecycle window_move` passed
the first four tests before adding the child/no-pointer cases. Two clippy
invocations caught fixture `expect`/`unwrap` calls, corrected to explicit
assertions without allowances or changing the tests' meaning. Their chained
test/doc/example commands did not run. Initial plain `rustc` lacked Cargo's
path; an unquoted inherited PATH produced shell export diagnostics; the explicit
path above corrected the environment. No runtime test failure occurred in these
development checks.

Windows `cargo fmt --all --check`, `cargo clippy -p alo-shell --all-targets
--locked -- -D warnings`, and `cargo test -p alo-shell --locked` returned exit 0.
Windows executes zero Linux-only shell runtime tests. Final Linux checks passed:

```text
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

138 unit, 115 real-client lifecycle and three socket tests passed, plus three
compile-fail doctests; zero failed/ignored. This includes all six new move tests.

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and `WAYLAND_DISPLAY=wayland-0`:

```text
/root/alo-os-target/debug/examples/nested_check --offscreen
/root/alo-os-target/debug/examples/nested_check --popups --cursor
```

The first offscreen run failed: the fixture's continuous cursor-motion injection
changed the pointer from (4,5) to (10,14) between initiating press and the client's
move request. The resulting (4,1) delta correctly differed from the oracle's
(10,10). Client coordination also ended after renderer failure; the chained
nested check did not run. Corrected only fixture coordination: stop cursor-loop
injection at the interactive-move stage. No production code or pixel expectation
changed. Re-ran affected all-target clippy, rebuilt `nested_check`, ran both full
graphics commands and fmt check; all exit 0. Offscreen passed all 12 coordinated
stages, including both complete 1,600-pixel movement frames and release. Nested
passed 115 submitted client surfaces. Expected Mesa fallback diagnostics and
deliberate malformed-client protocol refusals remain; neither is a test skip.

Full independent Windows/Linux workspace, rustdoc and pinned BPF publication
gates belong to the supervisor and have not run for this task.

## Published report reconciliation

At iteration start the tree was clean. Read the constitution, delivery order,
shared-main and report rules, current queue, STATE tail, relevant feature/roadmap,
ADRs 0002/0010 and application-adapter/application-verb contracts. Seven published
reports were absent from STATE; all are reconciled here and in shared progress
documents. Source reports remain unchanged. Contributor-reported tests/gates
below were reviewed, not rerun by this worker.

- `docs/autonomy/updates/settings-refusals-carry-no-credential.md`: four redaction
  tests close the earlier person-settings TOML Debug exposure while retaining
  line/column and known schema words. Machine-description raw TOML diagnostics
  remain a named follow-up; this does not certify all credential diagnostics.
- `docs/autonomy/updates/where-a-credential-goes.md`: two owned local-service
  listeners verify credential pairing; a production daemon refusal test observes
  no accepted connection or departure. This is not authenticated HTTPS provider
  evidence. Its proposed store/tier interpretation is superseded below.
- `docs/autonomy/updates/what-actually-protects-a-credential.md`: loaded-kernel
  Unix-socket reachability demonstrates that bounding does not isolate a store
  from same-process turns. The v0.01 key store and v0.5 Secret portal are distinct;
  no tier moves. Its ADR 0017 conflict, API-isolation and connection-lifetime
  claims are corrected by the next report.
- `docs/autonomy/updates/what-the-agent-cannot-reach.md`: outbound session-bus
  access is allowed by ADR 0017; the inbound agent socket stays unchanged.
  Separate login identities, not `Secret`'s API, provide isolation. The protocol
  test exhaustively checks all four `ToAnAgent` variants. A compromised daemon
  running as the person can retrieve keys. Actual separate-agent-login bus
  refusal is still unmeasured. The libsecret singleton observation must not be
  applied to the later zbus implementation.
- `docs/autonomy/updates/the-persons-own-bus.md`: accepted ADR 0022, derived uid
  path and eight bus/four-refusal-shape tests. Socket ownership refusal is not a
  real separate-user login test; `nothing_was_sent` alone is not a boundary
  measurement. Its missing libsecret package blocker is superseded below.
- `docs/autonomy/updates/libsecret-cannot-be-told-which-bus.md`: contributor
  installed six ordinary development packages after coordination; libsecret
  0.21.7 compiles/links. Its API cannot accept an explicit GDBus connection, so
  the required uid-derived routing could not use that binding. This measured
  binding blocker is resolved by the accepted amendment below.
- `docs/autonomy/updates/the-client-is-given-the-connection.md`: approved ADR
  0022 amendment to `secret-service` over explicitly addressed `zbus`, encrypted
  DH sessions with Rust crypto. Intended/decoy listeners and a default-session
  control verify actual routing. Store implementation exists; it is not wired
  into authenticated daemon operation. Real-store locked/missing/denied states,
  authenticated HTTPS, concurrent retrieval, logout and actual connection lifetime
  need a running Secret Service fixture, reported missing by Claude. No singleton
  or per-retrieval closure claim is inherited. Image package/unit and sign-in are
  desktop dependencies scheduled in phases 7/4; native selection remains phase 5.

The first five implementation/audit reports list contributor workspace
fmt/clippy/tests, warnings-denied rustdoc, supervisor tests and pinned BPF gates;
those are retained reported evidence. The last two report package/link and bus
routing observations respectively; do not infer full gates from those reports.
ADR 0021 remains proposed. Claude retains credential/security ownership. Reports
arriving during publication will be reconciled next iteration.

## Remaining scope and proposed progress

This completes client-initiated interactive movement as a component.
Interactive resize is next, followed by remaining window operations and native
launcher/dock/controls. Configurable raw shortcuts stay phase 3 after underlying
operations. No full window-management, compositor or release checkbox is ticked.
Full direct DRM/seat entry, populated devices/hotplug, GPU/recovery evidence and
physical laptop/GPU workstation acceptance remain owed at their delivery phases;
physical acceptance follows the phase 7 integrated VM image. WSLg pixels and
socket tests are not hardware certification. No OS was installed physically.

All four progress documents carry the component and credential reconciliation.
No staging, commit, push, identity change, other checkout edit, tools/dev-loop
change, worker/loop launch or unrelated host change.
