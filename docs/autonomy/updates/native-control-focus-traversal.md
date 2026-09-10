# Native control focus traversal

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `4e2a78b`. Read CLAUDE, DELIVERY, SHARED_MAIN, report guidance,
current QUEUE/STATE, relevant v0.01 feature/roadmap sections, ADRs 0002/0010 and
the native-control contract. No AGENTS.md found. Every published task report
already had a STATE reference at iteration start; no reconciliation due.
Selected this component and acceptance checks in QUEUE before implementation.

The host can now traverse visible native control names in both directions or
select either endpoint. Minimise, maximise/restore and close retain their visual
order. Disabled names remain accessible; fully clipped controls are skipped,
partially visible controls retained. An unfocused strip starts at the relevant
endpoint, and traversal wraps. Missing/stale publication and competing pointer
ownership refuse and clear focus. Identical refresh preserves position.

Each request uses the existing focus setter to renew private selection identity,
including one-item wrap. A pending F1 opening therefore cannot survive traversal
back to the same action. This selects names without client keyboard focus,
input events or window-command effects. No agent API, new wording, palette,
configuration or engine change; ADRs 0002/0010 remain intact.

Paths under `crates/alo-shell/`:

- `src/window_control_focus.rs`: additive `WindowControlFocus` and
  `Server::navigate_window_control`; bounded three-item traversal without allocation.
- `src/window_control_presentation.rs`, `src/lib.rs`: live private traversal
  snapshot and public export, retaining the existing ownership/focus boundary.
- `tests/window_controls/focus.rs`, `mod.rs`: four private-client tests for
  order/wrap/endpoints, disabled and partial/full clipping, typing, competing
  client/native held buttons, cancelled release ownership, hide/reveal, remap,
  death, and F1 cancellation/opening on single-control traversal.
- `examples/support/nested_reader_frame_check.rs`: traverse the published strip
  before both existing F1 reader submissions, checking each selected action.
- `tests/window_controls/name_fallback.rs`: bounded request orchestration repair
  described below; server, vocabulary, target and every assertion preserved.

Rustdoc, the native-control contract and all four shared progress documents
describe the additive surface, evidence and remaining work.

## Verification

Ubuntu WSL2: existing bpffs confirmed with `findmnt -n -t bpf /sys/fs/bpf`;
WSLg socket `/mnt/wslg/runtime-dir/wayland-0` present. `pkg-config --modversion
wayland-client egl gbm libinput libudev xkbcommon` reports 1.24.0, 1.5,
26.0.8-1ubuntu0.3, 1.31.1, 259, 1.13.1. No dependency installation needed.
Private display resources only; no kernel mutation or outer machine lock.

Before every build/test/lint/format/doc command, measured `(Get-PSDrive C).Free`
and refused below `12GB`. All readings exceeded 53 billion bytes; lowest observed
reading: 53,245,390,848 bytes. Separate desktop Linux
target `/root/alo-os-target` retained. Reserve is headroom, not a running quota.

Linux prefix: `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env
PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`.
Commands:

```sh
cargo test -p alo-shell --locked control_focus -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
env RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
env RUST_BACKTRACE=1 cargo test -p alo-shell --locked name_fallback_publishes_all_pages_for_focus_hover_and_disabled_names -- --nocapture
cargo build -p alo-shell --example nested_check --locked
```

Four focused tests pass. Initial clippy refused unchecked array indexing; changed
to checked access, final clippy passes, including after the fixture repair.
Final full affected suite passes 168 unit, 262 lifecycle, three socket and four
compile-fail doctests, none ignored. Rustdoc and example build pass.

Windows: `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings`,
`cargo test -p alo-shell --locked --quiet` pass. Windows shell targets have zero
test cases because this crate is Linux-only. The Linux-only fixture repair was
formatted and verified by the final Linux suite/clippy.

