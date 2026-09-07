# Direct-display session lifetime

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Responsible contributor: single Codex desktop worker in `C:\dev\alo-os`
- Status: ready for integration as a completed device-lifetime component;
  direct-display backend and compositor remain unfinished

## Change and decisions

The shell now obtains its display descriptor through the login seat, retires it
on pause and reacquires it only while active. `session_device.rs` owns lazy
acquisition, scoped descriptor access, explicit shutdown and drop cleanup.
`direct_session.rs` owns the pinned Smithay libseat notifier and calloop dispatch.
`session_device_check` connects that owner to existing DRM output discovery.
No raw path-open fallback, VT switch, modeset, daemon/agent verb or context API.
Public Rust methods are documented; external adapter/agent contracts are unchanged.

Follow ADRs 0001/0002: trusted native shell plumbing, unmodified pinned engines.
One configured device is sufficient for this delivery component. Pause destroys
the old descriptor rather than allowing cached discovery to imply continuing
authority. Acquisition, close and reported notifier failures require a fresh
session; activation cannot erase them. Explicit shutdown reports cleanup errors;
drop is a best-effort backstop. libseat determines permissions and open semantics
(Smithay's backend ignores the requested open flags). No unsafe code or lockfile
version changes. Required native libraries were already installed.

Smithay forwards seat callbacks into a calloop channel. Two nonblocking readiness
passes drain that handoff before synchronous access. A real calloop channel test
forwards pause/activate in one batch and verifies original close errno delivery,
actual descriptor closure and terminal refusal on subsequent polls/access.
Kernel revocation can still occur during an ioctl; callers must handle its error.

This is a discovery-stage owner, not a renderer lifecycle implementation. Callers
must poll while idle and must not retain duplicated descriptors or scanout objects
across polls. A later renderer needs ordered teardown/rebuild and a resource
generation/lifetime design. Pinned Smithay 0.7.0 acknowledges disable before the
shell callback and uses internal unwraps for several dispatch/disable/registration
failures. No such panic was triggered here; these are source-inspected limits,
not a claim of graceful handling of every libseat failure. Recorded in quirks;
production failure recovery and renderer pause ordering remain open without an
engine patch or a decision conflicting with the accepted ADR.

## Executed verification

PowerShell in `C:\dev\alo-os`, all exit 0:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Ubuntu WSL2, separate target `/root/alo-os-target`, all exit 0:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --lib session_device
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --examples --locked
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

The first focused test ran eight new lifetime cases; the final suite includes the
ninth, calloop integration case: 19 unit + 64 real-client lifecycle + 3 socket
tests = 86 passed, zero ignored. Tests cover lazy/repeated acquisition, initial
inactivity, independent backend inactivation, pause/activate, open/close errno,
notifier loss, explicit shutdown and drop. Real Unix socket peer EOF proves fd
closure even when the injected manager close operation reports failure. Windows
intentionally excludes these Linux tests. No build/test/lint failures or relaxed
gates. Final checks ran after the event-loop handoff correction.

Additional real-library refusal (expected exit **1**, asserted by PowerShell):

```powershell
wsl -d Ubuntu -- env LIBSEAT_BACKEND=seatd SEATD_SOCK=/run/alo-session-check-no-seat.sock timeout 15s /root/alo-os-target/debug/examples/session_device_check /dev/dri/card0
```

Result: connect-stage ENOENT (2), no device operation or fallback. Pinning the
diagnostic to an absent seatd socket avoids activating any host session or embedded
seat backend. Local log: `.git/alo-session-device-refusal.log`. WSLg regression
log `.git/alo-session-device-wslg.log` records 115 client surface submissions,
popup/reactive/cursor callbacks, unmap/remap, isolated refusal and disconnect.
This is scripted protocol/GLES submission, not pixel readback or physical input.

Prerequisites: WSLg socket exists, no `/dev/dri/card0`; pkg-config reports libseat
0.9.2, libudev 259, GBM 26.0.8-1ubuntu0.3, EGL 1.5 and xkbcommon 1.13.1.
An initial shell PATH probe mishandled inherited space-containing PATH; subsequent
commands used explicit `env PATH=...`. WSL also emitted existing unknown-key
configuration diagnostics; no host configuration was read or changed to fix them.
No package install, shared kernel/cgroup/BPF/service mutation or worker launch.

## Reconciliation and remaining acceptance

At iteration start every published task report was already referenced in STATE;
no report required consolidation. Reports arriving during publication reconcile
next iteration. Own report consolidated into all four shared progress documents.
Acceptance was selected in QUEUE before implementation; no Claude assignment
taken, delegation, other-checkout edits, staging, commits or pushes.

Next executable component: atomic DRM property/capability discovery and test-only
configuration validation, then scanout/page flips and direct input. Running-renderer
pause ordering, parent leave and production session entry remain. Successful
libseat acquisition/reacquisition and DRM discovery need a DRM-equipped development
login or VM; physical acceptance needs the named ordinary laptop and 24-GB-or-larger
GPU workstation records in docs/hardware.md, including real session switching,
native display, input and suspend/resume. None is certified here. Other v0.01 work
is preserved. Supervisor full Windows/Linux/BPF publication gates have not run
for this change; release and compositor boxes remain unchecked.
