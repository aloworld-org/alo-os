# Explicit output retirement

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop/integration worker in C:\dev\alo-os.
Status: **recovered, full publication gates passed**. The owner explicitly
authorized correcting the test and checking the feature. Original failures below
are preserved; see the recovery section for current evidence.

## Change and decisions

Added `FrameTarget::retire` (legacy targets explicitly refuse), direct target
terminal retirement, and `Server::retire_output`. The server validates identity,
then retires the backend before sending leaves and withdrawing the output global.
Failed retirement preserves protocol state and pending callbacks. Success clears
popup output constraints and allows a new output lifetime. Direct retirement
attempts halt further submission and never retry a failed disable on drop.

Source: `crates/alo-shell/src/output_retirement.rs`, `presentation.rs`,
`direct_target.rs`, `server.rs`, `output_metadata.rs` and their protocol tests.
This is trusted compositor plumbing under ADR 0002, not an agent verb or adapter
contract change. No engine patch or release scope change. Session pause here is
a future caller of the explicit operation, not an implemented seat event loop.

Use Wayland `disable_global`, retaining inert binding data until display teardown,
because immediately destroying the global can disconnect clients with queued
binds. The API still emits `global_remove`. This retains one inert global per
retired lifetime; reclamation before display teardown is not implemented.

Acceptance recorded in QUEUE before implementation: real leave/global-removal,
pending callback, fresh lifetime and failed-disable checks; no repeated backend
I/O; WSLg regression; affected formatting, lint, tests and rustdoc.

## Original worker verification and halt

Ubuntu WSL2: Rust 1.98.0; WSLg socket at
`/mnt/wslg/runtime-dir/wayland-0`; pkg-config Wayland client 1.24.0, EGL 1.5,
GBM 26.0.8-1ubuntu0.3, libinput 1.31.1 and libseat 0.9.2. No `/dev/dri`.
Initial login shell lacked Rust on PATH; sourcing `/root/.cargo/env` verified
the installed compiler. No dependencies installed or host configuration changed.

Linux commands use `/mnt/c/dev/alo-os`,
`PATH=/root/.cargo/bin:/usr/lib/llvm-22/bin:/usr/sbin:/usr/bin:/sbin:/bin`,
`LLVM_PREFIX=/usr/lib/llvm-22`, `CARGO_TARGET_DIR=/root/alo-os-target`.

1. `cargo fmt --all` passed. `cargo test -p alo-shell --locked output_retirement
   -- --nocapture` initially failed `UnsafeRuntime`: tempfile's permissions did
   not meet the server's existing 0700 requirement. Corrected only the fixture's
   permissions, preserving the security check.
2. The same fmt and focused test commands passed: the real Wayland client checked
   successful disable/withdrawal/rebind, callback retention, failed disable and
   terminal no-retry behavior (one test, both scenarios).
3. Added no-I/O retirement and expanded delayed-bind/new-identity assertions.
   A compile attempt failed because DRM connector handles require `NonZeroU32`,
   not `TryFrom<u32>`; corrected that construction. Local initial log:
   `.git/output-retirement-linux.log`.
4. Final `cargo fmt --all` passed. The same focused test command ran two tests:
   unused-target retirement passed; real protocol test failed with
   `Scanout(ResourceError { failure: ResourceFailure { stage:
   "validate prepared scene", source: Kind(InvalidData) }, cleanup: [] })`.
   Log: `.git/output-retirement-linux-final.log`. Command chain exited 1.
   This was the second runtime test failure this iteration; no implementation
   retry followed, under the owner's explicit halt instruction.

Read-only diagnosis: the new fixture sets connector 2, which aliases its CRTC 2.
`AtomicPlan::new` correctly rejects duplicate connector/CRTC/plane IDs. The
injected `ScanoutDevice` also compares against connector-1 wire requests, so
changing only the connector is not a coherent replacement fixture. Fix the
fixture on authorized resumption; do not relax either production validation or
the existing transport assertions. The final expanded delayed-bind assertion is
not a completed acceptance result because the combined test failed.

Windows, final source: `cargo fmt --all --check`,
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`,
`cargo test -p alo-shell --locked` all exited 0. Linux shell tests are cfg-excluded
on Windows (zero executed tests), so this is not retirement runtime evidence.
Reviewed tracked diff and the new retirement module; `git diff --check` passed.

The failed Linux chain did **not reach**:
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`,
`cargo test -p alo-shell --locked`,
`RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked`, or
`cargo build -p alo-shell --examples --locked`.
Final Linux fmt check and WSLg offscreen/nested/invalid-EGL regressions also remain
owed. Supervisor full Windows/Linux workspace, rustdoc and BPF gates were not
run by this worker. No staging, commit or push.

