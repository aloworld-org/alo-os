# Native control scene composition

Date: 2026-09-09. Workstream: native desktop. Responsible contributor: desktop
integration worker in `C:\dev\alo-os`. Status: ready for integration.

## Change and decisions

`src/window_control_scene.rs` in `crates/alo-shell` adds borrowed, per-frame native
content. `Nested::submit_control_scene` and `render_control_scanout` use the same
GLES painter: client roots/popups, native strip, complete label, then custom cursor
or compositor arrow. Existing entry points pass None. No native content is cached.
Cursor textures remain retained through frame finish; returned surface identities
keep their previous order. Native pixels add no client identities or callbacks.
Clients behind opaque native content remain in the drawn identity list.

`RenderError::ControlScene` refuses mismatched output/layout/label viewports,
clipped labels and labels covering any control before client import. Offscreen
validation precedes allocation too. This prevents silently painting incomplete
names or obscuring controls; it does not supply an alternate full-text reader.
The host must provide a larger label or a separate full-text surface.

Composition and live window authority stay separate. The caller must refresh the
mapping without interleaved dispatch, publish only successful composition and
retire on failure/removal. An opaque label still needs host overlay input policy
before interactive use. Existing normal typing and input routes are unchanged.
No new strings, agent surface, palette, approval or ADR; uses accepted ADRs
0002/0010, existing action vocabulary, Inter and token painters.

Acceptance: complete layer ordering, geometry happy/refusal cases, unchanged
surface identities and callbacks, removal and recovery after rejection. One new
unit test covers disabled labels, mismatched geometry, overlap and clipping.
`examples/support/window_control_scene_check.rs` adds eight complete 57,600-pixel
custom-cursor frames and four arrow frames using a real mapped root, child and
popup in both schemes. Custom cursor positions cover strip and label separately.
Expected strip/arrow masks are independent of production painters; label pixels
are compared against the prepared raster (not independent shaping evidence).
Existing client callback/membership assertions still run after preparation.

## Executed checks

Started clean at `003c928`. Read constitution, delivery/ownership/report guidance,
current queue and journal, relevant feature/roadmap, ADR and contract sections.
All published reports already had STATE references at iteration start. Selected
the bounded composition component and acceptance in QUEUE before implementation.
Reports arriving during publication are reconciled next iteration.

Ubuntu Rust 1.98.0; pkg-config: wayland-client 1.24.0, EGL 1.5, GLES 3.2,
xkbcommon 1.13.1, libudev 259, libinput 1.31.1, GBM 26.0.8-1ubuntu0.3,
libseat 0.9.2. WSLg Wayland socket verified. No dependency install or shared
maintenance. Fresh Windows C: preflights before build/test/lint phases all exceeded
12 GiB; minimum observed 60,159,377,408 bytes. Separate desktop target retained.
No kernel mutation, outer fixture lock, cleanup, service/mount change, WSL restart
or keep-alive helper. Two tool calls returned unusually late; their owned process
sessions were polled to completion, without restarting or launching replacements.

Linux used `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked control_scene -- --nocapture
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --example nested_check --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
```

Focused test passed. Full Linux shell suite: 157 unit, 207 lifecycle, three socket
and three compile-fail doctests passed, none ignored. Initial clippy found a
collapsible conditional and test-module placement; fixed without exemptions.
Final affected clippy passed after the final code/test edits. Examples built.
Full tests cover final production logic; later changes were fixture assertions
and rustdoc wording. Linux format and warnings-denied rustdoc passed.

Windows affected checks passed (shell tests run zero Linux-only cases):

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

First run failed because the fixture incorrectly expected maximize enabled before
any output submission. Corrected its independent mask and added an explicit
OutputUnavailable assertion; no production policy or assertion weakened. Second
run exited zero: twelve new full composition frames, removal/refusal recovery,
existing label lifetime/event checks and all 28 nested stages passed. Mesa fallback
and intentional truncated-SHM diagnostics skipped no assertions. No test timeout.
Standalone unchanged strip/label painter executables were not rerun.

## Remaining evidence and progress

This completes shared scene rendering, not installed usable controls. Next: nested
host transaction connecting fresh live snapshots, submission, publication and
failure/removal retirement, with label overlay input policy. Alternate full-text
access, native navigation/cursor selection and direct installation remain.
Offscreen normal-transform pixels do not prove flipped nested on-screen control
submission, real parent input delivery, direct DRM/libinput/scanout or hardware.
VM boot/update recovery and physical laptop/GPU acceptance remain later release
evidence. Existing parent cursor-leave notification limitation is unchanged.

Contract and all four progress documents updated. Full supervisor Windows/Linux/
workspace/rustdoc/BPF publication gates remain pending; no feature/release tick.
No staging, commit, push, dev-loop change, worker launch, other-checkout edit,
credential/identity access or unrelated host changes.

Final tracked and new-file diff reviewed; git diff --check passed.
