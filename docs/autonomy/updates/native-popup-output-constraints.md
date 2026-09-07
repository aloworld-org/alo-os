# Native popup output constraints

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop worker / integration owner
- Status: ready for integration

## Change and decisions

Menus in the development compositor can fit within the single output using the
client's flip, slide and resize permissions. `src/popup_placement.rs` in
`crates/alo-shell` owns arithmetic validation and output-to-parent conversion;
`popups.rs` retains protocol lifetimes. The committed scene supplies parent buffer
and window-geometry origins, including nested popups and shadows. Smithay 0.7.0
performs the protocol-ordered flip/slide/resize algorithm; no engine patch or
second positioning algorithm is introduced (ADR 0002).

`Server::render` retains the last positive target extent, matching the output
mode published even when submission fails. Empty targets preserve it. Initial
and explicit requests use that extent. No output means unconstrained placement
for protocol fixtures; no flags means the application did not authorize an
adjustment. Impossible fits can remain clipped, rather than silently granting
adjustments the client did not request. Automatic reactive placement is still
unfinished. Existing popups do not move merely because the output changes.

Existing one-million positioner operand bounds remain. Translated origins and
output extents are limited to sixteen million before narrowing f64 to i32,
leaving headroom for upstream intermediate arithmetic. Extreme coordinate trees
refuse before conversion. Unsafe initial requests receive popup_done; unsafe
reposition requests dismiss their descendant tree. Smithay still owns configure
serials, and only acknowledged commits change rendering/input geometry.

Four real Unix-socket tests cover permitted/absent flags, flip precedence and
failed flips, sliding negative positions, shrinking oversized geometry, nested
parent/window offsets, pointer coordinates, acknowledged commits, failed-submit
callback retention, empty output, explicit outputless fixtures, extreme target
and parent origins, unsafe repositioning and unaffected client connections.
The WSLg fixture now submits constrained buffers at (304,13), then (0,0), checks
both callbacks/output enter and unmap leave, and continues cursor/lifecycle checks.

Read constitution, delivery/ownership rules, reports README, current queue/state,
shell feature/roadmap, ADRs 0001/0002 and app-adapter contract. Acceptance was
recorded in QUEUE before code. Public rustdoc updated; no agent/context surface,
new scope, dependencies, upstream patch, permission request or ADR exception.

## Published report reconciliation

Reconciled `kernel-enforcement-plan-and-supervisor.md`, the only published report
not referenced at iteration start. Reviewed its audit, release-tier references,
ADR 0015's recording passage and standalone supervisor entry/repository code.
The contributor reports Ubuntu WSL2 kernel 6.18.33.2, Rust 1.98.0, tool fmt,
all-target clippy and release build passing, plus manual no-handoff, invalid task,
audit-heading, incomplete-handoff, lock and stop/status checks. These were not
rerun here. No workspace gates or Windows checks ran for that contributor task.

The plan distinguishes outstanding v0.01 network-boundary enforcement/attribution
from broader v0.5 kernel boundaries/records and early filesystem-hook work.
It changes no enforcement behavior or release boxes. The reported tension between
kernel records and the current two-map test remains a workstream design concern;
ADR 0015 itself says only turns are recorded. This integration neither resolves
that concern nor changes the ADR. Genuine mid-attach refusal remains unforced,
and all physical measurements remain owed. No source report/tool was modified
or launched. Kernel work remains Claude's assignment.

## Executed checks

PowerShell in `C:\dev\alo-os`, all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2 commands, separate target `/root/alo-os-target`, all passed:

```powershell
wsl -d Ubuntu -- pkg-config --modversion wayland-server egl xkbcommon
wsl -d Ubuntu -- test -S /mnt/wslg/runtime-dir/wayland-0
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::constraints
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::constraints *> .git/alo-popup-constraints-wire.log
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor *> .git/alo-popup-constraints-graphics.log
```

Linux: 65 shell tests pass (3 unit, 59 client lifecycle, 3 socket ownership),
none ignored; focused and wire runs each pass four constraint tests. Full shell
output is retained in `.git/alo-popup-constraints-tests.log`. Windows intentionally
excludes Linux protocol tests. Both platforms' affected all-target clippy and fmt
pass; Linux warnings-denied rustdoc and example build pass. No test assertion or
lint failures occurred, and no gate/lint was weakened.

WSLg run exits 0 and submits 92 client surfaces, including constrained popup
buffers, existing popup/child repositioning and cursor regression. Unmap/remap,
isolated refusal and disconnect checks complete. Native prerequisites: Wayland
server 1.24.0, EGL 1.5, xkbcommon 1.13.1 and WSLg socket present. No installation
or shared kernel/cgroup/BPF/service changes. Linux target remains separate from
the other checkout. Wire/submission evidence is not pixel readback or real input.

## Remaining acceptance

Next useful component: automatic reactive popup placement on output/parent
changes, with configure/commit and descendant coordination. Parent-leave backend,
direct display/input, session integration and all other unfinished v0.01 items
remain. Actual parent input and physical laptop/GPU workstation records are owed.
No compositor or release checkbox is ticked. Supervisor full independent
Windows/Linux/BPF publication gates have not run for this change.

CHANGELOG, ROADMAP, QUEUE, STATE and COMPOSITOR record this component and limits.
Source/tests/docs diff reviewed. No staging, commit, push, delegation, loop launch,
dev-loop edit, other-repository changes or unrelated host modifications. Reports
arriving during publication are reconciled next iteration.
