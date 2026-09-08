# Configurable window command dispatch

Date: 2026-09-08. Workstream: native desktop/window management.
Responsible contributor: single desktop development worker in C:\dev\alo-os,
also the integration owner. Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/shortcut_dispatch.rs` connects the existing alo-shortcuts
model to next/previous native window selection and cooperative close. The caller
supplies current bindings; rebinding and clearing apply on the next request.
Conflicting personal bindings fire nothing. Unimplemented actions explicitly
refuse rather than pretending success. Close uses actual keyboard ownership,
including popup roots, never stacking as a fallback. Each call queues one XDG
close without killing, retrying or dismissing the popup. Switching retains the
existing detailed activation errors. Cargo.lock adds only the existing workspace
alo-shortcuts dependency to alo-shell.

This is a complete action-dispatch component, not completed keyboard shortcuts.
Raw layout matching, consumed press/release and repeat isolation, nested/direct
input wiring, settings persistence and controls remain next. It would be incorrect
to call this method directly for every raw key event. Public rustdoc and
COMPOSITOR.md state that boundary and distinguish queued work from client success.
No agent endpoint, context reader, user-visible strings or new release scope.
Agent application verbs retain their grants/proposals/single approvals.

Read ADRs 0002/0010 and application-verb contracts. Native Rust and the accepted
shortcut model determine the implementation; no decision exception is needed.
Confirmed pinned Smithay 0.7.0 EGLDisplay::new is unsafe (display.rs:201), while
workspace unsafe_code remains forbid. Selected the first independent delivery
step 3 component without changing that policy. Acceptance recorded in QUEUE
before implementation; no worker, loop or other checkout used.

Five new real Wayland socket tests cover rebound and cleared bindings, reverse
cycling, personal conflicts, all eight unsupported actions, no keyboard/empty
display, close focus differing from stacking, one request with client survival,
and popup-focused close preserving its grab. The existing GLES cycling fixture
now additionally resolves a custom Ctrl+Alt+Space binding for each direction and
checks the magenta/red client pixels after production dispatch.

## Executed verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18). Prerequisite commands:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
test -d /dev/dri
```

Versions respectively: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1,
26.0.8-1ubuntu0.3, 0.9.2. WSLg socket present; /dev/dri absent. No dependency
installation, shared kernel/BPF state, service or host configuration changes.

Windows commands from C:\dev\alo-os, all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows executes zero Linux-only runtime tests. Linux commands from
/mnt/c/dev/alo-os, PATH=/root/.cargo/bin:/usr/bin:/bin and isolated
CARGO_TARGET_DIR=/root/alo-os-target:

```sh
cargo test -p alo-shell --offline shortcut_dispatch
cargo test -p alo-shell --locked shortcut_dispatch
cargo test -p alo-shell --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Initial locked invocation correctly required a lockfile update for the new
workspace dependency; offline invocation updated it but exposed a test typo
(`clear` instead of `unbind`). Corrected it, then four focused tests passed.
Added the popup test before the full run: 138 unit, 100 lifecycle, three socket
tests and three compile-fail doctests passed, none failed/ignored. Final affected
clippy, rustdoc, example build and fmt passed. No runtime assertion failed or
gate was weakened. Registry inspection used grep since Ubuntu rg is absent;
initial quoting/path lookups were corrected.

With XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir and WAYLAND_DISPLAY=wayland-0,
after rebuilding examples, both commands passed (exit zero):

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Configured command GLES pixels passed in both directions. Existing golden pixels,
truncated-SHM refusal and nested regression (115 client surfaces) passed. Expected
Mesa fallback warnings and intentionally invalid client diagnostics occurred.
These are real Wayland/GLES integration checks, not raw physical keyboard input,
DRM scanout, GPU acceleration certification or physical machine acceptance.

## Reconciliation and remaining work

At iteration start the tree was clean. Compared all published report filenames
against STATE: only `three-primary-model-choices.md` was unreconciled. Reviewed
its documentation-only diff-check evidence and retained the three primary source
choices, no-agent opt-out, undecided paired-machine UI placement, separate privacy
settings, proposed ADR 0021 and unchanged Alo hosting tier. Consolidated into all
four progress documents without a runtime or release completion claim. Claude
retains security/model-choice work. Later-arriving reports reconcile next time.

CHANGELOG, ROADMAP, QUEUE and STATE include this component and remaining work.
Full independent Windows/Linux workspace test/lint/rustdoc and BPF publication
gates belong to the supervisor and were not run by this worker. Physical keyboard,
populated libinput/hotplug, safe standalone GLES, complete DRM/seat entry, GPU
context/disable recovery and certified laptop/GPU workstation records remain
owed. Launcher/dock, other window operations, clipboard and all later delivery
requirements remain unfinished. No feature or release checkbox is marked done.
Source/test/documentation diff reviewed and git diff --check passed. No staging,
commit, push, tools/dev-loop edit, other repository changes or physical install.
