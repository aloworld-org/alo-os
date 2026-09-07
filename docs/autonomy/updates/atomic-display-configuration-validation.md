# Atomic display configuration validation

Date: 2026-09-07. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in C:\dev\alo-os.
Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/atomic_test.rs` builds the thirteen required properties for
one connector, CRTC and primary plane: ACTIVE, exact owned MODE_ID/FB_ID, CRTC
routing, zero origins, full-mode destination and 16.16 source dimensions.
`DisplayResources` freezes the plan before allocation, refusing missing or aliased
required handles, object collisions and empty geometry before acquiring resources.
Later caller mutation cannot change the request or its mode. Discovery must still
come from this same descriptor; this trusted shell API does not authenticate a
caller-fabricated snapshot or promise containment of upstream raw metadata parsing.

`DisplayResources::test_and_release` consumes the candidate, submits exactly
TEST_ONLY | ALLOW_MODESET on its borrowed allocation descriptor, then explicitly
releases blob/framebuffer/buffer on both acceptance and refusal. The original
kernel error precedes every cleanup error; no retry, legacy fallback, page-flip
request or active commit. Cleanup failure requires retirement of that device.
The consuming API deliberately avoids implying a reservation for later scanout.
This follows ADR 0002's native Rust/Smithay backend with unchanged pinned engines;
ADR 0001 agent/context boundaries and public agent/adapter contracts are unchanged.

`atomic_output_check /dev/dri/cardN --test-only` offers a developer diagnostic.
Its direct file opening remains a fixture: production uses DirectSession. Success
means the kernel accepted that candidate at that instant, not that a later commit
will succeed, nor that pixels were drawn. Resources remain unbound and unmapped.

Six new tests cover exact thirteen-property routing/geometry and immutable
snapshots, missing/aliased handles and colliding objects, pre-allocation refusal,
fixed flags and single submission with EINVAL/EACCES/ENODEV, real atomic ioctl
ENOTTY with fd survival, and acceptance/refusal crossed with simultaneous cleanup
failures. Tests reside in atomic_test_tests.rs and display_resources_tests.rs.

## Verification

Executed Windows commands (exit 0): `cargo fmt --all`;
`cargo fmt --all --check`;
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`;
`cargo test -p alo-shell --locked`. Windows excludes Linux shell tests.
Initial Windows/Linux format checks requested a second formatting pass on the
expanded diagnostic match arm; applied cargo fmt, final checks clean. No test
failures, lint exemptions, ignored tests or gate changes.

Executed Ubuntu WSL commands from /mnt/c/dev/alo-os, with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```text
cargo test -p alo-shell --lib --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo test -p alo-shell --lib real_atomic_ioctl --locked -- --nocapture
```

All exit 0. Initial 41 unit tests passed; final suite: 41 unit, 64 client lifecycle,
3 socket tests and 1 compile-fail lifetime doctest, **109 checks**, none failed or
ignored. Real atomic TEST_ONLY ioctl refused ENOTTY (25), descriptor survived.
Additional integration commands, all exit codes asserted in PowerShell:

```powershell
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/dri/card0 --test-only
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/null --unknown
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Respectively expected exit 1/ENOENT, expected exit 1/usage refusal before open,
and exit 0: 115 client surfaces submitted, popup/reactive/cursor callbacks,
unmap/remap, isolated refusal and disconnect verified. No pixel readback,
physical input, successful DRM test or active scanout measured. Logs:
`.git/alo-atomic-test-{linux,refusal,absent-card,argument,wslg}.log`.
Source/new tests and tracked diff inspected; `git diff --check` passes.

WSLg socket and pkg-config prerequisites verified: libseat 0.9.2, libudev 259,
GBM 26.0.8-1ubuntu0.3, EGL 1.5, xkbcommon 1.13.1. No packages needed.
An optional process diagnostic had a PowerShell-to-bash pipe quoting error;
verification results came from completed commands/logs, not that diagnostic.
Initial queue/state reads corrected to docs/autonomy paths. No host changes.

## Remaining work and integration

Next: scanout ownership with pixel initialization, active commit/page-flip lifetime
and retirement, renderer pause ordering, direct input and production session entry.
Existing parent-leave and libseat disable-before-notify limitations persist.
Successful discovery, allocation, TEST_ONLY and resource retirement require a
DRM-equipped VM/development login; /dev/dri is absent here. Certified business
laptop and GPU workstation (24 GB VRAM or more) physical display/input, session
switching, suspend/resume and all hardware checklist evidence remain owed.
WSLg is development evidence only. Compositor and release remain unchecked;
all other v0.01 scope is retained. Supervisor full Windows/Linux/BPF publication
gates have not run for this change. No staging, commit, push or dev-loop edits.

All published reports were already referenced in STATE at iteration start;
no contributor report needed reconciliation. Existing evidence/limits retained;
reports arriving during publication reconcile next iteration. No Claude assignment,
other checkout, shared kernel/BPF/cgroup/service mutation or additional worker.
Proposed changes consolidated into CHANGELOG, ROADMAP, QUEUE and STATE; native
compositor implementation notes are updated in COMPOSITOR.md.
