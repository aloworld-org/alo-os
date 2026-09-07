# Direct-display resource ownership

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Contributor: single desktop worker in `C:\dev\alo-os`
- Status: ready for integration; independent supervisor gates pending

## Change and decisions

`display_resources.rs` owns one full-mode XRGB8888 dumb buffer, its registered
framebuffer and an exact advertised mode blob. `resource_device.rs` contains the
pinned drm-rs ioctl transport. The public `DisplayResources` borrows the session
descriptor, exposes IDs for future TEST_ONLY use, and provides explicit release
with every cleanup error retained. Partial allocation unwinds in reverse order;
one failed destruction does not skip the others. Drop is best effort. Release
attempts each handle once; errors require device retirement, not continued reuse.

Dimensions and advertised format are checked before allocation; returned size,
format and pitch are checked before framebuffer registration. Invalid blob IDs
are refused without truncating them into destruction targets. A compile-fail
doctest prevents resources escaping their owning fd's lifetime. The candidate
must come from fresh discovery on that same descriptor; the Rust snapshot alone
does not prove device identity or reserve the output.

The existing `atomic_output_check` gains optional `--allocate` and explicitly
releases its unbound resources. Its direct path open is a developer diagnostic;
production must allocate inside `DirectSession::with_device`. No successful
card open occurred here. Opening a primary card can implicitly acquire master,
as described by the [kernel DRM userland documentation](https://docs.kernel.org/gpu/drm-uapi.html#primary-nodes-drm-master-and-authentication).

This is a complete allocation/lifetime component, entered in QUEUE before code.
Atomic TEST_ONLY request construction remains the next component. No mapping,
pixel initialization, atomic commit, scanout or rendering is implemented here.
These resources must stay unbound: active scanout retirement needs a separate
owner. Dumb allocation is a small candidate for validating an initial full-mode
configuration, not a replacement for the Smithay production renderer. Depth-24,
bpp-32 ADDFB registers XRGB8888; it is not a legacy modeset fallback. Other formats,
modifiers and hardware constraints require subsequent kernel validation.

Native Rust and unmodified pinned engines follow ADR 0002. No new dependency,
unsafe repository code, agent verb, UI string or adapter contract was introduced.
Public Rust additions have rustdoc. Source inspection of drm-rs 0.14.1 confirms
Mode preserves the raw kernel timing struct; allocation helpers assume nonzero
kernel handles and hide the dumb allocation length. Quirks documents these
limits; normalized fault injection is not raw malformed-kernel containment.

## Executed verification

Windows PowerShell in this checkout, final checks exit 0:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2 with this checkout's separate target directory, final checks exit 0:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --lib display_resources --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --examples --locked
```

Initial focused tests passed seven cases; final full shell suite includes eight
new unit cases plus the lifetime doctest: **35 unit + 64 lifecycle + 3 socket +
1 doctest = 103 passing**, none ignored. Tests cover happy release/drop, exact
mode preservation, padded stride, every allocation-stage failure, invalid
format/dimensions/layout/blob IDs, simultaneous cleanup failures, and real
CREATE_DUMB refusal with descriptor survival. Windows excludes Linux shell tests.
Initial Linux clippy refused three collapsible cleanup conditionals; corrected
without exemptions, and final clippy/tests passed. No tests failed or were weakened.

Additional integration, with exit codes asserted in PowerShell:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --lib real_non_drm_allocation_refuses --locked -- --nocapture
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/dri/card0 --allocate
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/atomic_output_check /dev/null --unknown
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Respectively: exit 0, CREATE_DUMB ENOTTY (25) and caller fd survives; expected
exit 1, absent card ENOENT (2); expected exit 1, usage refusal before device open;
exit 0, **115 client surfaces** submitted with popup/reactive/cursor callbacks,
unmap/remap, isolated refusal and disconnect. Logs are local under
`.git/alo-display-resources-{refusal,absent-card,argument,wslg}.log`.
GLES submission is not pixel readback or physical input evidence.

Prerequisites checked: WSLg socket exists; `/dev/dri` does not. pkg-config reports
libseat 0.9.2, libudev 259, GBM 26.0.8-1ubuntu0.3, EGL 1.5 and xkbcommon 1.13.1.
No installation needed, no shared kernel/cgroup/BPF/service mutation. Initial
document/path searches were corrected to actual repository paths; no other
checkout was inspected or changed.

## Integration and remaining work

At iteration start, every published task report was already referenced in STATE;
no unreconciled report required consolidation. Existing contributor evidence and
limits remain unchanged. Own report is consolidated into CHANGELOG, ROADMAP,
QUEUE and STATE; COMPOSITOR and quirks also updated. Reports arriving during
publication are for the next iteration. No Claude assignment taken or worker launched.

Next component: construct the full-mode atomic TEST_ONLY request using these
owned resources; test kernel refusal, cleanup and stale/incomplete snapshots.
Then scanout/page flips, renderer pause ordering, direct input and production
entry. Existing parent-leave and libseat disable-order limitations remain.
Successful DRM allocation/destruction and atomic tests require a DRM-equipped
development login or VM. Certified laptop and 24-GB-or-larger GPU workstation
display/input, session switching, suspend/resume and all hardware checklist
records remain owed. WSL does not certify hardware. Full v0.01 scope preserved;
compositor and release stay unchecked. Supervisor full Windows/Linux/BPF gates
have not run for this change. No staging, commit, push or dev-loop modification.
