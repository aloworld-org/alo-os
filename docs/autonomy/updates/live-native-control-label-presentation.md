# Live native control label presentation

Date: 2026-09-09. Workstream: native desktop. Responsible contributor: desktop
Codex worker in C:\dev\alo-os, sole integration owner.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Added `crates/alo-shell/src/window_control_label_target.rs`, exported through
`src/lib.rs`: a read-only per-frame selector returning the current WindowControl
and output-contained LabelGeometry. Hover uses the shared clipped hit test;
explicit native focus selects an action in the current strip, including disabled
controls. No client keyboard focus is inferred and no mapping or authority is
retained. All held native presses, including cancellation, and existing competing
input ownership dismiss labels. Missing/foreign/hidden/unmapped roots, absent
presentation, nonhits, fully clipped controls and explicit dismissal return None.

Size uses the existing shaper bounds, shrinks to the output, aligns to the visible
control's left edge and prefers a four-pixel gap below, then above, then clamps.
Invalid geometry and selected outputs below nine pixels refuse before shaping.
The existing shaper retains full strings/provenance and clipping status. This adds
no palette, font, user-facing vocabulary, input owner or agent surface. These are
routine choices consistent with ADR 0002's native shell and ADR 0010's tokens.
No new approval or scope decision was required.

User-readable change: window-control labels can now follow current hover or
explicit native focus, keep disabled names accessible and disappear when a window
is hidden or input is held. Label boxes stay within the output.

Acceptance checks are in `tests/window_controls/labels.rs`: three real-client
tests cover disabled hover/focus through actual text preparation, normal typing,
no configure/close/focus side effects, below/above/clamped and tiny output placement,
invalid geometry, NaN/gap/outside/clipped hits, explicit/absent dismissal, held and
cancelled presses, client-held buttons, hide/reveal, unmap and foreign roots.
`examples/support/window_control_label_check.rs`, called by the existing minimize
fixture, adds fourteen complete 5,760-pixel light/dark frames for visible labels
and cleared frames after hiding, over three hide/restore cycles. All pixels are
compared to the prepared CPU raster or black ground. This proves live selection,
placement, painting and removal together, not independent font shaping or desktop
scanout. The contract documents freshness, clipping and overlay responsibilities.

## Executed verification

Started clean at 671d83a. Read constitution, delivery order, report guidance,
shared-main ownership, current queue/journal and relevant features/roadmap/ADRs/
native-control contract. Listed reports not referenced in STATE: none. No report
reconciliation outstanding at iteration start; reports arriving during publication
belong to the next iteration. Claude's checkout/workstream remained untouched.
Selected this complete component and acceptance checks in QUEUE before coding.

Ubuntu prerequisites: `rustc --version` returned 1.98.0 (88d9e12ae).
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm
libseat` returned 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3,
0.9.2. `test -S /mnt/wslg/runtime-dir/wayland-0` passed. No dependencies installed.

Every build/test/lint/format command was preceded by `(Get-PSDrive C).Free` and
refusal below `12GB`. Lowest command preflight: 62,309,507,072 bytes. This is a
preflight reading, not a running disk quota. No cleanup, shared maintenance,
service/session/mount change, WSL restart or keep-alive helper was performed.
Fixtures use private protocol resources; no new kernel mutation or outer lock.

Linux commands ran via `wsl -d Ubuntu --exec env`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked window_control_labels_live -- --nocapture
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

The first focused command failed to compile because the new test attempted `?`
on TextError, which does not implement std::error::Error. Corrected the fixture
conversion without changing assertions; the second focused invocation passed all
three tests. All subsequent listed commands passed. Full Linux shell suite:
156 unit, 198 lifecycle, three socket and three compile-fail doctests; no ignored
tests. Existing malformed-client/keymap diagnostics were expected refusal cases.
Clippy passed first run. No repeated-failure loop, suppression or weakened test.

Windows commands in C:\dev\alo-os all passed:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

Formatting ran twice as edits progressed. Windows executes zero Linux-only shell
tests. These are affected-target checks, not independent workspace gates.

WSLg commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/control_labels_check
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

All passed without timeout: all 28 nested stages plus fourteen new live label
frames, 72 complete 57,600-pixel prepared label frames and 896 complete control
frames. Mesa fallback warnings and deliberate malformed-SHM diagnostics skipped
no assertions. Linux rustdoc and both formatting checks passed.

## Remaining work and integration

The selector requires current host input every frame: pass Dismissed on leave,
focus loss or backend deactivation; never carry focused actions across target
replacement/remapping. None/error means discard the previous label. No persistent
focus dispatcher is implemented. Production nested control composition/event
ownership, focus lifetime, label overlay hit policy and readable full-text access
when clipping occurs are the next component, then matching direct integration.
On a small output the label may overlap the strip; this read-only API adds no hit
area or click-through policy. Usable window controls remain incomplete.

CHANGELOG, ROADMAP, QUEUE and STATE updated in this change; no feature/release tick.
The new contract section describes the additive Rust surface and exact limits.
Independent full Windows/Linux workspace lint/tests/rustdoc and BPF gates remain
the supervisor's work. On-screen interaction, direct DRM/input/scanout, integrated
VM boot/update recovery and certified laptop/GPU records are not proved by these
fixtures; machine acceptance remains at the scheduled delivery phases.

Diff and new files reviewed; `git diff --check` passed. No staging, commit, push,
dev-loop modification, worker/loop launch, private credential/identity access,
other-checkout edit or unrelated host action. Release remains unverified.
