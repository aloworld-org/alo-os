# Native control pointer feedback

Date: 2026-09-09. Workstream: native desktop window management.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor publication gates pending.

## Change and decisions

Completed the hover/pressed feedback component selected in QUEUE before
implementation. `crates/alo-shell/src/window_control_feedback.rs` adds read-only
`Server::window_control_feedback`: an explicit live root, current geometry and
optional pointer position produce the existing snapshot with per-control idle,
hovered or pressed state. The existing hit test, availability planner, input-busy
predicate and private press identity/geometry/intent checks are shared rather
than maintained as a second policy. No focus/stacking fallback is introduced.

Cancelled and disabled-at-press gestures retain release ownership but cannot
look armed, even if the pointer returns or availability improves. Another root,
hidden/remapped lifetime, changed geometry and unavailable operation cannot
inherit pressed presentation. Client buttons and popup/move/resize ownership
suppress feedback. Reads send no wire events and cannot cancel, rearm or execute
a transaction. Hosts must route events first and use leave/reset/removal hooks;
reading an outside position is not observing motion. Stored snapshots stay frozen.

`window_controls.rs` supplies immutable feedback presentation and a synthetic
`with_pointer_feedback` builder for isolated views. Disabled controls stay idle;
every application clears previous feedback. Bounds, action labels, default idle
painting and existing API behavior remain unchanged. `window_control_paint.rs`
uses existing appearance tokens: hover adds a contrasting single border over the
alternate ground; pressed inverts ordinary ground/ink with a double-width border.
The inset borders distinguish states without hue alone. Disabled strikes and
transparent gaps remain intact. ADRs 0002/0010 are followed: native Rust, no
terracotta or new palette, no new hardcoded user-facing strings. This is a routine
presentation choice within accepted scope, requiring no new ADR or owner decision.

Public rustdoc and `docs/contracts/native-window-controls.md` describe authority,
refresh and host routing boundaries. No new agent verb, protocol, stored format,
dependency or user-facing vocabulary is added. Native externalized labels and
production nested/direct composition remain subsequent components; this task
completes feedback only, not usable controls or window management.

## Acceptance and executed checks

Two new unit tests cover clipped/half-open/invalid hits, clearing earlier feedback,
unchanged bounds/labels, disabled refusal, non-color border distinctions and the
absence of terracotta. Three new private-display real-client tests in
`tests/window_controls/feedback.rs` cover hover/press/release, frozen snapshots,
read-only outside queries, ordinary typing/pointer isolation, client-held input,
disabled-to-enabled refusal, live output retirement, out-and-back and duplicate
cancellation, seat reset, target/geometry mismatch, hide/reveal, unmap/remap and
foreign-display refusal. Fixtures use private resources, not kernel-global state;
no outer machine lock is added.

Prerequisites verified through Ubuntu WSL: Rust 1.98.0 and
`pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm
libseat` returned 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3,
0.9.2. The WSLg socket `/mnt/wslg/runtime-dir/wayland-0` exists. No package,
mount, session or shared-service maintenance was performed.

Every build/test/lint/format invocation had a PowerShell `(Get-PSDrive C).Free`
preflight refusing below `12GB`. Minimum reading: 69,308,678,144 bytes. No cleanup
was performed. This reserve is headroom, not a continuous allocation quota.

Linux commands from `/mnt/c/dev/alo-os`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked window_controls_feedback
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --examples --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
```

Focused tests passed all three new lifecycle cases on their first run. Initial
all-target clippy failed because extending the shared graphics fixture left the
old baseline `expected` helper unused in `nested_check`. Restored its actual use
for baseline frames; the second clippy run passed. No lint exemption or weakened
test was introduced. Full tests passed 152 unit, 195 lifecycle, three socket
tests and three compile-fail doctests, none ignored. Examples, warnings-denied
rustdoc and Linux formatting passed. Malformed-client/keymap diagnostics were
existing refusal fixtures, not skipped failures.

Windows commands from `C:\dev\alo-os`:

```powershell
cargo fmt --all
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
```

All passed; `cargo fmt --all` ran twice as edits progressed. Windows executes
zero Linux-only shell tests. Affected checks are not full workspace gates.

WSLg commands with `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/window_controls_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Both passed without timeout. The standalone painter checks 896 complete
5,760-pixel GLES frames against independent literal masks/palette values: the
128 idle regressions plus hover/press for every slot, both schemes, all eight
availability masks, maximize/restore and four clipping origins. Every paint/hit
boundary remains checked. The nested fixture passes all 28 offscreen stages,
retains twelve existing live snapshot frames and adds eight full light/dark
frames from routed hovered/pressed/explicitly cancelled/out-and-back-cancelled
state. Complete visible, minimized and restored 6,400-pixel scene assertions
remain intact. Mesa fallback diagnostics and deliberate malformed-SHM refusal
skipped no assertions. On-screen mode was not rerun; no physical input or direct
DRM/display submission claim follows from offscreen GLES.

## Contributor reconciliation and remaining evidence

Started from clean `4323118`. Read constitution, delivery/shared ownership rules,
report guidance, current queue, journal tail and relevant feature/roadmap/ADR/
contract sections. The only published report not already referenced in STATE was
`docs/autonomy/updates/one-keyring-fixture-two-crates.md`. Reviewed its dev-only
fixture manifest, alo-secrets dependency/image guards and kernel-loop rename
accounting locations. It reports eight real-keyring and nine unit tests passing
after the move, but supplies no exact commands. It adds two guards and three
rename-accounting tests, recording initial guard-test mistakes. These are
contributor evidence, not independently rerun credential/tooling checks here.
Production credential architecture is unchanged; authenticated daemon HTTPS is
still Claude's next task. No provider, store or release checkbox is promoted.
Reports arriving during publication reconcile next iteration.

CHANGELOG, ROADMAP, QUEUE and STATE are updated in this change. Full independent
Windows/Linux workspace tests/lint/rustdoc and pinned BPF publication gates remain
the supervisor's work. Next: externalized native control labels with clipping,
fallback and rendered text checks, then production nested/direct composition.
On-screen control interaction, real direct input/scanout and populated session
recovery are unproved by these fixtures. Integrated VM boot/update recovery and
certified laptop/GPU workstation records remain owed at their scheduled phases;
no hardware certification or release verification is claimed.

Tracked diff and new source/test/report files reviewed; `git diff --check` clean.
No staging, commit, push, dev-loop edit, worker/loop launch, credential/identity
access, other-checkout changes or shared-system maintenance.
