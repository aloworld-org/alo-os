# Direct-display resource discovery

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Responsible contributor: single Codex desktop worker in `C:\dev\alo-os`
- Status: ready for integration as a completed discovery component, not a
  completed direct-display backend or compositor

## Change, decisions and acceptance

The shell can choose a connected display, advertised progressive mode and
compatible CRTC using a caller-owned DRM session descriptor. Query errors retain
their kernel cause; unusable or absent outputs refuse. This is discovery only:
it does not reserve resources, acquire master, configure client capabilities,
force-probe a connector or change scanout. No agent IPC/verb/context surface,
scope expansion, unsafe code or upstream patch is introduced (ADRs 0001/0002).

`crates/alo-shell/src/drm_inventory.rs` owns resource/connector/encoder ioctls.
`direct_output.rs` owns policy and public diagnostics. Internal eDP/LVDS/DSI
panels are preferred for the laptop-first release, followed by stable connector
ID, not driver iteration order. A valid advertised preferred mode wins; otherwise
the first valid mode in the driver's order wins. Timings are retained verbatim.
Disconnected/unknown and writeback ports, missing modes or compatible CRTCs,
interlaced/doublescan/stereo/multiscanned and malformed timings are ineligible.
The lowest compatible CRTC ID is only a possible route, never a free-resource
claim. The 32-bit encoder mask bound is checked before upstream filtering.

Pinned `drm = 0.14.1` is already Smithay's dependency; only alo-shell's dependency
edges changed in Cargo.lock. Its existing `drm-ffi = 0.9.1` is a test-only direct
dependency to construct realistic kernel mode records through safe conversions.
Seven tests in `direct_output_tests.rs` cover happy selection, order independence,
exact timings, fallback, missing/disconnected resources, unsupported modes,
hot-unplug between snapshots, preserved query errors, and real kernel ENOTTY
without closing the borrowed descriptor. The synthetic snapshots do not emulate
successful hardware ioctls or prove actual encoder wiring.

`examples/direct_output_check.rs` is a developer diagnostic for an explicitly
named session-owned development card. It opens read-only and performs no modeset;
opening a primary node can implicitly give its first opener DRM master. Production
must instead obtain the descriptor through session management. This iteration
opened only `/dev/null` and attempted the nonexistent `/dev/dri/card0`.

Acceptance was recorded in QUEUE before code. Parent-leave remains unavailable
through pinned Smithay's Winit wrapper, so this bounded direct-display component
advances the next executable part of delivery step 2 without an engine patch.
No additional approval was needed. Single worker, clean initial tree, supervisor
owns publication; no stage/commit/push or other checkout modifications occurred.

## Executed verification

PowerShell in `C:\dev\alo-os`, all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, independent target `/root/alo-os-target`, all passed:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --lib direct_output
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --examples --locked
wsl -d Ubuntu -- /root/alo-os-target/debug/examples/direct_output_check /dev/null
wsl -d Ubuntu -- /root/alo-os-target/debug/examples/direct_output_check /dev/dri/card0
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

The two direct diagnostic commands intentionally exit 1: resource query ENOTTY
(25), and missing card ENOENT (2), respectively. The harness checked both exit
codes equal 1. All other commands exit 0. The Linux shell suite passes 77 tests:
10 unit, 64 real-client lifecycle, 3 socket ownership; none ignored. Windows
intentionally excludes Linux tests. No test/clippy failures or gate relaxation.
The first focused command updated only the two lockfile dependency edges.

Additional integration: real non-DRM kernel ioctl refusal is exercised by both
the library test and executable. WSLg regression submits 115 client surfaces;
popup/cursor, reactive placement, callbacks, unmap/remap, isolated refusal and
disconnect pass. Local logs: `.git/alo-direct-output-tests.log`,
`.git/alo-direct-output-refusal.log`, `.git/alo-direct-output-missing-card.log`,
`.git/alo-direct-output-graphics.log`. These are scripted protocol/GLES submissions,
not pixel readback, actual parent input, successful DRM discovery or certification.

Prerequisites checked: WSLg socket `/mnt/wslg/runtime-dir/wayland-0` exists;
pkg-config reports wayland-client 1.24.0, libinput 1.31.1, libudev 259,
GBM 26.0.8-1ubuntu0.3, EGL 1.5. `/dev/dri` is absent. No packages needed installation.
No kernel/cgroup/BPF pins/services or unrelated host settings were changed.

## Report reconciliation and remaining work

At iteration start three published reports were not referenced in STATE:
`kernel-supervisor-runs-gates-through-wsl.md`,
`network-egress-enforcement-policy.md`, `network-egress-gap-reproduced.md`.
Reviewed each and relevant source/tests; integrated their evidence and limits
in STATE, ROADMAP and QUEUE. No user-visible security behavior changed, so their
requested absence of a changelog announcement is preserved. Contributor Linux/
BPF gates are reported evidence, not rerun in this desktop task; Windows was not
run for the network reports. Corrected supervisor end-to-end publication is not
inferred from the report's successful gates/local commit. Source reports unchanged;
reports arriving during publication reconcile next iteration.

All four progress documents and COMPOSITOR now record this discovery component.
Next: session device acquisition and pause/resume lifetime, then atomic test/commit,
page flips/scanout, direct input and production entry. Parent-leave support remains
pending without a patched engine. Successful resource ioctls need a DRM-equipped
VM or development machine; physical acceptance still needs the ordinary laptop
and 24-GB-or-larger GPU workstation required by `docs/hardware.md`, including
native-resolution boot, input, suspend/resume and the rest of the release checks.
This absence does not exhaust independent implementation work. Full supervisor
Windows/Linux/BPF publication gates have not run for this change. Compositor and
release remain unchecked; no hardware claim. No worker/loop launch, delegation,
dev-loop edit, credential access, physical disk installation or unrelated change.
