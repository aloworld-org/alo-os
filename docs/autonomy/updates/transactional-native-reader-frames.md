# Transactional native reader frames

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `4477c1f`. Read CLAUDE, DELIVERY, SHARED_MAIN, report guidance,
current QUEUE and STATE tail, relevant v0.01 features/roadmap, ADRs 0002/0010
and the native-control contract. No AGENTS.md found. Every published report
already had a STATE reference at startup; no contributor reconciliation was due.
Selected this component and acceptance criteria in QUEUE before implementation.

The native host can now submit a complete live reader page, navigation, feedback
and original strip with the client scene before making reader hits available.
Unsupported targets, stale pages, invalid placement and failed submissions cannot
leave active reader targets. Pending client callbacks survive refusal.

- `src/window_control_reader_frame.rs`: one live preparation/submission/publication
  transaction. Original reader appearance is retained, chrome is freshly shaped,
  feedback is revalidated, and only successful submission including the strip
  root publishes exact reader/page-visit identity and opaque hit geometry.
- `src/window_control_reader_scene.rs`: opaque borrowed scene, constructed only
  by live server preparation. Backend validation, painting and frozen inspection
  are distinct from live hit authority. Unrelated page/chrome content cannot be
  supplied as a constructed scene by the public caller.
- `src/scene_native.rs`, `scene_drawing.rs`, `nested.rs`: shared native scene
  selection/painter and actual nested EGL submission. Existing control/offscreen
  signatures remain intact. `FrameTarget::submit_reader` refuses by default;
  direct targets do not silently drop the reader and claim success.
- Reader appearance retention, exact reusable hit-map extraction and server
  publication storage are small changes in their existing responsible modules.
  Ordinary strip/label rendering clears reader publication; strip retirement
  and ordinary rendering clear both. Failed reader frames dismiss the reader
  permanently and cancel pointer execution while retaining release obligations.
- `tests/window_controls/reader_frame.rs`: four real-client tests for publication,
  callbacks, successful refresh/navigation/removal, geometry replacement, stale
  identity, away-and-back visits, unsupported/omitted/failed frames and typing.
- `examples/support/nested_reader_frame_check.rs`: actual WSLg live reader
  submission through `nested_check --reader`. Optional `--trace` in that example
  reports pump/dispatch/render timing without changing its deadline.

Source paths above are under `crates/alo-shell/`. Public rustdoc and
`docs/contracts/native-window-controls.md` describe the added surfaces.
No new palette, production wording, focus operation, agent verb or engine patch.
ADRs 0002/0010 are preserved. Separate scene selection avoids giving the reader
scene a second responsibility for existing complete-label composition.

Publication intentionally retains only identity and five rectangles, not page
rasters. Identical refreshes preserve held pointer state; changed geometry or
page/session identity cancels it before painting. A mismatching publication query
conservatively clears the saved publication. Failed submission cannot prove old
pixels disappeared: the host must subsequently submit a successful removal.

## Verification environment and commands

Ubuntu WSL2, private desktop target `/root/alo-os-target`; WSLg Wayland socket
present and existing `/sys/fs/bpf` mounted. `pkg-config --modversion
wayland-client egl gbm xkbcommon libinput libudev` returned 1.24.0, 1.5,
26.0.8-1ubuntu0.3, 1.13.1, 1.31.1 and 259. No prerequisite installation needed.

Before every format/build/test/lint/doc/graphical command:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

All readings exceeded 12 GiB (over 53 billion bytes throughout development).
This is individual operational preflight, not a continuous disk quota.

Linux commands use `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked reader_frame -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --examples --locked
cargo build -p alo-shell --example nested_check --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

The focused test ran initially and after the fixture repair. Clippy ran initially,
after documentation repair and after the final scene-module split. Full tests
ran before and after that split. The single-example builds covered geometry and
timing instrumentation repairs; the final examples command passed as well.

Graphical commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir
WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls --trace
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader --trace
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check --pressed
```

Reader and controls each ran initially, diagnostically, and finally without
tracing. Every external 30-second limit and the nested client's ten-second
deadline remained unchanged.

