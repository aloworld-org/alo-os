# Externalized native reader navigation

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Worded navigation model complete;
reader rendering/input integration, usable window management and release unfinished.

## Change and decisions

`crates/alo-shell/src/window_control_reader_words.rs` declares four reader-only
strings with translator notes, under `shell.name-*`. Registration composes with
the shortcut vocabulary and clones before insertion so a collision on any key
adds nothing. English stays in Word declarations; each resulting Said retains
its own provenance. No new language restriction or hand-assembled UI sentence.
The format-1 translation keys are additive; existing translations may be partial.

`window_control_reader_navigation.rs` prepares the complete worded chrome model
from a borrowed page: position, previous/next, dismissal and boundary availability.
Both numeric gaps are filled with decimal integers in the existing 1..=128 page
range. Wording remains available for disabled navigation. Missing declarations,
unfilled gaps, invalid page metadata and excessive text refuse atomically. Empty
translation sentences already refuse at translation validation; the chrome also
defensively checks blank text. Each label retains separate fallback provenance.
"Done reading" refers only to dismissing the reader, never closing its window.

The trusted Server semantic Previous/Next method validates live reader identity
before boundary checks. Requests never wrap, steal input focus or dispatch a
window operation. It reuses the existing checked selection and retirement path;
it adds no configured key binding or input authority. Public chrome is frozen
data, not proof of live publication, valid shaping/fit or frame submission.

This is a complete navigation model component of the selected full-name reader
work. Chrome layout/raster, input ownership and transactional reader publication
remain separate implementation work. Existing complete-label refusal stays intact.
Consistent with ADR 0002's native Rust shell and ADR 0010: no new engine, palette,
font or agent signal. No design decision conflicts with an accepted ADR.

Exports are in `lib.rs`; three tests in `tests/window_controls/reader_navigation.rs`
reuse a narrowly exposed `reader::begin` helper and register in `mod.rs`. The
public contract is updated in `docs/contracts/native-window-controls.md`. The
four shared progress documents identify the complete component and exact next
work; no feature or release checkbox is ticked.

## Checks actually executed

Started clean at 41ab2ba. Read CLAUDE, DELIVERY, SHARED_MAIN, report guidance,
current QUEUE/STATE tail, relevant features/roadmap, ADR 0002/0010 and native-control/
translation contracts. No additional AGENTS.md found. Compared report filenames
against STATE: all published reports were already referenced. No reconciliation
was needed; reports arriving during publication belong to the next iteration.
Selected this component and acceptance in QUEUE before code changes.

Ubuntu prerequisites checked using:

```sh
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
stat -f -c %T /sys/fs/bpf
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
Socket present; existing bpffs reports bpf_fs. No maintenance or installation.
C: free-space preflights stayed above 12 GiB; minimum observed 54,672,064,512 bytes.
The first combined format/focused-test command used one preflight for both;
subsequent phases each had their own immediate reading. No cleanup performed.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked native_reader -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
cargo build -p alo-shell --examples --locked
```

Final affected clippy, tests, rustdoc (with `RUSTDOCFLAGS=-Dwarnings`), fmt and
example build all pass. Focused acceptance: seven tests pass. After the final
lint repair and added 4096/4097-byte boundary assertion, the full affected suite
passes 162 unit, 223 real-client lifecycle, three socket and four compile-fail
doctests, none ignored. Intentional malformed-client diagnostics remain visible.

New integration evidence: a real private Wayland client keeps keyboard focus and
receives ordinary key press/release during forward/backward reader navigation;
it receives no close request. Reordered German position, translated next label,
source previous/dismiss labels, first/last/single-page boundaries, stale boundary
navigation and republication refusal pass. Additional checks cover every missing
word prefix, invalid metadata including usize::MAX, 128-page numeric boundary,
4096-byte acceptance/4097-byte refusal, atomic registration at every collision,
and missing/invented translation gaps and blank-translation refusal.

WSLg commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_label_pages_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both pass: 72 complete 57,600-pixel GLES page frames forward/reverse with
translation/fallback, light/dark and four scales; nested eight strip submissions,
two labels, two expansions, two dismissals, two ordinary removals and exhausted-
space/strip refusal-recovery, six client surfaces and unmap/remap/disconnect.
Mesa/EGL discovery warnings remain visible; successful rendering followed. No
graphical timeout. These are existing graphical regressions, not rendered new
chrome, parent navigation-key delivery, on-screen reader UI or scanout evidence.

Windows commands pass:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows shell tests contain zero cases because the crate is Linux-only. This is
affected compilation evidence. Full independent supervisor Windows/Linux/workspace/
rustdoc/BPF gates have not run this iteration and remain required before publication.

## Development failures and repairs

Original focused failure: expected a single page for the maximise label in a
180x28 box; measured result was Page 1 of 2. The test now explicitly prepares a
180x128 reader, verifies Page 1 of 1 and both refused navigation directions.
Production layout and limits were unchanged. Splitting navigation tests into
their own file accidentally removed the existing reader test imports; compiler
errors identified this and the original imports were restored. The next focused
run exposed that blank translations fail before chrome construction; the test now
asserts that refusal at the actual translation boundary. Seven focused tests then
passed. Clippy rejected unwrap_err assertions and the resulting single-element
loop. Replaced assertions with exact Option error comparisons and tests of both
4096/4097-byte limits. Final clippy and full affected acceptance pass afterward.
No lint exemptions, assertion suppression, changed deadlines or ignored checks.

Logs in `.git/alo-loop/externalized-native-reader-navigation/` retain original
`focused.txt`, import failure `focused-final.txt`, blank-translation failure
`focused-repaired.txt`, passing `focused-acceptance.txt`, original `linux-clippy.txt`,
and final `linux-clippy-final.txt`, `linux-tests.txt`, `linux-rustdoc.txt`,
`linux-fmt.txt`, `linux-examples.txt`, `page-gles.txt`, `nested-controls.txt`,
`windows-fmt.txt`, `windows-clippy.txt`, `windows-tests.txt`. PowerShell records
native stderr as NativeCommandError even on successful commands; actual exit
codes and terminal test results were inspected.

## Remaining work and authority

Next executable reader component: bounded chrome layout and raster preparation
with full wording, visible source provenance, unchanged text scale and refusal
when it cannot fit. Then keyboard/pointer ownership and transactional reader
composition/submission/publication/retirement; native navigation/cursor selection
and direct integration follow. Image integration and release physical laptop/GPU
workstation evidence remain owed at their delivery phases. No hardware certification.

Tests use private resources, with no kernel-global mutation or outer machine
lock. No stage/commit/push, extra worker/loop, dev-loop edit, other-checkout edit,
credential/identity access, package/service/mount/session change, WSL restart or
keep-alive helper, cleanup or unrelated host change. Source/new-file diff reviewed
and `git diff --check` passes. Publication belongs to the supervisor.
