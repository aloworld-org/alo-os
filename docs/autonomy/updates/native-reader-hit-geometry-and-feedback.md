# Native reader hit geometry and feedback rendering

Date: 2026-09-10. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Geometry and feedback composition
complete; transactional reader publication and backend activation unfinished.

## Change and decisions

`WindowControlReaderInteraction` now prepares exact page/chrome hit geometry and
composes complete reader wording with visible pointer feedback. Geometry and
painting live separately in `window_control_reader_interaction.rs` and
`window_control_reader_interaction_paint.rs`; existing chrome stores its original
page/strip geometry and scheme. The additive public contract is documented in
`docs/contracts/native-window-controls.md`.

The constructor refuses mismatched page bounds, output or strip placement and
any gutter extending off output or overlapping page, wording or strip controls.
Command rows reserve two opaque pixels outside their text rasters. This preserves
every original glyph, source-language mark, translation and scale. Enabled rows
have an underline, hover a one-pixel outline, and armed press a two-pixel outline;
disabled rows keep full wording without the interactive affordance. Shape changes
avoid relying on colour alone. Existing Cream/Navy and Charcoal/Cream tokens
follow ADRs 0002/0010; no new palette, strings, bindings, engine or agent surface.

Hit tests preserve fractional coordinates, include left/top and exclude right/
bottom edges, and refuse nonfinite/outside positions without clamping. Only
painted opaque page/row/gutter areas consume; transparent gaps and unused capacity
do not. Disabled rows still produce semantic command hits, allowing the live
pointer policy to consume without acting. Invalid or inconsistent semantic
feedback refuses before any draw call; renderer errors still invalidate the
whole frame. Preparation and feedback are bounded, with no text raster copies.

This completes geometry and feedback composition, not publication. Matching
geometry does not prove page content or lifetime. The host must validate the live
page, prepare its matching chrome, refresh feedback, submit and publish one
transaction, and retire input/pixels on failure or replacement. Transactional
reader submission/publication/retirement and coordinated backend key/pointer
activation are next, followed by native navigation/cursor selection and direct
integration. Full-name access and usable window management remain unfinished.

## Acceptance and development diagnosis

Three new unit tests check every pixel's hit coverage, fractional/edge/nonfinite
positions, geometry refusals, complete text exclusion from feedback primitives,
scheme tokens, all four scales and invalid/disabled feedback. A new real private-
client test derives pointer hits from actual page/chrome geometry, traverses
forward/backward, consumes disabled boundaries, rejects retired hits and preserves
normal typing with no keyboard leave or application close.

The GLES example compares every pixel of complete 640x480 frames and every
interaction hit against a separate geometric/raster reference. It retains all
pages in forward/reverse order, translation/source fallback, light/dark and four
text scales. Idle, hover and press are explicit acceptance phases. Idle also
checks every framebuffer byte remains untouched after inconsistent feedback.

The first focused run failed a collision fixture: at `(244, 40)` its command
gutter started below the short page, so it correctly did not collide. Corrected
the fixture to `(244, 8)`, where the command gutter actually overlaps the page;
retained the refusal assertion and reran focused/full acceptance successfully.
Original failure: `focused.txt`; repair: `focused-repaired.txt`.

The original expanded GLES invocation timed out at 30 seconds with no progress
output (`gles-reader.txt`). A known-good `nested_check --controls` passed; no
reader process remained in `pgrep`. Instrumented diagnostic rerun under the same
30-second limit (`gles-diagnostic.txt`) initialized GLES at 106.5 ms and steadily
completed 46 plain plus 138 interaction frames by 29.77 s before exit 124. This
measures excessive combined fixture workload in that rerun, not a driver stall;
the uninstrumented original's exact progress is unknown. The new matrices had
quadrupled frame comparisons and added full-screen hit checks plus refusal
readbacks. Split acceptance into unchanged plain chrome and explicit idle/hover/
pressed invocations, each with the original 30-second deadline and every assertion
and matrix case retained. No timeout increase, ignored assertion or gate change.
The first separated idle check passed 72 frames in 14.27 s (`gles-idle.txt`).

