# Synchronous scene replacement

Date: 2026-09-07. Workstream: native desktop compositor. Responsible contributor:
single desktop development worker in C:\dev\alo-os, also progress integration owner.
Status: ready for integration; supervisor owns staging, commit, full gates and push.

## Change and decision

`ActiveScene::replace` consumes prepared pixels on the original borrowed descriptor
and frozen route/mode. Allocation, upload, TEST_ONLY and blocking enable retain the
old allocation until successful commit. Old retirement then skips disable, preventing
it from blanking the new scene. Failed submission preserves old identities/storage.
Successful submission returns `SceneReplacement` with any retirement error separately:
cleanup failure must not be mistaken for a scene that never committed. Any cleanup
failure prevents further replacement until the caller disables and retires the device.
Failed disable quarantines the current allocation without retry on drop.

Implementation: `crates/alo-shell/src/scene_replacement.rs`, `scene_scanout.rs` and
the internal scanout/allocation test transports. New tests live in
`scene_replacement_tests.rs` and `scene_identity_tests.rs`. API rustdoc and
`docs/autonomy/COMPOSITOR.md` describe submission versus retirement outcomes.
No agent verb or application-adapter contract changes. ADR 0002's native Rust and
safe pinned blocking transport are retained; no engine patch or unsafe exception.
No new release scope or owner decision needed.

Acceptance was recorded in QUEUE before code: multi-frame ordering, immutable
active allocation, size/allocation/upload/commit refusals, cleanup/quarantine,
surface identity preservation, WSLg regression and affected Rust gates.

## Executed evidence

Ubuntu WSL2: Rust 1.98.0; /run/user/0/wayland-0 is a socket. pkg-config versions:
wayland-client 1.24.0, EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2,
libinput 1.31.1. No install required; /dev/dri absent. WSL emits existing unknown
autoMemoryReclaim/sparseVhd configuration-key diagnostics; host config untouched.

Windows PowerShell, executed exit 0:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Shell code/tests are cfg-excluded on Windows: zero tests there, not Linux evidence.
Ubuntu, from /mnt/c/dev/alo-os with PATH=/root/.cargo/bin:/usr/bin:/bin and
CARGO_TARGET_DIR=/root/alo-os-target, executed exit 0:

```sh
cargo test -p alo-shell --lib replacement --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Full affected suite: 92 unit, 65 client lifecycle, 3 socket and 2 doctests = 162
passed, zero failures/ignored. Five new replacement tests cover three distinct
framebuffer/blob pairs, pixel changes, commit-before-release with no intervening
disable, all candidate failure stages and clean retry, size refusal before I/O,
post-commit cleanup versus refused-commit cleanup, quarantine/no drop retry and
real server-side Wayland resource identities over a Unix socket. Identity fixture
does not dispatch/advertise those objects or claim client callback integration.
After strengthening resource-ID bounds and duplicate-release assertions, reran
fmt, all-target clippy and `cargo test -p alo-shell --lib display_resources --locked`
(34 passed, zero failures/ignored).
Logs: `.git/scene-replacement-linux.log`, `.git/scene-replacement-final-tests.log`.

With XDG_RUNTIME_DIR=/run/user/0 and WAYLAND_DISPLAY=wayland-0:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-replacement-egl.json timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

First two exit 0: existing real SHM 1,056-pixel scene, callback preservation,
non-DRM activation and truncated SHM refusal; nested submits 135 client surfaces.
Third exits 1 with invalid EGL Display, no success. Logs:
`.git/scene-replacement-{gles,nested,refusal}.log`. The first refusal wrapper lost
its shell status variable; direct tool invocation independently confirmed exit 1.
Initial test clippy slicing/expect/panic diagnostics were corrected without
exceptions; final checks pass. No repeated test-failure loop or gate reduction.

## Limits and integration

Successful DRM commits and resource destruction are injected, not measured kernel
scanout. WSLg regressions exercise existing GLES/client paths, not the new KMS
replacement ioctl on hardware. No callback is emitted by replacement itself;
direct FrameTarget membership/callback scheduling is the next useful component.
Default cursor, pause ordering, direct input/session entry, safe cookie-bearing
asynchronous transport/retirement, real GPU context-loss faults and existing
libseat/parent-leave/unmap-panic limits remain. Business laptop and >=24 GB GPU
workstation physical display/input/session/suspend/resume records remain owed.
No physical certification or release-complete claim.

All published reports were already referenced in STATE at iteration start; no
unreconciled report or Claude-owned task was taken. Reports arriving during
publication reconcile next iteration. CHANGELOG, ROADMAP, QUEUE and STATE updated
with this component and limits. Full Windows/Linux/BPF workspace publication gates
are the supervisor's pending work. No staging, commit, push, worker/loop launch,
other checkout access, shared kernel/BPF/cgroup/service mutation or physical install.
