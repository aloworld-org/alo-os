# Native control label rendering

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Component and decisions

Completed the native text-rendering component selected in QUEUE before writing
code. `window_control_label.rs` prepares the existing control `Action::said`
wording, including disabled actions, into bounded opaque RGBA text. The complete
`Said` and translation/source-fallback provenance remain available; clipping never
replaces or shortens the externalized words. Missing vocabulary, invalid geometry,
empty/oversized text, invalid font data and unsupported glyphs refuse explicitly.
There is no additional English vocabulary, input authority or client context.

ADRs 0002/0010 and the design brief guide routine choices: native Rust shaping
through pinned, unpatched cosmic-text 0.14.2; bundled Inter; existing cream/navy
and charcoal/cream tokens; no terracotta. Fontconfig is disabled, and an explicit
private font database avoids host font discovery. `from_fonts` permits a primary
face and explicit fallbacks for other scripts. The Inter font and OFL license are
under `crates/alo-shell/fonts/`, with upstream revision and SHA-256 provenance.
This adds dependencies and a font asset, not a new release feature or engine patch.

Text uses 14px/20px metrics multiplied by the existing 75..300% TextScale, with
word/glyph wrapping and nominal four-pixel padding. Bearings may extend into that
padding. Boxes are bounded to 2048x512 (at most 1,048,576 RGBA pixels); text is
bounded to 4096 UTF-8 bytes. Full shaping catches missing glyphs even offscreen.
The separate `window_control_label_paint.rs` coalesces equal scanline pixels,
clips to the validated viewport and propagates frame errors. Public rustdoc and
`docs/contracts/native-window-controls.md` define full-text/fallback presentation
and host ownership obligations. No surface, mapping, focus or transaction changes.

This completes a useful immutable label renderer. Live hover/focus selection,
output-bounded placement, dismissal, disabled label access in that live UI and
label-overlay input ownership remain the next component, followed by production
nested/direct composition. No automatic tooltip is installed and no usable-control,
window-management, accessibility-conformance or release completion is claimed.

## Tests and actual verification

Four new unit tests cover translation and source fallback, all three disabled
actions, Greek/Cyrillic/extended Latin and combining marks, 75/100/200/300% scaling,
wrapping, output/text clipping with retained words, invalid dimensions/origins,
missing vocabulary, oversized text, invalid/empty fonts, missing glyphs and valid
rendering after a refusal. These use only private memory/font data. Existing
real-client suites still exercise normal typing, focus and pointer isolation.

Initial focused test: three passed, European-script test failed because a glyph's
negative bearing entered the nominal padding and was incorrectly marked clipped.
Fixed actual ink clipping to use the box edge; did not weaken that assertion.
The next focused run passed all four. Initial all-target Linux clippy reported
potentially panicking vector indexing in raster preparation, painting and tests.
Changed production indexing to checked access with errors and test indexing to
checked assertions. Final clippy passed with no exemptions. No repeated failing
test loop, ignored tests or weakened gate.

Prerequisites verified in Ubuntu: Rust 1.98.0; `pkg-config --modversion
wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat` returned 1.24.0,
1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2. `test -S
/mnt/wslg/runtime-dir/wayland-0` passed. No package installation, shared session,
mount/service changes, kernel mutation, cleanup or outer machine lock was needed.

Before every build/test/lint/format command, PowerShell `(Get-PSDrive C).Free`
was checked against `12GB`. Minimum reading through the verification commands:
67,215,818,752 bytes. No cleanup; the reserve is not a continuous allocation quota.

Linux commands used `PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`, through `wsl -d Ubuntu --exec env`:

```sh
cargo fetch --manifest-path /mnt/c/dev/alo-os/Cargo.toml
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked control_labels -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --offline control_labels -- --nocapture
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

The first focused test failure and initial clippy failure are described above;
clippy ran twice. The offline focused run refreshed the lockfile after disabling
fontconfig. Final full shell suite passed 156 unit, 195 lifecycle, three socket
tests and three compile-fail doctests, none ignored. All examples built, rustdoc
passed with warnings denied and Linux formatting passed. Existing malformed-client
and keymap diagnostics were refusal fixtures, not skipped failures.

Windows commands in `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed; formatting ran as edits progressed. Windows executes zero Linux-only
shell tests. These are affected-target checks, not supervisor workspace gates.

WSLg commands also set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_labels_check
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

All passed without timeout. New label fixture verifies 72 complete 57,600-pixel
GLES frames against the prepared CPU raster and untouched black ground: all three
actions, translated/source fallback, light/dark, 100/200/300% and four clipping
origins including full occlusion. It asserts actual ink exists separately; the
CPU shaping tests validate text/script/scale/refusal behavior. This checks upload/
drawing/clipping, not an independent implementation of typography. Existing
896 complete control painter frames and all 28 nested offscreen stages pass,
retaining live snapshot/feedback and complete minimize/restore scene assertions.
Mesa fallback warnings and deliberate malformed-SHM diagnostics skipped no checks.

## Contributor reconciliation and evidence limits

Started clean at `024b309`; read constitution, delivery/shared ownership rules,
report guidance, current queue, journal tail and relevant feature/roadmap/ADR/
contract sections. The sole published report not yet referenced in STATE was
`docs/autonomy/updates/the-daemon-fetches-and-connects.md`. Read it and reviewed
its two daemon fixtures in `crates/alo-agentd/src/doing.rs`. It reports real-store
four-refusal no-provider/no-fallback evidence and a connection only to the chosen
provider after retrieval. It provides no exact verification commands or numeric
results; those tests were not independently rerun in this desktop task.

The report's authenticated-HTTPS key-delivery assertion is blocked on a
trust-anchor decision: production currently uses compiled Mozilla roots and
loopback classification precludes the proposed owned fixture. This reconciliation
does not approve system roots, an extra-root setting or an ADR change. It retains
that evidence limit and Claude's independent lifetime/concurrency/logout work.
The source report and Claude's checkout/workstream were not edited. Reports
arriving during publication are reconciled next iteration.

CHANGELOG, ROADMAP, QUEUE and STATE updated together. Full independent Windows/
Linux workspace lint/tests/rustdoc and pinned BPF gates remain the supervisor's
work. On-screen native label interaction, direct DRM/input/scanout and populated
session recovery are not proved here. Integrated VM boot/update recovery and
certified laptop/GPU workstation records remain due at their scheduled phases.
No hardware or release certification follows from these offscreen fixtures.

Tracked changes and new source/test/font/report files reviewed; whitespace clean.
No staging, commit, push, dev-loop edit, worker/loop launch, credential/identity
access, other-checkout edits or unrelated host changes.
