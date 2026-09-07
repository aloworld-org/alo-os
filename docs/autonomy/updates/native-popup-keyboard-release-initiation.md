# Native popup keyboard-release initiation

- Date: 2026-09-07
- Workstream: native desktop compositor and release-progress integration
- Contributor: single desktop worker / integration owner
- Status: ready for integration

## Change, decisions and acceptance

Applications can open a grabbing menu on key release in the development
compositor. `crates/alo-shell/src/keyboard.rs` retains the latest delivered real
key event, replacing the previous held-press-only policy. Only matched releases
are forwarded. New accepted key events, focus/lifetime loss and successful grab
consumption invalidate authority; synthetic cleanup never creates it.
`popup_grabs.rs` preserves exact parent/seat validation, single-use root serials
and active submenu inheritance. One stored event bounds memory and authority
without introducing a timeout or retaining historical serials. Duplicate and
invalid input cannot invent events. Pointer-release initiation remains separate.

Three new real socket tests in `tests/popups/keyboard_grabs.rs` exercise release
initiation, submenu focus/cleanup, invalid/unmatched events, pending-grab replay,
new press/release supersession, foreign/stale serials, focus loss, unmap/remap and
actual synthetic-release serial refusal. Independent clients remain mapped.
The existing press-path refusal tests remain intact. The graphics example adds
`--keyboard-release-grabs` to submit a real popup using its received release
serial, then verify input, outside dismissal, output leave and disconnect.

Acceptance was recorded in QUEUE before code. This is one complete keyboard-release
component, not complete release-triggered input or compositor acceptance. ADRs
0001/0002, v0.01 compositor scope and the invocation-only contract remain unchanged:
no agent/context surface, engine patch, dependency or new release scope. No new
owner decision or approval required. Filesystem security remains Claude's work.

## Executed verification

Windows PowerShell at repository root (all final checks passed):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2 from the same checkout, separate target `/root/alo-os-target`:

```powershell
wsl -d Ubuntu -- test -S /run/user/0/wayland-0
wsl -d Ubuntu -- pkg-config --modversion wayland-server egl xkbcommon
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked --test client_lifecycle popups::keyboard_grabs
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- bash -lc 'PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 cargo test -p alo-shell --locked --test client_lifecycle popups::keyboard_grabs > .git/alo-keyboard-release-wire.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --keyboard-release-grabs > .git/alo-keyboard-release-graphics.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --keyboard-grabs > .git/alo-keyboard-release-press-regression.log 2>&1'
wsl -d Ubuntu -- bash -lc 'WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --grabs > .git/alo-keyboard-release-pointer-regression.log 2>&1'
```

Linux: 55 tests pass (3 unit, 49 client lifecycle, 3 socket ownership), none
ignored; focused/wire keyboard-grab runs each pass six tests. Windows intentionally
excludes Linux protocol tests. Clippy, fmt and warnings-denied rustdoc pass.
Initial focused test compilation failed on a nonexistent `Fixture::counts` helper;
corrected to the existing serialized backend query. All executed assertions pass;
no lint allowances or gate changes. WSLg socket present; native versions are
Wayland server 1.24.0, EGL 1.5, xkbcommon 1.13.1. No installation or shared kernel,
cgroup, BPF or service changes were needed.

All three graphics fixtures exit 0. Additional wire evidence in the release log:
key press serial 3, matched release serial 4, then xdg_popup.grab with serial 4;
GLES callback/output enter, keyboard input, consumed outside click, popup_done/
output leave, parent focus restored and disconnect cleanup. Logs stay local under
`.git` as named above. These are scripted backend and protocol/submission checks,
not pixel readback, actual parent input or hardware certification.

## Integration and remaining work

Read updates README and checked all published report names against STATE at
iteration start: all eight task reports were already referenced; no pending
contributor reconciliation. Reports arriving during publication belong to the
next iteration. CHANGELOG, ROADMAP, QUEUE and STATE consolidate this component;
COMPOSITOR documents the updated backend policy. No source report was rewritten.

Next: pointer-release initiation and its ownership/serial invalidation policy.
Repositioning/output constraints, parent-leave backend, direct display/input,
production session and all other unfinished v0.01 requirements remain. Physical
laptop and GPU workstation records and actual parent input remain owed. Supervisor
full independent Windows/Linux/BPF test/lint/rustdoc publication gates have not
run for this change. No release box is certified. No staging, commit, push,
worker/loop launch, dev-loop edit or changes to another checkout/repository.
