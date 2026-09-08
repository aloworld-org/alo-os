# Trusted window tiling and restoration

Date: 2026-09-08. Workstream: native desktop. Responsible contributor: single
Codex desktop development worker in `C:\dev\alo-os` (integration owner).
Status: **verified, ready for publication after owner-authorized recovery**.
The original halt below is retained as history; the final recovery evidence
supersedes its pending checks and authorization requirement.

## Change and decisions

Added `Server::set_window_tiled` and a shared normal/maximized/tiled transaction
owner in `crates/alo-shell/src/window_mode.rs`. Initial normal geometry survives
side switches and maximize transitions. Fresh configure boundaries gate actual
committed-size placement. All four tiled states describe edges abutting either
the output or split. Restore clears layout flags while retaining unrelated
activation. Exact half-output hints are checked at request and committed response;
invalid anchors suspend without discarding normal geometry. Valid replacement
outputs reconfigure; failed output submission/retirement cannot replace authority.
Hidden mappings retain memory; unmap/disconnect forget it. Existing operation
exclusion is shared by placement, sizing and pointer move/resize.

These are routine choices within accepted ADR 0002 and v0.01 independent tiling,
not linked-neighbour resizing or remembered arrangements. No engine patch,
agent verb or application-adapter authority was added. The two native-window
contracts describe the shared transactions and new `WindowModeError`. The initial
attempt proposed aliasing `WindowMaximizeError`; recovery rejected that source
break and preserved its original enum, as detailed below. The new `Tile` error
variant belongs only to the new tile entry point; existing `Maximized`
placement/sizing refusals now cover tiled memory too.

Real-client cases are in `tests/window_tiling/transactions.rs` and its
`transactions/lifetime.rs` module. Exact tiled-state wire capture retains
all duplicates. A conforming half-buffer helper and six full-frame graphical
stages were added to `examples/support/window_tile_check.rs` and the offscreen
fixture. Those graphical stages have **not** been built or executed.

## Executed checks and halt

The initial tree was clean. Read CLAUDE, DELIVERY, SHARED_MAIN, reports README,
current QUEUE and STATE tail, feature/roadmap sections, ADRs 0002/0010 and the
native-window/application-adapter contracts. No AGENTS.md found by `rg --files`.
All published report filenames were already referenced in STATE at iteration
start; no reconciliation was needed. Reports arriving during publication belong
to the next iteration. Supervisor owns pulling and publishing; worker did not
stage, commit, push or edit another checkout. Claude assignments were untouched.
QUEUE recorded this selected component and acceptance before implementation.

Ubuntu prerequisites: Rust 1.98.0, WSLg Wayland socket present; pkg-config found
wayland-client/server 1.24.0, EGL 1.5, GBM 26.0.8-1ubuntu0.3, libudev 259,
libinput 1.31.1, libseat 0.9.2 and xkbcommon 1.13.1. The first prerequisite
shell command incorrectly expanded inherited PATH with spaces, producing export
warnings; subsequent cargo commands used the fixed explicit PATH below.
No dependency installation or host/service/kernel changes were needed.

