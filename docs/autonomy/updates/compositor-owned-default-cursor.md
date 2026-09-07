# Compositor-owned default cursor

Date: 2026-09-07. Workstream: native desktop compositor. Responsible contributor:
single desktop development worker in `C:\dev\alo-os`, also progress integration
owner. Status: ready for integration; supervisor publication gates remain pending.

## Change and decisions

The compositor now draws its ordinary pointer on both nested and prepared direct
frames. `cursor.rs` adds `Cursor::Arrow { location }`: a pointer-enabled server
returns the last accepted location on empty desktop, client focus loss, cursor
destruction and disconnect. Authorized hidden/client cursors keep their behavior.
Legacy unpositioned `Default` remains for callers without a pointer seat: nested
keeps the host arrow and offscreen/direct adds no pixels. Cursor-aware targets must
handle Arrow; the default FrameTarget implementation explicitly refuses it.

`default_cursor.rs` owns a 12x18 original mask with a tip hotspot (0,0), black outline
and white interior. Scale one and floored fractional coordinates match the current
output model. Neutral black/white supplies contrast on both dark/light content
without introducing a second branded palette or an external theme dependency.
Geometry clips before integer conversion; non-finite positions refuse, while
finite fully offscreen positions produce no pixels. `scene_drawing.rs` uses safe
Smithay solid drawing above all windows/popups. No engine patch, unsafe code, new
agent verb, background context reader, or application-adapter contract change.
This implements ADR 0002's existing native compositor scope.

Arrows own no Wayland surface, output membership or callback. Shared rendering
failures propagate before submission. Nested hides the parent cursor only after
successful scene submission; its legacy host-arrow behavior remains available.
All public snapshot semantics are documented in rustdoc and COMPOSITOR.md.

## Verification actually executed

Ubuntu WSL2, Rust 1.98.0; pkg-config verified wayland-server 1.24.0, EGL 1.5,
GLES 3.2, xkbcommon 1.13.1, udev 259, libinput 1.31.1, GBM 26.0.8-1ubuntu0.3,
libseat 0.9.2. `/run/user/0/wayland-0` is a socket; `/dev/dri` is absent.
No dependency installation or shared kernel/cgroup/BPF/service changes needed.
Existing WSL configuration and Mesa diagnostics were left untouched.

Windows PowerShell, repository root, all exit 0:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux commands below use `wsl -d Ubuntu -u root -- env
PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target` from the
repository root. All final commands exit 0:

```text
cargo test -p alo-shell --lib default_cursor --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
env RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Three new geometry tests and one new real-socket protocol test; full shell suite:
100 unit + 66 client lifecycle + 3 socket ownership + 2 doctests = 171 passing,
none failed/ignored. Windows cfg-excludes Linux shell execution. Initial full
Linux run found nine popup coordinator tests whose simulated target supported
only unpositioned Default. The fixture now explicitly simulates owned-arrow
support with no extra client identities; an independent legacy target test still
requires refusal. Existing popup/client-cursor refusals remain. Clippy's type
complexity and manual-contains findings were corrected without lint exceptions.

With `XDG_RUNTIME_DIR=/run/user/0 WAYLAND_DISPLAY=wayland-0`:

```text
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both exit 0. Offscreen tests compare every 33x32 output pixel to an independent
row-span golden shape at seven placements, including fractional/negative locations,
all edges and an extreme offscreen position, above real SHM window/child/popup
content and after disconnect. Actual protocol stages switch custom -> hidden ->
custom -> destroyed/default. Restored client pixels, unchanged arrow/background
surface identities, non-finite coordinate refusal/recovery, callback preservation,
non-DRM DirectTarget refusal and truncated-SHM import refusal also pass. The nested
regression submits 115 client surfaces. Invalid graphics initialization:

```text
env __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-default-cursor-egl.json timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Exits 1 with invalid EGL Display, as required. No successful scanout claim.
`git diff --check` passes; tracked diff and both new Rust files reviewed.

## Remaining work and proposed progress

Record this component in CHANGELOG/ROADMAP/QUEUE/STATE; leave compositor/release
unchecked. Next independently executable component: truthful per-backend output
identity/mode/physical metadata through FrameTarget, with real protocol assertions
and refusal preserving membership. Then pause/retirement, direct renderer/input/
session entry. Safe asynchronous cookie transport, GPU context-loss tests and
existing libseat/parent-leave limitations remain. This step does not test parent
cursor visibility by physical observation or certify scaling beyond scale one.
Successful DRM display/input/session/suspend/resume acceptance on the physical
business laptop and >=24 GB GPU workstation remains owed. Supervisor full
Windows/Linux/BPF gates, commit and push are pending; worker staged/committed/
pushed nothing and did not modify tools/dev-loop or another checkout.
