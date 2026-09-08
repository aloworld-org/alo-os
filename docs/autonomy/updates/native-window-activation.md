# Native window activation

Date: 2026-09-08. Workstream: native desktop window management.
Responsible contributor: single desktop worker in `C:\dev\alo-os`.
Status: ready for integration; activation component complete, feature unfinished.

## Change, decisions and acceptance

`crates/alo-shell/src/window_activation.rs` adds trusted Server::activate_window:
validate focus, select the mapped root, then use existing raising/pointer routing.
Foreign, dead, unmapped and popup targets and absent keyboards refuse before
mutation. Input and post-focus raising errors are distinguished; the latter is
defensive propagation, not an injected-failure or rollback claim.

`keyboard.rs` synchronizes XDG Activated after its central focus operation.
Mapped popup focus activates the grabbed root, not an unrelated client. Clearing
focus deactivates; held keys release before transfer, same-root grabs survive,
other-root selection dismisses them and popup teardown restores the parent.
Unmap resets role state; fresh mapping does not regain focus automatically.
Dead clients never cause automatic selection of another application. Pending
state preserves other flags and suppresses identical configures, including
selection before acknowledgement. Success queues configures, not client rendering.

The first socket focus-loss assertion found that Smithay 0.7's SeatHandler
callback runs on entry/replacement but not clearing. Inspected the pinned source
and moved synchronization after our own focus operation, observing the resulting
handle state. No engine patch or lint exception. Activation is separate from
raise-only operations but shared by existing nested/direct focus paths.

ADR 0002 and v0.01 focus/window-management requirements authorize native Rust
plumbing. Public rustdoc and COMPOSITOR.md document the boundary. No new scope,
user-facing strings, context reader, agent endpoint or activation-token protocol.
Agent application focus still needs grant/proposal/approval-controlled adapter
wiring. Native controls and configurable shortcuts remain next.

Recorded component acceptance in QUEUE before implementation. Rechecked pinned
EGLDisplay::new's unsafe signature; standalone direct GLES remains restricted by
workspace policy. Delivery permits this independent step-3 component. No unsafe
exception or alternative engine architecture is authorized by this work.

## Verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18); eight native dependencies checked:
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat`
returned 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present; /dev/dri absent. Initial ordinary-shell Rust lookup missed
the documented root toolchain; corrected PATH, with no dependency installation.
No shared kernel/BPF state, services or other checkout changes.

Windows commands from this checkout passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows runs zero Linux-only shell runtime cases. Linux commands from
`/mnt/c/dev/alo-os`, PATH=/root/.cargo/bin:/usr/bin:/bin and isolated
CARGO_TARGET_DIR=/root/alo-os-target, passed:

```sh
cargo test -p alo-shell --locked window_activation
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Focused run passed the first three tests; full run includes the added fourth.
Full Linux totals: 138 unit, 91 client-lifecycle, three socket and three compile-
fail doctests; zero failures/ignored. Four new real socket/SHM tests exercise
activation/deactivation configures, rapid unacknowledged and acknowledged state,
idempotence, raised order, key recipient isolation, explicit clearing, foreign/
unmapped/dead/popup/no-keyboard refusal, fresh remap, same-root grabs, other-root
dismissal and popup teardown restoring the parent without deactivating it.

Development corrections: private grab-field access and a fixture serial-field
typo caused compile errors; used the existing popup focus helper and key_serial.
One runtime assertion exposed the Smithay clearing behavior described above;
subsequent focused and full tests passed. Clippy required as_chunks for constant
wire-state decoding; corrected without suppression. Initial document paths and
quoted WSL registry reads were corrected; no claim rests on unsuccessful reads.

Additional Linux commands passed in the same environment:

```sh
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

After the example rebuild, with XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir and
WAYLAND_DISPLAY=wayland-0, both exited zero:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Extended window_raise_check.rs exercises activate_window with real GLES readback:
selecting the magenta second window changes the sampled pixel; selecting the
first restores its red pixel. offscreen_client.rs also asserts real wire
activation/deactivation on the second client and final activation of the first.
The full eight-stage offscreen fixture preserves golden scene/cursor pixels and
truncated-SHM refusal. Nested regression submitted 115 client surfaces. Expected
Mesa fallback and deliberately invalid-client diagnostics accompanied success.
This is nested/offscreen development evidence, not DRM scanout or hardware.

Source, tests and documentation diff reviewed; final git diff --check passed.
Full independent Windows/Linux workspace/rustdoc/BPF gates belong to the
supervisor and are not claimed by this worker.

## Reconciliation and remaining acceptance

Read report instructions and compared all published report filenames to STATE
at iteration start: none unreferenced. Claude retains security/model-choice
ownership; no accepted-policy changes or release checkbox changes. Reports
arriving during publication are reconciled next iteration.

All four shared progress documents are updated in this change. Next component:
native window switching/close controls and configurable shortcut integration.
Move/resize/minimise/maximise/tile, launcher/dock, clipboard and subsequent
delivery work remain unfinished. Standalone safe GLES, full DRM/seat entry,
populated input/hotplug and GPU/disable recovery remain owed. WSLg evidence
cannot certify the required physical laptop and GPU workstation; hardware
records still require those machines. No release verification claim.

No staging, commit, push, other repository changes, worker/loop launch,
tools/dev-loop changes or physical installation; supervisor owns publication.