All Linux cargo commands used `/mnt/c/dev/alo-os`,
`PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
and `CARGO_TARGET_DIR=/root/alo-os-target`.

Executed commands, in order:

1. Windows `cargo fmt --all`: passed on the then-current tree.
2. Linux `cargo test -p alo-shell --locked window_tiling_transaction`: failed
   compilation because a new `assert_eq!` compared `WindowPlacementError`, which
   does not implement PartialEq. Corrected to an exact `matches!` variant check.
3. Same focused command: **7 passed**, zero failed.
4. Windows `cargo fmt --all`: passed after graphical/test additions.
5. Same focused Linux command: **9 passed**, zero failed (144 unit and three
   socket cases filtered; 153 other lifecycle cases filtered).
6. Windows `cargo fmt --all`: passed after internal naming/contract edits.
7. Linux `cargo clippy -p alo-shell --all-targets --locked -- -D warnings`:
   failed on six missing private documentation items in the shared mode code.
   Added documentation. Review also replaced newly introduced `expect` calls,
   without changing lint configuration or assertion requirements.
8. Same affected-target clippy command: failed because the replacement test
   helper at `tests/window_tiling/transactions.rs:41` uses `panic!`, prohibited by
   the workspace's `clippy::panic` rule. **Halted after repeated verification
   failures. No further build/test/lint commands were run.**

The nine passing cases precede the final internal field/method renaming, error
wording and test-helper edit, so they are not evidence for the final tree.
No full regression, final fmt check, successful clippy, rustdoc, example build,
WSLg offscreen/nested run, Windows affected tests or independent supervisor
workspace/Linux/BPF gates are claimed. A read-only whitespace check is recorded
in STATE; it is not a verification gate substitute.

Checked Windows C: before every cargo command; all readings exceeded 12 GiB.
Initial reading: 15,552,393,216 bytes (about 14.48 GiB). Last build/lint preflight:
15,541,661,696 bytes (14.4743 GiB). Final status reading: 15,534,075,904 bytes
(14.4672 GiB). WSL virtual free space was not treated as extra capacity. No
cleanup, second worker/loop, tools/dev-loop edit, private credential access,
Git identity change or unrelated process termination occurred.

## Remaining work and resumption requirement

Owner authorization to resume after the repeated failures is required. On
resumption, change the exact tiled-flags helper to propagate a missing-event
error (for example, return Result and use `ok_or(...)?`) while preserving exact
flags and duplicate detection. Do not relax the panic/expect/unwrap lints.
Then format final Rust, pass affected all-target clippy and all nine focused
cases plus the full shell regression suite, run rustdoc and build examples.
Run the new six 6,400-pixel tile boundaries and existing WSLg offscreen/nested
regressions, inspect the final source/tests/contracts diff, and record actual
results before requesting supervisor publication. Check C: before each command.
No retry loop is authorized by this report.

Also review the shared error alias and output/commit lifetime integration on the
final tree. The graphical fixture must demonstrate both exact half buffers,
request/ack preservation and normal restoration; code alone does not provide
that evidence. Neither tiling nor window management is completed by this halt.
Native controls remain the next component after these transactions are verified.

WSL/socket tests cannot certify direct DRM/seat entry, populated input/hotplug,
GPU/recovery or a certified laptop/workstation. Those records remain owed in their
delivery phases; physical acceptance follows phase 7 VM image integration.
Configurable operation dispatch follows underlying operations in phase 3.
Release exit remains unchecked.

Proposed shared progress: record this implementation as in progress/blocked,
retain the earlier completed geometry evidence, and name the helper/lint failure
and all unrun gates. All four progress documents were updated with that status.

## Owner-authorized recovery, 2026-09-08

Confirmed the halted supervisor had exited and no competing build was running.
Preserved all unfinished work on top of `08a1b62`. The exact tiled-state helper
now returns a missing-configure error through `Result`; every caller propagates
it. Sorted exact comparisons still retain duplicates. A regression asserts that
no configure event fails for both tiled and normal expectations. No lint changed.

Review rejected the proposed maximize error alias: adding a variant to an
exhaustively matchable published enum is a source break even if that method never
returns it. Restored the original `WindowMaximizeError` enum, its four variants,
conversion from resize geometry errors and its error text. Shared transactions
remain internal; the new tiling entry point alone exposes `WindowModeError`.
A compile-time contract regression imports and exhaustively matches all four
original variants without a wildcard. No versioning or ADR change is needed
because the existing contract is preserved.

Affected Linux all-target clippy passed with warnings denied after the helper
and compatibility corrections. Final Linux focused command
`cargo test -p alo-shell --locked --test client_lifecycle window_` passed all
69 selected tests, including the nine tile transaction cases, missing-event
regression, exhaustive maximize contract and existing window-operation tests.

Built `nested_check` and executed the offscreen and popup/cursor checks. All
28 offscreen stages passed; tile stages 23 through 28 each compare all 6,400
pixels, proving request/ack preservation, both exact half buffers and original
normal restoration. The nested regression submitted 115 client surfaces and
passed. Full publication gates subsequently passed, as recorded below.
Recovery logs remain local under `.git/tiling-resume-gates.log`.

## Final recovery gates

All commands returned exit 0 on the final code. Windows C: was checked before
every phase, stayed above the 12 GiB reserve, and had about 14.16 GiB free at
completion. No cache cleanup or other checkout build was performed.

Windows, from `C:\dev\alo-os`:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
```

Linux, from `/mnt/c/dev/alo-os`, using `/root/alo-os-target`, Rust from
`/root/.cargo/bin`, LLVM 22 on PATH and `LLVM_PREFIX=/usr/lib/llvm-22`:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
```

The BPF filesystem precondition was checked before the real kernel tests.
The shell contributed 144 unit tests, 164 lifecycle cases, three socket tests
and three doctests, all passing. Existing ignored hardware/platform checks
elsewhere remain excluded evidence, not newly executed acceptance. Windows
does not execute the Linux-only compositor runtime cases.

From `crates/alo-bounding-kernel`, on its pinned toolchain:

```text
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
```

Graphics build: `cargo build -p alo-shell --example nested_check --locked`.
With `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and `WAYLAND_DISPLAY=wayland-0`:

```text
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Both returned exit 0. Every offscreen stage and existing graphical regression
ran, including six tile/restore full-frame boundaries. `git diff --check` passed.
This recovery was verified interactively, not by attributing checks to the
halted supervisor. No supervisor source, lint configuration, gate or authority
boundary was weakened. All four progress documents now identify native controls
as the next integration work; this is not full window-management acceptance.
