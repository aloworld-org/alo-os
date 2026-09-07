# Native popup keyboard grabs

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop development worker / integration owner
- Status: ready for integration

## Change and decisions

Keyboard-opened menus now obtain explicit popup grabs without a pointer device.
`crates/alo-shell/src/keyboard.rs` retains only the latest accepted real key
press's serial and focused surface. A later accepted key event, focus change or
successful grab invalidates it. Synthetic releases never create authority.
`popup_grabs.rs` accepts this serial on the exact parent, keeping the existing
seat check, ordered submenu ownership, dismissal and input cleanup. The root
serial is consumed even before a popup maps, so destroying a pending grab cannot
reopen it. Active submenus may inherit the chain's original serial.

This intentionally bounded policy avoids an arbitrary timeout or accumulating
historical input serials. Duplicate/unmatched input does not invent events.
Release-triggered initiation remains refused and needs its own serial-lifetime
policy. This completes keyboard press initiation, not all popup interaction or
the compositor feature. ADRs 0001/0002 and the adapter boundary are unchanged:
trusted backend input only, no agent verb/context reader, engine patch, dependency
or new release scope. Acceptance was recorded in QUEUE before implementation.

Three tests in `tests/popups/keyboard_grabs.rs` use actual socket-delivered key
serials and a keyboard-only seat: map/focus, inherited submenu grab, key cleanup,
release/supersession/foreign/stale/focus-loss/unmap-remap refusals, duplicate input
and pending-grab destruction/replay. The shared keyboard fixture records serials.
The existing graphics fixture accepts `--keyboard-grabs`, submits a real SHM popup
through GLES and checks output enter/callback, keys, consumed outside clicks,
popup_done/output leave, restored parent focus and disconnect cleanup.

## Executed verification

Windows PowerShell, repository root:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, repository root, separate Linux target for this checkout:

```powershell
wsl -d Ubuntu -- test -S /run/user/0/wayland-0
wsl -d Ubuntu -- pkg-config --modversion wayland-server egl xkbcommon
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::keyboard_grabs
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::keyboard_grabs
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --keyboard-grabs
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --grabs
```

All checks pass: Linux shell 52 tests (3 unit, 46 socket-client, 3 socket ownership),
none ignored; Windows intentionally excludes Linux protocol tests. Both WSLg
fixtures exit 0. Native versions: Wayland server 1.24.0, EGL 1.5, xkbcommon 1.13.1;
WSLg socket exists. No dependency install or shared kernel/service change needed.
All executed test assertions passed; no lint allowances or gate changes added.

Local evidence, not committed: `.git/alo-keyboard-popup-wire.log` (three tests),
`.git/alo-keyboard-popup-graphics.log` (keyboard initiation), and
`.git/alo-keyboard-popup-pointer-regression.log` (pointer initiation regression).
Graphics checks measure submission/protocol behavior with scripted backend input,
not pixel readback or physical keyboard/mouse input. Existing Mesa initialization
diagnostics do not prevent submission. WSLg does not certify hardware.

## Reconciliation and remaining work

The sole unreferenced published report at iteration start was Claude's
`kernel-enforcement-for-file-renames.md`. Reviewed its implementation in
`alo-bounding-kernel/src/deciding.rs`, the two-hook loader, and the real-kernel test
cases. Consolidated into CHANGELOG, ROADMAP, QUEUE and STATE without modifying
the report or its code. Six kernel tests, read regression, approved agentd move,
full Linux gates and pinned BPF checks remain contributor-reported evidence,
not independently rerun here; no shared-kernel tests were launched this iteration.
Retained unhooked mutation operations, conservative exchange handling, hard-link
identity limits, non-Linux exclusions and physical acceptance. Filesystem security
remains Claude's workstream; its next assignment is the owner's decision.

Next desktop component: release-triggered popup initiation and bounded serial
lifetime with real-client happy/refusal tests. Repositioning/output constraints,
parent-leave notifications, direct display/input, production session integration
and all remaining v0.01 requirements remain unfinished. Physical laptop and GPU
workstation acceptance, actual parent input and supervisor full independent
Windows/Linux/BPF gates remain owed. No release/capability box is certified.
No staging, commit, push, worker/loop launch, dev-loop edit or other-repository
change. Reports arriving during publication are reconciled next iteration.
