# Live native reader selection

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `56add16`. Read CLAUDE, DELIVERY, SHARED_MAIN, updates guidance,
current QUEUE and STATE tail, relevant v0.01 features/roadmap, ADRs 0002/0010 and
the native-window-controls contract. No AGENTS.md found. All published reports
already had STATE references at iteration start; no reconciliation was due.
Selected the component and acceptance in QUEUE before implementation. No work
assigned to the filesystem contributor or another checkout was touched.

The native shell can now explicitly open the full name selected on its published
control strip. Native focus takes precedence over fresh pointer hover; disabled
names remain accessible without executing a window command or stealing typing.
This completes the explicit opening component, not usable full-name access.

- `crates/alo-shell/src/window_control_reader_selection.rs` adds
  `Server::open_presented_window_control_reader` and `Nested::open_control_reader`.
  Selection reuses the live publication and current backend pointer position;
  it does not reuse a cached action or infer application keyboard focus.
- Before returning page zero, every page's complete navigation wording and hit
  geometry must fit at the person's text scale. Later decimal page numbers can
  wrap differently. Existing bounded page preparation and errors are retained;
  temporary chrome is dropped each iteration. No pixels or input authority are
  published. Failed capacity preparation preserves an existing reader.
- `tests/window_controls/reader_selection.rs` adds four private-client tests:
  focus/hover/disabled selection with typing, absent/retired/busy refusal, atomic
  geometry/vocabulary refusal with existing-reader preservation, and a measured
  font-wrap boundary where page one fits but page ten does not. The latter checks
  that opening refuses rather than trapping the person on an unreadable page.
- `examples/support/nested_reader_frame_check.rs` opens its two backend-owned
  readers through the new live selection wrapper and explicitly focused Close
  name. All original twelve transactional submissions and refusal assertions
  remain. Light and dark selected readers reach actual EGL submission.

The host calls this on an explicit opening request after pumping, retains the
reader across navigation, and renders with the same strings/style/chrome.
Per-frame submission still validates independently. Keeping selection separate
from rendering preserves ADR 0002's native architecture and existing publication
authority; ADR 0010 palette rules, externalized wording and normal typing remain.
No engine patch, new palette, production string or agent capability was added.
Public rustdoc, contract and all four progress documents updated.

## Verification

Ubuntu WSL2 prerequisites checked: WSLg socket and existing `/sys/fs/bpf` mount.
`pkg-config --modversion wayland-client egl gbm xkbcommon libinput libudev`
returned 1.24.0, 1.5, 26.0.8-1ubuntu0.3, 1.13.1, 1.31.1 and 259. No installation
or shared maintenance needed. Private-client fixtures mutate no kernel-global
resources; no outer machine lock was added. Desktop target `/root/alo-os-target`.

Every build/test command checked Windows C: free space first:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

All observed readings exceeded 53 billion bytes, above 12 GiB. No cleanup.
This is preflight headroom, not a running-build disk quota.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked reader_selection -- --nocapture
cargo test -p alo-shell --locked --quiet
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo build -p alo-shell --example nested_check --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Final results: four focused tests pass; full affected suite passes 168 unit,
250 lifecycle, three socket and four compile-fail doctests, none ignored.
Affected clippy, example build and warnings-denied rustdoc pass. Final full suite
and clippy were rerun after the test indexing correction described below.

Graphical commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir`
and `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Final normal runs both pass. Reader: twelve original page/feedback submissions,
two backend-owned selected reader submissions, navigation/dismissal, removal,
geometry refusal/recovery and actual pump calls. Controls: existing strip, label,
expanded-name and refusal/recovery regression passes. Both preserve the six-client
unmap/remap/refusal/disconnect checks. No timeout or assertion was weakened.

Windows affected commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

All pass. Windows shell tests have zero cases because the crate is Linux-only.
Full supervisor workspace/Windows/Linux/rustdoc/BPF gates have not run for this
change. No claim of those independent gates or publication is made.

## Preserved failures and diagnosis

Logs: `.git/alo-loop/live-native-reader-selection/`.

- `focused.txt`: initial test compile failed because `Action::said` is infallible
  and `pointer_motion` takes separate x/y arguments. Corrected test calls;
  `focused-fixed.txt` and `focused-final.txt` pass.
- `linux-clippy-final.txt`: added page-ten test used unchecked vector indexing,
  prohibited by repository lint. Replaced with checked slice destructuring;
  `linux-clippy-fixed.txt` and `linux-tests-final.txt` pass.
- `wslg-reader.txt`: initial reader run failed the unchanged ten-second client
  deadline, followed by client connection reset during teardown. No reader
  submission marker appeared. No surviving nested/client/cargo process was found;
  WSLg socket and bpffs still existed. No process was stopped or service restarted.
- Investigated with the known controls regression and existing instrumentation:
  the two graphical commands above with `ALO_NESTED_TRACE_SUBMISSION=1` and
  `--trace`. `wslg-controls-trace.txt` passes and measures an empty-scene submission
  at 2.403636 ms after painting and 8.575840773 s after swap. The reader trace
  passes with another empty-scene submission at 1.003495 ms after painting and
  2.991342836 s after swap. These are measured upstream swap delays, before reader
  selection. They support a submission-delay hypothesis but do not establish the
  original failure's exact cause/location. The intermittent delay is unresolved.
- `wslg-reader-final.txt` and `wslg-controls-final.txt` pass with tracing disabled
  and original deadlines. Their success does not prove the original cause or a
  repair to upstream graphics. Existing Mesa and intentional malformed-client
  diagnostics remain visible.

PowerShell Tee-Object logs include NativeCommandError wrappers for successful
native stderr. Native exit status was checked after each command; two enclosing
PowerShell batches themselves returned 1 from stderr despite successful final
native tests/docs. Their logs show successful native completion, and the batches
only advanced past commands whose native exit status was zero. Graphical batches
explicitly print native exit status and end with that status.

## Remaining evidence and integration

Explicit native opening/preflight is complete. Next: attach it to a native
activation gesture and automatic full-name fallback in strip presentation, then
native cursor selection and direct backend integration. This API itself installs
no activation key, shortcut or pointer gesture, and does not automatically replace
an oversized tooltip. Full-name access/window management remain unfinished;
no feature/release checkbox is promoted.

Private-client checks and WSLg native-focus selection-to-submission do not prove
synthetic parent key/button delivery, readback of submitted frames, direct scanout,
VM boot or physical input. Integrated VM and physical laptop/GPU workstation
records remain owed in their delivery phases. Supervisor independent gates and
publication remain pending. Reports arriving during publication reconcile next
iteration; release verification is not claimed.

No stage/commit/push, supervisor modification, second worker/loop, other-checkout
change, credential/identity access, WSL helper/restart, cleanup or shared package/
service/mount/session changes. Final tracked/new source, tests, example and docs
were inspected; format and diff whitespace checks pass.
