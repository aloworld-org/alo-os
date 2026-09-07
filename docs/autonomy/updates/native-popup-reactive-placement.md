# Native popup reactive placement

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop worker / integration owner
- Status: ready for integration

## Change and decisions

Mapped menus whose applications opt into reactive positioning now receive updated
geometry when the output or committed parent geometry changes. Rendering and
pointer input continue using the old geometry until the application acknowledges
and commits the configure. Nested menus constrain against committed ancestors,
including their window geometry, rather than speculative parent configurations.

`crates/alo-shell/src/popup_reactive.rs` owns scheduling separately from protocol
lifetimes. Dispatch cleanup and pre-render cleanup refresh placement after pruning
lost parents, and clean up input grabs if unsafe placement dismisses a tree.
The existing bounded placement helper and pinned Smithay constraint algorithm
remain authoritative (ADR 0002); there is no engine patch or second algorithm.
Comparing with Smithay's latest pending server geometry avoids duplicate events
while a client has outstanding configures. Smithay retains serial validation.

Both committed and latest requested positioners must permit reactivity. Thus an
explicit request can withdraw permission immediately; newly granted permission
waits for its acknowledged commit, as required by Smithay's send_configure API.
Initial unmapped popups wait for their first configured buffer before automatic
updates. Empty targets preserve the last valid extent; failed submission can
announce a valid extent but never consumes frame callbacks. Unsafe arithmetic
dismisses the popup tree child-first and cannot revive a dismissed role.
No parent-size prediction is added: placement uses committed parent geometry.

Five new socket tests cover output opt-in isolation, unchanged geometry, multiple
outstanding configures and individual acknowledgement, commit ordering, failed
and empty targets, nested committed placement and pointer coordinates, reactive
permission changes, initial map/unmap, unsafe output, terminal child-first
dismissal, stale acknowledgement and independent-client survival. The WSLg
fixture additionally submits a reactive SHM popup at (304,13), changes its
parent's window origin, commits the reactive configure and submits at (304,15),
checking callbacks and output leave. Existing popup/cursor lifecycle checks stay.

Read constitution, delivery/ownership rules, updates README, current queue/state,
relevant feature/roadmap, ADRs 0001/0002 and app-adapter contract. Acceptance was
recorded in QUEUE before implementation. No new agent/context authority or scope.
All published reports present at iteration start were already referenced in
STATE; no report needed reconciliation. Source reports were not edited. Reports
arriving during publication will be reconciled next iteration.

## Executed checks

PowerShell in `C:\dev\alo-os`, final checks all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, separate target `/root/alo-os-target`, final checks all passed:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::reactive
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::reactive *> .git/alo-popup-reactive-wire.log
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor *> .git/alo-popup-reactive-graphics-final.log
```

Initial focused run passed four tests before adding the fifth stale-ack test;
final wire run passes all five. Final Linux shell suite passes 70 tests: 3 unit,
64 client lifecycle, 3 socket ownership, none ignored. Full shell output is in
`.git/alo-popup-reactive-tests.log`. Windows intentionally excludes Linux protocol
tests. Both platforms' affected all-target clippy and fmt pass; Linux rustdoc is
warnings-denied. Supervisor full workspace/BPF gates have not run for this change.

Development corrections: first clippy rejected indexing in the scheduler; checked
access replaced it. The next clippy rejected test unwraps; explicit presence
assertions and optional geometry comparisons replaced them. No lint was relaxed.
The first combined WSLg run completed reactive submission but failed the subsequent
cursor count assertion (8 observed versus the old expected 6); its fixture counts
now include the extra menu and two callbacks, including the final leave count.
The corrected combined run exits 0. The initial failure log is retained at
`.git/alo-popup-reactive-graphics.log`. No recurring failure or lowered gate.

Prerequisites verified in Ubuntu: WSLg socket present, pkg-config versions
wayland-server 1.24.0, EGL 1.5, xkbcommon 1.13.1, libinput 1.31.1 and libseat 0.9.2.
No package installation or shared kernel/cgroup/BPF/service mutation was needed.
An initial login-shell cargo lookup had no result; all build commands use the
explicit installed cargo PATH above. Target remains separate from Claude's tree.

## Remaining acceptance and progress integration

CHANGELOG, ROADMAP, QUEUE, STATE and COMPOSITOR record this completed component.
Compositor item 33 remains unchecked. Next: an unpatched parent-leave notification
backend, then direct display/input and production session integration. Pinned
Smithay's Winit wrapper still exposes neither cursor leave nor its event loop;
do not bypass it with an unapproved engine patch. All remaining v0.01 scope stays.
Actual parent-input behavior and physical laptop/GPU workstation records are owed.
Scripted WSLg is protocol/GLES submission evidence, not pixel readback, real input
or hardware certification. No release verification is claimed.

Source/tests/docs diff reviewed. No staging, commit, push, worker/loop launch,
delegation, dev-loop edit, other-repository change or unrelated host change.
