# Trusted window minimization and restoration

Date: 2026-09-08. Workstream: native desktop. Responsible contributor: desktop
integration worker in `C:\dev\alo-os`. Status: ready for supervisor integration,
subject to independent publication gates.

## Change and decisions

Native shell controls can hide a window and restore its current buffer without
losing placement or maximize/restore geometry. Hidden windows cannot receive
keyboard focus, pointer hits or submitted frame callbacks. Hiding releases held
input, dismisses the owning popups and cancels move/resize authority, including
late resize anchoring. Restoring does not raise or steal focus. Cycling skips
hidden windows while preserving their original ring positions. Ordinary client
commits cannot reveal them; unmap/disconnect retires hidden state.

`crates/alo-shell/src/window_minimize.rs` owns the trusted transition and typed
refusal. `surfaces.rs` distinguishes visibility from buffered mapping lifetime;
`window_maximize.rs` retains memory on that buffered lifetime, and
`window_switch.rs` filters the stable ring by visibility. Targeted resize
cancellation remains in `resize_transaction.rs`. Public rustdoc and
`docs/contracts/native-window-minimize.md` describe the API and limits; the
maximize contract explicitly records hidden-target refusal and retained memory.

These are routine native-shell choices consistent with ADR 0002 and v0.01 window
management. No visual text or palette changes (ADR 0010), agent verb, adapter
authority, upstream engine patch or new release scope. Separate restoration and
activation avoid an implicit focus transfer; separate buffered and visible
lifetimes avoid discarding valid normal geometry merely because a window hides.
The queue recorded this component and its acceptance checks before implementation.

## Executed verification

Ubuntu WSL2 prerequisites checked: Rust 1.98.0; the WSLg Wayland socket exists;
pkg-config finds wayland-client/server 1.24.0, EGL 1.5, GBM 26.0.8-1ubuntu0.3,
libudev 259, libinput 1.31.1, libseat 0.9.2 and xkbcommon 1.13.1. No dependency
installation or shared service/kernel modification was required.

Before every build/test/lint command, PowerShell measured
`(Get-PSDrive C).Free / 1GB` and refused below 12 GiB. Observed readings stayed
above 15.49 GiB. This is a preflight, not a running disk quota. No cleanup or
second workstream was started.

Linux commands ran via `wsl -d Ubuntu -- bash -lc`, from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
and `CARGO_TARGET_DIR=/root/alo-os-target`:

```text
rustc --version
test -S /mnt/wslg/runtime-dir/wayland-0
pkg-config --modversion wayland-client wayland-server egl gbm libudev libinput libseat xkbcommon
cargo test -p alo-shell --locked window_minimize
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo build -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

All final checks above passed. The focused suite first passed five tests. Adding
the move/resize case exposed a fixture duplicate acknowledgement of the initial
configure in the move case. Restricted that acknowledgement to the resize case,
which actually receives a new configure; all six focused tests then passed.
No production workaround, assertion weakening or lint allowance was added.
Full affected Linux tests passed: 141 unit, 143 real-client lifecycle and three
socket tests, plus three compile-fail doctests. Warnings-denied all-target clippy,
example builds and rustdoc passed.

Six new real-client tests cover:

- Hidden-root keyboard/pointer refusal, held-event release and recipient isolation;
  duplicate visibility calls and restoration without activation.
- Hidden buffer commits, callback withholding, output leave/re-entry and retained
  maximize normal geometry; operation without keyboard or output prerequisites.
- Three-window cycling in both directions, all-hidden refusal and ring retention.
- Foreign, child, popup, unbuffered and dead target refusal; unmap/remap and
  disconnect cleanup without hidden-state inheritance.
- Grabbed popup dismissal and keyboard retirement without reopening on restore.
- Move/resize cancellation, unrelated-window isolation, late committed resize
  responses and stale press refusal after restoration.

Graphical commands used `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```text
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both returned exit 0. Offscreen passed all 21 existing stages and four new full
6,400-pixel frames: hide, restore, hide and restore a real SHM window at its saved
position. Every pixel is independently checked as white window or black background
through production `render_scanout`; duplicate calls are checked at each boundary.
The nested popup/cursor regression submitted 115 client surfaces and passed.
Expected Mesa fallback messages and deliberate malformed-client protocol errors
were emitted; no assertion or stage was skipped. The offscreen transport remains
a fixture; these are GLES/protocol observations, not successful DRM commits.

Windows commands from `C:\dev\alo-os` all passed:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows executes zero Linux-only runtime cases; those tests are not additional
compositor acceptance evidence. Final Linux `cargo fmt --all --check` and
`git diff --check` passed. Source, new test/graphics files, contracts and progress
diff were inspected. No full workspace or BPF gate is claimed by this worker.

## Reconciliation and remaining work

Initial tree clean. Read constitution, delivery/shared-main/report rules, current
queue and STATE tail, relevant feature/roadmap sections, accepted ADRs 0002/0010
and native maximize/application contracts. Compared every published report filename
with STATE: none awaited reconciliation at iteration start. Publication arrivals
reconcile next iteration. Claude's assigned filesystem/credential work is untouched.

Proposed changelog/roadmap/queue update: trusted minimization/restoration is a
completed component, not a completed minimize feature or window-management gate.
Next: XDG client minimize policy, pre-map/mapped/duplicate/refusal behavior and
truthful WM capability advertisement, using this primitive. Then tiling and native
controls. No release checkbox changes. All four shared progress documents are
updated in this change by the integration owner.

Independent supervisor Windows/Linux workspace fmt/clippy/tests, Linux rustdoc,
pinned BPF gates and concurrent-main integration remain pending. WSLg does not
certify direct DRM/seat entry, populated physical input/hotplug, GPU/recovery or
certified laptop/workstation acceptance. Those machine records remain owed at
their delivery phases, with physical acceptance after integrated-image VM checks.
Configurable shortcut input/settings integration remains phase 3 after window
operations; normal keyboard routing and component tests remain active here.

No staging, commit, push, worker/loop launch, tools/dev-loop edit, other checkout,
credential/identity access, host cleanup or unrelated host changes. Supervisor
owns publication.
