# Native control pointer routing

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Completed the queued primary/motion router component, selected in QUEUE before
implementation. `crates/alo-shell/src/window_control_routing.rs` adds the trusted
`Server::route_window_control_pointer` boundary: current explicit painted root,
viewport/origin, position and timestamp; typed native/client outcomes and refusals.
Native primary gestures and held motion are consumed; other input uses the existing
client route. Disabled hits and failed native releases never fall through. Keyboard
and axis APIs are unchanged. No agent verb, vocabulary, dependency or stored format
was added. Public rustdoc and `docs/contracts/native-window-controls.md` document
the additive surface and its host responsibilities.

The router checks the current painted root on every event. Replacement or removal
permanently cancels the old transaction even at identical geometry; returning to
the old target does not rearm it. The existing transaction component still checks
mapping/visibility identity, motion/geometry/intent and live operation policy.
The small private `Press::targets` helper exposes only an identity comparison.

The router performs fallback itself so callers cannot mistake a cancelled or
failed native release for ordinary client input. No result or error may be
forwarded or retried automatically. Existing client-grab refusal is retained.
Owned motion deliberately does not alter ordinary seat position or focus; backend
cursor/presentation integration remains separate. A host must supply every motion,
cancel removed UI immediately, use leave/reset hooks and route the matching release
after input loss. ADRs 0002/0010 and the accepted release scope remain unchanged.

## Acceptance and executed verification

Four new private-display real-client tests in `tests/window_controls/routing.rs`
cover exactly-once close and duplicate ownership, owned motion suppression,
ordinary keyboard/secondary-button input, primary misses, target replacement and
removal during motion/press/release, disabled-to-enabled state, out-and-back and
non-finite excursions, live release failure consumption, missing pointer capability,
client-held refusal and invalid button refusal. No kernel-global mutation or outer
machine lock is used; these fixtures own private displays and client resources.

Ubuntu prerequisites verified: Rust 1.98.0; `pkg-config --modversion wayland-client
egl glesv2 xkbcommon libudev libinput gbm libseat` returned 1.24.0, 1.5, 3.2, 1.13.1,
259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2. The WSLg socket at
`/mnt/wslg/runtime-dir/wayland-0` exists. No installation or shared maintenance.

Every build/test/lint/format invocation had a PowerShell `(Get-PSDrive C).Free`
preflight refusing below `12GB`. Lowest build/check preflight: 72,154,546,176 bytes.
Initial read-only inventory observed 71,570,845,696 bytes. No cleanup; the reserve
is operational headroom, not a continuous disk quota.

Linux, from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls_routing
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
```

All passed. Focused run: four new lifecycle tests. Full run: 150 units, 192 lifecycle
tests, three socket tests and three compile-fail doctests; none ignored. Existing
malformed-client and keymap diagnostics are refusal fixtures. No build/test failure,
weakened assertion, lint exemption or test retry. Read-only searches initially used
incorrect root queue/journal paths and Windows wildcard paths; corrected before
selection. An incorrect new field spelling was fixed before compilation completed.

Windows, from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed; formatting ran twice as edits progressed. Windows executes zero
Linux-only shell tests. Affected checks are not the independent workspace gates.

WSLg, with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
```

Both passed. `examples/support/window_minimize_check.rs` now drives cancellation
and minimize through the combined router. Its complete 6,400-pixel visible
cancellation, hidden minimize and restored client-frame assertions remain intact.
All 28 offscreen stages and twelve live-snapshot frames pass. The unchanged painter
regression passes 128 complete 5,760-pixel frames. Mesa fallback and deliberate
malformed SHM diagnostics skipped no assertions. No on-screen pointer claim; the
nested example was rebuilt but its on-screen mode was not rerun.

## Integration and remaining evidence

Started clean. Reviewed constitution, delivery/ownership rules, report guidance,
current queue, journal tail and relevant feature/roadmap/ADR/contract sections.
Every published task report was already referenced in STATE; none needed new
reconciliation. Claude's workstream and checkout remain untouched. Reports arriving
during publication reconcile next iteration; release remains unverified.

CHANGELOG, ROADMAP, QUEUE and STATE record the routing component without promoting
a feature checkbox. Next: native hover/pressed feedback with disabled/cancelled
states and full-frame tests, then externalized native labels and production
nested/direct composition. This API does not automatically install controls in
backends; usable controls remain unfinished. On-screen interaction, direct
input/scanout and populated session recovery are not proved by WSLg offscreen
checks. Integrated VM boot/update recovery and certified laptop/GPU workstation
records remain owed at their scheduled phases; no hardware certification claim.

Full independent Windows/Linux workspace tests, lint, rustdoc and pinned BPF gates
belong to the supervisor and remain pending. Tracked diff and new source/test/report
files reviewed; whitespace checked. No staging, commit, push, dev-loop edit, worker
or loop launch, credential/identity access, unrelated checkout changes or shared
service/session/mount/package maintenance.
