# Client window minimize requests

Date: 2026-09-08. Workstream: native desktop. Responsible contributor: desktop
integration worker in `C:\dev\alo-os`. Status: verified, ready for publication
after owner-authorized recovery. The initial halt below is retained as history;
the recovery evidence at the end supersedes its pending checks.

## Change and acceptance

`src/window_minimize.rs` under `crates/alo-shell` shares the trusted visibility
transition through Surfaces; `src/surfaces.rs` routes XDG minimize requests and
advertises Maximize plus Minimize. The requesting role alone is reachable.
Buffered-root validation ignores pre-map intent, without a premature configure
or retained hiding. Duplicate requests are inert. This follows ADR 0002 and the
existing v0.01 window-management scope; no engine patch, agent API, visual strings
or ADR change. XDG has no client unminimize handshake; restoration remains trusted.

Added `tests/window_minimize/client_requests.rs`: pre-map/remap behavior and
capabilities; input retirement, recipient isolation, duplicate requests, hidden
commits/callbacks, restoration and disconnect. Updated maximize capability
assertions. Added real-client stage 22 in the offscreen fixture, checking every
hidden frame byte and all 6,400 restored pixels using production GLES rendering.
That graphical fixture has NOT run. Native minimize/maximize contracts updated.

Acceptance remains focused happy/refusal tests, affected full tests/clippy,
format/rustdoc/examples and WSLg graphical checks. This component is unfinished;
rendered controls and tiling remain subsequent work, no feature checkbox changed.

## Actual verification and blocker

PowerShell C: free-space check before every formatting/test command:
`$free=(Get-PSDrive C).Free; if($free -lt 12GB){throw 'Storage reserve below 12 GiB'}`.
Readings 15.497 GiB or higher. No cleanup, second workstream or dependency install.
Preflight is not a continuous disk quota.

Ubuntu commands use `wsl -d Ubuntu -- bash -lc`, `/mnt/c/dev/alo-os`,
`PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`
and `CARGO_TARGET_DIR=/root/alo-os-target` for tests.

- `rustc --version`: 1.98.0.
- `test -S /mnt/wslg/runtime-dir/wayland-0`: passed.
- `pkg-config --modversion wayland-client wayland-server egl gbm libudev libinput libseat xkbcommon`:
  1.24.0, 1.24.0, 1.5, 26.0.8-1ubuntu0.3, 259, 1.31.1, 0.9.2, 1.13.1.
  Initial inherited PATH probe produced splitting errors; explicit PATH rerun passed.
- Windows `cargo fmt --all`: passed three times as Rust edits were completed.
- Linux `cargo test -p alo-shell --locked client_minimize`: run twice, both exit 1.
  Each run passed the input/isolation/duplicate/hidden-commit/disconnect test.
  The pre-map test first failed with actual [[2,4]] versus expected [[2,3]].
  Protocol enum constants replaced numeric expectations. The second run failed
  with actual [[4,2]] versus expected [[2,4]]: capability iteration is unordered.

The owner explicitly requires repeated test failures to halt. No third run,
assertion weakening, production workaround or further build followed. Resume must
compare the exact advertised capability set independent of order in both minimize
and maximize tests; then execute all outstanding acceptance checks. The remainder
of the pre-map test has not yet executed past that assertion. Full affected tests,
clippy, rustdoc, example build and graphical fixtures were not run this iteration.
Independent supervisor Windows/Linux workspace and BPF gates also remain unrun.

## Reconciliation, limits and next action

Initial tree clean. Every published report already referenced in STATE; none to
reconcile. Publication arrivals reconcile next iteration. Claude's assignments
untouched. All four shared progress documents updated with this blocker, without
completion credit. Owner must authorize resuming the halted iteration; no external
hardware or credentials are required to fix this fixture assertion.

WSLg/protocol tests cannot certify DRM/seat, populated physical input/hotplug,
GPU/recovery or certified laptop/workstation behavior. Physical acceptance remains
phase 8 after integrated-image VM validation; configurable shortcuts remain phase
3 after underlying window operations. Ordinary keyboard tests remain active.
No staging, commit, push, worker/loop launch, other checkout, credential/identity,
company-managed files, host cleanup or tools/dev-loop changes occurred before the
halt. That halted attempt was not ready-for-integration evidence.

## Owner-authorized recovery, 2026-09-08

The owner authorized correcting the assertions, verifying the feature and
restarting the loop. Confirmed the supervisor and worker had exited, no competing
build was active, and main remained at `7e5e456`. Preserved the unfinished change.

`tests/support/wm_capabilities.rs` compares exact sorted capability vectors and
the number of capability events. It preserves duplicates, uses protocol enum
constants, and checks both initial and remapped events. Four helper tests prove
both wire orders pass and missing, duplicate and extra capabilities fail. Raw
captured wire events are not rewritten. Both minimize and maximize tests use it.
No production workaround or relaxed capability advertisement was required.

Review also found the offscreen fixture still required 21 stages after the new
client-minimize stage was added. Its final assertion now requires all 22; no
stage or pixel assertion was removed.

Linux `cargo test -p alo-shell --locked --test client_lifecycle client_` passed
11 selected tests, including both new minimize cases and all four client
maximize cases. All publication and graphical checks subsequently passed.
Local recovery logs live in
`.git/client-minimize-resume-gates.log`, not in the published tree.

## Recovery verification

All commands below returned exit 0. C: was measured before every phase and
remained above 12 GiB (approximately 14.49 GiB at completion). No cleanup occurred.

Windows, from `C:\dev\alo-os`:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
```

Linux used the checkout and target directory above, with LLVM 22 on PATH and
`LLVM_PREFIX=/usr/lib/llvm-22`. The existing BPF filesystem was checked/mounted
before kernel tests. Workspace commands:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
```

The shell contributed 141 unit tests, 149 client lifecycle tests (including the
four new assertion-helper tests), three socket tests and three doctests. All
passed. Existing platform/hardware exclusions elsewhere remain exclusions, not
newly executed acceptance. Windows does not execute Linux compositor cases.

From `crates/alo-bounding-kernel`, on its pinned toolchain:

```text
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
```

Graphical verification built with
`cargo build -p alo-shell --example nested_check --locked`, then used
`XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir` and `WAYLAND_DISPLAY=wayland-0`:

```text
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

All 22 offscreen stages completed. The real client minimized its window twice;
the hidden frame was entirely black, and trusted restoration reproduced all
6,400 expected window/background pixels at the preserved geometry. The existing
maximize, trusted minimize, input, popup and cursor graphics regressions passed.
`git diff --check` passed. No stage, assertion or test was skipped for recovery.

The first local gate launcher treated Cargo's ordinary progress on stderr as a
PowerShell error before checking its exit status. Corrected only that local
launcher to check native exit codes; the complete gate sequence above then ran
successfully. No supervisor source or gate policy changed. This recovery was
verified interactively before publication, not by claiming the halted supervisor
had resumed. Progress documents now identify tiling as the next component.
