# Mapping-bound reader pointer transactions

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Pointer transaction and semantic
feedback component complete; rendered reader interaction remains unfinished.

## Change and decisions

`crates/alo-shell/src/window_control_reader_pointer.rs` adds explicit trusted-host
pointer ownership. Hits name covered content or previous/next/dismiss commands;
only the primary button on an enabled command arms. A release acts once on the
same reader, page visit and target. Leaving the command, page away-and-back,
replacement, cancellation and competing buttons disarm permanently. Repeats cannot
rearm. All eight mouse buttons retain matching release ownership through inactive
reader retirement. Disabled/content/non-primary presses consume without action.
Missing pointer seats and existing client grabs refuse acquisition. Normal client
focus and keyboard state remain unchanged; dismissal never closes an application.

`motion` returns whether to withhold motion/axes; `button` shares the existing
ReaderKeyRoute outcomes. `feedback` revalidates enabled hovered/pressed commands
against the live reading session. Reusing ReaderKeyCommand/ReaderKeyRoute keeps
semantic navigation consistent across input; their existing variants and key
behavior are unchanged. Unique Arc identities reuse the reader's page-visit
protection without counters or wraparound. Primary release and permanent drag-away
cancellation avoid activating a later target. Multiple held buttons disarm all
commands while preserving release drainage.

This is a trusted semantic component, not automatically active UI. The host must
offer hits only after a reader frame publishes, cancel on frame failure/removal,
leave, focus/seat/session loss, geometry changes and competing keyboard interaction,
and route owned releases while inactive. Geometry hit testing, feedback painting,
transactional reader composition/publication/retirement and coordinated backend
activation are subsequent integration. No new agent verb, user-facing string,
palette, font, engine, configured shortcut or ADR decision. This is native Rust
under ADR 0002 and retains ADR 0010's palette. Additive API/rustdoc and contract:
`docs/contracts/native-window-controls.md`.

## Checks actually executed

Started clean at ce5b73d. Read CLAUDE, DELIVERY, SHARED_MAIN, reports README,
current QUEUE/STATE, relevant feature/roadmap, ADR 0002/0010 and native-control/
translation contracts. `rg --files -g AGENTS.md` found none. Compared published
report filenames against STATE: every report already referenced; none needed
reconciliation. Selected this component and acceptance in QUEUE before coding.

Ubuntu prerequisites checked through `wsl -d Ubuntu --cd /mnt/c/dev/alo-os
--exec sh -c`:

```sh
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
stat -f -c %T /sys/fs/bpf
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
Existing bpffs reports bpf_fs. Explicit follow-up `if test -S
/mnt/wslg/runtime-dir/wayland-0; then echo WSLg-socket-present; else exit 1; fi`
also passed. No shared maintenance required.

Every build/test/lint/format/doc command had an immediate Windows C: reserve
preflight: `$free = (Get-PSDrive C).Free; Write-Output "C free bytes: $free";
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }`. All exceeded 12 GiB;
minimum observed command preflight: 53,049,700,352 bytes. These are individual
readings, not a continuous reserve guarantee or additional WSL capacity.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked native_reader_pointer -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Focused run passed the first four new tests. Added the fifth explicit host routing
test and wire assertions, then the full suite passed. Clippy passed initially,
then rejected a direct `axes[0]` assertion in that fifth test. Replaced it with
`first().map(...)` and retained the exact length/value assertions. Clippy rerun
passed; final full suite after repair passed: 165 unit, 233 private-client
lifecycle, three socket and four compile-fail doctests, none ignored. Rustdoc
with warnings denied passed. Intentional malformed-client and invalid keymap
diagnostics remain visible; no failing test was suppressed.

Five new private-client tests cover forward/reverse and boundary navigation,
enabled hover/pressed feedback, dismissal, repeats, competing buttons, leaving to
content/outside/another command, cancellation, clear_input, strip retirement,
page away-and-back, replacement on the same strip, foreign server, missing pointer
seat and invalid button codes. Existing client press/release and normal typing
arrive with no keyboard leave or application close. Additional host integration
evidence uses routing decisions to withhold covered motion, axes and all eight
buttons; cancellation/inactive repeats/releases drain, then the real client sees
exactly one ordinary motion, one scroll value and one press/release pair.

Windows commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Affected Windows clippy/tests passed (zero shell cases because this code is
Linux-only). Format ran during development; final check identified the multiline
wrapping needed after the checked-lookup repair. Ran cargo fmt, then fmt --check
passed. This final change was whitespace only. No assertion, lint exemption,
timeout or gate was weakened. Reviewed tracked and new source/test/document diffs;
`git diff --check` passed. Git no-index new-file diff returns 1 for differences,
not an implementation failure.

Logs: `.git/alo-loop/mapping-bound-reader-pointer-transactions/`: `focused.txt`,
`linux-clippy.txt`, `linux-clippy-final.txt` (indexing failure),
`linux-clippy-repaired.txt`, `linux-tests.txt`, `linux-tests-final.txt`,
`linux-rustdoc.txt`, `windows-clippy.txt`, `windows-tests.txt`, `fmt.txt`,
`fmt-check.txt`, `fmt-final.txt` (wrapping failure), `fmt-repaired.txt`,
`fmt-check-repaired.txt`. PowerShell NativeCommandError wrapping appears for
native stderr even on success; actual command exit codes were inspected.

## Progress and evidence limits

All four shared progress documents updated, with this report referenced in STATE.
No feature or release checkbox promoted. Next: published hit geometry and
interaction feedback painting connected to transactional reader frame submission/
publication/retirement, coordinated backend key/pointer activation, then native
navigation/cursor selection and direct integration. Explicit pointer transactions
are complete; full-name access and usable window management remain unfinished.

Evidence is private-client semantic state and explicit host routing, not parent
pointer/key delivery, rendered feedback or on-screen reader navigation. No new
paint path, GLES or nested graphical run was added or claimed. Scanout, integrated
VM boot and physical laptop/GPU workstation acceptance remain owed in their
delivery phases. Independent supervisor Windows/Linux workspace, rustdoc and BPF
gates have not run this iteration; supervisor owns integration and publication.
Reports arriving during publication reconcile next iteration; no release verdict.

No staging/commit/push, dev-loop edits, additional worker/loop, other-checkout edit,
credential/identity access, cleanup, shared mount/service/package/session change,
WSL restart/helper or unrelated host change. Tests used private resources without
kernel mutation or an outer machine lock; desktop Linux target remained separate.
