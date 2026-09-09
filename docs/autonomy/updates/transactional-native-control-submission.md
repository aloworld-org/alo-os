# Transactional native control submission

Date: 2026-09-09. Workstream: native desktop. Responsible contributor: desktop
integration worker in `C:\dev\alo-os`. Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/window_control_frame.rs` owns a complete strip transaction:
explicit live root, origin and pointer feedback are refreshed against the actual
backend viewport, submitted with clients/popups/cursor, then published through the
existing mapping-bound lifetime. No client dispatch occurs between those stages.
The private frame-target adapter preserves existing output/callback publication.
`Nested::render_window_controls` uses its owned parent position, independently
of client motion frozen during native grabs. Selection remains the host's choice.

`FrameTarget::submit_controls` is additive: legacy implementations refuse Some,
forward None to their existing scene path. Nested implements the shared painter.
Distinct `RenderError` variants preserve unsupported-backend, omitted-root and
live snapshot refusal details. A root omitted by the backend cannot gain native
input authority or callbacks. Backends remain trusted to report actual drawing.
Any failure retires authority/focus and cancels execution but retains release
ownership. Explicit None and ordinary `Server::render` remove authority even on
failed removal. Same-frame refresh preserves valid gestures and native focus.

This completes the strip transaction component selected in QUEUE before coding.
Labels remain excluded from this high-level path until their opaque overlay input
policy is implemented. The next component composes/dismisses fresh labels with
that policy, then full-text alternatives, navigation/cursor selection and direct
integration remain. No window-management feature or release box is ticked.
No new vocabulary, palette, font, agent surface, approval or ADR; this reuses
ADRs 0002/0010 and existing mapping, rendering and callback contracts.

## Acceptance and verification

Started clean at `4c10814`. Read constitution, delivery order, current queue,
journal tail, ownership/report guidance and relevant feature/roadmap, ADR and
contract sections. One published report lacked a STATE reference:
`an-administrator-set-that-rule.md`. Reviewed its ordering and origin handling
against code and retained its evidence limits in all four progress documents.
The report supplies mutation claims but no exact commands/results listing;
production still supplies no bound and pre-turn refusals have no record entry.
Settings integration remains unfinished. No contributor source/report was edited.
Reports arriving during publication are reconciled next iteration.

Ubuntu Rust 1.98.0; pkg-config verified wayland-client 1.24.0, EGL 1.5, GLES 3.2,
xkbcommon 1.13.1, libudev 259, libinput 1.31.1, GBM 26.0.8-1ubuntu0.3 and libseat
0.9.2. WSLg socket verified. All build/test/clippy/doc commands had fresh Windows
C: preflights above 12 GiB; minimum 58,797,293,568 bytes. No routine package install,
shared service/mount/session change, kernel mutation, outer fixture lock, cleanup,
WSL restart or helper. Targets remained separate at `/root/alo-os-target`.

Linux commands used `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked control_frame -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
```

Three new real-client tests passed: fresh operation availability after output
publication; normal typing/focus; same-frame held close exactly once; successful
callbacks; failed submission, omitted target, foreign target and invalid geometry;
failed/successful ordinary removal; pending callbacks; cancelled release ownership;
replacement; and legacy refusal before submission. Full shell suite passed twice:
157 unit, 210 lifecycle, three socket and three compile-fail doctests, none ignored.
The final run includes the distinct diagnostic variants and exact refusal assertions.
Focused tests also passed again after initial test-helper lint fixes. Initial clippy
caught an expect and panic in test helpers; replaced with exhaustive array access
and explicit submission-call accounting, with no exemption or weakened assertion.
Final affected clippy passed. Examples built. Both format checks passed.

Windows affected checks passed; Linux-only shell tests execute zero cases here:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

Graphical commands additionally set
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

The first --controls invocation failed its unchanged internal ten-second client
deadline before the new checks; the client connection reset as the fixture exited.
No causal diagnosis was established. Rustdoc was running concurrently in this
checkout. One unchanged rerun with no build running from this worker passed in
4.65 seconds: six actual EGL strip submissions in light/dark, two ordinary removals,
two refusal/recovery sequences, cancelled releases and actual client callback,
output metadata, unmap/remap and disconnect checks. No timeout or test was weakened;
no process/service was stopped to obtain that result. The initial failure remains
part of the evidence rather than being relabelled a success.

Offscreen regression passed all 28 stages, twelve full scene frames and existing
label/input/lifetime checks. Mesa fallback and intentional malformed-client/SHM
refusals skipped no assertions. These graphical runs preceded only the distinct
unsupported/omitted-target diagnostic refinement, which the final full suite and
clippy cover. No graphical success-path logic changed afterwards.

## Limits and remaining work

Actual nested EGL submission proves the backend accepted the frame, not a captured
on-screen pixel result or physical scanout. New gesture input is synthetic trusted
routing, not a physical pointer or parent-event injection. Existing offscreen
complete-frame comparisons cover the shared painter but not independent on-screen
orientation. Direct DRM/libinput integration, usable label input/navigation,
VM image boot/recovery and physical laptop/GPU acceptance remain owed in their
scheduled phases. WSL evidence never certifies hardware. Existing missing parent
cursor-leave notification limitation remains.

User-readable change: failed desktop frames cannot leave their window controls
authorized to act; pending callbacks and owned releases remain accounted for.
Contract and four progress documents carry this and the contributor reconciliation.
Full independent supervisor Windows/Linux/workspace/rustdoc/BPF publication gates
remain pending. No staging, commit, push, dev-loop edit, worker launch, credential
or identity access, other-checkout modification or unrelated host action.

Final warnings-denied rustdoc passed after the diagnostic refinement. Tracked and
new-file diff reviewed; git diff --check passed. Full supervisor gates remain pending.
