# Linux graphics development baseline

This is delivery item 32, supporting the v0.01 Smithay compositor in ADR 0002.
`tools/graphics-check` is a developer executable, included in workspace gates
but never installed by `image/Containerfile`. It is not a compositor, an agent
verb, an application adapter, or a public OS contract. Its diagnostics are
developer output, not the shell's translated vocabulary.

## Prepare Ubuntu

Run in Ubuntu, with root only for package installation and the BPF baseline:

```sh
apt-get update
apt-get install -y build-essential pkg-config \
  libwayland-dev libegl1-mesa-dev libgles2-mesa-dev libxkbcommon-dev \
  libudev-dev libinput-dev libgbm-dev libseat-dev
```

The Rust/BPF setup is documented in `LOOP.md`: stable Rust at least 1.97,
`nightly-2026-06-01` with rust-src, rustfmt and clippy, LLVM 22 and
`bpf-linker 0.11.0` built with `--no-default-features --features llvm-22`.
Inspect `findmnt -n -o FSTYPE /sys/fs/bpf` before the kernel tests; if absent,
`mount -t bpf bpffs /sys/fs/bpf`. Do not overlay an existing mount. Confirm
`ls -A /sys/fs/bpf` is empty after tests; do not remove unfamiliar pins.
Keep the mount check and test commands in the **same WSL invocation**: WSL can
stop an idle distribution between calls, losing this mount. A separate successful
preflight is not evidence that the next invocation still has bpffs.

For the existing Windows checkout and Ubuntu toolchain, enter the environment
from PowerShell without passing Windows PATH quoting through two shells:

```powershell
wsl -d Ubuntu -u root -- env PATH=/root/.cargo/bin:/usr/lib/llvm-22/bin:/usr/sbin:/usr/bin:/sbin:/bin LLVM_PREFIX=/usr/lib/llvm-22 CARGO_TARGET_DIR=/root/alo-os-target bash
```

Then use the commands below inside Ubuntu. The root login is needed for the
existing BPF integration tests, not for graphics. On another setup use the
developer's own Rust installation, session and Linux target directory.

```sh
cd /mnt/c/dev/alo-os
cargo test -p alo-graphics-check --locked
cargo clippy -p alo-graphics-check --all-targets --locked -- -D warnings
cargo build -p alo-graphics-check --locked
timeout 30s /root/alo-os-target/debug/alo-graphics-check check
timeout 30s /root/alo-os-target/debug/alo-graphics-check render
```

`check` reports pkg-config versions and connects to the selected Unix socket.
`render` additionally creates a 320x200 window, verifies that winit selected
Wayland, clears a GLES framebuffer and submits it once, then exits. Run against
a real Wayland session; WSLg supplies `WAYLAND_DISPLAY` and `XDG_RUNTIME_DIR`.
No implicit `wayland-0` or X11 success is accepted. A socket connection by itself
does not verify the Wayland protocol or EGL; only `render` exercises those.
An external timeout bounds a broken display server or driver; exit 124 is a
failed check, never success. This does not measure presentation on a panel.

Exercise graphics initialization refusal on Mesa, keeping the real socket:

```sh
timeout 30s env __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-egl-vendor.json \
  /root/alo-os-target/debug/alo-graphics-check render
```

Expect exit 1 and `Smithay graphics initialization`, with no submitted-frame
message. The override is confined to that child, not written to the session.
Missing session configuration and missing/stale/non-socket paths are also
covered by the ordinary tests. No graphics test is ignored in the workspace;
the live display probe is an explicit integration command because CI may have
no graphical session.

## Dependency choices

