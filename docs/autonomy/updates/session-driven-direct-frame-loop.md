# Session-driven direct frame loop

Date: 2026-09-08. Workstream: native desktop compositor.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; one component, not a completed compositor.

## Change and decisions

`crates/alo-shell/src/direct_loop.rs` adds `DirectSession::run_compositor`:
fresh atomic discovery, a GLES `DirectTarget` borrowing the active descriptor,
and synchronous seat polling, client dispatch and rendering. A trusted scheduler
chooses frame timestamps, idle dispatch or stop. Seat polls before and after the
scheduler prevent a pause during pacing from permitting another frame. Idle
iterations still poll. The target is owned by the loop and retired/dropped before
the active scope closes its descriptor. There is no automatic retry or resume.

The private target health check stops immediately after a committed frame with
failed resource cleanup, even if the scheduler would otherwise remain idle.
Loop, retirement, protocol-flush and descriptor-close outcomes remain separate.
Retirement flushes queued output events without further client dispatch. Failed
disable preserves advertised state and quarantines resources under the existing
retirement policy; discard the server before recovery. Flush does not guarantee
that a blocked client has consumed events. No agent API or context capture is added.

This is the next native Rust component under ADR 0002 and the v0.01 compositor
feature/roadmap. Caller-owned, current GLES rendering keeps graphics creation
separate from seat ownership: the renderer must not retain/duplicate the scoped
DRM descriptor. Native renderer creation, direct input and boot entry are still
required. No engine patch or new release scope; no public agent, adapter, daemon,
configuration or image contract changed. Errors are internal diagnostics, not
hardcoded session UI strings. The libseat early-disable acknowledgement limit
in `docs/quirks.md` remains; this code cannot restore revoked kernel authority.

## Verification

Ubuntu WSL2: Rust 1.98.0 (88d9e12ae 2026-08-18). pkg-config confirms
Wayland 1.24.0, EGL 1.5, GLES 3.2, xkbcommon 1.13.1, udev 259,
libinput 1.31.1, GBM 26.0.8-1ubuntu0.3 and libseat 0.9.2. WSLg socket
exists; `/dev/dri` is absent. No dependency install or shared kernel changes.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked direct_loop
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
# XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0:
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
# Same WSLg environment; expected exit 1:
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-egl-vendor.json \
  timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Final checks pass: four focused tests, then 115 shell unit tests, 67 client
lifecycle tests, three socket tests and three compile-fail doctests. The production
transaction/target tests exercise stop, idle, polling on either side of scheduling,
replacement refusal, post-commit cleanup failure, single disable and preservation
of advertised output on failed disable. The additional real-descriptor/calloop
integration injects pause plus activation during both drawing and idle pacing;
retirement and target drop write distinct bytes through the live descriptor,
then the peer observes exactly one manager close and EOF. This does not claim
real libseat/DRM execution. The existing real Wayland protocol suite also passes.

Affected Linux clippy, rustdoc and example builds pass. WSLg offscreen golden
pixels/refusals pass; nested popup/cursor regression submits 115 client surfaces.
Both standalone regressions exit 0; invalid EGL exits 1 as expected. Existing
Mesa diagnostics and deliberate invalid-client protocol errors remain expected.
Windows `cargo fmt --all --check`,
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`, and
`cargo test -p alo-shell --locked` pass; this Linux-only crate runs zero tests
on Windows. Final diff review and `git diff --check` pass.

Initial focused tests refused unsafe temporary runtime directories: fixtures now
explicitly set mode 0700, following the existing socket contract, without changing
the safety check. Initial clippy found a test unwrap; the fixed scheduler defaults
to Stop and still asserts the exact poll and frame counts. A local Linux check
script initially had CRLF line endings that corrupted shell arguments and an
environment value; that invocation is invalid evidence. The corrected LF script
passes all checks in `.git/direct-loop-final-checks.log`. PowerShell's redirected
native stderr marked its wrapper unsuccessful despite completed checks; direct
WSLg invocations confirmed exit statuses. The earlier log
`.git/direct-loop-checks.log` retains the invalid invocation. No repeated test
failure, lint exemption or weakened assertion.

## Remaining work and progress integration

Next component: native offscreen GLES initialization with explicit ownership and
initialization/refusal evidence, followed by direct input and boot/session entry.
Successful physical DRM scanout, real seat pause/resume, failed-disable recovery,
GPU context loss, asynchronous transport and certified laptop/GPU-workstation
acceptance remain owed. WSLg is a development fixture, never hardware certification.

All four shared progress documents track this component without ticking the
compositor or release. At iteration start all published report filenames were
already referenced in STATE.md; no new report required reconciliation. Claude's
network request-boundary work remains assigned to Claude. Reports arriving during
publication belong to the next iteration. No stage, commit, push, other checkout
changes, worker launch or tools/dev-loop changes. Full independent workspace,
rustdoc and BPF publication gates belong to the supervisor.
