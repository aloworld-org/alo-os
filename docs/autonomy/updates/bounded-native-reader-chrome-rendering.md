# Bounded native reader chrome rendering

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Wording layout/raster component
complete; interactive reader, usable window management and release unfinished.

## Change and decisions

`crates/alo-shell/src/window_control_reader_chrome.rs` adds atomic preparation of
four complete wording rows, in position/previous/next/dismiss order. The consumed
externalized model supplies separate Said values and frozen previous/next
availability. Prepared rows are immutable and retain all wording and provenance.
Source guillemets from Showing::InDevelopment remain real shaped text; the
existing Showing::ToAPerson behavior is unchanged. Unavailable navigation names
remain readable. This is wording raster preparation, not interactive button
feedback or input authority.

The caller supplies a bounded capacity distinct from the page and strip. Matching
page/strip/output viewport, complete output containment and no page/control
intersection are mandatory. Vertical layout uses each row's full natural wrapped
height, four-pixel padding and four-pixel separation. Geometry is 9..=2048 by
9..=512; total row raster allocation cannot exceed 1,048,576 RGBA pixels (4 MiB).
The column is a simple bounded layout that permits long translated rows to wrap
without changing their reading order or reducing the person's text size. Unused
capacity is not painted. A caller must choose a different placement on refusal;
this does not automatically move or repaginate the name.

Complete text/glyph validation, padding, shaping metrics and rasterization reuse
`window_control_label.rs`, which now factors private shape_said/prepare_said
helpers. Blank whitespace is also rejected defensively. Missing vocabulary,
excessive text and missing glyphs retain their errors; invalid placement returns
Placement; exhausted space or clipped actual ink returns LineTooLarge. A late
failure returns no partial chrome. The shaper remains reusable after refusal.
`window_control_label_pages.rs` adds only a crate-private viewport accessor for
composition validation. The prepared chrome painter reuses the label painter;
a frame failure must cause the caller to discard the entire frame.

Consistent with ADR 0002's native Rust shell and ADR 0010's existing scheme tokens.
No new palette, font, engine, vocabulary, configured binding or agent surface.
The new public API and evidence limits are documented in
`docs/contracts/native-window-controls.md`. Exports are in `lib.rs`.

## Checks actually executed

Started clean at 9855d76. Read constitution, delivery order, shared-main ownership,
report guidance, current QUEUE/STATE, relevant features/roadmap, ADR 0002/0010 and
native-control/translation contracts. No AGENTS.md found. Compared every published
report filename against STATE: none required reconciliation. Reports arriving
during publication reconcile next iteration. Selected acceptance in QUEUE before
implementation. All four shared progress documents updated without release ticks.

Ubuntu prerequisite checks:

```sh
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
stat -f -c %T /sys/fs/bpf
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present; bpffs reports bpf_fs. No prerequisites needed installation.
Before every build/test/format/lint/doc command, Windows C: was measured with
`(Get-PSDrive C).Free` and the command refused below `12GB`. Every reading passed;
minimum command preflight was 54,645,133,312 bytes. No cache cleanup performed.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked native_reader_chrome -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Final affected clippy, full tests, rustdoc (with `RUSTDOCFLAGS=-Dwarnings`), examples
and fmt pass. Full suite after final source changes: 165 unit, 224 real-client
lifecycle, three socket and four compile-fail doctests, none ignored. Intentional
malformed-client and nonexistent-keyboard-layout diagnostics remain visible.

Three new unit tests exercise mixed/reordered translations, marked source words,
light/dark and 100/125/200/300 percent scale, exact natural row height and full
raster equivalence, exact-height acceptance and one-pixel-short refusal, wrapped
rows, 9/10-pixel widths that lose ink, page/control/output overlap, mismatched
page viewport, out-of-range dimensions, late missing vocabulary, 4097-byte text,
unsupported glyph and successful reuse after refusal. One new private-client
test prepares first/middle/last pages and reverse traversal with matching chrome,
checks availability and overlap refusal, retires the reader, and verifies normal
client typing with no keyboard leave or close request.

WSLg commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_reader_chrome_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both pass. The new example compares every pixel of 72 complete 640x480 GLES
frames, with page plus all four chrome rows, first-to-last and reverse order,
partial translation/source marking, light/dark and four text scales. It checks
opaque text rows and unchanged background outside them. The existing nested
regression passes eight EGL strip submissions, two labels, two expanded names,
two dismissals, two ordinary removals, exhausted-space and strip refusal/recovery,
six client surfaces and unmap/remap/disconnect. Mesa/EGL discovery warnings remain
visible; successful rendering followed. No graphical timeout or assertion failure.

Windows commands pass:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows shell tests have zero cases because the implementation is Linux-only;
these are affected compilation checks. Full independent supervisor workspace,
Windows/Linux/rustdoc/BPF gates have not run this iteration and remain mandatory.
No claim that affected checks substitute for them.

## Development failures and repairs

Initial focused compilation failed because TextScale's TextError does not
implement std::error::Error. Test fixtures now map this known setup error to a
fixture diagnostic, matching existing tests. The next run found a Windows pipe
had replaced a literal source-mark character with `?`; the assertion now uses
Rust's explicit Unicode escape for the actual guillemet. A Python edit script
also stopped while reading the existing multilingual example with Windows' default
cp1252 codec, before creating that example or the integration test. Subsequent
edits use explicit UTF-8 mode, preserving the existing translations. No production
limits or assertions were weakened. Focused acceptance passed after repair;
additional wrapping/viewport tests then passed in the final full affected suite.

Local logs: `.git/alo-loop/bounded-native-reader-chrome/`:
`focused.txt` (compile failure), `focused-repaired.txt` (wrong fixture character),
`focused-acceptance.txt` (passing four focused tests before the final added unit
test), `linux-tests.txt` (final full suite), `linux-clippy.txt`, `linux-rustdoc.txt`,
`linux-examples.txt`, `linux-fmt.txt`, `chrome-gles.txt`, `nested-controls.txt`,
`format.txt`, `windows-fmt.txt`, `windows-clippy.txt`, `windows-tests.txt`.
PowerShell wraps native stderr in NativeCommandError even on successful native
commands; actual exit codes and final summaries were inspected. The edit-script
codec failure is retained in the supervisor worker event transcript.

## Remaining evidence and authority

The new evidence is complete chrome layout/raster and real GLES readback, plus
private Wayland client integration. It is not parent navigation-key delivery,
interactive button feedback, on-screen reader publication, direct scanout or
integrated VM boot. Next: keyboard/pointer ownership and interaction feedback
with transactional reader composition/submission/publication/retirement; then
native navigation/cursor selection and direct integration. Wording raster
preparation is complete; full-name access and usable window management are not.
Physical laptop/GPU workstation acceptance remains owed during release validation
after image integration. No hardware certification or release checkbox change.

Tests use private resources; no kernel mutation or outer machine lock. No shared
package/mount/service/session change, WSL restart/helper, cache cleanup, unrelated
host change, additional worker/loop, dev-loop edit, other-checkout edit,
credential/identity access or Git stage/commit/push. Source/new-file diffs reviewed
and `git diff --check` passes. Supervisor owns independent gates and publication.
