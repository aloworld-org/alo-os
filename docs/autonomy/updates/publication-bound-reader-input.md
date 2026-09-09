# Publication-bound reader input coordination

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `5518bcb`. Read CLAUDE, DELIVERY, SHARED_MAIN, updates README,
current QUEUE/STATE, relevant features and roadmap, ADRs 0002/0010 and the native
window-controls contract. No AGENTS.md found. All published reports already had
STATE references at iteration start; no reconciliation was due. Selected this
component and its acceptance criteria in QUEUE before implementation.

A reusable backend-owned coordinator now permits reader navigation only against
the page actually submitted. Geometry changes, lost activation and competing
devices disarm old commands, while cancelled releases remain owned and ordinary
typing keeps its existing route. This is a complete ordered-input component;
attachment to parent event pumps remains an explicit next component.

- `crates/alo-shell/src/window_control_reader_input.rs`: `WindowControlReaderInput`
  owns keys, pointer feedback/releases and observed publication identity. `key`
  accepts the trusted host's semantic mapping; None preserves ordinary typing.
  `pointer` derives hits from actual backend positions and live publication,
  never caller-supplied semantic hits. `synchronize` handles activation without
  an input event. Deactivation retires publication until a fresh frame. Hosts
  still perform ordinary focus/seat teardown, map keys and forward exactly once
  only on Forward. Motion determines axis consumption too.
- `window_control_reader_frame.rs`: private continuous publication identity
  survives only identical successful reader/page/geometry refreshes. New identity
  cancels both owners even if geometry changed away and back with no intervening
  event. This closes a gap that live reader/page identity alone cannot detect.
- Key presses cancel pointer execution; pointer presses cancel key execution;
  motion alone preserves keys. Navigation/dismissal cancels both. Missing, stale,
  failed or removed publication cannot acquire new input. Client-owned keys and
  buttons refuse acquisition. Release obligations survive all cancellation.
- `tests/window_controls/reader_input.rs`: four real-client tests cover navigation,
  refresh, repeats, ordinary typing, geometry discontinuity, inactive drain,
  fresh-frame reactivation, removal, frame refusal, client grabs, cross-device
  cancellation, nonfinite pointer positions and dismissal without closing apps.
  Existing frame fixture helpers are shared with sibling tests.
- `examples/support/nested_reader_frame_check.rs`: all existing native reader
  submissions now use the coordinator and its frame pointer owner, retaining
  every prior geometry, feedback, navigation, removal and refusal assertion.
- `nested.rs`: optional `ALO_NESTED_TRACE_SUBMISSION` prints elapsed bind, paint
  and upstream submission boundaries. It changes neither rendering nor deadlines;
  useful when existing event-loop tracing locates a stall inside rendering.

Public rustdoc and `docs/contracts/native-window-controls.md` document the API.
All four progress documents updated with evidence and remaining work. No new
agent surface, production wording, focus operation, palette or engine patch.
ADRs 0002/0010 remain intact. Normal input stays in this implementation phase.

## Verification environment and exact commands

Ubuntu WSL2, private desktop target `/root/alo-os-target`. WSLg socket and existing
`/sys/fs/bpf` checked. `pkg-config --modversion wayland-client egl gbm xkbcommon
libinput libudev` returned 1.24.0, 1.5, 26.0.8-1ubuntu0.3, 1.13.1, 1.31.1 and 259.
No packages, services, mounts or sessions were changed.

Every format/build/test/lint/doc/graphical command had this Windows preflight:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

Minimum observed: 52,863,983,616 bytes; all above 12 GiB. Individual preflight is
operational headroom, not a continuous quota. No cleanup performed.

Linux commands use `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked reader_input -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --example nested_check --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Focused tests ran initially, after the return-type repair, and finally after the
client-grab fixture repair. Clippy ran initially, after final coordinator/test
changes and after diagnostic instrumentation. Full Linux tests ran before/after
the fixture repair and again after instrumentation. Example built before graphics
and after instrumentation. Rustdoc ran before/after instrumentation. Final checks
are logged with `-published` suffixes; that name means the final candidate checks,
not that any code has been committed or published. The final three commands also
set `RUSTDOCFLAGS=-Dwarnings` (only rustdoc uses it).

Graphical commands additionally use `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir
WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls --trace
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader --trace
ALO_NESTED_TRACE_SUBMISSION=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --reader --trace
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Listed in execution order. Last two are final normal acceptance runs, both pass.
Every external 30-second limit and the existing ten-second client deadline stayed
unchanged. No assertion removed, diagnostic suppressed or fixture skipped.

