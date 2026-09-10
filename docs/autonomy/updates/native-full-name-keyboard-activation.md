# Native full-name keyboard activation

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration after recovery acceptance; supervisor gates pending.

## Change and decisions

Started clean at `1c629f0`. Read CLAUDE, DELIVERY, SHARED_MAIN, report guidance,
current QUEUE/STATE, relevant v0.01 features/roadmap, ADRs 0002/0010 and native
control contract sections. No AGENTS.md found. Every published report already
had a STATE reference; no reconciliation was due. Selected the component and
acceptance in QUEUE before implementation. No filesystem workstream changes.

The native-focused control's full name can now be opened with F1, once on release.
The nested host retains the reader for its existing rendering and navigation.
F1 was selected as local name help, with explicit native focus required so hover
does not take an application's help key. Any held client key, including modifiers,
refuses acquisition. This does not implement configurable desktop shortcuts or
native focus navigation. Disabled names remain readable; no window command runs.

Paths below are under `crates/alo-shell/`:

- `src/nested_reader_session.rs`: new borrowed session configuration and owned
  F1 transaction. Atomic all-page preparation happens at press; release installs
  an unpublished page-zero reader. Repeats do not reopen; cancelled releases drain.
  Existing reader navigation and normal typing route through the original adapter.
- `src/window_control_presentation.rs`: private native focus selection identity
  renews on every explicit focus request. Away-and-back cannot revive a gesture;
  reader mapping identity independently refuses retirement and republication.
- `src/keyboard.rs`: inspect whether any ordinary keys are held, preventing
  acquisition of client-owned F1 and modifier chords.
- `src/nested.rs`, `nested_control_input.rs`, `nested_reader_input.rs`, `lib.rs`:
  opt-in `pump_reader_session` uses the exact key adapter and ordered pointer/
  activation routing. Mode changes, input competition, errors and loss cancel
  opening, retaining release ownership. The old pumps remain available.
- `tests/window_controls/reader_opening.rs`: four real private-client tests for
  opening/repeats/submission/navigation/dismissal/typing, focus roundtrip and
  republication, pointer/key competition and loss, client F1/chords and disabled
  names, preparation error and mode-change release draining. Existing selection
  fixture helpers are shared without weakening their tests.
- `examples/support/nested_reader_frame_check.rs`: two new F1 adapter-to-EGL
  submissions, both schemes, and session-pump calls. All original assertions
  remain. Synthetic host key events do not prove actual parent F1 delivery.

Rustdoc and the native contract describe host obligations: preserve the slot,
render before acquiring reader input, remove dismissed pixels, and retire controls
and remove the old reader before changing preparation configuration. No new
wording, palette, engine patch or release scope. All four progress documents updated.

## Verification

Ubuntu WSL2 prerequisites: WSLg socket exists; existing
`findmnt -n -t bpf /sys/fs/bpf` reports mounted bpffs. Graphics pkg-config versions
for wayland-client, egl, gbm, xkbcommon, libinput and libudev respectively:
1.24.0, 1.5, 26.0.8-1ubuntu0.3, 1.13.1, 1.31.1, 259. No installation needed.
Private resources only: no kernel-mutating test fixture or outer machine lock.

Before every build/test/lint/format/doc command:

