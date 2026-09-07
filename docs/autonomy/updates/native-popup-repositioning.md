# Native popup repositioning

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop worker / integration owner
- Status: ready for integration

## Change and decisions

Applications can reposition a popup without destroying its role. In
`crates/alo-shell/src/popups.rs`, the same arithmetic bounds protect initial and
explicit positioners before Smithay computes geometry. `surfaces.rs` prunes
parent lifetimes before accepting a request. Smithay sends reposition token,
popup geometry and surface configure in order, and owns configure serial
validation. On a configured buffer commit, its committed geometry updates the
snapshot consumed by the existing shared render/input scene. No duplicate serial
history or engine patch is introduced (ADR 0002). Parent links, grab authority
and descendant stacking remain unchanged. Unsafe placement dismisses child-first;
dismissed roles cannot reposition or revive. Rustdoc describes the new behavior.

Three new socket tests in `tests/popups/reposition.rs` verify exact event order,
two outstanding configure transactions, no movement on request/ack alone,
intermediate and final acknowledged commits, stale-ack isolation, descendant
pointer coordinates, failed-submit callback retention and terminal dismissal.
The existing dismissal tests now explicitly request an arithmetic-unsafe position:
valid repositioning is implemented, so unsupported-operation dismissal is obsolete.
Their lifetime/isolation assertions remain. The WSLg popup fixture repositions
before mapping, then submits that geometry and its nested child through GLES.
This covers both initial-map and mapped-popup reposition paths across fixtures.

Acceptance was written into QUEUE before code. Read constitution, delivery order,
shared-main ownership, updates README, current queue/state, shell feature/roadmap,
ADRs 0001/0002 and app-adapter contract. No public agent/context contract changed,
new release scope, dependency, upstream patch, approval or ADR exception.

## Published report reconciliation

One report at iteration start was not referenced in STATE:
`kernel-protection-for-deletion-and-links.md`, owned by Claude. Reviewed its
hook/loader sources and retained evidence in all four progress documents. It
reports seven loaded-kernel tests, per-hook leftover-pin refusal, teardown,
open/rename/agentd regressions and full Linux/BPF gates passing. Four hooks and
six pins are present. These are contributor-reported measurements, not this
worker's reruns. Genuine mid-attach failure was not induced. Unwatched symlink,
directory, creation and attribute operations, already-open handles, non-filesystem
operations and pre-existing hard links remain explicitly limited; physical
acceptance is outstanding. Filesystem implementation remains Claude's assignment.
Reports arriving during publication are reconciled next iteration.

## Executed checks

PowerShell in `C:\dev\alo-os`, final checks passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, separate target `/root/alo-os-target`:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin bash -c 'pkg-config --modversion wayland-server egl xkbcommon; test -S /mnt/wslg/runtime-dir/wayland-0'
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::reposition
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- bash -lc 'PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::reposition > .git/alo-popup-reposition-wire.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor > .git/alo-popup-reposition-graphics.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups > .git/alo-popup-reposition-submission.log 2>&1'
```

Linux: 61 tests pass (3 unit, 55 real-client lifecycle, 3 socket ownership), none
ignored. Focused and wire runs each pass three tests. Windows excludes Linux
protocol tests. Final all-target clippy and fmt pass on both platforms; Linux
warnings-denied rustdoc and example build pass. Initial clippy found prohibited
indexing/unwrap in the new tests; an intermediate edit had a numeric-token typo,
then clippy rejected explicit panic. Corrected to optional geometry assertions
and asserted-present serials without changing lint configuration or test intent.
No test assertion failed; the first focused and full Linux runs passed.

Prerequisites: WSLg socket present, Wayland server 1.24.0, EGL 1.5, xkbcommon
1.13.1. No package installation or shared kernel/cgroup/BPF/service mutation.
WSLg logs confirm token 93, configure (34,-6,16,16), acknowledged popup submission
at buffer origin (35,-5), child at (43,7), callbacks/output enter, unsafe-request
child-first dismissal/output leave and client teardown. Combined cursor run
submits 84 client surfaces and completes cursor/unmap/remap/refusal/disconnect.
The standalone popup run exits 0. Logs remain local in `.git`.

## Remaining acceptance and integration

Output constraints and automatic reactive placement are the next useful
component. Parent-leave notifications, direct display/input, usable session and
all other unfinished v0.01 requirements remain. WSLg checks are scripted protocol
and GLES submission, not pixel readback or actual parent/physical input. Certified
laptop/GPU workstation records and actual parent-input observation remain owed.
No feature/compositor/release box is ticked. Full independent Windows/Linux/BPF
publication gates have not run for this change; the supervisor owns them.
CHANGELOG, ROADMAP, QUEUE, STATE and COMPOSITOR consolidate this component.
No staging, commit/push, worker/loop launch, dev-loop edit, other-checkout change,
or unrelated host modification. Source, tests and documentation diff reviewed.
