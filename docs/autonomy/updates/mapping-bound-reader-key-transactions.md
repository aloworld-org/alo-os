# Mapping-bound reader key transactions

Date: 2026-09-09. Workstream: native desktop compositor.
Responsible contributor: desktop integration worker in `C:\dev\alo-os`.
Status: ready for independent supervisor gates. Explicit keyboard transaction
component complete; rendered reader input integration and window management unfinished.

## Change and decisions

`window_control_reader_keys.rs` adds an explicit trusted host transaction for
previous/next/dismiss commands. A press captures reading-session identity, page
visit and semantic command; a matching release validates the live server binding
and executes once. Repeats cannot rearm or execute. Concurrent command presses
are consumed but disarmed. Boundary navigation consumes without wrapping;
dismissal cannot close an application. Cancel retains each owned release.

`window_control_reader.rs` now distinguishes readers opened against the same
strip and renews a page-visit identity only on actual page changes. This prevents
replacement and away-and-back activation without counter overflow. Same-page
render validation preserves the visit. `keyboard.rs` provides a private existing
XKB held-key query: client-owned keys and absent seats refuse acquisition. Valid
evdev codes 1..=767 bound retained ownership; unrecognized keys/releases forward.
No client focus, XKB state, window operation, vocabulary or configured shortcut
is changed. Consistent with ADR 0002; ADR 0010 palette remains unchanged.

This is deliberately an explicit host component: opening a reader does not
activate it. Hosts must offer semantic presses only after reader publication,
cancel on frame failure/removal, focus/seat/session or binding change, and drain
releases even when inactive. Backend activation, pointer ownership, interaction
feedback and transactional reader rendering/publication are subsequent integration.
The additive contract is in `docs/contracts/native-window-controls.md`.

## Checks actually executed

Started clean at b4cb63b. Read required constitution, delivery, ownership and
report guidance, current queue/STATE, relevant feature/roadmap, ADR 0002/0010 and
native-control/translation contracts. No AGENTS.md found. Every published report
already had a STATE reference; none required reconciliation. Selected the
component and acceptance in QUEUE before implementation.

Ubuntu prerequisites checked with:

```sh
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
stat -f -c %T /sys/fs/bpf
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg socket present, existing bpffs is bpf_fs. No maintenance needed.
Each build/test/lint/format/doc command had an immediate Windows C: reserve
preflight using `(Get-PSDrive C).Free`, refusing below `12GB`. All exceeded the
reserve; minimum observed command preflight: 54,526,406,656 bytes.

Linux commands ran through `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env`
with `PATH=/root/.cargo/bin:/usr/bin:/bin` and
`CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell --locked native_reader_keys -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo doc -p alo-shell --no-deps --locked
```

Focused test passed the first three new cases before the client-key refusal test
was added. Initial clippy failed on five missing private-item documentation
errors in the new module. Added the required documentation, then reran the exact
clippy command successfully. No exemptions or gate changes. Final full Linux
suite after all source changes: 165 unit, 228 private-client lifecycle, three
socket and four compile-fail doctests pass, none ignored. Final affected clippy
passes; rustdoc passes with `RUSTDOCFLAGS=-Dwarnings`.

Four new private-client integration tests verify first/last boundaries,
forward/reverse navigation, repeat suppression, press-time command mapping,
concurrent keys, dismissal, cancellation/repeat/release drainage, missing reader,
strip retirement, clear_input, page away-and-back, replacement on the same strip,
foreign server, absent keyboard, invalid codes and existing client key ownership.
Normal client typing and matching client-owned releases arrive without keyboard
leave or application close. These tests use private resources, no kernel mutation.

Windows checks pass:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Format ran twice during implementation; final check passes. Windows shell test
cases are zero because this implementation is Linux-only. These affected checks
do not replace independent supervisor Windows/Linux workspace, rustdoc and BPF
gates, which have not run this iteration. No new GLES/on-screen test was run:
this change introduces no paint path. Evidence is trusted semantic transactions
against private Wayland clients, not parent navigation-key delivery, published
reader UI, direct scanout or integrated VM boot. Physical laptop/GPU workstation
acceptance remains owed in release validation after image integration.

Logs: `.git/alo-loop/mapping-bound-reader-key-transactions/`: `focused.txt`,
`linux-clippy.txt` (original failure), `linux-clippy-repaired.txt`, `linux-tests.txt`,
`linux-rustdoc.txt`, `windows-clippy.txt`, `windows-tests.txt`, `fmt.txt`.
PowerShell's NativeCommandError wrapping of native stderr appears even on success;
actual exit codes and test summaries were inspected. Intentional malformed-client
and invalid keyboard-layout diagnostics remain visible in the full Linux suite.

## Progress and remaining work

All four shared progress documents record this component and its limits, with no
release checkbox changes. Next: reader pointer ownership and interaction feedback
connected to transactional composition/submission/publication/retirement, then
backend reader key mapping/activation, native navigation/cursor selection and
direct integration. This report completes only explicit key transactions.
Reports arriving during publication reconcile next iteration.

Source and new-file diffs reviewed; `git diff --check` passes. No staging, commit,
push, dev-loop edits, other-checkout edits, added worker/loop, credential/identity
access, cleanup, shared service/mount/package/session changes, WSL restart/helper
or unrelated host changes. Supervisor owns independent gating and publication.
