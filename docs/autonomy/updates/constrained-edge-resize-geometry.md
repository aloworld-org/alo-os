# Constrained edge resize geometry

- Date: 2026-09-08
- Workstream: native desktop compositor
- Contributor: single desktop development worker and progress integration owner,
  `C:\dev\alo-os`
- Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

The native shell can calculate a window resize from any edge/corner using the
application's committed geometry and size limits. The opposite edge stays
anchored to the actual size the client commits, even if that differs from the
suggestion. Invalid targets and excessive values refuse without changing the
window, configuration, stacking or keyboard/pointer recipient.

This is one complete geometry component of interactive resize, not a finished
interactive-resize feature. `crates/alo-shell/src/window_resize.rs` exposes an
immutable mapped-root snapshot and typed eight-edge calculation. `scene.rs`
shares its existing effective geometry calculation rather than introducing a
second interpretation of window shadows, child bounds or XDG geometry.
Public rustdoc and `docs/contracts/native-window-resize.md` describe this additive
trusted Rust API; no agent verb, app-adapter or wire contract changes.

Decisions consistent with ADR 0002's native Rust boundary:

- Keep geometric calculation separate from pointer authorization and the
  asynchronous configure/ack/commit lifecycle. A snapshot holds no surface
  handle or authority. This makes old-buffer placement impossible to mistake
  for an acknowledged or committed resize in this component.
- Measure each delta from the fixed initial pointer location and round halves
  away from zero. Do not accumulate rounding drift or flip edges on crossing.
- A drag calculates the nearest allowed size, clamping selected dimensions at
  positive client limits. The existing exact `request_window_size` API retains
  its refusal semantics. Unselected axes cannot silently resize to satisfy a
  newly incompatible constraint. Current limits must be revalidated by the
  future transaction caller; snapshot limits remain fixed.
- Bound initial geometry, computed placement, dimensions and pointer deltas to
  one million logical pixels, matching native placement's arithmetic envelope.
  Impossible limits and non-finite deltas refuse. Actual client dimensions may
  violate their advertised limits; anchoring follows actual dimensions within
  the supported range, not the requested dimensions.

No user-facing UI strings, colour changes or ADR 0010 changes. No new release
scope, permission requirement, engine patch or routine dependency installation.

## Acceptance and verification

Three unit tests exhaust all eight edge/corner directions, opposite anchors for
an actual client size different from the suggestion, crossing, rounding, zero
and finite client bounds, impossible limits, unsupported unchanged dimensions,
NaN/infinities, extreme deltas/dimensions and both placement boundaries.

Four real-client tests in `tests/window_resize/mod.rs` cover pending versus
committed limits/geometry, unchanged old buffers after configure and ack,
resized buffer commit, effective geometry intersection, foreign/child/popup/
unmapped/dead targets, remap eligibility, oversized surface-tree refusal and
recovery. Normal keyboard isolation, pointer delivery, unchanged configuration
and stacking remain tested. None of these tests claim press-serial authority
for a resize: no resize-request handler is present yet.

Prerequisites checked in Ubuntu WSL2:

```text
export PATH=/root/.cargo/bin:/usr/bin:/bin
rustc --version
test -S /mnt/wslg/runtime-dir/wayland-0
pkg-config --modversion wayland-server wayland-client egl gbm libinput libudev xkbcommon libseat
```

Rust 1.98.0; socket present; library versions respectively 1.24.0, 1.24.0, 1.5,
26.0.8-1ubuntu0.3, 1.31.1, 259, 1.13.1 and 0.9.2. Initial default PATH did not
find rustc; the explicit PATH resolved it. No service, shared kernel/BPF or
other checkout changes.

Linux commands from `/mnt/c/dev/alo-os`, with the explicit PATH above and
isolated `CARGO_TARGET_DIR=/root/alo-os-target`:

```text
cargo test -p alo-shell --locked window_resize
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Focused tests passed six cases before the final oversized-tree case was added.
Two development clippy attempts failed on missing private documentation and a
test-helper panic; corrected with documentation and Result-returning tests,
without lint allowances. Their chained later gates did not run. Final commands
all returned exit 0: 141 unit, 119 lifecycle and three socket tests, plus three
compile-fail doctests; no failures or ignored entries. No runtime test failed.

Windows commands from `C:\dev\alo-os`:

```text
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

All passed; Windows runs zero Linux-only shell runtime cases.

Graphical checks with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```text
/root/alo-os-target/debug/examples/nested_check --offscreen
/root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both returned exit 0. The added `examples/support/resize_geometry_check.rs`
compares every pixel of the 80x80 GLES framebuffer for all eight edges, after
valid calculations and invalid-delta refusal. The full existing 12-stage
offscreen regression also passes, including actual acknowledged buffer resizing
and pointer movement; those existing checks are not interactive-resize evidence.
Nested regression submitted 115 client surfaces and passed popup/cursor,
remap/refusal/disconnect checks. Expected Mesa fallback and deliberately malformed
client diagnostics did not skip or fail these checks. No DRM scanout is measured.

Changed source, tests, example and documentation reviewed; `git diff --check`
passed. All four progress documents identify this
component, evidence and next implementation. No staging, commit or push.

## Reconciliation and remaining scope

Read constitution, delivery order, current queue/STATE tail, shared-main/report
rules, relevant features/roadmap, accepted ADRs 0002/0010 and application contracts.
Initial working tree was clean. Every published report filename was already
referenced in STATE at iteration start; none awaited consolidation. Reports
arriving during publication are reconciled next iteration. Claude's credential
and security work remains assigned to Claude.

Next: integrate XDG interactive-resize press authority, invalid serial/edge and
foreign-target refusal, current-limit revalidation, resizing configure state,
acknowledgement and actual-buffer/geometry commit anchoring, final-release
configuration and leave/unmap/disconnect cancellation. Include ordinary keyboard
isolation and real resized GLES frame tests. Complete remaining window operations
before configurable raw keyboard shortcut integration and native controls.
This calculation component neither sends a configure nor applies a placement.

Full independent Windows/Linux workspace, rustdoc and pinned BPF gates remain
the supervisor's work and are not claimed here. Direct DRM/seat entry, populated
devices/hotplug, GPU/recovery and physical laptop/workstation records remain owed
at their delivery phases; physical acceptance follows integrated VM image checks.
WSLg is not hardware certification. Interactive resize, full window management,
compositor and release checkboxes remain open. No supervisor modification, worker
launch, physical installation, other repository/checkout or unrelated host change.
