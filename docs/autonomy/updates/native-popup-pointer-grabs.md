# Native popup pointer grabs

- Date: 2026-09-07
- Workstream: native desktop compositor
- Contributor: single desktop development worker / release-progress integration owner
- Status: ready for integration

## Change and decisions

Pointer-opened menus can retain keyboard focus and dismiss on outside clicks.
`crates/alo-shell/src/popup_grabs.rs` owns seat-scoped input policy separately
from `popups.rs`'s protocol/buffer lifetimes. A root grab requires the display's
seat and Smithay's active pointer press serial on the parent surface tree. A
submenu requires the current topmost grabbing parent and may inherit the chain's
original serial, or use a new active press on that parent. A dismissed chain's
serial cannot authorize another root grab. Requests from another client, stale
serials, unsupported initiation and non-topmost siblings are dismissed. Grabs
after mapping, non-grabbing popup parents and out-of-order destruction of active
grabbing ancestors produce isolated protocol errors.

`keyboard.rs` routes focus to the topmost mapped grabbing popup, restores the
surviving parent, and releases held keys before changing recipients. Trusted
backend selection of the original root preserves popup focus; selecting another
root or clearing focus dismisses the chain. `pointer.rs` restricts hits to the
owning client, matching XDG owner-events behavior: the client's parent surfaces
remain reachable. Outside presses dismiss child-first and consume both the press
and its later release. Presses re-hit the current scene even without motion or
while another button's implicit grab retains its previous target. Duplicate
transitions cannot dismiss a menu. Unmap, destruction, disconnect and backend
leave cancel input without redirecting releases to another application.

Design reasons: preserve the existing explicit parent/lifetime forest and input
cancellation paths instead of adding a second popup manager with independently
tracked mapping state. Smithay remains pinned and unpatched (ADR 0002). The
initial implicit drag is ended before popup ownership changes, preventing a new
menu from inheriting the opening button's release. Pending unbuffered children
retain grab ownership but receive no keys until mapped. Input remains trusted
backend plumbing; no agent verb, context reader, grant or adapter API is added
(ADR 0001 and the application-adapter contract). No new scope or ADR exception.

Acceptance was recorded in QUEUE before implementation. This completes the
pointer-triggered component, not all popup initiation or the compositor feature.
Keyboard and release-triggered root initiation remain refused and require a
follow-up policy/serial component. Touch is not advertised by this seat.

## Executed verification

Windows PowerShell, repository root:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Ubuntu WSL2, repository root, this checkout's separate Linux target:

```powershell
wsl -d Ubuntu -- test -S /run/user/0/wayland-0
wsl -d Ubuntu -- pkg-config --modversion wayland-server egl xkbcommon
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo check -p alo-shell --all-targets --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::grabs
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::grabs
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --grabs
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
git diff --check
```

Final checks pass. Linux shell: 49 tests (3 unit, 43 real-client, 3 socket), none
ignored. Six new grab tests exercise real request/event serials, focus surface
IDs, input delivery/isolation, terminal dismissal and cleanup. Windows intentionally
runs zero Linux protocol tests. The WSLg socket exists; native package versions
are Wayland server 1.24.0, EGL 1.5 and xkbcommon 1.13.1. No dependency installation
or shared kernel/cgroup/BPF/service change was needed.

Intermediate clippy runs identified test unwraps, a collapsible condition and
duplicated example-fixture dead code/missing documentation. Fixed the assertions
and condition and placed graphics evidence in the existing example's separate
support module; no lint allowance or gate change. An intermediate examples build
succeeded with those dead-code warnings before restructuring; the final focused
clippy and example build are clean. All executed test assertions passed. An
initial WSL source-search quoting error was corrected; an accidental overly broad
source search was stopped. No repository or host configuration was changed for it.

Additional local evidence (not committed):

- `.git/alo-popup-grabs-tests.log`: all 49 Linux tests pass.
- `.git/alo-popup-grabs-wire.log`: six grab tests pass over real Unix sockets,
  including protocol errors confined to the offending clients.
- `.git/alo-popup-grabs-graphics.log`: `--grabs` exit 0; real grabbing SHM popup
  receives a GLES submission callback and output enter, keyboard focus and a key
  press/release, then outside-click dismissal/output leave and parent-focus return.
  Disconnect leaves no popup or toplevel. Input is explicitly scripted backend
  input; no parent keyboard/mouse observation or pixel readback is claimed.
- `.git/alo-popup-grabs-regression.log`: `--popups --cursor` exit 0, 67 submitted
  client surfaces across frames, nested popup/cursor callbacks, offscreen
  withholding, output leaves, malformed-client refusal and disconnect cleanup.

Mesa emitted the existing driver-probe/EGL/Zink diagnostics recorded in the
graphics baseline, then successfully submitted these frames. This is development
graphics evidence, not certified hardware acceptance.

## Reconciliation, remaining work and publication

Read updates/README and checked the five published task reports at iteration
start against STATE. All were already referenced; no unreconciled report required
integration. Filesystem security remains Claude's workstream; its source/report
and recorded evidence limits are unchanged. This report is integrated into the
four shared progress documents in this change.

Next useful component: keyboard-triggered popup grabs with delivered-key serial
validation and release-triggered opening policy. Repositioning/output constraints,
parent-leave notifications, direct display/input and production session integration
remain, along with all other unfinished v0.01 requirements. No physical laptop or
GPU workstation acceptance, actual parent-input observation or release verification
is claimed. WSLg cannot certify hardware.

The supervisor still owes independent complete Windows/Linux/BPF publication
gates and concurrent-main integration/retests. No staging, commit, push, new
worker, dev-loop edit or other-checkout changes. Reports arriving during
publication are reconciled next iteration. No compositor or hardware box moves.
