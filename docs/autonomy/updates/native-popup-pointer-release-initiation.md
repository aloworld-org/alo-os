# Native popup pointer-release initiation

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop worker / integration owner
- Status: ready for integration

## Change, decisions and acceptance

Applications can open a grabbing menu on a matched mouse-button release in the
development compositor. `crates/alo-shell/src/pointer.rs` retains one actual
release serial and recipient, invalidated by newer accepted button events,
pointer focus/lifetime loss and grab consumption. Synthetic releases never create
authority. `popup_grabs.rs` validates the recipient's parent tree and existing
seat/parent ownership. Subsurfaces can initiate parent menus and active submenus
inherit their chain serial. Single stored authority bounds memory without a
timeout or retaining historical serials. Existing pointer-press and keyboard
policies remain intact; no agent/context API or public contract changed.

The first focused run found a real edge case: Smithay retains focus after ending
an implicit drag until the next motion. The final button release now re-hits the
scene before retaining authority, refusing a release outside its recipient even
if the pointer later returns. Another still-held button continues to own its
implicit drag. This respects ADR 0002 without an upstream patch or unsafe code.
ADRs 0001/0002, the v0.01 compositor feature and adapter boundary were read; no
new scope, dependency, exception or owner decision was needed.

Three tests in `tests/popups/pointer_release.rs` use real Unix-socket serials:
subsurface initiation, mapped submenu inheritance, consumed and pending-grab
replay; nine refusal cases cover newer presses/releases, foreign and fabricated
serials, motion out/back, unmap/remap, synthetic release, final drag release
outside and child destruction. Invalid/unmatched events do not invent authority;
motion within the same recipient preserves it. Independent clients remain mapped.
Existing tests cover pointer disconnect, input isolation and presentation failure.
The nested example adds `--pointer-release-grabs` for real SHM/GLES submission.

Acceptance was recorded in QUEUE before implementation. Filesystem security
remains Claude's workstream. All nine published task reports at iteration start
were already referenced in STATE; no unreconciled report required integration.
Reports arriving during publication are reconciled next iteration.

## Executed verification

PowerShell at `C:\dev\alo-os` (final checks passed):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, with this checkout's separate target directory:

```powershell
wsl -d Ubuntu -- bash -lc 'test -S /mnt/wslg/runtime-dir/wayland-0 && pkg-config --modversion wayland-client egl xkbcommon'
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::pointer_release
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- bash -lc 'PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::pointer_release > .git/alo-pointer-release-wire.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --pointer-release-grabs > .git/alo-pointer-release-graphics.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --grabs > .git/alo-pointer-release-press-regression.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --keyboard-release-grabs > .git/alo-pointer-release-keyboard-regression.log 2>&1'
```

Linux: 58 tests pass (3 unit, 52 client lifecycle, 3 socket ownership), none
ignored; focused and wire runs each pass three tests. Windows intentionally
excludes Linux protocol tests. Final fmt, affected-target clippy and warnings-
denied Linux rustdoc pass. Initial focused run: two passed, one failed on the
final drag-release refusal scenario. Fixed the implementation as described above;
the unchanged assertion and all subsequent tests pass. No lint or gate weakened.

WSLg socket and native packages verified: Wayland client 1.24.0, EGL 1.5,
xkbcommon 1.13.1. No installation or shared kernel/cgroup/BPF/service changes.
All three graphics fixtures complete successfully. The pointer-release wire log
records press serial 3, release serial 4 and `xdg_popup.grab` serial 4, followed
by GLES callback/output enter, keyboard routing, consumed outside click,
popup_done/output leave, parent focus return and disconnect cleanup. Logs remain
local under `.git`. These are scripted backend input and submission/protocol
checks, not pixel readback or observations of actual parent/physical input.

## Integration and remaining work

CHANGELOG, ROADMAP, QUEUE and STATE consolidate this complete component;
COMPOSITOR documents its policy and quirks records the Smithay drag behavior.
The final rustdoc rerun also passed after the public input documentation change;
WSL emitted existing unknown-key warnings for `wsl2.autoMemoryReclaim` and
`wsl2.sparseVhd`. No host configuration was read or changed to address them.
Source and documentation diff reviewed, including
new files; whitespace check passes. No staging, commit, push, worker/loop launch,
dev-loop edit, shared-kernel mutation or other-checkout/repository changes.

Next useful component: XDG popup repositioning, with configure/ack ordering and
shared input/render placement. Output constraints, parent-leave backend, direct
display/input, production session and all other unfinished v0.01 requirements
remain. Actual parent input and certified laptop/GPU workstation physical records
are owed. Supervisor full independent Windows/Linux/BPF publication gates have
not run for this change. No compositor or release certification is claimed.
