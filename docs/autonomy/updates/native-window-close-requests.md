# Native window close requests

Date: 2026-09-08. Workstream: native desktop window management.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; close-request component complete, feature unfinished.

## Change and decisions

`crates/alo-shell/src/window_close.rs` adds `Server::request_window_close` and
`WindowCloseError`. `surfaces.rs` resolves only this display's live mapped
toplevel root. Each explicit call queues exactly one XDG close event. Ordinary
dispatch flushes it; success is neither a receipt nor confirmation of closure.
An application can ignore it or present a save dialog. No focus change, forced
resource destruction, process kill, retry or timeout is added. Unmapped, stale,
foreign and popup roots refuse; a properly configured remap becomes eligible.
The helper separates role lookup/lifetime from close policy and delivery.

ADR 0002, v0.01 window management and the application-verbs close contract justify
cooperative native close behavior. No new agent endpoint, context reader or
adapter bypass exists. No engine patch, dependency change, unsafe/lint exception
or user-facing UI string was added. Public rustdoc and COMPOSITOR.md describe
the success boundary. Native controls, shortcuts and application integration
remain separate, so this component does not complete window management.

Selected and recorded acceptance in QUEUE before implementation. Delivery item
33's next direct renderer component remains dependency-blocked: pinned Smithay
0.7.0 EGLDisplay::new is unsafe (registry source display.rs:201); the current
workspace forbids unsafe. No exception or upstream change is authorized. The
queue explicitly permits independent delivery-step-3 work in that situation.

## Verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18). Prerequisites checked:
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat`
reported 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
`test -S /mnt/wslg/runtime-dir/wayland-0` passed. `/dev/dri` absent. No packages,
shared kernel state, services or BPF pins changed.

Windows, from the checkout, passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

This Linux-only crate runs zero Windows runtime tests. Linux commands from
`/mnt/c/dev/alo-os`, with `PATH=/root/.cargo/bin:/usr/bin:/bin` and the separate
`CARGO_TARGET_DIR=/root/alo-os-target`, passed:

```sh
cargo test -p alo-shell --locked window_close
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Three new real Wayland socket tests in `tests/window_close/mod.rs` pass.
The shared client fixture now records actual XDG close events. Checks cover:

- Only the selected client receives one event, with no implicit retry. An ignored
  request preserves render eligibility and real keyboard wire delivery/focus.
  Another explicit call sends one more request. Voluntary role/surface destruction
  removes only that window and the retained handle refuses thereafter.
- Unmap and configured-but-bufferless targets refuse; fresh remap works. Abrupt
  disconnect and a stale handle after a replacement client maps still refuse.
- Another display's live root and a mapped popup refuse without sending close
  to either client or dismissing the popup.

Integration uses actual protocol dispatch, SHM clients and keyboard events;
render eligibility uses the existing controlled FrameTarget, not physical scanout.
Final Linux totals: 138 unit tests, 83 client-lifecycle tests, three socket tests,
three compile-fail doctests; no failures or ignored cases.

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`, passed:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both exited 0: real GLES golden pixels/truncated-SHM refusal and nested popup/
cursor regression with 115 submitted client surfaces. Expected Mesa and invalid
client diagnostics appeared. These are existing graphics regression checks, not
new close-control UI or physical-display evidence.

Initial test compilation placed a shared test module at Cargo's standalone test
root; moved it to `tests/window_close/mod.rs` and all subsequent checks passed.
Initial read commands used wrong root document paths and PowerShell glob syntax;
corrected. Ubuntu lacks rg, so local registry inspection used grep. An invalid
multi-file documentation patch was rejected before applying; corrected it.
No runtime test failure or gate weakening. Source/test/docs diff reviewed and
`git diff --check` passed. Full independent workspace Windows/Linux/rustdoc/BPF
publication gates belong to the supervisor and were not run by this worker.

## Reconciliation and remaining work

One published report was unreferenced at iteration start:
`docs/autonomy/updates/model-choice-and-what-alo-can-verify.md`. Reviewed report,
new production Service-door gap-test source and proposed ADR 0021 revision.
Consolidated all four shared documents. Two contributor tests demonstrate that a
forwarding service still receives local provenance, quiet indicator/empty egress
record and ThisMachineOnly permission. Contributor Linux workspace, rustdoc,
supervisor and BPF gates are reported evidence, not tests rerun by this worker.
The owned relay fixtures use no external provider or credential. The privacy gap
is open. ADR 0021 remains proposed; D1-now/D4-later is a recommendation, not an
accepted policy. Future qualification requires runtime supervision regardless of
ownership; the mechanism is missing. Paired-machine configuration is typed but
unreachable. Claude retains security ownership; no release scope/tier changes.

Next independent component: toplevel activation/stacking integrated with rendered
order and input, then native switching/close controls and configurable shortcuts.
Move/resize/minimise/maximise/tile, launcher/dock and clipboard remain unfinished.
Standalone safe GLES, full direct DRM/seat entry, populated input/hotplug and
GPU-context-loss/failed-disable recovery remain owed. Physical certified laptop
and GPU workstation records still require real machines, not WSL certification.

All four shared progress documents updated; no compositor, window-management or
release checkbox ticked. No staging, commit, push, other checkout/repository edit,
worker/loop launch, tools/dev-loop modification or physical installation.
Reports arriving during publication are reconciled next iteration.
