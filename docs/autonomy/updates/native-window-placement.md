# Native window placement

- Date: 2026-09-08
- Workstream: native desktop compositor, delivery phase 2 window primitives
- Contributor: single desktop development worker, C:\dev\alo-os
- Status: implementation and corrected graphics fixture verified on the combined tree; ready for publication.

## Implementation

`crates/alo-shell/src/window_placement.rs` adds trusted `Server::place_window`
and the dispatch-thread `window_buffer_origin` accessor. Bounded logical
coordinates anchor committed, clamped XDG window geometry. Scene traversal now
uses this origin for drawing, hit testing and popup output constraints. Unmap
resets placement; foreign, child, popup, dead and unmapped roots refuse. Invalid
coordinates refuse before mutation. Existing pointer position is refreshed with
existing grab policy. Size, activation and stacking are preserved.

The one-million-per-axis bound leaves room for existing popup arithmetic and
permits intentional negative/offscreen positions without clamping requests.
Private surface data preserves existing FrameTarget signatures; custom targets
must honor the documented origin or refuse placement. Smithay renderer state
and placement must be read on the display thread. No engine patch, unsafe
exception, agent endpoint, UI string or new scope. ADRs 0002/0010 and application
grant/proposal/approval contracts remain unchanged. Interactive dragging is not
implemented. Graphics validation was corrected in the recovery below; complete
desktop acceptance remains separate from this primitive.

Five real socket tests in `tests/window_placement/mod.rs` cover movement and
stationary-pointer refresh, no configure/activation/stacking changes, committed
geometry and local coordinates, invalid/extreme coordinates, remap/disconnect,
foreign/child/popup refusal, popup constraints and translated input, and reactive
popup coalescing with acknowledgement versus commit.

## Executed checks

Ubuntu WSL2 prerequisites checked with:

```sh
export PATH=/root/.cargo/bin:/usr/bin:/bin
rustc --version
pkg-config --modversion wayland-client egl gbm xkbcommon libudev libinput gl libseat
test -S /mnt/wslg/runtime-dir/wayland-0 && echo WSLg-present
if test -d /dev/dri; then echo DRM-present; else echo DRM-absent; fi
```

Rust 1.98.0; library versions in order: 1.24.0, 1.5, 26.0.8-1ubuntu0.3,
1.13.1, 259, 1.31.1, 1.2, 0.9.2. WSLg present, DRM absent. No dependencies,
shared kernel/BPF state, services or other checkout changed.

Linux commands from /mnt/c/dev/alo-os, with the PATH above and isolated
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_placement
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

The first focused run passed three tests and failed one: the test read private
Smithay thread-local data outside its owning display thread and observed zero.
Moved that observation through Fixture::backend and documented the requirement;
all four then passed. Added the reactive-popup test before the full suite.
Final full suite passed 138 unit, 109 lifecycle and three socket tests, plus
three compile-fail doctests: 250 runtime tests total, none failed or ignored.
Final affected clippy, rustdoc, example build and Linux fmt passed. Intermediate
clippy runs caught missing helper docs and unchecked fixture indexing; fixed
with documentation and checked access, without lint suppression.

Windows commands from C:\dev\alo-os:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Affected Windows clippy/tests/fmt passed after adding the missing Linux cfg on
the new example module. Windows runs zero Linux-only runtime cases. Subsequent
Linux-only test accessor changes were formatted and passed Linux gates. Final
diff check passed. No full supervisor workspace/BPF gates were run by this worker.

## Graphical failure and exact remaining work

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen &&
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Offscreen exited 1 in the new `window_placement_check.rs`: translated pixel 0
for delta (5,6) was magenta `[255,0,255,0]`, expected transparent `[0,0,0,0]`.
The scene has a second magenta root. The test moves only the first root but its
oracle translates the entire prior framebuffer, incorrectly erasing the second
root's stationary contribution. The fixture client then failed its coordination
wait because the renderer had exited. Expected Mesa fallback diagnostics also
appeared. Earlier raising/activation/configured-switching/pending-size checks
printed success, but the full offscreen regression did not finish. The chained
nested popup/cursor command **did not run**.

