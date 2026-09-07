# Truthful output metadata

Date: 2026-09-07. Workstream: native desktop compositor. Responsible contributor:
single desktop/integration worker in `C:\dev\alo-os`.

Status: **ready for integration; recovery gates passed**. On 2026-09-08 the owner explicitly asked
to fix and resume the desktop loop. The original worker's halt and failed checks
are preserved below; the recovery fixes the client handshake without weakening
assertions. Normal offscreen integration now passes twice. Complete publication
gate results are recorded in the recovery section.

## Change and decisions

`crates/alo-shell/src/output_metadata.rs` defines validated output identity,
millimetres and millihertz. FrameTarget defaults to an explicitly virtual output;
Nested supplies its nested identity. DirectTarget uses a frozen connector ID,
kernel connector dimensions retained through DirectOutput discovery, and
progressive timing arithmetic rounded to millihertz. Unknown make/model/subpixel
information stays unknown. Partial/absent dimensions normalize to unknown;
overflow, unsupported timings and malformed wire strings refuse.

Presentation freezes identity on the first successful submission. Replacement
refuses before drawing. Globals/current/preferred modes publish only after
successful submission, preserving metadata, membership and callbacks on refusal.
Server validates metadata before accepting a proposed popup extent. Reactive
popup negotiation retains its existing desired-extent behavior even on failed
submission, independently of the last submitted wl_output mode.

This follows ADR 0002's native single-display design and v0.01 scope. No engine
patch, EDID parser, hotplug or multi-output support was added. ADR 0001 and agent/
application-adapter contracts are unchanged. Public Rust documentation covers
OutputMetadata, FrameTarget::metadata, RenderError and DirectOutput::physical_size;
existing struct literals were updated. Wire strings are backend descriptive data.

Acceptance: validation/refusal tests, real wire metadata and membership/callback
assertions, WSLg integration, affected fmt/clippy/tests/rustdoc and inspection.
The normal offscreen integration criterion was unmet at the original halt and
is now satisfied by the recovery runs below.

## Original worker checks (before the halt)

Ubuntu WSL2, Rust 1.98.0. `/run/user/0/wayland-0` is a socket.
`pkg-config --modversion wayland-client egl glesv2 gbm libseat libinput` returned
1.24.0, 1.5, 3.2, 26.0.8-1ubuntu0.3, 0.9.2, 1.31.1. No installation needed.
`/dev/dri` is absent; no shared kernel/BPF/cgroup/service tests were run.

Windows: `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy -p alo-shell --all-targets --locked -- -D warnings` and
`cargo test -p alo-shell --locked` all passed. Shell tests are cfg-excluded there
(zero executed). These preceded final test error-propagation and offscreen timing
fixture edits; supervisor full gates remain pending.

Linux from `/mnt/c/dev/alo-os`, via `wsl -d Ubuntu -u root -- env`, with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo test -p alo-shell output_metadata --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

Passed: 102 unit + 67 lifecycle + 3 socket + 2 doctests = 174 checks. Three new
tests and expanded direct wire assertions. Coverage includes fractional refresh,
overflow/unsupported timing, malformed strings/dimensions, no initial global on
failure, exact known metadata, failed resize/identity refusal preserving events,
membership and callbacks, and subsequent successful resize. Direct wire fixture
checks connector name, geometry and refresh through seven commit/refusal stages.
After the last offscreen timing-only edit, Linux fmt/all-target clippy/example
build were rerun successfully; full tests/rustdoc preceded that fixture edit.

Initial implementation checks found a missing test module, a non-Clone protocol
event and a misplaced integration module; corrected. First full shell run found
five popup regressions: rollback of desired extent broke existing negotiation,
and deferred global creation needed another bind roundtrip. Restored negotiation
and added the roundtrip in the popup-chain test, retaining all assertions; full
suite passed. Clippy unwrap/slicing findings were corrected with Result
propagation/iterator skip, without exemptions; final clippy passed.