Windows affected commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Format applied after source changes and checked after final instrumentation.
Windows clippy/tests pass; shell test cases are zero on Windows (Linux-only crate).
Full independent workspace/Windows/Linux/rustdoc/BPF supervisor gates are pending.

## Failures, investigation and repairs

Logs: `.git/alo-loop/publication-bound-reader-input/`. Native exit codes determine
outcomes; PowerShell labels native stderr NativeCommandError even on success.
Expected malformed-client and keymap refusals and Mesa diagnostics remain visible.

1. `focused.txt`: initial new test returned unit via a trailing `?`, causing
   E0308. Corrected the test return expression. `focused-repaired.txt`: three
   then-present tests pass. No production fix was necessary.
2. `linux-tests.txt`: the fourth test wrongly expected a reader to remain usable
   after querying it during a client pointer grab. `control_reader_binding`
   refuses busy input; `read_window_control_page` permanently dismisses that
   reader. Test now asserts retirement and opens a fresh reader for the separate
   frame-refusal check. `focused-final.txt`: all four pass; final full suites
   pass. Original refusal expectations remain, with added lifecycle evidence.
3. `nested-reader.txt`: first normal graphical run hit the ten-second client
   deadline before reader results. Client connection-reset panic followed server
   teardown. Process inspection found no surviving nested/reader process; socket
   and bpffs remained present. An initial shell-form process query had quoting/
   missing-rg errors; corrected direct `ps -eo pid,ppid,stat,etime,comm` query with
   PowerShell filtering succeeded. No process was stopped.
4. `nested-controls-diagnostic.txt`: unchanged controls fixture passed. Tracing
   measured its early empty render from 22.37 ms to 2.260 s.
   `nested-reader-diagnostic.txt`: reader run then failed, with an empty render
   from 17.38 ms to 13.862 s, before this task's reader path was reached.
5. Added optional finer stage instrumentation; `nested-reader-stages.txt` measured
   an early zero-root frame: bind complete at 40 microseconds, paint complete at
   2.376 ms, upstream submission complete at 8.568 s. This run completed all 12
   coordinated reader submissions, but later exceeded the same client deadline;
   its client protocol-error assertion failed following forced server teardown.
   Delay is measured within Smithay backend submit (pre-present/EGL swap), not
   the reader or painter. Exact upstream cause remains unknown. Read pinned
   Smithay source: its default GL attributes already use vsync=false; no engine
   patch or speculative vsync change was made. strace/gdb were unavailable; no
   shared package maintenance was attempted.
6. Final normal `nested-reader-final.txt` passes all reader and lifecycle checks;
   final normal `nested-controls-final.txt` passes the control regression and
   lifecycle checks. These passes do not establish why earlier submissions
   stalled. No shared restart, timeout increase, retry loop or diagnostic hiding.

## Final evidence and remaining work

- Final Linux: 168 unit, 242 lifecycle, three socket and four compile-fail doctests,
  none ignored (`linux-tests-published.txt`). Four new input tests pass.
- Final affected Linux clippy and warnings-denied rustdoc pass
  (`linux-clippy-published.txt`, `linux-rustdoc-published.txt`). Example build
  passes. Windows affected clippy/tests and final format/diff checks pass.
- WSLg reader: 12 complete page/feedback EGL submissions, both schemes,
  publication-coordinated pointer next/dismiss, removal and refusal/recovery.
  Existing control regression and six-client lifecycle checks pass as well.
- Evidence is actual nested submission driven by explicit host events plus real
  private-client input coordination, not automatic parent key/pointer routing,
  readback of the submitted frames, physical scanout or an integrated VM boot.
- Full-name access/window management are not complete. Next: attach the coordinator
  to ordered nested parent events and key mapping; preserve actual backend pointer
  position and route loss/removal releases; then reader selection/cursor and direct
  backend integration. Physical laptop/GPU workstation acceptance remains owed
  during release validation. No feature/release checkbox promoted.

Tracked diff and new source/test/report reviewed. No stage/commit/push, dev-loop
edit, second worker/loop, other-checkout change, credentials/identity access,
cleanup, WSL helper/restart or shared mount/service/package/session maintenance.
Fixtures use private resources; no kernel mutation or outer machine lock.
Reports arriving during publication reconcile next iteration. Supervisor owns
integration, independent gating and publication; release verification not claimed.