Smithay is an unmodified crates.io release, pinned to **0.7.0**, with transitive
dependencies in the root `Cargo.lock`. Its
[upstream backend documentation](https://smithay.github.io/smithay/smithay/backend/index.html)
describes winit for nested development and DRM for direct displays. Selected
features compile both paths: winit/EGL/GLES, Wayland frontend with system
libraries, DRM/GBM, libinput/udev, libseat session handling, and desktop helpers.
Compiling direct-display support now exposes its native prerequisites before
compositor work begins. It does not open DRM devices or acquire a seat.
Defaults are disabled to avoid pulling unrelated renderer and Xwayland scope
into this prerequisite check. Smithay's winit dependency itself includes X11;
the probe checks the actual display handle and refuses an X11 result.

This follows ADR 0002 without patching an engine or changing ADR 0001's agent
boundary or the application-adapter contract. The one-frame probe is only an
integration fixture. Item 33 still owes a functioning server, real clients,
input routing, disconnect handling and direct-display integration. Physical
keyboard, pointer, display and certified-machine records remain required.

## Evidence collected 2026-09-07

Ubuntu **26.04 LTS**, x86_64, under WSL2 kernel
**6.18.33.2-microsoft-standard-WSL2**. Stable `rustc 1.98.0
(88d9e12ae 2026-08-18)` uses LLVM 22.1.8; the pinned nightly reports
`1.98.0-nightly (14210df0e 2026-05-31)`, LLVM 22.1.6. `bpf-linker` is 0.11.0.
The initial pkg-config checks lacked Wayland server, EGL, xkbcommon, udev,
libinput, GBM and libseat development metadata; the installation above succeeded.
bpffs was absent and was mounted for the existing kernel tests.

The probe reported Wayland server/client **1.24.0**, EGL **1.5**, GLES **3.2**,
xkbcommon **1.13.1**, udev **259**, libinput **1.31.1**, GBM
**26.0.8-1ubuntu0.3**, and libseat **0.9.2**. EGL/GLES numbers are pkg-config API
versions, not a GPU capability measurement. It connected to
`/run/user/0/wayland-0`. Both `check` and `render` exited **0** within the 30-second
timeout. The actual framebuffer was **320x200**, with a Wayland display handle.

Mesa emitted `failed to get driver name for fd -1`, `MESA-LOADER: failed to
retrieve device information`, `ZINK: failed to choose pdev` and `egl: failed to
create dri2 screen` before the successful submission. Acceleration was not
measured; these diagnostics must not be turned into a GPU support claim.
The invalid EGL-vendor command exited **1**, reported
`Smithay graphics initialization: Egl(DisplayNotSupported)`, and never reported
a submitted frame. A missing graphical runtime therefore does not pass merely
because the Unix socket is reachable.

The first full Linux test attempt stopped at
`a_turn_is_bounded_by_the_kernel` with `NoPinDirectory`: WSL had restarted after
the separate bpffs setup invocation. The corrected run mounts/checks bpffs in
the same invocation as the tests, matching the supervisor. No test or kernel
requirement was changed. Rustdoc did not run after that first failed attempt.

Final verification completed successfully:

- Linux: `cargo fmt --all --check`;
  `cargo test -p alo-graphics-check --locked` (three unit and three CLI tests);
  `cargo clippy -p alo-graphics-check --all-targets --locked -- -D warnings`.
- Linux baseline: `cargo clippy --workspace --all-targets --locked -- -D warnings`;
  `cargo test --workspace --locked --quiet` (tests and doctests);
  `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked`.
  The successful test/rustdoc invocation began with
  `(mountpoint -q /sys/fs/bpf || mount -t bpf bpffs /sys/fs/bpf)` and ended with
  `ls -A /sys/fs/bpf`, which was empty. Local worker logs are in
  `.git/graphics-baseline-clippy.log`, `.git/graphics-baseline-tests-mounted.log`
  and `.git/graphics-baseline-docs.log`; they are not release artifacts.
- In `crates/alo-bounding-kernel`: `cargo fmt --all --check` and
  `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`.
- Windows: `cargo fmt --all --check`;
  `cargo clippy -p alo-graphics-check --all-targets --locked -- -D warnings`;
  `cargo test -p alo-graphics-check --locked` (one CLI refusal test).
- `git diff --check` and review of the source/documentation diff and lockfile;
  no existing locked package version was removed or upgraded.

The supervisor's independent Windows/Linux publication gates have not been
run by this worker. No new test ignore or lint exemption was added. Physical
display/input testing, real compositor clients and certified-machine release
acceptance have not been performed by this step.
