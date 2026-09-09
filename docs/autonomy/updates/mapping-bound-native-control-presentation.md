# Mapping-bound native control presentation

Date: 2026-09-09. Workstream: native desktop. Responsible contributor: desktop
Codex worker in C:\dev\alo-os, sole release-progress integration owner.
Status: ready for integration; independent supervisor publication gates pending.

## Change, decisions and acceptance

Added `crates/alo-shell/src/window_control_presentation.rs`: one server-owned
explicitly painted root, existing visibility identity, viewport/origin, observed
maximize/restore intent and native label focus. Reuse of the existing Arc identity
detects hide/reveal and unmap/remap even between host observations. No protocol
identity is treated as a mapping lifetime, and no focused/stacked fallback target
is selected. Keeping the state on Server avoids multiple independently retained
host focus caches. This is the complete target/focus lifetime component of the
next nested composition task, not completion of usable window controls.

`present_window_controls` accepts current successful host composition; None or a
failed candidate retires the old strip. Replacement and geometry/intent changes
cancel execution but retain owned release consumption. The same live frame keeps
focus and a valid held gesture. Fresh feedback, label selection and pointer routing
revalidate this state before using existing policy/selectors/router. Native focus
is explicitly selected from visible controls, including disabled names, without
client keyboard authority. Pointer routing clears it; observed competing/held input
clears it rather than letting it return when the grab ends. No hover is cached.

`pointer_leave` and `clear_input` now retire presentation before capability-dependent
cleanup. Nested/direct pointer deactivation reaches this hook even without pointer
capability. Existing release ownership and keyboard routing remain intact. Hosts
must still retire on their own submission/output-loss paths, publish only after
composing the actual mapping and observe ownership changes as they happen. The
contract explicitly forbids mixing this lifecycle with independently supplied
low-level painted targets. No actual nested event pump opts in yet.

These routine choices follow ADR 0002's native Rust shell and ADR 0010's existing
tokens; no new palette, strings, shortcuts, protocol, agent surface or release
scope. No additional approval was needed. Public/private rustdoc and the additive
`docs/contracts/native-window-controls.md` section describe the precise boundary.

User-readable change: native control labels lose focus when their window
presentation is retired, and held native actions stay cancelled after replacement
or remapping without leaking the release to an application. Unchanged frames keep
valid interactions. Disabled names and ordinary client typing remain available.

Five new real-client tests in `tests/window_controls/presentation.rs` cover stable
refresh, disabled native focus, live feedback, fresh/uncached hover, invalid label
geometry, exactly-once close, duplicate release, ordinary typing/configure isolation,
non-strip/clipped focus refusal, replacement/relayout/invalid/foreign/removal,
unobserved hide/reveal, unmap/remap, death, observed maximize intent, client-held
input and all existing leave/nested/direct/reset paths with and without pointer.
Tests use private protocol fixtures; no new kernel mutation or outer machine lock.

The nested offscreen fixture adds ten complete 5,760-pixel light/dark GLES frames:
same-frame focused label, disappearance after unobserved hide/reveal, no focus
resurrection after republishing, explicit fresh focus and explicit retirement.
Every pixel is compared with the prepared label raster or cleared background.
This proves lifecycle through selection/preparation/painting/removal together,
not independent font shaping, automatic event-pump dispatch or on-screen scanout.

## Executed verification

Started clean at `8bb8165`. Read constitution, delivery order, shared-main/report
guidance, current queue/journal and relevant feature/roadmap/ADR/contract sections.
No published report lacked a STATE reference at iteration start. Selected this
component and acceptance in QUEUE before implementation. Reports arriving during
publication reconcile next iteration; no release verification claim.

Ubuntu prerequisite check: Rust 1.98.0 (88d9e12ae); `pkg-config --modversion
wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat` returned 1.24.0,
1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3 and 0.9.2. The WSLg socket
`/mnt/wslg/runtime-dir/wayland-0` exists. No dependency installation or maintenance.

Every build/test/lint/format command had a fresh Windows `(Get-PSDrive C).Free`
preflight with refusal below `12GB`. Lowest command reading: 62,192,496,640 bytes.
This is preflight headroom, not a running quota. No cleanup, shared service/session/
mount change, WSL restart, keep-alive helper, other-checkout edit or process stop.

Linux commands ran through `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin` and `CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked window_controls_presentation -- --nocapture
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --no-deps --locked
cargo fmt --manifest-path /mnt/c/dev/alo-os/Cargo.toml --all --check
```

First focused invocation: four pass, one fails because the fixture assumed focus
had not sent an existing configure. Captured the synchronized post-focus configure
list and compared it unchanged, preserving the no-presentation-side-effects test.
Focused rerun passed all five. First clippy failed on eight undocumented private
fields/methods; documented them and the extracted example helper, with no exemption.
Clippy then passed. Full Linux shell suite passed 156 unit, 203 lifecycle, three
socket and three compile-fail doctests, none ignored. Expected malformed-client
and invalid-keymap diagnostics were refusal cases. Examples and rustdoc passed.
Final review added happy-path feedback/hover and label-geometry assertions to the
same test; all five focused tests and affected clippy passed again. No production
behavior changed after the full suite/graphics checks. No repeated-failure loop.

Windows commands in C:\dev\alo-os passed:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

Formatting ran as edits progressed, with final Windows/Linux checks. Windows shell
tests execute zero Linux-only cases. These are affected-target checks, not full
independent workspace gates. The final assertion additions were rerun on Linux.

WSLg executions set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/control_labels_check
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

All exited zero without timeout: ten new complete lifetime frames, the fourteen
existing live label frames and all 28 nested stages; 72 complete 57,600-pixel
prepared-label frames; 896 complete control painter frames. Mesa fallback warnings
and deliberate truncated-SHM refusal diagnostics skipped no checks.

## Remaining work and integration

Next: actual nested strip/label composition and event pumping using the completed
lifetime, label overlay hit policy and readable full-text access when clipping
occurs; native keyboard navigation and matching direct backend integration remain.
The existing selector can put a label over the strip on a small output; this change
adds no overlay hit area or click-through decision. It installs no native controls.

Independent supervisor full Windows/Linux tests/lints/rustdoc and pinned BPF gates
remain pending. WSLg component checks do not prove direct DRM/libinput/scanout,
on-screen interaction, integrated VM image boot/update recovery or certified
laptop/GPU hardware acceptance. The integrated image and physical records remain
at their scheduled phases. No feature/release checkbox was promoted.

CHANGELOG, ROADMAP, QUEUE and STATE updated together. Tracked diff and new files
reviewed; `git diff --check` passed. No staging, commit, push, dev-loop changes,
worker/loop launch, private credential/identity access or unrelated host changes.
