# Adaptive native control labels

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: **blocked at graphical verification; unfinished changes preserved**.
No staging, commit or push. This report does not authorize publication.

## Change and decisions

`WindowControlLabels::prepare_expanded` keeps a fitting requested box and otherwise
tries available output-contained space below then above the control row. The
four-pixel gap, existing 2048x512 allocation cap, text scale, vocabulary, font,
full Said and source/translation provenance are retained. At most three rasters
are prepared. Malformed geometry, missing vocabulary/glyphs and foreign selection
refuse; no candidate can publish clipped words or obscure controls. Selection
matches action, bounds and availability, independent of transient hover feedback.
This follows ADRs 0002/0010 without a new palette, agent surface or policy.

The labeled Server/Nested transaction now uses this helper. Successful submission
publishes the expanded rectangle for pointer exclusion. Exhausted space or failed
submission retires authority while preserving callbacks and releases. Normal
client typing remains routed. Low-level preparation and scene validation retain
strict clipping behavior. New unit tests cover both placement sides, disabled
names, source fallback, translations, 100/200/300% scales with exact raster equality,
fitting-box preservation, 4000-byte refusal, tiny outputs and foreign/invalid input.
A new real-client test covers expanded-area exclusion, typing, refusal/callback
retirement, failed submission, recovery and release drainage. Existing transaction
clipping assertions now expect expansion for a short name; exhausted-space refusal
has explicit long-name coverage. No timeout or production limit was changed.

Sources: `crates/alo-shell/src/window_control_label_expansion.rs`,
`window_control_frame.rs`, module registration in `lib.rs`, label unit/client tests,
and nested/offscreen graphical fixtures. Contract:
`docs/contracts/native-window-controls.md`.

Automatic expansion is implemented, but not yet graphically verified. Full-text
access when neither box fits remains a subsequent bounded paged-reader component;
then native navigation/cursor selection and direct integration. This is not a
completed usable-control/window-management feature or release acceptance.

## Verification actually executed

Started clean at `677159e`. Read constitution, delivery, ownership, reports, current
queue and STATE tail, relevant feature/roadmap, ADR 0002/0010 and native-control
contract sections. Selected this component and acceptance in QUEUE before code.
Reconciled the published report named below. No other contributor report modified.

Ubuntu Rust 1.98.0; pkg-config confirmed wayland-client 1.24.0, EGL 1.5, GLES 3.2,
xkbcommon 1.13.1, libudev 259, libinput 1.31.1, GBM 26.0.8-1ubuntu0.3 and libseat
0.9.2. WSLg socket existed at prerequisite inspection. Every build/test/lint
command had a fresh Windows C: preflight refusing below 12 GiB. Minimum observed
47,775,219,712 bytes; final diagnostic reading 57,751,719,936 bytes. No cause for
free-space changes was established. Separate desktop target retained; no cleanup,
package/mount/service/session changes, kernel mutation, outer fixture lock, WSL
restart or keep-alive helper.

Linux commands used `wsl -d Ubuntu --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked expanded_labels -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked label_frame -- --nocapture
cargo test --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --locked --quiet
cargo clippy --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --all-targets --locked -- -D warnings
cargo build --manifest-path /mnt/c/dev/alo-os/Cargo.toml -p alo-shell --examples --locked
```

- Two focused expansion unit tests passed.
- First focused client run: five passed, new expanded transaction test hit the
  existing fixture callback timeout while using a 4000-byte name and 9px preferred
  box. The near-limit refusal remains tested in the shaper suite; the
  transaction fixture now uses 500 bytes, still explicitly unable to fit. No
  fixture deadline or refusal assertion was loosened. Next focused run: six pass.
  The workload change resolved that run; no host scheduling cause was established.
- Full Linux shell suite passed: 159 unit, 216 lifecycle, three socket and three
  compile-fail doctests, none ignored. Expected malformed-client/keymap diagnostics
  are refusal paths. This run includes the final production implementation.
- Initial clippy rejected one needless borrow in the nested example. Removed it
  without exemptions; final affected Linux clippy passed. All examples built.
- Windows `cargo fmt --all`, `cargo clippy -p alo-shell --all-targets --locked --
  -D warnings` and `cargo test -p alo-shell --locked` passed. Shell tests are
  Linux-only, so Windows executed zero test cases.

Graphical command additionally supplied
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

**FAILED: exit 124**, after EGL/Mesa driver/device initialization warnings
(including ZINK unable to choose a physical device and dri2 screen creation
failure), before any control-transaction success output. The Windows tool call
also experienced a long wall-clock delay (about 4794 seconds); its cause is not
established. This is not evidence of label submission or a diagnosis of a code
regression. No bypass, timeout increase, WSL restart or repeated graphical attempt.
The second timeout in this iteration triggers the owner's repeated-failure halt.

The following are **pending, not executed successfully**: `nested_check --controls`,
`nested_check --offscreen` (not reached because the command chain stopped), final
Linux fmt check, warnings-denied affected rustdoc and all independent supervisor
Windows/Linux/rustdoc/BPF gates. The offscreen fixture adds two complete 57,600-pixel
expanded-label comparisons and the nested fixture adds two expanded submissions;
compiled checks are not runtime evidence. No new graphical integration result,
parent-event delivery, direct scanout, VM boot or physical certification is claimed.

## Published contributor report reconciliation

Reviewed `docs/autonomy/updates/recording-a-question-nobody-was-asked.md` and relevant
`alo-agentd::doing`, `alo-turn`, `alo-record` code and record-file contract. The
published work adds NeverPutAnywhere, records pre-turn policy refusals, carries
one wording into response/record and fails closed if recording fails. It includes
persisted response/record agreement and no-connection/no-fallback write-failure
tests. Its mutation checks are reported without exact commands/results, so this
iteration claims no executed contributor verification. Production still supplies
no configured organisation bound. Older readers report the new tag as an unreadable
line; shortening such a record refuses. Shared progress entries correct the old
unrecorded-refusal limitation without completing settings or release scope.

## Handoff

Preserve this checkout. Owner/supervisor must review the graphical timeout and
provide a working WSLg/EGL verification environment or authorize resumed diagnosis.
Any shared environment repair requires the coordinated idle maintenance handoff;
no repair is authorized by a timeout. Complete the pending component checks before
STEP DONE or publication. Reports arriving during publication reconcile next
iteration. CHANGELOG, ROADMAP, QUEUE and STATE record this blocked status and the
reconciled contributor evidence. No dev-loop edit, extra worker, other-checkout
change, private credential/identity access or unrelated host changes.

Final tracked/new-file diff review and git diff --check passed. Work remains unstaged.
