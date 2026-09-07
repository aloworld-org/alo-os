# Native popup presentation

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`;
integration owner for the four shared progress documents.
Status: ready for integration, pending the supervisor's independent gates.

## Change and decisions

Nested applications can display and interact with opt-in non-grabbing XDG popups.
`crates/alo-shell/src/scene.rs` shares front-to-back ordering and buffer placement
between `nested.rs` rendering and `pointer.rs` hit testing. Newest popups precede
their own parent's tree, without crossing a foreground window or the cursor.
Placement adds the parent's committed window-geometry origin and subtracts the
popup's, with both geometries clamped to their buffer trees. Floating-point bounds
and offsets avoid overflow from full-range client geometry and child positions
before the existing renderer clips to the output. Reasons: XDG geometry excludes
shadows; input and graphics must use the same placement; pinned engines remain
unmodified under ADR 0002. No new dependency, scope, or owner decision is required.

`FrameTarget::submit_popups` is additive and defaults to explicit refusal of live
popups on older targets. No callback or membership update occurs on unsupported
targets or failed submission. `Popup::location` and the backend trait have rustdoc;
`COMPOSITOR.md` documents the internal contract. Popup focus remains valid through
its mapped parent; live unmap, dismissal and parent loss clear held buttons, while
disconnect clears state without redirecting releases to another client. No agent
verb, context reader, daemon IPC or application-adapter surface changes (ADR 0001
and `docs/contracts/app-adapters.md`).

Acceptance was recorded in QUEUE before implementation. Four real Unix-socket
tests in `tests/popups/presentation.rs` cover geometry commits and extreme values,
foreground isolation, empty input regions, sibling/subsurface stacking and offsets,
implicit drag routing, unmap/dismissal/parent loss/role destruction/disconnect,
and callback/output retention on failed or unsupported presentation.
`examples/support/popup_check.rs` exercises actual SHM buffers through GLES.

At iteration start, updates/README and the published report inventory were checked.
`descriptive-task-reporting.md` and `native-popup-protocol-lifetimes.md` were already
referenced in STATE; no published report awaited reconciliation. This report is
integrated in the same change. Filesystem security remains Claude's workstream.

## Checks actually run

Windows PowerShell, repository cwd `C:\dev\alo-os`; Ubuntu WSL2 commands inherit
`/mnt/c/dev/alo-os`. WSLg socket `/run/user/0/wayland-0` exists; pkg-config reports
Wayland server 1.24.0, EGL 1.5, xkbcommon 1.13.1. No packages needed installation;
no shared kernel, BPF, cgroup or service state was changed. The Linux build target
is exclusively `/root/alo-os-target` for this checkout.

```powershell
wsl -d Ubuntu -- sh -c 'test -S /run/user/0/wayland-0 && pkg-config --modversion wayland-server egl xkbcommon'
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle popups:: -- --nocapture
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
git diff --check
```

Final results: formatting and all affected-target clippy checks pass on Windows
and Linux; warnings-denied Linux rustdoc and example build pass. Linux shell:
40 tests pass (3 unit, 34 client, 3 socket), zero ignored; Windows intentionally
runs zero Linux tests. Focused popup wire run: 9 pass. These are component checks,
not a claim that the supervisor's full workspace/BPF gates have run.

Intermediate corrections: first new-test compile found a private fixture field
and unused import; added a reusable input-region helper. The first subsurface
test wrongly expected a release event after destroying its receiving surface;
changed the scenario to live synchronized unmap, asserting the synthetic release
and refused duplicate release before destruction. The corrected tests all pass.
A Linux fmt check overlapped the last combined-fixture edit and reported the
unformatted assertion; formatting and a subsequent Linux check passed. No lint
configuration, test gate, or assertion was disabled.

Additional local evidence:

- `.git/alo-popup-presentation-wire.log`: 9 real-client tests pass, protocol
  configure/ack/buffer events, local pointer coordinates, release/leave and
  deliberate invalid-configure refusals.
- `.git/alo-popup-presentation-graphics.log`: popup-only run exits 0, 34 submitted
  surfaces across frames; actual popup callback/output entry and dismissal leave;
  offscreen popup callback withheld.
- `.git/alo-popup-cursor-graphics.log`: final combined fixture exits 0, 56 submitted
  surfaces across frames; popup origin (8,11) from parent/popup geometry offsets,
  offscreen popup and i32::MAX child callbacks withheld; then cursor submission
  at scripted pointer (26,35), hotspot (2,3), output leave, unmap/remap, malformed
  client refusal and abrupt disconnect. End state has no roots or popup snapshots.
  The counts depend on scheduling and are observations, not fixed acceptance
  thresholds. Mesa emits the previously documented WSLg driver-probe diagnostics
  before successful EGL rendering. No pixel readback or physical observation is
  claimed by callback/wire checks.

## Remaining work and integrated progress

CHANGELOG describes popup interaction, ROADMAP keeps the compositor unchecked,
QUEUE records this completed component and selects nested popup parent chains
as the next useful component, and STATE references this report and its limits.
Popup opt-in remains explicit. Nested chains, explicit popup grabs, repositioning
and output constraints, parent-leave notification backend, direct display/input,
window management and production session entry remain unfinished. Delivery steps
3-8 and every other v0.01 requirement remain in scope.

The supervisor still owes the complete independent Windows/Linux test, lint,
rustdoc and BPF publication gates. Actual parent input/cursor observation and
certified laptop/GPU workstation display/input acceptance remain external machine
evidence; WSLg is not hardware certification. No staging, commit, push, dev-loop
edit, worker launch or other-checkout modification was performed. Reports arriving
during publication are reconciled next iteration; no release verification claim.