Windows affected checks:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Logs: `.git/alo-loop/transactional-native-reader-frames/`. PowerShell wraps native
stderr as NativeCommandError even when the native exit code is zero; outcomes
use exit codes. Expected malformed-client/keymap refusals and Mesa discovery
diagnostics remain visible. No diagnostic suppression or gate weakening.

## Failures, diagnosis and repair

1. Initial clippy found 18 missing private-item comments, with no Rust compile
   errors. Added responsibility/identity comments; subsequent clippy passed.
   Preserved in `linux-clippy-initial.txt`.
2. Two of the four initial protocol tests failed during ordinary removal/recovery:
   the mock target did not implement cursor submission, so the existing default
   refused the positioned cursor. Added explicit cursor support to the protocol
   target; real graphics remain separately tested. All four repaired tests pass.
   Logs: `linux-focused.txt`, `linux-focused-repaired.txt`.
3. Initial WSLg reader check refused invalid placement. The fixture assumed a
   Close label began at the strip's x rather than the Close control's x. Measured
   page bounds are (75,40,140,28), overlapping the original navigation capacity
   (162,40,154,156). Moved navigation below the page to (162,74,154,126), retaining
   text scale, all labels and two-pixel gutters. The example logs both rectangles
   and asserts the original collision and repaired separation. Logs:
   `nested-reader.txt`, `nested-reader-repaired.txt`, `nested-reader-final.txt`.
   Connection-reset client diagnostics followed the failed fixture's server exit.
4. The first existing controls regression hit its ten-second client deadline
   before reporting output negotiation. Process inspection found no surviving
   nested/reader test process. Added optional event-loop timing and ran an isolated
   diagnostic control. That passed, measuring an early empty-scene render interval
   from 17.573 ms to 7.524 s. The original uninstrumented stall location/cause
   remains unknown; this diagnostic pass does not prove it. The traced reader
   run passed (early empty-scene interval 17.889 ms to 2.223 s), and final normal
   reader/control runs passed. No shared service, WSL or process restart occurred.
   Logs: `nested-controls.txt`, `nested-controls-diagnostic.txt`,
   `nested-reader-repaired.txt`, `nested-controls-final.txt`.

## Final evidence and limits

- Linux: 168 unit, 238 lifecycle, three socket and four compile-fail doctests
  pass, none ignored (`linux-tests-final.txt`). Four new transaction tests pass.
- Affected Linux clippy, all example builds and warnings-denied rustdoc pass
  (`linux-clippy-final.txt`, `linux-examples-final.txt`, `linux-rustdoc.txt`).
  Windows affected clippy/tests pass (`windows-clippy.txt`, `windows-tests.txt`);
  shell test cases are zero on Windows because this crate is Linux-only.
- Final normal WSLg reader run passes 12 complete EGL page/feedback submissions,
  both schemes, geometry-derived semantic next/dismiss, two removals and
  refusal/recovery sequences. Existing six-client lifecycle checks pass too.
- Final normal controls regression passes its existing strip/label/expanded-name,
  dismissal, removal, refusal/recovery and six-client lifecycle checks.
- Existing pressed-state GLES readback passes all 72 complete 307,200-pixel
  frames and exact hit checks, forward/reverse pages, translation/fallback,
  light/dark and four scales in 13.71 seconds (`gles-pressed.txt`). This is
  component pixel evidence, not readback of the new nested submitted frames.
- Final format and diff checks pass. Tracked changes and all new source/test/
  example files reviewed. All four shared progress documents updated in this
  change, referencing this report. No feature or release checkbox promoted.

This completes the explicit reader frame composition/submission/publication
component. Next: coordinated backend key/pointer activation against publication,
ownership cancellation/draining, native reader selection/navigation/cursor
selection and direct integration. No automatic parent-event routing is installed
by this API. WSLg evidence is actual submitted native content with explicit host
semantic input, not parent keyboard/pointer delivery, physical scanout or VM boot.
Physical laptop/GPU workstation records remain owed during release validation.
The independent supervisor Windows/Linux workspace/rustdoc/BPF gates are pending;
they must pass before publication. No release verification claim.

No stage/commit/push, supervisor modification, second worker/loop, other-checkout
edit, credential/identity access, cleanup, WSL helper/restart or shared mount/
service/package/session maintenance. Tests use private resources; no kernel
mutation or outer machine lock. Later published reports reconcile next iteration.
