# Nested native control event routing

Date: 2026-09-09. Workstream: native desktop. Responsible contributor: desktop
integration worker in `C:\dev\alo-os`. Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/nested_control_input.rs` owns the actual parent position
independently of client focus/motion. `Nested::pump_seat` uses it to route through
the published mapping-bound strip. Native gestures consume motion without losing
the actual next release position; out-and-back motion cannot rearm execution.
Inactive input retires controls and forgets position, but drains owned releases.
Fresh motion is required before new buttons/scroll. Invalid coordinates retire
presentation before refusal using the existing pointer coordinate limits.
Scroll dismisses native label focus. Client buttons/scroll and normal keyboard
input retain their existing routes. No agent surface or user-facing string added.

The pump retains the first error, stops subsequent routing in that pump and clears
input before returning. Native errors retain their typed cause through the additive
`RenderError::WindowControl` variant. The existing explicit `nested_pointer` API
remains available; neither it nor the keyboard-only pump is silently repurposed.
One adapter belongs to one backend/server lifetime. Its coordinates are not a
window authority token. The existing presentation/transaction policy, vocabulary,
tokens and ADRs 0002/0010 remain authoritative. No new approval or ADR needed.

`tests/window_controls/nested_input.rs` adds four real-client happy/refusal tests:
exactly-once close and duplicates, actual client typing/buttons/scroll, owned
out-and-back motion without client leakage, inactive/reactivated release draining,
fresh-motion requirements, and NaN/infinite/out-of-range motion retirement.
`examples/support/window_control_label_check.rs` adds eight complete 5,760-pixel
light/dark frames through this adapter, proving label hover/press dismissal,
minimization and deactivation. The actual root leaves the mapped set on release.

## Executed checks

Started clean at `54a63a8`. Read constitution, delivery/ownership/report guidance,
current queue/journal and relevant feature, roadmap, ADR and contract sections.
All published task reports already had STATE references at iteration start.
Selected this bounded event-routing component and acceptance in QUEUE before
implementation. Reports arriving during publication reconcile next iteration.

Ubuntu Rust 1.98.0 (88d9e12ae); pkg-config reports wayland-client 1.24.0, EGL 1.5,
GLES 3.2, xkbcommon 1.13.1, libudev 259, libinput 1.31.1, GBM 26.0.8-1ubuntu0.3,
libseat 0.9.2. `/mnt/wslg/runtime-dir/wayland-0` exists. No dependency installation,
shared maintenance, kernel mutation, outer fixture lock, cleanup, helper or WSL
restart. Each format/build/test/lint command had a fresh Windows C: preflight,
refusing below 12 GiB. Minimum reading was 60,557,508,608 bytes; this is operational
headroom, not a running quota. Targets remained checkout-private.

Linux commands used `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked window_controls_nested_input -- --nocapture
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

Four focused tests passed. Full shell tests passed 156 unit, 207 lifecycle, three
socket and three compile-fail doctests, none ignored. First clippy found four
test-helper lock unwraps and one unchecked index; converted these to explicit
errors without exemptions or changed assertions. Clippy then passed, and focused
tests reran after those test-only fixes. Examples and warnings-denied rustdoc pass.
No production code changed after the full Linux suite. Expected malformed-client
and invalid-keymap refusal diagnostics skipped no checks.

Windows affected commands passed (shell tests run zero Linux-only cases):

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

WSLg set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Exit zero, no timeout: eight new complete label frames and real minimization,
existing ten lifetime/fourteen live-label frames and all 28 nested stages pass.
Mesa fallback and deliberate truncated-SHM diagnostics skipped no assertions.
The separate unchanged control-label and strip-painter executables were not rerun
in this iteration. The supervisor owns independent full publication gates.

## Remaining evidence and progress

This completes the parent-event adapter and its seat-pump wiring, not the whole
nested desktop integration or window-management feature. Next: compose strip and
labels, establish overlay hit policy and readable full clipped-name access;
native navigation, cursor integration during native grabs and corresponding direct
integration remain. Parent cursor-leave notification is still absent in Smithay
0.7. Synthetic adapter inputs and GLES offscreen pixels do not prove actual parent
event delivery, on-screen interaction, direct DRM/libinput/scanout, integrated VM
boot/update recovery or certified laptop/GPU hardware acceptance. Physical records
remain at release validation after image integration. No feature/release tick.

CHANGELOG, ROADMAP, QUEUE and STATE updated together. Contract and public rustdoc
updated. Tracked/new diff reviewed and whitespace checked. Independent supervisor
Windows/Linux/workspace/rustdoc/BPF gates remain pending. No staging, commit, push,
dev-loop edit, worker launch, other-checkout edit, credential/identity access or
unrelated host changes.