This is the second runtime validation failure this iteration; work halted under
the owner's explicit repeated-failure rule. No retry or weakened expectation.
Next iteration must correct the oracle to include the stationary second root,
or isolate the intended root and its popup subtree in the graphical scene. Keep
full-frame positive/negative translation, clipping, restoration and unchanged
stacking checks. Then rerun offscreen and nested regressions and affected gates.
Source and tracked diff were reviewed. This was the halt state before the owner
authorized recovery below, not a continuing instruction to leave the fixture broken.

Interactive move/resize, remaining operations, rendered controls, launcher/dock,
clipboard and full v0.01 scope remain. Safe standalone GLES, direct DRM/seat
entry, populated device/hotplug and GPU recovery evidence remain at their phases;
physical laptop/GPU workstation acceptance follows integrated VM image checks.
No hardware certification from this WSL fixture.

## Reconciliation

Read constitution, delivery/shared-main/report instructions, current queue,
STATE tail, feature/roadmap and relevant ADR/contract sections. Initial tree
clean. Sole published report absent from STATE at iteration start:
`docs/autonomy/updates/three-model-choices-in-the-backend.md`.
Reviewed report and source test references. Format 2 settings persist/resolve
providers, format 1 stays compatible; authenticated providers refuse pending
keyring support. Contributor reports six choosing/two daemon tests and workspace/
BPF gates; these were not rerun by this worker. Raw TOML refusal Debug can retain
a pasted credential despite safe user messages, and needs a follow-up. Alo has
no endpoint; paired machines remain unreachable. ADR 0021 stays proposed. Claude
retains that workstream. Keyring work remains delivery phase 5, not a reason to
skip executable desktop work. Reports arriving during publication reconcile next
iteration. All four shared progress documents updated; no release checkbox ticked.

No staging, commit, push, other repository changes, tools/dev-loop edits,
worker/loop launch or physical installation.

## Owner-authorized recovery

The owner asked to fix the failed test and resume development. No supervisor or
worker was active in the desktop checkout when recovery started; existing work
was preserved. The renderer was not changed to satisfy a mistaken expectation.

The GLES expectation now computes opaque fixture layers independently of the
production placement helper: the root's red/green halves, yellow child and blue
popup move together above the unmoved magenta second root. It checks all 6,400
pixels at baseline and each requested position, including negative/top-left and
positive/bottom-right clipping, restores the exact original framebuffer, and
asserts unchanged stacking and second-window origin after each move.

After `cargo fmt --all` and rebuilding `nested_check`, both the complete
`--offscreen` and `--popups --cursor` commands above returned exit 0. Intentional
malformed-client refusals and Mesa fallback diagnostics remain expected output;
they are not ignored test failures.

### Combined-tree verification

Rebased the unpublished task onto main through `66d2e15` without conflicts.
The interactive recovery ran the same full publication gates as the supervisor:

- Windows: workspace fmt check, all-target clippy with warnings denied, and
  workspace tests with `--locked` all returned exit 0.
- Linux: workspace fmt, all-target warnings-denied clippy, workspace tests,
  warnings-denied rustdoc, plus the pinned BPF target's fmt and release clippy
  all returned exit 0. The standard suite reported 13 existing ignored test
  entries; none were added by this task and none are counted as passed evidence.
- Re-ran `cargo test -p alo-shell --test client_lifecycle --locked
  window_placement`: five passed, zero failed, zero ignored, 104 filtered out.
- Rebuilt `nested_check` on the combined tree and reran the full `--offscreen`
  and `--popups --cursor` regressions: both returned exit 0, including the new
  complete-frame placement checks.
- Supervisor fmt, all-target clippy, 11 tests and release rebuild passed. The
  rebuild picks up current embedded worker instructions; no supervisor source
  or failure policy changed.

Local logs: `.git/placement-recovery-windows.log`,
`.git/placement-recovery-linux.log`, `.git/placement-recovery-graphics.log` and
`.git/placement-recovery-supervisor.log`. No tracked build artifacts or logs.
Only recovery documentation changed after these code gates; diff checks precede
publication. The next desktop task is interactive move/resize, not another
attempt to fix this now-verified expectation.
