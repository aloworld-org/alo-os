# Cooperative window sizing

Date: 2026-09-08. Workstream: native desktop/window management.
Responsible contributor: single desktop development worker in C:\dev\alo-os,
also the integration owner. Status: ready for integration.

## Change and decisions

`crates/alo-shell/src/window_size.rs` adds Server::request_window_size and typed
WindowSizeError. Trusted native controls can suggest a positive logical size to
a mapped same-display toplevel. The API validates raw signed dimensions before
Smithay Size construction, refuses violations of committed client min/max limits
and returns the configure serial or None for an unchanged latest server state.
Zero client limits leave that axis unconstrained. Exact requests refuse rather
than silently clamp so a caller can explain the requested operation's refusal.
The positive i32 protocol range is accepted: this operation allocates no buffer.

Smithay retains ownership of configure serials, acknowledgements and duplicate
suppression. Only pending size is changed; activation, focus, stacking and the
current buffer stay intact. Normal clients can choose another size. Committed
buffer geometry remains authoritative for input and rendering, even after an
acknowledgement. Unmap resets server size state; configured remaps are eligible
again. Public rustdoc and docs/autonomy/COMPOSITOR.md document these limits.
No agent endpoint, command verb, background context, UI string, unsafe exception,
engine patch or new release scope. Agent arrangements still require the existing
grant, proposal and single approval. Interactive Resizing state, drag handles,
placement, tile/maximise and the complete resize feature are not implemented by
this primitive.

Read CLAUDE, delivery/shared-main/report rules, current queue, STATE tail, v0.01
features and roadmap, ADRs 0002/0010 and application-verb/adapter contract sections.
Recorded acceptance in QUEUE before coding. Pinned Smithay EGLDisplay::new at
display.rs:201 is still unsafe under workspace forbid; selected the independently
executable window operation under DELIVERY rather than changing the policy.
Native Rust, client-cooperative protocol sizing and committed geometry follow
the accepted architecture. No additional owner decision was needed.

Four real Wayland socket tests in tests/window_size cover size/serial delivery,
duplicate suppression before and after ack/commit, activation and keyboard
preservation, unaffected clients, client size choice, buffer-driven input extent,
pending versus committed min/max limits, per-axis bounds, zero/negative requests,
maximum positive protocol dimensions and foreign/child/popup/unmapped/dead target
refusals. Fresh remap proves the previous requested state is discarded.
Shared fixture events now retain XDG sizes; a 32x24 SHM helper supplies real
replacement pixels. The ten-stage offscreen fixture checks an unchanged complete
framebuffer during a pending size suggestion and every pixel after an actual
client ack and 32x24 buffer commit. Existing refusal and lifecycle checks remain.

## Executed verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18). Prerequisite commands:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
test -d /dev/dri
```

Versions respectively: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1,
26.0.8-1ubuntu0.3, 0.9.2. WSLg socket present; /dev/dri absent (the last
prerequisite test exits 1). No dependencies or shared kernel/BPF state, services,
other checkout or host configuration changed. Linux commands below ran from
/mnt/c/dev/alo-os with PATH=/root/.cargo/bin:/usr/bin:/bin and the isolated
CARGO_TARGET_DIR=/root/alo-os-target:

```sh
cargo test -p alo-shell --locked window_size
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Final focused run: four passed. Full shell suite: 138 unit, 104 lifecycle,
three socket tests and three compile-fail doctests passed, none failed/ignored.
Affected clippy, rustdoc, examples and fmt passed. The first focused run caught
Smithay's negative Size constructor panic in the test caller (two other tests
passed). Changed the API to validate a raw tuple before constructing Size;
the corrected three tests passed, then added the fourth input/buffer test.
First Linux all-target clippy found the new shared SHM helper unused by the
example. Extended the graphics fixture to exercise an actual acknowledged resize;
reran clippy and all subsequent checks successfully. No suppression or gate
weakening. Routine root-document path/glob and shell quoting lookups corrected;
registry search used grep because Ubuntu rg is absent.

Windows, from C:\dev\alo-os, all passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Windows runs zero Linux-only shell runtime cases. Final format check also passed
in Linux. Source, tracked diff, new files and documentation reviewed; harmless
Git CRLF-normalization notices appeared for two fixture files.

With XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir and WAYLAND_DISPLAY=wayland-0,
both rebuilt graphical commands passed, exit zero:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Pending-size framebuffer preservation, duplicate/invalid requests and acknowledged
32x24 replacement pixels passed. Ten-stage offscreen regression includes the
existing truncated-SHM refusal; nested popup/cursor regression rendered 115 client
surfaces. Expected Mesa fallback and deliberately invalid-client diagnostics
occurred. These are real Wayland/GLES checks, not DRM or physical certification.

## Reconciliation and remaining work

Initial tree clean. Compared published report filenames with STATE; the sole
unreconciled report was deferred-desktop-and-hardware-acceptance.md. Reviewed its
documentation-only diff-check evidence and retained owner sequencing: underlying
window operations precede phase 3 shortcut integration; physical acceptance
follows phase 7 VM image checks in phase 8. No runtime or release tier changed.
Updated QUEUE/ROADMAP/STATE accordingly; no functional changelog entry was needed
for that scheduling clarification. Claude retains security/model-choice work.
Reports arriving during publication reconcile next iteration.

All four shared progress documents record this component and its remaining work.
Next independently executable component: root placement shared by drawing, input
and popup constraints, then interactive resizing and the remaining window
operations. Shortcuts integration follows those operations per DELIVERY.
Launcher/dock, clipboard and all remaining v0.01 requirements stay unfinished.
Full independent Windows/Linux workspace test/lint/rustdoc and BPF publication
gates belong to the supervisor and were not run by this worker. Safe standalone
GLES, full DRM/seat entry, populated input/hotplug, GPU/disable recovery and
certified laptop/GPU workstation acceptance records remain owed at their phases.
No release/feature checkbox ticked; no staging, commit, push, tools/dev-loop edit,
other repository changes, worker/loop launch or physical installation.