With `XDG_RUNTIME_DIR=/run/user/0 WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Passed nested wire name/unknown dimensions/refresh checks and popup/cursor,
callback, refusal/disconnect regression: 115 client surfaces. Local evidence:
`.git/output-metadata-nested.log`.

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

**Failed twice.** First, synthetic mode dimensions lacked timing fields and
metadata validation stopped before expected ENOTTY. Added explicit synthetic
clock/sync/totals and rebuilt. Second, non-DRM refusal was reached, but the client
failed at `examples/support/offscreen_client.rs:126`: expected membership `(4, 0)`,
received `(0, 0)`; direct command exited 1. Likely cause: first-success global
publication requires another bind roundtrip before enter events. This has **not
been corrected or verified** in that fixture. No further repair/retry followed
the repeated failure. First wrapper misleadingly returned zero despite logging
failure; `.git/output-metadata-offscreen.log` is failure evidence, not a pass.

```sh
env __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-output-metadata-egl.json \
  timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

Correctly exited 1 with invalid EGL Display; this does not replace the failed
normal offscreen check. `git diff --check` passes. Tracked changes and new source/
tests inspected; no assertion or lint gate was weakened.

## Remaining work

The owner has explicitly authorized recovery from this halt. The original
remaining handshake work is resolved and complete publication gates passed.
After this component: output retirement on session pause, client leaves/global
removal, pending callbacks and failed-disable refusal; direct renderer/input/
session wiring. No successful physical DRM scanout or hardware dimensions were
measured. Safe async transport, GPU context-loss and existing parent-leave/libseat
limits remain. Physical business laptop and >=24 GB GPU workstation records and
remaining release acceptance are owed; WSLg never certifies hardware.

All four shared progress documents retain this unfinished step. All reports
published at iteration start were already referenced in STATE; none awaited
reconciliation. Later reports reconcile next iteration. Supervisor full Windows/
Linux workspace/BPF publication gates had not run at the halt. No other checkout,
credentials, git identity, workers, supervisor tools or physical disks changed.

## Owner-authorized recovery, 2026-09-08

The old supervisor PID was absent and status was HALTED; no second editor was
started. Reproduced the remaining failure with the built offscreen executable:
exit 1, membership (0, 0) rather than (4, 0). The first roundtrip discovers the
success-only wl_output global and queues a bind. A second roundtrip is necessary
to complete that bind before asserting surface enter events. Existing direct
protocol and output-metadata wire fixtures already used this handshake.

`examples/support/offscreen_client.rs` now completes that bind and asserts one
output and all four enters/callbacks. Before success, it additionally asserts
zero outputs and no output metadata. No sleeps, assertion removal, renderer
bypass or supervisor gate changes. Rebuilt all shell examples, then normal
`nested_check --offscreen` passed twice, retaining exact golden pixels, real SHM
scenes, cursor switching, callback preservation, disconnect and truncated-SHM
refusal. The nested `--popups --cursor` run passed with 115 client surfaces.
Invalid EGL with the same nonexistent vendor path still exits 1 as expected.
Expected protocol-refusal diagnostics and WSLg Mesa fallback warnings are not
compiler warnings and did not replace checks of command exit status.

Local logs: `.git/output-metadata-recovery-offscreen.log`,
`.git/output-metadata-recovery-nested.log`,
`.git/output-metadata-recovery-egl-refusal.log`.

The existing desktop supervisor is unchanged. Its fmt, all-target clippy with
warnings denied, 11 unit/publication tests and release build passed, using the
documented `--manifest-path tools/dev-loop/Cargo.toml` commands. In particular,
failed combined-tree gates preserve work without pushing. Claude's separate
kernel-publication follow-up is saved in `../claude-publication-review.md`;
the kernel checkout was not edited or started here.

Full publication gates executed successfully on the recovered code:

```sh
# Windows, then Ubuntu WSL2 with the documented PATH, LLVM_PREFIX and target dir:
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked --quiet
# Linux only:
RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked
# From crates/alo-bounding-kernel, using its pinned toolchain:
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
```

Both platform command chains exited 0; all 116 test-result groups in each log
passed, including zero-test/cfg-excluded groups. Existing ignored tests remain
ignored; this count is not a count of executed acceptance tests. Linux rustdoc
and pinned BPF clippy also exited 0. Linux used the supervisor's bpffs mount
precondition before kernel tests; no existing pins or unrelated processes were
removed. Logs: `.git/output-metadata-recovery-windows.log` and
`.git/output-metadata-recovery-linux.log`. `git diff --check` passed.

Shared progress documents now record recovery and the next desktop component.
Publication is a normal main push after a final remote check; restart only after
the checkout is clean and synchronized. WSL remains development evidence, not
certification. The constitution's physical acceptance is explicitly still owed.
