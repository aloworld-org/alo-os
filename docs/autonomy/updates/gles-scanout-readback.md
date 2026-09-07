# GLES scanout readback

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; supervisor owns staging, commit and push.

## Change and decisions

`crates/alo-shell/src/readback.rs` adds `readback_xrgb`, `RowOrder`,
`ScanoutPixels` and `ReadbackError`, exported through `src/lib.rs`. A complete
bound GLES target is copied through pinned Smithay 0.7's safe ExportMem API.
RGBA bytes become tightly packed, top-to-bottom B,G,R,0 bytes; `frame()` supplies
the existing `DisplayResources::with_frame` input. The owned CPU bytes remain
immutable. No live buffer mapping, atomic submission or callback is added.

Readback refuses empty extents and more than i32::MAX bytes **before export**:
the inspected pinned GLES copy/map implementation multiplies width, height and
four in signed i32. It checks mapping extent, ABGR/XBGR format and the pinned
GLES inversion flag before mapping, then exact length before allocating/copying.
Graphics failures retain the upstream GlesError; allocation uses try_reserve_exact.
No partial source is returned on refusal. Source channels are not unpremultiplied
and no colour-space conversion occurs; callers composite the output first.

Explicit row order is necessary because the mapping's GL inversion flag does
not describe the rendering transform. The real fixture verifies Smithay Normal
with TopToBottom and Flipped180 with BottomToTop. Other rotations are not inferred.
ABGR requests RGBA/UNSIGNED_BYTE, the GLES readback baseline, avoiding dependence
on optional BGRA export. Both opaque and alpha-bearing metadata are accepted;
the X byte is always zero. This synchronous CPU transfer is a correctness path,
not a zero-copy or performance claim.

ADR 0002 remains Rust/Smithay with unpatched engines. ADR 0001 and the application
adapter contract are unchanged: this trusted display-backend API introduces no
agent verb, screenshot request, context capture or background read. Diagnostic
strings are developer/backend data, not a native user-facing screen.

Selected acceptance criteria were recorded in QUEUE before implementation. The
step completes readback/conversion only, not the compositor feature. No new owner
decision or permission was needed. Claude's security workstream was not touched.
At iteration start every published task report was already referenced in STATE;
no pending reconciliation. Reports published later belong to the next iteration.

## Verification actually executed

Windows, repository root:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Final checks exit 0. Linux shell tests are cfg-excluded on Windows. Initial
Windows clippy found a missing private-item doc comment; fixed. Initial Linux
clippy found variable-range indexing and a test unwrap; replaced with checked
access and error propagation. No lint exemption, ignored test or weaker gate.
No test failed. An inspection quoting error and a patch context mismatch were
corrected without changing the intended task.

Ubuntu WSL, `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --lib readback --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Final commands exit 0. Six new tests: asymmetric channel/alpha/row conversion;
odd width/single row; empty/overflow extents; missing/wrong format, size and
inversion metadata; short/trailing byte refusal; patterned 1280x720 conversion
through padded allocation upload, TEST_ONLY, blocking enable and ordered disable.
That lifecycle test uses the existing fake ResourceDevice and transport, verifies
every visible pixel plus padding/tail, and does not prove a DRM ioctl succeeds.
Total: 82 unit + 64 client lifecycle + 3 socket + 2 doctests = **151 passed**,
zero failed/ignored. Local logs: `.git/readback-tests.log`,
`.git/readback-docs.log`; other check output was captured by worker tool events.

With `XDG_RUNTIME_DIR=/run/user/0`, `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/readback_check
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-readback-egl.json timeout 30s /root/alo-os-target/debug/examples/readback_check
```

First two exit 0. `examples/readback_check.rs` uses a real Wayland EGL context,
draws six different colours into a 3x2 offscreen GLES renderbuffer and checks
every XRGB byte in both Normal and Flipped180 orientations. This is new actual
pixel readback evidence, separate from DRM. Nested regression submitted **130
client surfaces**, with popup/cursor callbacks, unmap/remap, refusal and disconnect.
The invalid vendor invocation exits **1**, reports inability to obtain a valid
EGL Display, and never reports a pixel success. Local logs:
`.git/readback-gles.log`, `.git/readback-nested.log`, `.git/readback-refusal.log`.
Known Mesa/ZINK initialization diagnostics precede successful GLES use; no GPU
acceleration measurement is claimed.

Verified Rust 1.98.0, existing WSLg socket, and pkg-config versions: Wayland server
1.24.0, EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2, libinput 1.31.1.
No dependency installation was needed. Ubuntu lacks rg; used grep for local
pinned-source inspection after checking rg. No `/dev/dri` directory exists.

Reviewed source/tests/example and tracked documentation diff; `git diff --check`
passes. Four shared progress documents and COMPOSITOR updated in this change.
No stage/commit/push, dev-loop changes, worker launch, other-checkout access,
shared kernel/BPF/cgroup/service mutation or physical installation.

## Remaining work and proposed progress

Changelog: GLES output can now become a validated direct-display upload source.
Roadmap/queue: readback component complete; native compositor and release unchecked.
Next executable component: offscreen window/popup/cursor scene rendering into
ScanoutPixels, with real client pixel verification and import/draw/readback
refusal without premature callbacks. Direct FrameTarget, safe cookie-bearing
atomic transport, pending-buffer retirement, pause ordering, direct input and
production session entry remain unfinished. Parent-leave/libseat and pinned
unmap panic limits remain. No upstream engine patch or unsafe bypass.

Full supervisor Windows/Linux workspace tests/lints/rustdoc and BPF gates have
**not** run for this change. Successful DRM upload/presentation requires a
DRM-equipped development login/VM. Physical business-laptop and >=24 GB GPU
workstation display/input, session switching, suspend/resume and the remaining
hardware checklist require physical records. WSLg verifies none of those.
