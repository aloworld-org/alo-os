# Nested reader event routing

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `e04e215`. Read CLAUDE, DELIVERY, SHARED_MAIN, updates guidance,
current QUEUE/STATE, relevant v0.01 features/roadmap, ADRs 0002/0010 and the native
window-controls contract. No AGENTS.md found. All published task reports already
had STATE references at iteration start; no consolidation was due. Recorded the
selected component and acceptance in QUEUE before implementation, then moved
that selection beside its completed evidence in the current queue section.
No filesystem-security task or other contributor checkout was touched.

The nested name reader now receives ordered parent keyboard and pointer input.
PageUp/PageDown navigate and Escape dismisses without closing the application;
ordinary typing remains available. Input loss, removal and malformed events
cancel commands while preserving ownership of their matching releases. This is
the complete nested event attachment component, not complete full-name access.

- `src/nested_reader_input.rs`: the reader adapter on `NestedControlInput` routes
  before controls/clients, maps reader-local evdev navigation keys, validates
  pointer events, uses actual parent position for buttons/axes, and forwards only
  Forward events once. Motion consumed by the reader still updates that position.
  Scroll revalidates current geometry; bad coordinates/buttons/axes refuse even
  over native content. These reader keys are not configurable desktop shortcuts.
- `src/nested.rs`: `pump_reader_seat` accepts an explicitly selected live reader.
  Every full-seat pump uses the same coordinator, including None after removal.
  Activation precedes input, normal keyboard focus stays intact, and inactive
  ordinary keys are dropped. Errors no longer replace the ownership object;
  cancellation retains releases. Keyboard-only loss also clears cached position.
  Full-seat pumping is required to drain pointer releases; render-only pumping
  deliberately remains input-free. No reader opens automatically.
- `src/nested_reader_frame.rs`: `NestedReaderFrame` and `Nested::render_reader`
  connect the existing atomic frame transaction to this backend's own feedback
  owner. Input is moved aside only during synchronous submission and restored on
  both success/refusal. There is no event dispatch during that borrow.
- `tests/window_controls/nested_reader.rs`: four real-client tests exercise key
  navigation/repeats/boundaries/dismissal/typing, client-owned key refusal, actual
  pointer position and scroll interception, removal draining, deactivation,
  malformed events and key-error cancellation with a held release.
- `examples/support/nested_reader_frame_check.rs`: retains all original twelve
  coordinated page/feedback submissions and refusal assertions; adds two actual
  backend-owned reader submissions (light/dark) and reader-seat pump calls.

No upstream engine patch, new palette, production wording or agent capability.
ADRs 0002/0010 are preserved. Reader routing and frame preparation have separate
source files. Public rustdoc and the native-window-controls contract document
ownership, key mappings and evidence limits. All four progress documents updated.

## Verification and failures

Environment: Ubuntu WSL2, `/root/alo-os-target` for this checkout. WSLg socket
`/mnt/wslg/runtime-dir/wayland-0` and existing `/sys/fs/bpf` mount verified.
`pkg-config --modversion wayland-client egl gbm xkbcommon libinput libudev`
returned 1.24.0, 1.5, 26.0.8-1ubuntu0.3, 1.13.1, 1.31.1 and 259. No prerequisite
installation or shared maintenance was needed. Fixtures use private resources;
no kernel mutation or outer machine lock was added.

Each format/build/test/lint/doc/graphical command had a Windows preflight:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

All readings exceeded 53 billion bytes, above the required 12 GiB reserve.
This is a preflight, not a continuous quota. No cleanup was performed.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked nested_reader -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --example nested_check --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Focused tests first failed to compile: two assertions used `app.events.keys`
instead of the existing `app.events.keyboard.keys` field. Corrected those test
paths; the three then-present tests passed, followed by all four after adding
the client-ownership/backend-error test. No product behavior or assertion was
weakened. Logs: `focused.txt`, `focused-fixed.txt`, `focused-final.txt`.

Affected clippy and full tests passed, then passed again after the final
keyboard-only loss-path adjustment. Example build passed; full cargo test also
rebuilt examples after that adjustment. Warnings-denied rustdoc passed, including
a final run after public documentation review. Final Linux result: 168 unit,
246 lifecycle, three socket and four compile-fail doctests; none ignored.

Graphical commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both passed before and after the final loss-path adjustment. The reader run
preserves twelve explicit-host page/feedback EGL submissions, both schemes,
navigation/dismissal, removal and geometry refusal/recovery, and adds two
backend-owned reader submissions plus real parent event-pump calls. Controls
regression and six-client lifecycle checks pass. No graphical failure this
iteration. Existing ten-second client and 30-second command deadlines unchanged.
Expected malformed-client and Mesa diagnostics remain visible. Earlier upstream
submission stalls from `publication-bound-reader-input.md` are not explained or
claimed fixed by these passes.

Windows affected commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

All pass. Windows shell tests contain zero cases because the crate is Linux-only.
Logs are under `.git/alo-loop/nested-reader-event-routing/`; `*-final.txt` records
final Linux/graphical checks. Native exit codes establish results; PowerShell's
NativeCommandError wrapper also appears for successful native stderr output.

## Evidence limits and next work

The private-client tests drive the exact adapter installed in the Winit pump.
Graphical checks submit actual EGL reader frames and call the real parent pump,
but do not synthesize parent keyboard/button events, verify physical input or
read back those submitted frames. Existing offscreen painter evidence remains
separate. Native reader selection from full-name access, native cursor selection
and direct backend integration remain next. Full-name access/window management
are unfinished; no feature/release checkbox is promoted. Integrated VM boot and
physical laptop/GPU workstation records remain owed in their delivery phases.

Supervisor independent workspace Windows/Linux/lint/rustdoc/BPF gates and
publication remain pending. Tracked/new source, tests, example and docs reviewed.
No stage/commit/push, dev-loop modification, second worker/loop, other-checkout
change, credential/identity access, WSL helper/restart, cleanup or shared package/
service/mount/session changes. Published reports arriving during publication
are reconciled next iteration; release verification is not claimed.
