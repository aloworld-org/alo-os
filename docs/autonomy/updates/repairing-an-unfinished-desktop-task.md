# Repairing an unfinished desktop task

Date: 2026-09-09. Owner: desktop integration worker. Scope: the existing
development supervisor and its preserved adaptive-label task, not OS policy.

## Continue after a repairable failure

The owner asked the loop to solve blockers and continue. `STEP BLOCKED` now
requests another worker on the **same** dirty task, with the previous logs and
result. A failed independent gate uses the same path. Each repair must investigate
a specific hypothesis, preserve evidence, and rerun affected acceptance. Every
reported completion is followed by the whole independent gate again. There are
at most three repair workers per task/integrated failure, not endless blind retries.

Failed combined-tree verification can also enter recovery after integration.
The repair gets its own commit only after all gates pass; neither original nor
repair is pushed on failed verification. No worker stages, commits or pushes.

The OS-held single-writer lock, fixed remote/main checks, unchanged HEAD/index,
supervisor-edit refusal, owner STOP, disk reserve and read-only mount preflight
remain. Missing authority, unsafe Git state, process/authentication failures,
lost WSL lease, shared maintenance and exhausted recovery preserve work and ask
for a handoff. Recovery never authorizes a mount, another checkout's cleanup,
Windows changes or weakened acceptance. `STEP NEEDS INPUT` makes that distinction
explicit. Attempt logs remain local, under `repair-N` and `integration-N`.

## Preserved graphical work, investigated

The earlier `nested_check --controls` timeout remains unexplained. A controlled
run with `WAYLAND_DEBUG=client` passed and showed buffer submission and teardown;
then a normal run passed. Nothing was mounted, restarted or disabled. A pass
does not prove why the earlier command timed out.

The normal offscreen run exposed a separate failure: the client exhausted its
five-second acknowledgement wait while the server was still running the entire
light/dark native-scene matrix. That matrix subsequently printed its successful
pixel checks, followed by a closed-channel error. This observes the missed
acknowledgement, not the underlying reason this run was slow.

The two independent schemes now have explicit pre-submission stages. Neither
may publish an output or complete a callback. Every pixel, refusal, removal and
recovery assertion remains; the original 28 stages remain, plus two scene
stages. The per-stage five-second, overall fifteen-second and outer thirty-second
deadlines are unchanged. No client assertion was dropped or test ignored.

After rebuilding, `nested_check --offscreen` passed all 30 stages, including
eight custom-cursor, four arrow and two expanded-label complete 57,600-pixel
comparisons. `/usr/bin/time` reported 1.50 seconds. This is successful recovery
evidence, not proof of host scheduling or the original EGL timeout's cause.
Existing EGL/Mesa diagnostics remain visible even on passing runs.

## Verification

Windows developer-tool fmt/clippy (`--all-targets -- -D warnings`) and tests:
**29 passed**, none ignored. Seven new tests cover blocked-worker repair, fresh
gating after failure, integrated recovery, exhausted budget, non-retryable
handoffs, strict result parsing, and a real isolated Git remote that stays
unchanged until the repaired combined tree passes. The latter uses a controlled
gate callback, not a claim that a real compiler failure was induced.

Full combined-tree product verification **passed** on 6704741 plus this change:

```text
Windows: cargo fmt --all --check
         cargo clippy --workspace --all-targets --locked -- -D warnings
         cargo test --workspace --locked --quiet
Linux:   same fmt/clippy/test commands with CARGO_TARGET_DIR=/root/alo-os-target
         RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
BPF:     cargo fmt --all --check
         cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
         (crates/alo-bounding-kernel, its pinned nightly)
Graphics: cargo build -p alo-shell --examples --locked
          timeout 30s control_labels_check
          timeout 30s window_controls_check
          timeout 30s nested_check --offscreen
          timeout 30s nested_check --controls
```

Graphics used the binaries in `/root/alo-os-target/debug/examples`, with
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and `WAYLAND_DISPLAY=wayland-0`.
Both normal nested checks passed again in this full run. Windows and Linux
workspace commands exited zero; existing explicitly ignored environment/model
tests remain ignored, not claimed executed. No new ignore or gate exemption.
The developer-tool release build also passed. Fresh C: measurements preceded
every phase and stayed above 12 GiB. The latest remote was still 6704741 when
checked after these gates; no combined-tree rebase was needed.

Local diagnostics: `.git/alo-loop/graphical-recovery-trace.log`,
`graphical-recovery-normal.log`, and `automatic-recovery-670-*` gate directories.
The traced/normal pre-repair logs are retained, never replaced by a green result.

## Contributor reconciliation

Read `a-loop-that-says-whether-it-is-running.md` (305cab1) and
`a-lock-the-operating-system-holds.md` (6704741), and integrated both by fast-forward
without touching this task's dirty files. The second supersedes PID-based lock
recovery on Windows with an open handle; non-Windows retains its stated PID path.
It reports 34 Windows supervisor tests, distinct from this worker's 29 desktop
supervisor tests. Its reserve measures the checkout's C: filesystem, and its WSL
helper spans run/verify/publish. These are developer-tool changes, no release tick.
The backend plan's lack of ready entries is not backend/release completion.

Original adaptive-label implementation and earlier evidence remain in
`adaptive-native-control-labels.md`; this follow-up records the recovery without
rewriting the stopped worker's report. Paged full-name access where expansion
cannot fit, native navigation/cursor selection and direct integration remain.
