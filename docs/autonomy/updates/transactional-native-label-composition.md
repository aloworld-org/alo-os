# Transactional native label composition

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for integration after the checks recorded below; independent
supervisor publication gates remain pending.

## Change and decisions

Native control names now compose in the same transaction as their strip, clients,
popups and cursor. Fresh hover or identical live mapping-bound native focus selects
the label. The host supplies session vocabulary, existing text scale and a reusable
private shaper through `WindowControlLabelFrame`; `render_labeled_window_controls`
is additive on Server and Nested. Disabled names remain eligible. Missing selection
and held input dismiss labels. Missing vocabulary, invalid geometry, clipping and
control overlap refuse before backend submission. Backend failures retain pending
client callbacks and retire native authority. Only success publishes label bounds.

Opaque labels must not allow pointer input through to a client. A private overlay
module owns the submitted rectangle separately from its held button set. It
consumes covered motion, buttons and scroll, retaining bounds across an event batch
until frame replacement/retirement. Every owned release, including secondary and
chorded buttons, drains after removal, failure or backend deactivation, even without
fresh coordinates. A held overlay gesture suppresses label selection/native focus
and strip hover. Existing client grabs keep their releases; ordinary keyboard
focus and typing remain unchanged. Native labels execute no operation.

The nested adapter now uses the published axis route and drains overlay releases
while inactive. The shared painter, vocabulary, fonts, palette and ADRs 0002/0010
are unchanged. Pointer exclusion belongs beside presentation, not inside immutable
label pixels. This avoids granting authority to a retained raster and prevents
motion followed by a press in one input batch from clicking through stale pixels.

Sources: `crates/alo-shell/src/window_control_frame.rs`,
`window_control_overlay.rs`, `window_control_presentation.rs`,
`window_control_label_target.rs`, `window_control_feedback.rs`,
`nested_control_input.rs`, `presentation.rs`, `server.rs` and exports in `lib.rs`.
Contract: `docs/contracts/native-window-controls.md`.

## Acceptance and verification

Started clean at `59f60c7`. Read required constitution, delivery, ownership/report
guidance, current queue, STATE tail and relevant feature/roadmap/ADR/contract
sections. Every published report already had a STATE reference. Selected the
component and acceptance in QUEUE before code. Reports arriving during publication
are reconciled next iteration; no other contributor's report was edited.

Ubuntu Rust 1.98.0 verified. pkg-config reported wayland-client 1.24.0, EGL 1.5,
GLES 3.2, xkbcommon 1.13.1, libudev 259, libinput 1.31.1, GBM
26.0.8-1ubuntu0.3 and libseat 0.9.2. WSLg socket verified. Each build/test/lint/doc
command had a fresh C: free-space check refusing below 12 GiB. No package install,
shared mount/service/session change, kernel mutation, outer machine lock, cleanup,
WSL restart or helper was used. Linux target remained `/root/alo-os-target`.

Linux commands used `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo check --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked label_frame -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

Focused runs passed the first three then four real-client tests. Five final new
tests are covered by the full Linux suite: 157 unit, 215 lifecycle, three socket
and three compile-fail doctests, none ignored. The final full run follows the
review-added overlay-held hover suppression and assertion. Tests demonstrate:
fresh focus/hover labels; focus not transferred to replacement targets; dismissal;
normal client typing; real underlying-client click/scroll exclusion with positive
controls after removal; primary/secondary release drainage after failure/leave;
client-grab priority; clipping/invalid-size/missing-vocabulary refusal before any
submission; and submission failure preserving pending callbacks before recovery.

Initial clippy caught two redundant tuple conversions in test placements. Removed
both without exemptions; final affected clippy passed. No test failures, retries,
changed deadlines or weakened assertions. Deliberate malformed-client/keymap/SHM
refusal diagnostics are expected successful test paths.

Windows affected commands (Linux-only shell tests execute zero cases on Windows):

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

Graphical commands additionally used
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Nested EGL integration passed eight strip submissions, two complete label
submissions, two label dismissals and two ordinary removals in light/dark, plus
clipping and strip refusal/recovery sequences. Synthetic input at the submitted
label is consumed and secondary releases drain after a real dismissal submission.
Real client callbacks, output metadata, unmap/remap and disconnect also pass.
Offscreen regression passed all 28 stages, twelve complete 57,600-pixel scene
frames and existing label/lifetime/input full-frame checks. Mesa fallback warnings
skipped no assertions. EGL success proves backend acceptance, not on-screen pixel
capture, physical parent-event delivery, direct scanout or certified hardware.
The standalone unchanged label and strip painter executables were not rerun.

## Remaining work and progress integration

This completes nested label composition/dismissal and opaque pointer ownership,
not usable desktop controls or the whole window-management feature. The next
component is alternate full-text access for constrained/clipped names; native
navigation/cursor selection and direct integration follow. Current clipped labels
refuse explicitly. Integrated VM image and subsequent physical acceptance remain
required in their delivery phases; this task supplies neither evidence.

CHANGELOG describes native labels sharing desktop submission without click-through.
ROADMAP and QUEUE record this completed component and the exact remaining work;
STATE references this report and its evidence limits. No feature/release tick is
promoted. Independent supervisor Windows/Linux/rustdoc/BPF gates have not run here.
No staging, commit, push, dev-loop edit, extra worker, other-checkout change,
private credential/identity access or unrelated host changes.

Final verification note: examples rebuilt and `nested_check --controls` passed
again after the held-overlay hover guard. The full Linux suite and affected clippy
also passed after that guard. Offscreen regression preceded only that guard;
its existing painter is unchanged. Warnings-denied rustdoc and both format checks
passed. Minimum observed C: preflight was 56,455,680,000 bytes. All required focused
checks are complete; full independent supervisor gates remain pending.

Tracked and new-file diffs reviewed; git diff --check passed. No publication action
was taken by this worker.
