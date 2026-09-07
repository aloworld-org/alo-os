# Atomic display property discovery

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Contributor: single desktop development worker in `C:\dev\alo-os`
- Status: ready for integration; supervisor publication gates pending

## Change and acceptance

`crates/alo-shell/src/atomic_output.rs` discovers a fresh output after negotiating
universal-plane and atomic client capabilities. It validates standard connector,
CRTC and plane property names, unique IDs, types, mutability and atomic flags.
The lowest-ID compatible primary plane with formats supplies the route. A malformed
primary schema refuses instead of hiding it by trying another plane. Coordinate
ranges must admit an unscaled full-mode rectangle, with source sizes in 16.16 units.
The returned handles and advertised formats are a snapshot, not a reservation.

`atomic_inventory.rs` owns ioctl transport through the existing borrowed-descriptor
wrapper. Errors retain the operation and errno. Plane compatibility uses a fresh
resource list; a disappeared CRTC or oversized mask refuses. Enum names are resolved
from metadata, not numeric ordering, and property names remain bytes. The caller's
descriptor remains owned by the caller. `atomic_output_check` is an explicitly
named-device developer diagnostic. Production uses `DirectSession::with_device`.
Opening a primary node in the diagnostic can implicitly acquire DRM master; no
real card was opened in this iteration.

This is a complete **property discovery** component. It does not allocate a
framebuffer/blob, issue TEST_ONLY or change scanout. Client-capability changes
persist on that open file description even when later discovery fails, including
through duplicated descriptors. Reacquisition must rediscover. No legacy fallback,
new dependency, unsafe code, engine patch, context capture or agent verb was added.
External adapter/agent contracts remain unchanged; the additive Rust API has rustdoc.

Acceptance was entered in QUEUE before implementation. Native Rust and unmodified
engines follow ADR 0002; no agent authority changes under ADR 0001. Standard property
semantics follow the [Linux KMS documentation](https://docs.kernel.org/gpu/drm-kms.html)
and locally inspected pinned drm-rs 0.14.1 sources. Schema validation is deliberately
separate from kernel configuration validation because the latter needs owned
framebuffers and mode blobs. That is the next component, not a completed feature.

## Executed verification

PowerShell in this checkout, final commands all exit 0:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, separate target directory, final commands all exit 0:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --lib atomic_output --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --examples --locked
```

Eight new tests pass: complete/stable schema; every transport-stage failure and
stop ordering; missing/duplicate/wrong-type/immutable/non-atomic properties;
each required property and invalid source range; incompatible/cursor/empty-format/
missing planes; malformed type and aliased handles; valid fallback versus broken
primary refusal; real non-DRM ioctl errno and caller-descriptor survival. Final
Linux total: 27 unit + 64 lifecycle + 3 socket = **94**, zero ignored or failed.
Windows intentionally runs zero Linux shell tests. Initial Linux clippy rejected
15 test unwrap/unwrap_err calls; tests now propagate errors explicitly. No lint
allowance or weakened test. Final tests/clippy ran after the correction.

Additional integration, exit codes asserted in PowerShell:

```powershell
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/null
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/dri/card0
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

First: expected exit 1, universal-plane capability ioctl ENOTTY (25). Second:
expected exit 1, absent card ENOENT (2). Third: exit 0, **115 client surfaces**
submitted, popup/reactive/cursor callbacks, unmap/remap, refusal and disconnect.
Local logs: `.git/alo-atomic-discovery-refusal.log`,
`.git/alo-atomic-discovery-absent-card.log`, `.git/alo-atomic-discovery-wslg.log`.
This is protocol/GLES submission evidence, not pixel readback or physical input.

Prerequisites verified: `/run/user/0/wayland-0` exists, `/dev/dri` absent;
pkg-config libseat 0.9.2, libudev 259, GBM 26.0.8-1ubuntu0.3, EGL 1.5,
xkbcommon 1.13.1. No packages needed. A combined WSL source-search command had
quoting/tool-path errors; source inspection used the Ubuntu filesystem through
its Windows share instead. Existing WSL unknown-key warnings were left alone.
No kernel, BPF, cgroup, service or unrelated host changes.

## Reconciliation, limits and next work

Reconciled the two published reports not yet referenced in STATE at iteration
start: `network-egress-enforcement.md` and `kernel-supervisor-selects-its-own-task.md`.
Source-inspected their kernel/map/tests and supervisor selection/refusal paths;
did not run another worker, supervisor or shared-kernel test. Contributor-reported
Linux/BPF and Windows evidence remains labelled as such. All four shared progress
documents retain egress gaps and unfinished machine gates. Source reports untouched.

Pinned drm-rs raw property parsing assumes valid kernel lengths/C strings; our
post-parse schema checks cannot contain every malformed raw response. This
source-inspected limit is in quirks; no malformed-kernel panic was reproduced.
Successful atomic capability negotiation and property ioctls require a DRM-equipped
development login/VM. A real card may impose additional properties, formats,
modifiers or resource constraints: only TEST_ONLY with owned resources can check
them. Existing direct-session disable-order/parser limits remain unresolved.

Next: framebuffer/mode-blob ownership, atomic TEST_ONLY construction/refusal/cleanup;
then scanout/page flips, renderer pause ordering, direct input and production entry.
Parent leave remains blocked by pinned Winit's missing notification. Certified
business laptop and 24-GB-or-larger GPU workstation records still owe native display,
input, session switching, suspend/resume and the full hardware checklist. WSL never
certifies them. Other v0.01 scope remains unchanged. Supervisor full Windows/Linux/
BPF publication gates have not run for this change. No staging, commit, push,
dev-loop edit, delegation, other-checkout modification or release-complete claim.
