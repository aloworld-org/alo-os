# Mapping-bound native name readers

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Reader-state component complete;
the paged reader UI, usable window management and release remain unfinished.

## Change and decisions

`window_control_reader.rs` adds host-owned prepared reading state and checked
zero-based page selection. Opening uses the current published strip, explicit
action, immutable size/scheme/scale and full vocabulary. Disabled names remain
readable. The existing whole-line page preparer still refuses atomically.
The opaque reader grants neither input ownership nor window-operation authority.
Its pages can only be borrowed through a server check, carrying unabridged Said,
whole-line raster and one-based position/total metadata. No English position
string is assembled; externalized reader chrome belongs to the next component.

`window_control_presentation.rs` gives each strip publication lifetime a private
Arc identity, preserved on identical live refresh. Removal/republication cannot
revive it, even with identical geometry and surface. Existing live mapping checks
cover hide/remap, death, maximize/restore intent and geometry/root replacement.
Observed competing input renews the identity; access during competition refuses.
Readers rejected as stale or foreign, or explicitly dismissed, stay closed.
An invalid index preserves selection, but never prevents prior live validation.

The identity is a local lifecycle mechanism consistent with ADR 0002, not a new
agent grant or global lock. ADR 0010 palette restrictions and existing shaping,
fonts, text-scale and allocation limits are unchanged. Reopen on vocabulary/style
change; returned borrowed pages are frozen snapshots to discard across host events,
not proof of backend submission. Multiple prepared handles add no input authority.

Source/export changes: `crates/alo-shell/src/window_control_reader.rs`,
`window_control_presentation.rs`, `lib.rs`; tests in
`crates/alo-shell/tests/window_controls/reader.rs`, registered in `mod.rs`, reuse
the existing `presentation::present` fixture helper. Public Rust contract updated
in `docs/contracts/native-window-controls.md`.

Four new real-client integration tests exercise complete forward/backward page
traversal and reference raster/line/provenance agreement; out-of-range preservation;
disabled names and ordinary keyboard typing; explicit dismissal; removal and
identical republication; root/origin/viewport replacement; failed presentation;
unobserved minimize/restore and protocol unmap/remap; backend input loss; foreign
server refusal; competing press/cancellation; disconnect; invalid geometry and
non-strip action refusal. Fixtures use private Wayland clients/resources, no
kernel-global mutation. Existing suites retain their own machine locks.

## Verification actually executed

Started clean at d3fb280. Read constitution, delivery/ownership/report guidance,
current queue and STATE tail, relevant features/roadmap, ADRs 0002/0010 and native
control contract. Selected this complete reader-state component and acceptance in
QUEUE before code changes. Ubuntu prerequisites verified with:

