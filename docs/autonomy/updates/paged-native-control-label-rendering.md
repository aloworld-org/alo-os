# Paged native control label rendering

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor integration gates. No feature/release tick.

## Change and decisions

Long control names now have a bounded page preparation and rendering component.
`WindowControlLabels::prepare_pages` shapes the whole externalized name once and
partitions complete visual lines, preserving wrapping, glyph clusters, words,
translation/source provenance and text scale. Immutable pages expose line ranges,
bounds, pixels and the existing GLES scanline painter. Disabled controls retain
their names. Single-page output matches ordinary label preparation byte for byte.

The page type deliberately cannot become a `WindowControlLabel` supplied to the
ordinary complete-label scene; a compile-fail doctest enforces that distinction.
This prevents an integration caller from passing just the first page as a full
name. No live presentation state, client context or input authority is added.

Geometry and selection must match the layout, fit the output and leave controls
unobscured. Every glyph is checked before pagination, including later pages.
Insufficient line/ink space, excessive text, missing vocabulary/glyphs, invalid
geometry and foreign selection refuse atomically, with no partial set returned.
The existing 4096-byte wording and per-raster size limits remain. New limits of
128 pages and 4,194,304 aggregate pixels cap prepared reader pixels at 16 MiB;
both are checked before allocating page rasters. This bounds memory independently
of text length. It is an internal resource choice consistent with ADRs 0002/0010,
not a new palette, font, agent surface or product scope decision.

Sources: `crates/alo-shell/src/window_control_label_pages.rs` (page planning/API),
`window_control_label_page_raster.rs` (whole-line rasterization), shared shaping in
`window_control_label.rs`, module exports in `lib.rs`, page unit tests and
`examples/control_label_pages_check.rs`. The public native-control contract is
updated in `docs/contracts/native-window-controls.md`.

This completes the useful page preparation/rendering component, not the reader
feature. Next: live mapping-bound page navigation, externalized page position
wording, input ownership, submission/publication and retirement; then native
navigation/cursor selection and direct integration. Existing Server/Nested label
transactions still refuse names that cannot expand completely. Ordinary client
typing and applicable component tests remain intact. Full-name access and usable
window management are unfinished.

## Verification actually executed

Required constitution/delivery/ownership/report instructions, current queue,
STATE tail, relevant feature/roadmap, ADR 0002/0010 and native-control contract
were read. Started clean. The selected component and acceptance were written to
QUEUE before implementation. Every published task report was already referenced
in STATE at iteration start; there was no new contributor report to reconcile.
Reports arriving during publication belong to the next iteration.

Ubuntu Rust 1.98.0. `pkg-config --modversion wayland-client egl glesv2 xkbcommon
libudev libinput gbm libseat` confirmed respectively 1.24.0, 1.5, 3.2, 1.13.1,
259, 1.31.1, 26.0.8-1ubuntu0.3 and 0.9.2. The WSLg Wayland socket existed.
Each build/test/lint/format command had a fresh Windows C: reserve reading above
12 GiB. Lowest recorded preflight: 55,149,518,848 bytes. Desktop Linux target
remained `/root/alo-os-target`. No package install, cleanup, mount/service/session
change, WSL restart or keep-alive helper. These private graphics/socket fixtures
make no kernel-global changes; no outer machine lock was added.

Linux commands were invoked through `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked paged_names -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

Rustdoc additionally used `RUSTDOCFLAGS=-Dwarnings`. Final affected suite passes:
162 unit tests, 216 real-client lifecycle tests, three socket tests and four
compile-fail doctests; zero failures/ignored cases. Three new unit tests cover
full line coverage, full-raster equivalence, explicit/wrapped lines, translations
and fallback, disabled controls, light/dark, 100/125/200/300% text, out-of-range
page lookup and atomic happy/refusal boundaries. The full suite preserves normal
typing, callback, overlay and lifecycle coverage. All affected Linux clippy,
example compilation, rustdoc and formatting checks pass.

Graphical commands additionally used `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir
WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_label_pages_check
timeout 30s /root/alo-os-target/debug/examples/control_labels_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

All three passed on their first execution in this iteration. New page check:
72 complete 57,600-pixel GLES frames, all pixels compared, forward/reverse visits,
translation/fallback, light/dark and four scales. Existing label check: 72 full
frames with three controls and four clipping origins. Actual nested check: eight
strip submissions, two labels, two expanded labels, two dismissals, two ordinary
removals, exhausted-space/strip refusal and recovery; six submitted client
surfaces with unmap/remap/disconnect coverage. Mesa/EGL driver discovery warnings
occurred, but rendering/readback/submission succeeded without suppressed output.
Malformed-client diagnostics are asserted refusal paths. No graphical timeout.

Windows commands passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows shell cases are Linux-only: zero Windows test cases executed. This is
affected-target compilation evidence, not a native Windows desktop test.

## Development failures and repairs

Initial focused compilation used nonexistent `Vocabulary::new` in a test; changed
to the actual `Vocabulary::empty` API. Next focused run had two passes and one
failed budget assertion: 100 lines occupied exactly four permitted 2048x512
pages. The test now asserts acceptance at that boundary and refusal for 101 lines
(five pages). Production limits were unchanged; corrected focused run passed.
Initial clippy found unchecked indexing and missing private-field documentation;
replaced indexing with checked access and documented fields, with no exemptions.
Final clippy, tests and graphical acceptance all passed after these repairs.

Local evidence: `.git/alo-loop/paged-native-control-label-rendering/`, including
`development-diagnostics.txt`, `focused.txt`, `linux-tests.txt`,
`linux-tests-final.txt`, initial `linux-clippy.txt`, `linux-clippy-final.txt`,
`linux-examples.txt`, `page-gles.txt`, `label-gles.txt`, `nested-controls.txt`,
`linux-rustdoc.txt`, Windows clippy/tests and both fmt logs. PowerShell formats
native stderr as NativeCommandError records even on successful commands; actual
process exit statuses and final results were checked.

## Integration status and evidence limits

CHANGELOG, ROADMAP, QUEUE and STATE updated with this component and remaining
reader work. Tracked and new-file diffs reviewed; `git diff --check` passes.
Full independent supervisor Windows/Linux/workspace/rustdoc/BPF publication gates
remain pending and are not claimed here. No staging, commit, push, dev-loop edit,
extra worker, other-checkout changes, private credential/identity access or
unrelated host changes.

New evidence is offscreen GLES page readback on WSLg and existing nested EGL
submission. It is not live page navigation, actual parent key delivery, on-screen
pixel capture, direct scanout, VM image integration or physical certification.
Physical laptop/GPU workstation records remain owed at release validation.
No feature, compositor or release checkbox is promoted.