## Remaining work and progress integration

Owner-authorized recovery has corrected the changed-identity fixture and passed
full publication gates. The explicit retirement component is ready to publish.
Automatic seat pause handling, direct renderer/input/session integration,
post-descriptor-failure recovery, safe asynchronous transport and GPU context-loss
evidence remain unfinished. Physical laptop and GPU workstation acceptance are
still required; injected DRM and WSL cannot certify scanout or hardware.

CHANGELOG, ROADMAP, QUEUE and STATE record this as unfinished, with no release
checkbox changes. All published reports at iteration start were already referenced
in STATE. No Claude work taken, other checkout touched, worker launched,
tools/dev-loop changed, shared kernel tests run or physical installation performed.

## Owner-authorized recovery

The desktop supervisor was HALTED and no supervisor process remained. Preserved
the entire worker change. Reproduced the failure on Linux: unused retirement
passed and the expanded wire test returned InvalidData at validate prepared scene.
Changed the replacement connector to 4, distinct from CRTC 2 and plane 3. The
injected Device now has an independently configured expected connector; the
replacement device expects 4 and existing fixtures still expect 1. Atomic wire
requests are still compared exactly, including allocations, properties and flags;
the expected connector is never derived from the request being checked.

All eight direct-target tests passed after repair. Added an explicit regression
that connector aliases of either CRTC or plane still refuse before DRM I/O.
Expanded the wire test to prove wrong-target retirement refuses before backend
I/O, and that dropping a target cannot repeat its disable. Existing success,
failed disable, pending callbacks, late binding and fresh identity assertions
remain intact. A first additional assertion attempted to access a private field;
replaced it with an assertion through the public retirement operation, without
changing visibility. Clippy caught a direct panic in the new refusal test;
changed the test to return an error instead of adding a lint exemption.

Integrated published main through b010137 by fast-forward with local desktop
changes preserved; no overlapping files or conflicts. Focused tests passed on
that combined tree. No production-code workaround was needed for the fixture.

Commands already passed, using the documented Linux PATH and target directory:

```sh
cargo test -p alo-shell --locked direct_target_tests -- --nocapture
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Eight actual focused tests passed; the other test binaries contained no matches
and are not counted as evidence. Offscreen golden pixels and refusal paths pass;
nested regression passes with 115 client surfaces. Logs are local:
`.git/output-retirement-recovery-offscreen.log` and
`.git/output-retirement-recovery-nested.log`. These use real Wayland clients and
WSLg GLES; DRM retirement uses injected transport, not physical scanout.

Full publication gates passed after integrating b010137:

```sh
# Windows and Linux:
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
# Linux only, with LLVM_PREFIX=/usr/lib/llvm-22 and documented PATH/target:
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
# From crates/alo-bounding-kernel, pinned toolchain:
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
```

Both platform chains exited 0: 121 successful result groups each, including
cfg-excluded/zero-test groups. Existing ignored tests remain ignored; group counts
are not acceptance-test counts. Linux rustdoc and pinned BPF checks exited 0.
Logs: `.git/output-retirement-recovery-windows.log` and
`.git/output-retirement-recovery-linux-verified.log`. The earlier Linux recovery
log records the lint failure, not a pass. The supervisor's bpffs precondition was
checked; no other process's pins were removed.

All 11 desktop supervisor tests pass, including failed integrated-gate refusal.
Windows fmt/clippy reran successfully after the test correction. Invalid EGL
still exits 1, logged in `.git/output-retirement-recovery-egl-refusal.log`.
Final public-doc wording now describes identity lasting until output retirement,
not necessarily server destruction. The final affected Linux fmt, all-target
clippy, tests and rustdoc chain exited 0 after those doc edits: 105 unit, 67
lifecycle, 3 socket and 2 doctests, 177 checks total, none failed or ignored.
Evidence: `.git/output-retirement-recovery-shell-final.log`.

Shared progress documents incorporate recovery and the newly published kernel
audits without declaring their gaps closed. Claude's decision-proposal prompt
is `../claude-network-decision-proposal.md`. Its checkout was not edited or
started. Publication/restart follows a clean normal push, never a forced push.
Physical acceptance and automatic session wiring remain explicitly outstanding.