```sh
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
stat -f -c %T /sys/fs/bpf
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present, existing bpffs reports bpf_fs. No installation or maintenance.
Every build/test/lint/format command checked Windows C: first and exceeded 12 GiB;
lowest observed preflight was 54,677,307,392 bytes. Separate desktop target retained.

Linux commands ran through `wsl -d Ubuntu --exec env` (or the equivalent with
`--cd /mnt/c/dev/alo-os`), `PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`. Manifest path was
`/mnt/c/dev/alo-os/Cargo.toml` when not using the explicit working directory:

```sh
cargo test -p alo-shell --locked native_reader -- --nocapture
cargo test -p alo-shell --locked --quiet
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo build -p alo-shell --examples --locked
cargo doc -p alo-shell --no-deps --locked
cargo fmt --all --check
```

Final results: four focused tests pass; full affected suite has 162 unit tests,
220 real-client lifecycle tests, three socket tests and four compile-fail doctests,
all passing, none ignored. Final Linux clippy/examples/rustdoc/fmt pass. Rustdoc
used `RUSTDOCFLAGS=-Dwarnings` (also set in the final multi-phase command).

Graphical checks additionally used `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and
`WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/control_label_pages_check
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both pass: 72 complete 57,600-pixel GLES page frames in forward/reverse order,
translation/fallback, light/dark and four scales. Nested check: eight EGL strip
submissions, two labels, two expansions, two dismissals, two ordinary removals,
exhausted-space/strip refusal and recovery; six submitted client surfaces with
unmap/remap/disconnect checks. Mesa/EGL device-discovery warnings remain visible;
successful rendering and readback followed. Malformed-client diagnostics are
existing intentional refusal checks. No graphical timeout occurred.

Windows commands passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows executes zero shell cases because they are Linux-only. This is affected
compilation evidence, not a Windows desktop test. Last subsequent code edit fixed
a Linux-only test fixture; Linux fmt/clippy/full tests passed after that edit.
Full independent supervisor Windows/Linux/workspace/rustdoc/BPF gates are pending.

## Development failures and repair

Initial focused tests passed. Clippy then rejected seven test `expect()` calls;
replaced them with propagated Send+Sync test errors, without lint exemptions.
The expanded full suite had 219 lifecycle passes and one new retirement failure.
Inspection found replacement setup called `Fixture::root()`, whose implementation
returns the first mapped root, not necessarily the second client. This did not
establish replacement. Changed setup to select an explicitly unequal mapped root
and assert inequality, matching the existing presentation fixture. Final focused
and full affected acceptance pass. The original log lacks the failing case index;
the fixture defect is established by inspection, not a claim that the original
runtime identity was logged. No production assertion or deadline was weakened.

An early PowerShell queue edit decoded UTF-8 incorrectly. Rebuilt that document
from HEAD bytes plus the intended new ASCII section before further edits; reviewed
diff confirms no historical-text changes. Two unsuccessful context patches made
no source edits. No hidden cleanup or unrelated fixes.

Logs: `.git/alo-loop/mapping-bound-native-name-readers/`: `focused.txt`,
`focused-final.txt`, original `linux-clippy.txt`, final `linux-clippy-final.txt`,
original `linux-tests.txt`, final `linux-tests-final.txt`, `linux-examples.txt`,
`linux-rustdoc.txt`, `linux-fmt.txt`, `page-gles.txt`, `nested-controls.txt`,
`windows-fmt.txt`, `windows-clippy.txt`, `windows-tests.txt`. PowerShell renders
native stderr as NativeCommandError records even on exit zero; exit codes and
terminal results were checked. Original failure logs preserved.

## Contributor reconciliation and limits

Read and consolidated published reports not yet referenced in STATE:
`a-loop-that-chose-an-audit-heading.md` and `an-organisations-rule-off-a-disk.md`.
Checked task-section/scheduled filtering and daemon format/owner/startup handoff
against current code and ADR 0016/machine-description contract. Reports claim six
selection tests and 134 passing workspace test binaries plus policy mutations,
respectively, without exact verification command listings. Retained limits:
startup's `described.questions().clone()` is inspection-only; person-owned on-disk
attribution depends on runner UID; root/third-owner disk cases require privilege;
alo-image still supports only unmanaged format 1. Their historical idle mount
handoff is contributor evidence, not maintenance performed or authorized here.
No release checkbox moved. Source reports were not edited.

Updated CHANGELOG, ROADMAP, QUEUE and STATE, preserving these evidence limits.
Next reader work: externalized page position/navigation chrome, keyboard/pointer
ownership and transactional composition/submission/publication/retirement.
Then native navigation/cursor selection and direct integration. Full-name access
is unfinished. New evidence is live private-client reader-state/raster checking
and existing offscreen/nested GLES acceptance, not parent key delivery, on-screen
reader UI, direct scanout, integrated VM boot or hardware certification. Physical
laptop/GPU workstation records remain owed during release validation.

No staging, commit, push, extra worker, loop/dev-loop change, other-checkout edit,
credential/identity access, cache cleanup, WSL restart/helper, package/mount/
service/session change or unrelated host modification. Final source/new-file diff
review and `git diff --check` pass. Reports arriving during publication belong to
the next iteration; release verification is not claimed.
