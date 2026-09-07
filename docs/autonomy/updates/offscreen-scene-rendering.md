# Offscreen scene rendering

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`;
integration owner of the shared progress documents.
Status: ready for integration; supervisor publication gates remain pending.

## Change and acceptance

`crates/alo-shell/src/offscreen.rs` adds `render_scanout` and `PreparedScanout`:
an owned offscreen GLES scene becomes immutable XRGB upload pixels with the
identities of its drawn surfaces. Positive dimensions and signed GLES byte-count
limits are checked before allocation. Target creation/binding, import, drawing,
finish and readback errors return no prepared frame. Preparation sends no frame
callbacks or output membership events and is deliberately not a `FrameTarget`.
The trusted backend must upload and successfully submit the pixels before
returning their surface identities, without interleaving client dispatch.

`scene_drawing.rs` shares the existing fallible scene painter with `Nested`,
retaining imported textures through nested submission or offscreen readback.
Window/child, popup geometry and cursor hotspot stacking/clipping use the same
code. `ReadbackError` is preserved in `RenderError`; public Rust items document
the lifetime and submission boundary. No agent/adapter protocol changed.

Decisions: ADR 0002's native Smithay architecture and ADR 0001's context boundary
remain intact. A fresh full-frame ABGR renderbuffer and Normal transform reuse
the verified TopToBottom readback path; this favors complete immutable frames
and simple failure ownership before performance optimization. There is no unsafe
code, engine patch, capture verb or background context reader. Default/hidden
cursors add no pixels; a direct backend's default arrow is still outstanding.
The black clear remains the existing neutral diagnostic background.

Acceptance was recorded in QUEUE before implementation. One new unit test checks
valid/odd extents, zero/negative fields and signed overflow. One new real-protocol
integration test checks independent SHM buffers and callback preservation across
refused submission. Existing capability/input/popups tests remain intact.

The explicit `nested_check --offscreen` integration fixture uses an actual Wayland
client and WSLg GLES context. It checks every pixel of a 33x32 output (1,056 pixels):
red/green asymmetric window rows, a yellow child clipped at the right edge, a blue
popup shifted by parent/popup window geometry and a cyan cursor above both.
An extreme offscreen child is excluded. Repeated preparation clears an empty
scene and leaves the previously returned pixels intact. Preparation and injected
post-readback transport refusal send no callbacks; fixture-only successful
submission releases exactly four callbacks at time 77. Disconnect produces an
empty black scene. Truncating accepted SHM storage before import returns an import
error, disconnects the offending client with Bad pool size and sends no callback.
That is an actual buffer-import refusal; the transport refusal is fault injection.
No real GPU context-loss/draw/readback failure or kernel scanout is claimed.

## Executed verification

Windows PowerShell, repository root, all final commands exit 0:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux source is cfg-excluded on Windows (zero shell tests there).
Ubuntu WSL2, `/mnt/c/dev/alo-os`, environment
`PATH=/root/.cargo/bin:/usr/bin:/bin`, `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --lib offscreen --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Final exit 0; 83 unit + 65 client lifecycle + 3 socket + 2 doctests = 153 passed,
zero failed/ignored. Log: `.git/offscreen-linux-verified.log`. A prior PowerShell
stderr redirection reported NativeCommandError despite successful cargo output;
repeated the command group with redirection inside bash and explicit propagation
of WSL's exit status, obtaining exit 0. This was exit-status verification, not a
weakened gate. Full supervisor workspace/BPF gates have not run for this change.

WSLg, `XDG_RUNTIME_DIR=/run/user/0`, `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-offscreen-egl.json timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

First two commands exit 0; nested regression submitted 137 client surfaces and
passed popup/cursor callbacks, unmap/remap, protocol refusal and disconnect.
The invalid-EGL command exits 1 with invalid EGL Display and no pixel success.
Logs: `.git/offscreen-gles.log`, `.git/offscreen-nested.log`,
`.git/offscreen-refusal.log`. Corrected an exit-status variable lost in the first
refusal wrapper and ran the refusal directly to verify exit 1.
Known Mesa/ZINK startup diagnostics precede successful GLES, as in the baseline.

Development corrections: initial compilation required explicit Buffer/Physical
coordinate conversion and Texture import. The negative-size unit fixture first
hit Smithay's constructor assertion; now constructs deliberately malformed public
fields to exercise our refusal. The first scene fixture used an old root pointer
enter serial; waiting for the popup's actual enter fixes cursor authorization.
Clippy requested as_chunks_mut; fixed. No repeated failing test was looped, no
lint exception was added and no gate was lowered.

Prerequisites checked: Rust 1.98.0; WSLg socket present; pkg-config wayland-server
1.24.0, EGL 1.5, GLESv2 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2, libinput 1.31.1.
No routine dependency installation needed. `/dev/dri` is absent.

## Remaining work and progress integration

User-readable change: the compositor can prepare real window, popup and cursor
pixels for direct-display upload without prematurely telling clients their frame
was submitted. Shared progress updates retain the compositor/release checkboxes.
All reports published at iteration start were already referenced in STATE; no
unreconciled report required consolidation. Reports arriving during publication
are reconciled next iteration. Claude's filesystem/network workstream is untouched.

Next component: connect prepared scene pixels to consuming unbound upload and
blocking scanout ownership, with render/upload/TEST_ONLY/enable/refusal/cleanup
integration and callbacks only after successful submission. Direct FrameTarget,
default cursor, safe cookie-bearing atomic transport, pending-buffer retirement,
pause ordering, direct input and production session entry remain open. Existing
parent-leave, libseat and pinned unmap-panic limitations remain. Drawing/readback
context-loss and allocation failure evidence beyond propagated errors remains owed.
Actual DRM-equipped login/VM scanout is still needed; WSLg is not that evidence.
Physical business laptop and >=24 GB GPU workstation display/input, session
switching, suspend/resume and all hardware checklist records remain owed.

CHANGELOG, ROADMAP, QUEUE, STATE and COMPOSITOR updated in the same change.
Tracked and new-file diff reviewed; `git diff --check` passes. No staging, commit,
push, dev-loop edit, worker launch, other-checkout access, shared kernel/BPF/cgroup/
service changes or physical installation. Supervisor owns integration/publication.