## Commands and environment

Started clean at `4c294a9`. Read CLAUDE, DELIVERY, SHARED_MAIN, reports README,
current QUEUE/STATE, relevant features/roadmap, ADRs 0002/0010 and native-control
contract. `rg --files -g AGENTS.md` found none. Every published task report was
already referenced in STATE; no pending reconciliation at iteration start.
Selected the component and acceptance in QUEUE before coding.

Ubuntu prerequisites checked with `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec
sh -c` using `pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev
libinput gbm libseat`, `test -S /mnt/wslg/runtime-dir/wayland-0` and
`stat -f -c %T /sys/fs/bpf`. Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1,
26.0.8-1ubuntu0.3, 0.9.2; existing bpffs reports `bpf_fs`. Successful graphical
runs independently establish the Wayland-parent connection. No prerequisites
needed installation or shared maintenance.

Before every build/test/lint/format/doc command:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

Every reading exceeded 12 GiB. Minimum observed: 51,850,760,192 bytes. These are
individual preflights, not a continuous quota or additional WSL disk capacity.

Linux commands use `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked native_reader_interaction -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Diagnostic build additionally used `cargo build -p alo-shell --example
control_reader_chrome_check --locked`. Graphical acceptance additionally sets
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check --idle
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check --hover
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check --pressed
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Windows affected commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Logs: `.git/alo-loop/native-reader-hit-geometry-and-feedback/`. PowerShell wraps
native stderr as NativeCommandError even on success; command exit codes determine
results. Expected malformed-client/keymap refusal diagnostics and Mesa discovery
warnings remain visible. No diagnostics suppressed.

## Final results and limits

Final Linux suite: 168 unit, 234 lifecycle, three socket and four compile-fail
doctests pass, none ignored (`linux-tests-final.txt`). Focused repair passed all
three new unit tests plus the new private-client test. Affected Linux clippy,
example compilation and rustdoc with warnings denied pass (`linux-clippy-final.txt`,
`linux-examples-final.txt`, `linux-rustdoc-final.txt`). Windows affected clippy/tests
pass (`windows-clippy.txt`, `windows-tests.txt`); Linux-only shell cases are zero
on Windows. Final `cargo fmt --all --check` passes (`fmt-check-final.txt`).

Final graphical checks after the geometry/painter split all pass:

| Invocation | Complete 307,200-pixel frames | Elapsed | Log |
| --- | ---: | ---: | --- |
| plain chrome | 72 | 11.94 s | `gles-plain-final.txt` |
| `--idle` | 72 | 14.47 s | `gles-idle-final.txt` |
| `--hover` | 72 | 15.02 s | `gles-hover-final.txt` |
| `--pressed` | 72 | 13.70 s | `gles-pressed-final.txt` |

All 216 interaction frames additionally compare each hit's opaque coverage.
Idle also proves 72 invalid-feedback frames remain byte-for-byte untouched.
`nested-controls.txt` records passing existing strip/label/expanded-name submission,
dismissal, removal, refusal/recovery and ordinary six-client lifecycle regression.
No new reader backend input/publication claim is inferred from that control.

All four shared progress documents updated, with this report referenced in STATE.
Tracked/new source, tests and documentation reviewed; `git diff --check` passes.
No feature or release checkbox changed. Reports published during supervisor
publication are intentionally reconciled next iteration.

This evidence covers native geometry/composition, private-client semantic routing
and WSLg GLES readback. It does not establish actual parent navigation input,
on-screen interactive reader publication, scanout, integrated VM boot or physical
hardware acceptance. Laptop/GPU workstation records remain owed in release
validation. Independent supervisor Windows/Linux workspace/rustdoc/BPF gates are
pending and must pass before publication. No feature/release checkbox promoted.

No stage/commit/push, dev-loop edit, second worker/loop, other-checkout edit,
credential/identity access, cleanup, WSL restart/helper or shared mount/service/
package/session changes. Tests use private resources; no kernel mutation or outer
machine lock. Desktop Linux target remains separate. Reports arriving during
publication are reconciled next iteration.