```powershell
$free=(Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

Minimum observed preflight: 53,294,166,016 bytes, above 12 GiB. This is operational
headroom, not a quota. Desktop Linux target remains `/root/alo-os-target`.

Linux commands invoked through `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env`
with `PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo check -p alo-shell --locked
cargo test -p alo-shell --locked reader_opening -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
env RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --example nested_check --locked
```

Check and four focused tests pass. Initial clippy fails on three undocumented
private items; added their documentation. Final all-target clippy passes. Full
affected suite passes 168 unit, 258 lifecycle, three socket and four compile-fail
doctests, none ignored. Rustdoc and example build pass. A final prose clarification
to the session configuration rustdoc followed these checks; no executable change.

Graphical commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir`
and `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
env ALO_NESTED_TRACE_SUBMISSION=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --controls --trace
env ALO_NESTED_TRACE_SUBMISSION=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --trace
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Reader passes: two new F1-opened reader submissions, twelve existing page/feedback
submissions, two explicit backend-owned readers and two automatic twelve-page
fallback submissions, both schemes, existing refusal/lifecycle checks and actual
pump calls. The baseline trace passes six-client lifecycle/refusal checks.
Both normal controls runs and its diagnostic trace fail the client deadline;
the 30-second outer timeout does not fire. This is a failed acceptance gate.

Windows commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

All pass. Windows shell tests have zero cases because the crate is Linux-only.
Independent supervisor workspace/Windows/Linux/rustdoc/BPF gates have not run
for this task. No staging, commit or push.

## Failure diagnosis and recovery handoff

Logs remain in `.git/alo-loop/native-full-name-keyboard-activation/`:
`check.txt`, `focused.txt`, `clippy.txt` (initial refusal), `clippy-fixed.txt`,
`linux-tests.txt`, `rustdoc.txt`, `example-build.txt`, `fmt.txt`, `fmt-check.txt`,
`windows-clippy.txt`, `windows-tests.txt`, and graphical logs below. Native exit
statuses were captured and checked; PowerShell's NativeCommandError wrappers
also appear around successful cargo/Mesa stderr and are not the exit status.

- `wslg-reader.txt`: passing new and original reader acceptance.
- `wslg-controls.txt`: initial client deadline failure before control markers.
  Connection-reset panics in the fixture child accompany teardown.
- `wslg-controls-trace.txt`: instrumented reproduction. Second zero-root
  submission binds in 35.512 microseconds, finishes painting at 2.542634 ms,
  returns from upstream swap at 17.010975888 seconds; loop then reports
  `client deadline exceeded` at about 17.0276 seconds. No control branch or new
  opening gesture has run. The fixture's existing internal deadline is 10 seconds.
- `wslg-baseline-trace.txt`: control comparison without optional control/reader
  branches passes, around 2.55 seconds. This demonstrates usable graphics at that
  time, not a repair or proof of the original cause.
- `wslg-controls-final.txt`: investigated normal acceptance rerun still fails
  before control markers. No further blind retries performed.

Read the deadline loop and upstream submit timing instrumentation. After failed
runs, `ps -eo pid,ppid,stat,etime,args` shows no surviving nested-check/debug
fixture process; WSLg socket and existing bpffs remain present. One initial
process-filter shell command failed because Ubuntu lacks rg and WSL quoting
split the filter; the corrected direct `ps` with PowerShell filtering succeeded.

Measured: the traced failure stalls inside upstream empty-scene submission before
controls execute. Unproven: why swap waits intermittently, and whether the two
untraced failures stall at the identical call. Earlier published reports record
similar delays; they do not establish this run's cause. No justified source-level
repair was identified. No engine patch/configuration change, timeout increase,
suppressed diagnostics, assertion change or test bypass was made. Recovery must
continue this same task and rerun its affected acceptance; do not select new work.

## Remaining evidence and scope

Controls graphical acceptance now passes in recovery; the protocol-level wait
is measured below, without claiming the original cause was repaired.
Native focus navigation, cursor selection and direct integration are
still subsequent components. Full-name access/window management remain unchecked.
Evidence is private-client routing and nested EGL submission, not submitted-frame
readback, synthesized parent key delivery, direct scanout, VM or hardware.
Integrated VM and named physical laptop/GPU workstation records remain owed in
their delivery phases. No release certification; reports arriving during
publication reconcile next iteration.

No shared maintenance, package/mount/service/session change, WSL restart/helper,
cleanup, second worker/loop, supervisor edit, other checkout/repository or
credential/identity access. Tracked and new code/tests/example diff inspected.

## Recovery investigation and acceptance (2026-09-10)

Recovered the same preserved dirty task. Reread the required guidance, published
report reconciliation, original logs and complete code/test diff before editing.
No published reports lacked STATE references. No source repair was made: available
evidence does not justify changing the F1 transaction or graphics engine.

Hypothesis investigated: the empty-scene swap waits for parent Wayland pacing or
buffer availability, rather than control preparation or new input dispatch.
Read the installed Smithay `WinitGraphicsBackend::submit`: it calls
`pre_present_notify` and EGL `swap_buffers`. Added per-process `WAYLAND_DEBUG=client`
to the existing submission trace, without modifying source or shared settings.
In `recovery-controls-protocol.txt`, lines 406-421, painting completes at 1.348 ms;
parent surface #18 requests a frame at protocol timestamp 3834352.308 ms. The next
parent events arrive at 3841836.549 ms, including buffer #43 release, followed by
Mesa callback #45 done at 3841845.894 ms. Mesa then attaches and commits buffer #50;
swap returns at 7.495270185 s. This measures the parent event wait in this run.
It does not identify why the parent delayed, or prove the original untraced
failures had the same cause. The original 17.010975888-second failure is retained
above, unresolved. Read-only Weston tail showed window association messages,
not a specific error explaining the wait. No surviving nested-check process.
`strace` is absent; no shared package installation was needed for protocol tracing.

The diagnostic controls run passes all assertions, followed by one normal controls
acceptance and the affected reader acceptance, both passing. Same 30-second outer
timeout and 10-second internal deadline, no altered assertions or suppression.
No claim that a passing rerun fixes an intermittent graphics delay.

Exact recovery commands (same WSL environment/prefix as Verification above):

```sh
env WAYLAND_DEBUG=client ALO_NESTED_TRACE_SUBMISSION=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --controls --trace
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
cargo test -p alo-shell --locked --quiet
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
env RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Windows: `cargo fmt --all --check` and `git diff --check` pass. Existing Rust
formatting from the original attempt remains unchanged. Tests pass 168 unit,
258 lifecycle, three socket, four compile-fail doctests; none ignored. Clippy
and rustdoc pass. Controls includes eight strip, two label, two expanded-name
submissions, dismissals/removals and capacity/strip refusal-recovery. Reader
includes all original checks plus two F1-to-reader submissions and session pumps.
No executable changes in recovery, so the previously built example was reused.

Recovery logs share the original directory: `recovery-controls-protocol.txt`,
`recovery-controls-acceptance.txt`, `recovery-reader-acceptance.txt`,
`recovery-linux-tests.txt`, `recovery-clippy.txt`, `recovery-rustdoc.txt`,
`recovery-fmt-check.txt`. Every native exit status is zero. C: preflight before
each test/lint/doc/format command remained above 12 GiB; minimum recovery reading
53,252,653,056 bytes. Verified existing bpffs, WSLg socket and all six graphics
pkg-config prerequisites. No kernel mutation or shared maintenance performed.

Updated contract and all four progress documents to ready for integration, with
this uncertainty retained. Supervisor independent workspace/Windows/Linux/BPF
gates remain pending. Machine evidence limits above remain unchanged; no release
or full window-management completion claim. No staging, commit or push.
