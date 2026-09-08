# Scoped session pause polling

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; one lifetime component, not a completed compositor.

## Change and decision

`DirectSession::with_active_device` now lends a descriptor and nonblocking seat
poll to a trusted synchronous rendering loop. Previously the discovery-only
borrow could not poll the session while retaining a frame target. The new
`active_session.rs` owns that scope separately from notification/device policy.
`session_device.rs` latches interruption even when pause and activation arrive in
one batch, checks backend inactivation without a notification, and defers closing
the scoped descriptor until the caller retires and drops its target. The guard
also closes during unwind. Close failure remains terminal, never retried by drop.

`ActiveSessionResult` preserves the caller's result and manager-close result
independently and requires inspection. Public rustdoc describes polling before
frames and while idle, stopping on any poll error, invoking `Server::retire_output`
and returning only after dropping borrowed resources. A compile-fail doctest
proves the borrowed descriptor cannot escape. `DirectTarget` documents this path.

This is an additive native Rust component under ADR 0002 and the v0.01 compositor
feature/roadmap, with no engine patch or new release scope. No agent verb, adapter,
D-Bus, configuration or image contract changes. Keeping cleanup scoped avoids
self-referential descriptor/renderer ownership and preserves both failure causes.
The accepted libseat limitation in `docs/quirks.md` still applies: Smithay can
acknowledge disable before delivering the pause notification. An open descriptor
does not promise continuing kernel authority or successful disable after pause.

## Verification actually run

Ubuntu WSL2 kernel `6.18.33.2-microsoft-standard-WSL2`, Rust `1.98.0
(88d9e12ae 2026-08-18)`. Existing graphics packages were present: Wayland 1.24.0,
EGL 1.5, GLES 3.2, xkbcommon 1.13.1, udev 259, libinput 1.31.1,
GBM 26.0.8-1ubuntu0.3, libseat 0.9.2. WSLg socket present; `/dev/dri` absent.
No dependencies installed and no shared kernel, cgroup, BPF or service changes.

Linux commands used `PATH=/root/.cargo/bin:/usr/bin:/bin`,
`CARGO_TARGET_DIR=/root/alo-os-target`, from `/mnt/c/dev/alo-os` (or the equivalent
absolute `--manifest-path /mnt/c/dev/alo-os/Cargo.toml`):

```sh
cargo test -p alo-shell --locked scoped_
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS='-D warnings' cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
# Same WSLg environment, expected exit 1:
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-egl-vendor.json \
  timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

All final checks pass. Six new focused tests pass. Full shell checks: 111 unit,
67 client-lifecycle, three socket-ownership tests and three compile-fail doctests.
New integration evidence uses two-stage real calloop channels plus real kernel
Unix socket descriptors: pause/activate is sticky, retirement can still write
through the borrowed fd before close, EOF follows scope exit, and the next scope
acquires a new descriptor. Refusals cover initial inactivity, open denial, backend
inactivation, terminal notifier failure, independent operation/close errors and
exactly-once unwind cleanup. The notifier-loss case injects the terminal callback
state; it does not measure a real libseat transport failure.

Offscreen golden pixels/refusals and nested regression pass; 115 client surfaces
submitted. Existing Mesa diagnostics and deliberately invalid client protocol
messages appear in successful runs; these are not physical display evidence.
Local logs: `.git/scoped-session-checks.log`, `.git/scoped-session-egl-refusal.log`.

Windows: `cargo fmt --all --check`,
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`, and
`cargo test -p alo-shell --locked` pass. Windows excludes this Linux-only library
and runs zero shell tests; it is not duplicate execution of the Linux tests.
`git diff --check` and read-only diff review pass.

The first Linux clippy run found missing private field/method documentation,
test indexing and explicit test panics. These were corrected without lint
exemptions: explicit caller-invocation assertions, checked peer access and an
injected unwind payload. Focused and full tests have no failures this iteration.

## Remaining work and progress integration

Wire the direct GLES renderer, server output retirement and then direct input to
this scope. This API does not automatically retire an output; its trusted caller
must obey the stop/retire/drop contract. Real DRM acquisition, seat pause/resume,
failed-disable recovery, GPU context loss, safe asynchronous transport and physical
laptop/GPU-workstation acceptance remain owed. WSLg is not hardware certification.
Full independent Windows/Linux workspace, rustdoc and pinned BPF publication gates
belong to the supervisor and were not run by this worker.

CHANGELOG describes the pause-aware lifetime; ROADMAP and QUEUE retain unfinished
compositor status and name renderer/retirement wiring next. STATE references this
report and the report-reconciliation audit. No published report was unreferenced
at iteration start. Reports arriving during publication belong to next iteration.
No staging, commit, push, other checkout changes or worker launches.