With additional `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both pass first run. Reader evidence includes two traversal-selected F1 reader
EGL submissions, both schemes, with twelve existing page/feedback submissions,
two backend-owned readers, two automatic twelve-page fallbacks and pump calls.
Controls retains eight strip, two label, two expanded-name submissions and all
existing lifecycle/removal/refusal/recovery checks. Mesa driver-probe warnings
and intentional protocol-refusal diagnostics remain visible in logs; native
exit statuses are zero. No graphical timeout or changed graphical deadline.

## Failure investigation and targeted repair

Logs live in `.git/alo-loop/native-control-focus-traversal/`:
`focused.txt`, `clippy.txt`, `clippy-final.txt`, `clippy-repaired.txt`,
`linux-tests.txt`, `linux-tests-final.txt`, `linux-tests-repaired.txt`,
`fallback-diagnostic.txt`, `diagnostic-processes.txt`, `diagnostic-memory.txt`,
`rustdoc.txt`, `example-build.txt`, `windows-clippy.txt`, `windows-tests.txt`,
`fmt.txt`, `fmt-repair.txt`, `wslg-reader.txt`, `wslg-controls.txt`.

Original full suite failed in the existing
`name_fallback_publishes_all_pages_for_focus_hover_and_disabled_names`:
`tests/support/fixture.rs:113` timed out waiting for a backend request's reply
after three seconds; the backend subsequently hit SendError because the receiver
had exited. 261 other lifecycle tests passed. Process inspection after failure
found no surviving test process; WSL reported 5,351 MiB available. This does not
prove resource conditions at the instant of timeout.

Inspected fixture and test: one request bundled three independent selections,
each preflighting thirty pages before navigation and complete-page assertions.
Temporary elapsed-time instrumentation, then one isolated diagnostic run, measured
cumulative draw completion at 944.187 ms, 1.809122 s and 2.666046 s; complete backend
operation 2.666255 s, whole test 2.680943 s. Isolated assertions passed under the
original deadline. Removed instrumentation; the normal full suite reproduced the
same timeout. Thus the bundled request has measured little margin under its
three-second transport deadline. Scheduling/contention is a hypothesis, not a
proven original root cause; no production performance repair is claimed.

Targeted repair: submit each independent selection as one backend request,
carrying the same strings and accumulated target through all three on the same
server. All page counts, page ranges, navigation, out-of-range refusal, reader
dismissals, total target counts and no-close assertions remain. No fixture timeout,
suite concurrency, production code, gate or assertion was relaxed. Each complete
selection/preflight still must finish within the original three seconds. This
tests realistic separate host selection requests rather than packing three
independent host actions into one fixture RPC. The final full suite and clippy
pass after this repair; original failed logs remain preserved.

An initial PowerShell default-encoding rewrite affected historical QUEUE text.
Before further editing, reconstructed that file from the clean HEAD bytes plus
only this task's selected entry; inspected diff confirms history unchanged.
PowerShell renders native stderr as NativeCommandError records even on successful
commands; each actual native exit status was checked. No diagnostic suppression.

## Remaining work and evidence limits

Trusted focus traversal is complete. Navigation-key acquisition/release ownership
and ordered backend dispatch are the next component, followed by native cursor
selection and direct integration. This API alone does not deliver interactive
focus navigation or complete full-name access/window management. The host must
use reader navigation while a reading session owns the UI.

Evidence proves private-client routing and nested EGL submission, not actual
parent navigation-key delivery, submitted-frame readback, direct scanout, VM
integration or physical acceptance. Integrated VM and named laptop/GPU workstation
records remain owed in their delivery phases. No release box promoted.
Independent supervisor workspace/Windows/Linux/rustdoc/BPF gates have not run for
this task. Reports arriving during publication reconcile next iteration.

No cleanup, shared maintenance, mount/service/session change, WSL restart/helper,
second worker/loop, dev-loop edit, other checkout/repository or credential/identity
access. No staging, commit or push. Tracked and new diff inspected; final checks
are recorded in STATE.

Final cargo fmt --all --check and git diff --check pass; index empty.
