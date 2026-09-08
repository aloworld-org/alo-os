# Client window maximize and restore requests

Date: 2026-09-08. Workstream: native desktop/window operations, delivery phases
2/3. Responsible contributor: single desktop worker in `C:\dev\alo-os`, shared
progress integration owner.

Status: component verified after the owner-authorized storage pause; ready for
integration and publication. No window-management release checkbox is completed.

## Change and decisions

Applications can request maximization and restoration through their XDG toplevel
instead of needing a trusted Rust caller. `src/surfaces.rs` in `crates/alo-shell`
dispatches these requests to `window_maximize.rs`, retaining its mapped-root,
output, geometry and busy-operation validation. Each post-handshake request gets
a fresh configure, including unchanged requests and refusals. Refusal preserves
the latest pending mode, size and unrelated flags; only acknowledged commits can
place the actual buffer. Normal geometry survives repeated requests.

Initial configuration and remapping advertise only the implemented Maximize WM
capability. The pinned Smithay default advertised window menu, maximize,
fullscreen and minimize; explicitly replacing that set prevents clients from
offering unsupported controls. No upstream engine was patched.

Pre-map requests are deliberately declined: there is no committed normal
geometry to remember. Before the first empty commit they are answered together
by the normal initial configure, never by a premature configure. After initial
configure but before mapping they receive a normal-state refusal configure.
Intent is not saved for automatic execution later. A mapped client may ask again.
The pinned XDG XML explicitly permits compositor policy to decline both modes.
This avoids invented restore sizes while preserving the established handshake.

Accepted ADR 0002 keeps the implementation Rust/Smithay and ADR 0010 remains
unchanged. No new rendered strings, palette, agent verbs or adapter endpoints.
`docs/contracts/native-window-maximize.md` documents policy and evidence limits;
trusted API rustdoc now describes the shared transaction boundary.

## Acceptance and verification

Four new real-client tests in `tests/window_maximize/client_requests.rs` cover
repeated wire responses, request/ack/commit ordering, saved geometry, activation,
ordinary key delivery and foreign-client isolation, initial/pre-map/remap states,
capability advertisement, disconnect with a pending transaction, unavailable
output, busy resize refusal preserving resizing state and impossible restore
limits preserving maximize state until corrected.

The GLES fixture in `examples/support/offscreen_client.rs` now sends repeated
client maximize/unmaximize requests. `window_maximize_check.rs` verifies all
6,400 pixels per frame at five request, acknowledgement and committed-buffer
boundaries. This extends the existing transaction integration through the wire.

Ubuntu WSL2 prerequisites checked: Rust 1.98.0, WSLg socket and pkg-config versions
wayland-client/server 1.24.0, EGL 1.5, GBM 26.0.8-1ubuntu0.3, libudev 259,
libinput 1.31.1, libseat 0.9.2 and xkbcommon 1.13.1. An initial combined shell
search failed quoting before execution; a corrected prerequisite command passed.
Ubuntu lacks rg, so the registry-only fallback used grep; repository searches
used host rg. No package, system service or shared kernel/BPF state changed.

Initial `cargo test -p alo-shell --locked client_maximize` passed four tests.
After adding capability assertions, affected clippy passed, but the full test
run found one failure: Smithay's all-capabilities default survived additive set.
The implementation now replaces the set; the exact capability assertion remains.
The other 136 lifecycle tests passed in that run. No test was weakened or skipped.

After cleanup, the owner authorized finishing this preserved task and restarting
the desktop loop. The corrected capability replacement passed the four focused
tests and the complete Linux suite: 141 shell unit tests, 137 lifecycle tests and
three socket tests, plus three doctests. Full Linux workspace fmt, all-target
warnings-denied clippy, tests and warnings-denied rustdoc passed. Existing ignored
workspace cases retain their prior status and are not passing evidence.

Rebuilt the actual `nested_check` example and ran both `--offscreen` and
`--popups --cursor` with 30-second deadlines under WSLg. Both returned exit 0;
maximize/restore stages 17–21 each passed all 6,400 pixels. The nested regression
submitted 115 surfaces. Pinned BPF fmt and release clippy passed. No production
code was changed during recovery to accommodate a failed expectation.

Commands and output are retained locally in `.git/storage-resume-linux.log`;
Windows workspace gates are in `.git/storage-resume-windows.log`: fmt check,
all-target clippy with warnings denied and full workspace tests all returned
exit 0. This is independently executed Windows evidence, not inferred from Linux.

## Reconciliation and remaining work

Initial tree clean. Read constitution, delivery/shared-main/report rules, current
queue and STATE tail, relevant feature/roadmap sections, accepted ADRs 0002/0010
and native maximize/application-adapter contracts. Compared every published
report filename with STATE: no outstanding reports at iteration start. Reports
arriving during publication reconcile next iteration. Claude's filesystem and
credential assignments remain untouched.

Proposed progress updates: record client maximize/restore protocol policy as a
completed component once final checks pass. Next: trusted minimization and
restoration, with scene visibility, focus/held-input retirement, cycling,
unmap/disconnect and pixel evidence; then client minimize policy, tile and native
controls. Maximize controls, dock work areas, launcher, clipboard and remaining
desktop integration are still open. This is not completion of window management.
Configurable shortcuts remain in phase 3 after underlying operations.

WSLg proves protocol/GLES behavior only. Physical DRM/seat entry, populated input
and hotplug, GPU/recovery and certified laptop/workstation records remain owed in
their scheduled phases. Physical acceptance follows phase 7 VM image integration;
no hardware or release certification. Independent supervisor Windows/Linux
workspace/rustdoc/BPF gates and concurrent-main integration remain pending.
No staging, commit, push, tools/dev-loop edits, worker/loop launch, other checkout,
credential/identity access or unrelated host changes. Supervisor owns publication.

The paragraph above records the interrupted worker's handoff. During the
owner-authorized recovery, the interactive integration owner reviewed the saved
diff, ran all publication gates, completed progress documentation, and prepared
normal publication before restarting the clean-tree supervisor. No source was
discarded and no failed gate was bypassed. Main remained at the task's original
base during verification; a final fetch/diff check precedes publication.
