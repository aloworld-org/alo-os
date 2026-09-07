# Native popup chains

- Date: 2026-09-07
- Workstream: native desktop compositor
- Contributor: single desktop development worker / release-progress integration owner
- Status: ready for integration

## Change and decisions

Applications can open nested non-grabbing popups under mapped popup parents.
`crates/alo-shell/src/popups.rs` tracks the acyclic parent-before-child forest,
refuses unmapped/dismissed parents, and dismisses descendants before ancestors.
Unmapping, unsupported repositioning, role destruction and disconnect cannot
revive descendants or redirect their held pointer buttons to another client.
`surfaces.rs` prunes before accepting another role, including requests received
in the same dispatch batch after destruction.

`scene.rs` accumulates parent-relative XDG geometry and traverses iteratively,
placing descendants above ancestors and newest siblings first. Rendering and
pointer hits still consume the same scene. Iterative traversal avoids growing
the call stack with client-controlled popup depth; floating-point placement
continues to defer integer conversion until output clipping. Parents must already
be mapped, which prevents cycles without a new protocol or engine patch.
`Popup` and `FrameTarget` rustdoc describe the parent and placement contract.

This follows ADR 0002's pinned Rust/Smithay architecture and preserves ADR 0001's
agent boundary: no new agent/context API or application-adapter contract change.
No new release scope, dependency or owner decision was needed. Acceptance was
recorded in QUEUE before implementation. Grabs/repositioning still dismiss; this
is a complete chain component, not a completed compositor or desktop release.

## Executed verification

Windows PowerShell, repository root:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Ubuntu WSL2, repository root; this checkout's separate Linux target directory:

```powershell
wsl -d Ubuntu -- sh -c 'test -S /run/user/0/wayland-0 && pkg-config --modversion wayland-server egl xkbcommon'
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::chains
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-files --locked --test a_file_with_another_name_is_not_read
git diff --check
```

Final checks pass. Linux shell: 43 tests (3 unit, 37 real-client, 3 socket), no
ignored tests. Windows intentionally runs zero Linux protocol tests. Three new
chain tests cover three levels, accumulated geometry/pointer coordinates,
failed-submit callback/membership retention, successful submission, middle-parent
and ancestor cleanup, unmap/destruction/disconnect, ordered done events, held-button
release and isolation, unmapped/dismissed-parent refusal and terminal no-revival.
Six existing filesystem integration tests independently pass for reconciliation.

Intermediate compiler/lint checks found a missing test Proxy import and indexing/
unwrap use in new chain code/tests; corrected with iterators and slice matching.
No lint rules were changed. All executed test assertions passed. Prerequisites:
Wayland server 1.24.0, EGL 1.5, xkbcommon 1.13.1; WSLg socket present. No package
installation or shared kernel/cgroup/BPF changes were needed. WSL emitted existing
unknown-option diagnostics for autoMemoryReclaim/sparseVhd; host settings unchanged.

Additional integration evidence (local, not committed):
- `.git/alo-popup-chains-tests.log`: full shell tests pass.
- `.git/alo-popup-chains-wire.log`: all three chain tests pass over real Unix
  sockets, including per-role child-first popup_done and pointer cleanup.
- `.git/alo-popup-chains-graphics.log`: exit 0, 67 submitted client surfaces
  across frames; popup at (8,11), nested child at (16,23), both callbacks/output
  entries, descendant dismissal leaves, offscreen callback withholding, cursor
  callback/leave, malformed-client refusal and unmap/remap/disconnect regression.
  This is submission/protocol evidence, not pixel-readback or physical observation.

## Published report reconciliation

Read updates/README and compared published reports to STATE references at iteration
start. `hard-linked-files-are-not-read.md` was the only unreconciled report.
Reviewed its open-handle Unix link-count code and the real-kernel control/finding,
and reran the six alo-files integration tests above. Consolidated its user-readable
change and limits into CHANGELOG, ROADMAP, QUEUE and STATE without modifying
Claude's report or filesystem code. Its full Linux gates, kernel measurement and
mutation checks remain contributor-reported evidence, not gates rerun this turn.
Windows remains exposed; macOS and hardware untested; harmless same-folder aliases
also refuse. Existing kernel rename gap (6d) was already recorded and remains open.

## Remaining work and publication

Next useful compositor component: explicit popup grabs, validated seat/serial,
keyboard/pointer routing and outside-click dismissal. Repositioning/output
constraints, parent-leave backend support, direct display/input and production
session integration remain. All other v0.01 requirements remain in scope.
No physical laptop/GPU workstation acceptance, actual parent-input observation,
or release verification is claimed. WSLg cannot certify hardware.

The supervisor still owes its independent complete Windows/Linux/BPF test/lint/
rustdoc gates and concurrent-main integration before publication. No staging,
commit, push, dev-loop modification, worker launch, other checkout changes or
credentials were involved. Reports arriving during publication are reconciled
next iteration. Proposed shared-document updates are integrated in this change;
the compositor and hardware boxes remain unchecked.
