# Scanout-buffer initialization

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in C:\dev\alo-os.
Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/scanout_buffer.rs` initializes the complete mapped XRGB8888
allocation to black, including row padding and allocation tail. It validates
nonempty dimensions, format, checked row width, aligned/sufficient pitch and
mapped capacity before writing any bytes. Initialization is separate from the
resource-lifetime module so pixel-memory rules have one responsibility.

`DisplayResources::allocate` calls initialization after validating buffer metadata
and before framebuffer registration. `resource_device.rs` uses pinned drm-rs
MAP_DUMB/shared writable mmap and drops the mapping before returning to register
the framebuffer or unwind the buffer. A mapping failure retains its original
error and any buffer-destruction failure, with no registration or retry.
`atomic_output_check` requests RDWR for allocation/test-only (as DirectSession
already does) and reports black initialization and completed mapping lifetime.
Discovery-only remains read-only. No active modeset is submitted.

Black is deterministic memory initialization, not a replacement for design tokens
or a rendered shell. This follows ADR 0002 without patching pinned engines. No
agent capability/context capture or adapter contract change. Five new tests cover
padded dirty memory/tail, short mappings with no partial write, invalid geometry,
format and width overflow, map refusal plus cleanup failure, and real MAP_DUMB
ENOTTY with caller fd survival. Existing resource tests now assert initialization
and unmapping precede registration, and unchanged complete cleanup ordering.

The pinned wrapper constructs the mapped slice using a private capacity and its
mapping destructor can panic if munmap fails. These source-inspected limits are
recorded in `docs/quirks.md`, not claimed as recovered failures. No unsafe code,
upstream patch, ignored tests or weakened gates. Active scanout ownership and
page-flip retirement are explicitly the next component, not completed here.

## Verification

Windows, C:\dev\alo-os (all final commands exit 0):

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows excludes Linux shell tests. Ubuntu WSL, /mnt/c/dev/alo-os, using
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```text
cargo test -p alo-shell --lib --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo test -p alo-shell --lib real_map_dumb --locked -- --nocapture
```

All final commands exit 0: 46 unit + 64 client lifecycle + 3 socket + 1 lifetime
doctest = **114 checks**, zero failures or ignored tests. Initial clippy found two
test indices and unwrap_err forbidden by workspace lints; replaced with checked
access and Result propagation, then clippy passed. No test failures. Final
example RDWR correction was followed by affected clippy/example build and its
runtime refusal checks; final rustdoc/fmt checks also passed.

Additional integration, invoked from Windows with exit codes asserted:

```powershell
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/dri/card0 --allocate
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/null --unknown
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Absent card: expected exit 1/ENOENT. Unknown option: expected exit 1/usage before
open. WSLg: exit 0, 115 client surfaces, popup/reactive/cursor submissions and
callbacks, unmap/remap, isolated refusal and disconnect passed. Real MAP_DUMB test
exercised the same ioctl used before drm-rs mmap: ENOTTY (25), descriptor survives.
This does not exercise successful mmap/munmap or physical output. Local logs:
`.git/alo-scanout-{map-refusal,absent-card,argument,wslg}.log`.

Verified `/run/user/0/wayland-0` is a socket. pkg-config: libseat 0.9.2, libudev
259, GBM 26.0.8-1ubuntu0.3, EGL 1.5, xkbcommon 1.13.1. /dev/dri absent; no routine
packages needed. An initial compound WSL source-search command had quoting errors
and Ubuntu lacks rg; corrected with simple grep/source reads. Repository searches
used Windows rg. Initial queue/state reads corrected to docs/autonomy paths.
These diagnostics are not test failures or evidence of DRM availability.

Source, tests, diagnostic and documentation diff inspected; git diff --check
passes. Full independent Windows/Linux/BPF publication gates belong to the
supervisor and have not run for this change.

## Remaining work and integration

All published reports were already referenced in STATE.md at iteration start;
no unreconciled report or Claude task was taken. Existing evidence/limits retained.
Reports arriving during publication reconcile next iteration. CHANGELOG, ROADMAP,
QUEUE and STATE updated by the integration owner; COMPOSITOR and quirks updated.
No staging, commit, push, dev-loop changes, worker launch, other checkout changes,
shared kernel/BPF/cgroup/service mutation, or physical disk installation.

Next: active atomic commit ownership and page-flip retirement of initialized
buffers, injected commit/cleanup failures, renderer pause ordering, direct input
and production entry. Parent-leave and libseat disable-before-notify limits remain.
Successful DRM allocation, mapping, TEST_ONLY and retirement need a DRM-equipped
development login/VM. Certified business laptop and >=24 GB GPU workstation
physical display/input, session switching, suspend/resume and all hardware
checklist records remain owed. WSLg is development evidence only. The compositor
and release remain unchecked; the complete v0.01 scope is preserved.
