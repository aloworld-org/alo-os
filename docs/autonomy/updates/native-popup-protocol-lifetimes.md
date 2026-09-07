# Native popup protocol lifetimes

- Date: 2026-09-07
- Workstream: native desktop compositor
- Contributor: single desktop development worker, integration owner
- Status: ready for integration (protocol component only)

## Change and decisions

`crates/alo-shell/src/popups.rs` owns initial XDG popup configuration, buffered
surface snapshots and terminal dismissal. `surfaces.rs` forwards protocol commits
and prunes popups when their mapped toplevel parent disappears. Public methods
and the `Popup` snapshot have rustdoc. The current renderer does not consume
popup snapshots, so protocol acceptance requires explicit backend opt-in;
ordinary nested sessions retain their previous dismissal behavior. No application
adapter, agent verb, context reader or daemon contract changes (ADR 0001 and
docs/contracts/app-adapters.md). This is a complete protocol lifetime component,
not completed popup support or a completed compositor feature.

ADR 0002 remains in force: Rust and unpatched Smithay 0.7. Initial placement uses
its positioner implementation, with this backend accepting operands only within
plus/minus 1,000,000 to bound its geometry calculations. Grabs, popup parents,
repositioning and output-constrained placement remain future work; unsupported
grab/reposition requests dismiss. Unmap is terminal for this popup role; a new
role is needed, and a retained acknowledgement never revives it.

The initially investigated parent-leave component cannot use the pinned Winit
wrapper: its source explicitly drops CursorLeft and exposes no raw event-loop
hook. It remains a backend replacement/extension task, without an engine patch
or unsafe-code exception. This iteration selected the next independent protocol
component within the same first unfinished delivery task and recorded acceptance
criteria in QUEUE before implementation. No release scope was added.

## Verification actually executed

Windows PowerShell, repository root:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Ubuntu WSL2: checked `/run/user/0/wayland-0` with `test -S`; `printenv` returned
XDG_RUNTIME_DIR=/run/user/0 and WAYLAND_DISPLAY=wayland-0. `pkg-config --modversion
xkbcommon wayland-server egl` returned 1.13.1, 1.24.0, 1.5. No packages installed,
shared kernel state changed, or other checkout accessed. This checkout alone uses
/root/alo-os-target. WSL prints existing unknown .wslconfig key diagnostics;
the host configuration was not modified.

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle popups:: -- --nocapture
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --cursor
git diff --check
```

Final commands passed. Linux: 36 tests (3 unit, 30 client, 3 socket), zero ignored.
Windows intentionally runs zero Linux tests and is not protocol evidence.
Five new real Unix-socket tests cover initial geometry (7,10,16,16), no configure
before empty commit, missing/premature/stale acknowledgement refusal, isolated
client errors, disabled backend, absent/unmapped parents, extreme offsets,
buffer unmap and attempted revival, parent/role destruction, disconnect,
idempotent reposition dismissal and successful creation of a fresh role.
No executed assertion failed. An intermediate all-target compile caught the
test module's accidental standalone-test location and unused example helpers;
the module now follows the existing tests/<subject>/mod.rs layout, and the
example exercises its helpers. Clippy caught duplicate branches, which were
combined without suppressions or gate changes.

Additional integration evidence (local logs, not committed):

- `.git/alo-popup-lifecycle-wire.log`: five tests pass; real configure serials,
  SHM requests, dismissal and protocol errors recorded, exit 0.
- `.git/alo-popup-lifecycle-graphics.log`: WSLg exit 0; root/child callbacks
  [305,305], 22 submitted client surfaces across frames, popup handshake and
  dismissal coexist with GLES, unrendered popup callback withheld, then normal
  unmap/remap/refusal/disconnect and empty popup snapshots at teardown.
- `.git/alo-popup-cursor-regression.log`: WSLg exit 0; real client cursor buffer
  callback and output leave, 28 submitted surfaces, lifecycle checks pass.

## Integration and remaining acceptance

At iteration start, README and the sole published task report were read.
`descriptive-task-reporting.md` is already referenced in STATE; no published
report needed reconciliation. This report is referenced in the new STATE entry;
reports arriving during publication belong to the next iteration.

CHANGELOG, ROADMAP and QUEUE now describe this protocol component and preserve
the unchecked compositor feature. Next: popup-aware rendering and hit testing
using these snapshots before session opt-in; nested popup chains, valid grabs,
repositioning/output constraints, parent-leave backend support and direct display
remain unfinished. No hardware is certified: actual parent input observation,
direct display/input and the laptop/GPU workstation physical acceptance records
are still owed. Delivery steps 3-8 and all remaining v0.01 scope remain.

The supervisor's full Windows/Linux tests/lints/rustdoc and pinned BPF gates
have not been run by this worker for this change. No staging, commit, push,
dev-loop edits, gate weakening, sub-agents or other repository changes occurred.
