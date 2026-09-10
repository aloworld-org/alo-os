# Automatic native name fallback

Date: 2026-09-10. Workstream: native desktop and release-progress integration.
Responsible contributor: desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; independent supervisor gates pending.

## Change and decisions

Started clean at `a4ec1df`. Read CLAUDE, DELIVERY, SHARED_MAIN, updates guidance,
current QUEUE/STATE, relevant v0.01 features/roadmap, ADRs 0002/0010 and the
native-window-controls contract. No AGENTS.md found. Every published report was
already referenced in STATE at iteration start; no reconciliation was due.
Recorded the selected component and acceptance in QUEUE before implementation.
Desktop ownership retained; no filesystem work or other checkout touched.

The live native name presenter now automatically opens a paged reader when a
complete expanded label cannot fit. Names that fit keep the existing complete
label; disabled names remain accessible and no window command runs. This is a
complete automatic-fallback component, not completed desktop window management.

- `src/window_control_label_expansion.rs` privately distinguishes exhausted
  capacity (`None`) from invalid preparation (`Err`). Public `prepare_expanded`
  keeps its existing refusal behavior. No broad render error is treated as a
  reason to retry with paging; geometry, vocabulary, text and glyph checks remain.
- `src/window_control_name_fallback.rs` adds
  `Server::render_presented_window_control_name`. It validates the live strip and
  matching target output, selects native focus or fresh hover, and prefers the
  complete expanded label. The complete-label branch reuses the existing label
  transaction, including its bounded fresh preparation. On exhausted capacity it
  opens through the existing all-pages preflight and submits through the reader
  transaction. Only successful submission returns a reader with input authority.
  Errors retire native authority and cancel pointer feedback while retaining
  release ownership. Existing transactions preserve pending callbacks.
- `Nested::render_control_name` uses actual parent position and the same input
  owner as its reader pump/render methods. The host retains the returned reader
  across navigation. Calling the opener every frame would reset reading; this is
  documented explicitly, including the host's dismissal/resumption responsibility.
- `tests/window_controls/name_fallback.rs` adds four real private-client tests:
  preferred and expanded labels with ordinary typing; thirty-page automatic
  fallback from focus/hover/disabled names with publication-bound key navigation
  and every page's complete line range; oversized text, missing navigation,
  impossible chrome, wrong output and retired-publication refusals before any
  submission; failed/omitted-root submissions for both label and reader paths.
- `examples/support/nested_reader_frame_check.rs` adds two automatic fallback
  submissions (twelve pages, both schemes), keeping the existing twelve reader
  feedback submissions, two explicit backend-owned submissions, pump calls and
  every original refusal/lifecycle assertion.

Paths above are under `crates/alo-shell/`. The native API preserves ADR 0002's
Rust architecture, ADR 0010's existing palette, externalized wording, original
text scale and ordinary keyboard routing. No engine patch, new scope or agent
surface. Public rustdoc, contract and all four shared progress documents updated.

## Verification

Ubuntu WSL2 prerequisites checked: `/mnt/wslg/runtime-dir/wayland-0` is a socket;
`findmnt -n -t bpf /sys/fs/bpf` reports the existing mounted bpffs.
`pkg-config --modversion wayland-client egl gbm xkbcommon libinput libudev`
returned 1.24.0, 1.5, 26.0.8-1ubuntu0.3, 1.13.1, 1.31.1 and 259. No installation
or shared maintenance needed. Private-client fixtures do not mutate global
kernel state; no outer machine lock added. Desktop target `/root/alo-os-target`.

Before each build/test/lint/format/doc command, Windows C: was checked using:

```powershell
$free = (Get-PSDrive C).Free
Write-Output "C free bytes: $free"
if ($free -lt 12GB) { throw 'C reserve below 12 GiB' }
```

All readings exceeded 53 billion bytes, above the 12 GiB floor. Minimum observed
command preflight: 53,318,434,816 bytes. This is preflight headroom, not a quota.

Linux commands used `wsl -d Ubuntu --cd /mnt/c/dev/alo-os --exec env` with
`PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target`:

```sh
cargo check -p alo-shell --locked
cargo test -p alo-shell --locked name_fallback -- --nocapture
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked --quiet
cargo build -p alo-shell --example nested_check --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
```

Final results: check passes; four focused tests pass. After adding key-navigation
assertions and correcting fixture lint, all-target clippy passes and the full
affected suite passes 168 unit, 254 lifecycle, three socket and four compile-fail
doctests, none ignored. Example build and warnings-denied rustdoc pass.

Graphical commands additionally set `XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir`
and `WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --reader
timeout 30s /root/alo-os-target/debug/examples/nested_check --controls
```

Both pass on their first run this iteration. Reader checks include the two new
automatic twelve-page reader submissions in light and dark, twelve existing
page/feedback submissions, two existing explicitly opened backend-owned readers,
navigation/dismissal, geometry refusal/recovery and parent pump calls. Controls
retain eight strip, two label and two expanded-name submissions, removals and
refusal/recovery. Both keep six-client unmap/remap/refusal/disconnect checks.
This proves nested EGL submission, not submitted-frame readback or synthesized
parent keyboard/button delivery. Existing Mesa and intentional malformed-client
diagnostics remain visible. No timeout or assertion changed.

Windows commands:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

All pass. Windows shell tests have zero cases because this crate is Linux-only.
No claim is made that supervisor full workspace/Windows/Linux/rustdoc/BPF gates
have run. No staging, commit or push performed.

## Preserved failure and logs

Logs: `.git/alo-loop/automatic-native-name-fallback/`.
`check.txt` and `focused.txt` pass. Initial `clippy.txt` fails on needless borrows
and a needless Vec in new fixture text. Replaced those fixture expressions with
arrays and direct owned generic arguments; `clippy-fixed.txt` passes. No lint
allowance or gate change. `linux-tests.txt`, `example-build.txt`, `rustdoc.txt`,
`wslg-reader.txt`, `wslg-controls.txt`, `fmt.txt`, `windows-clippy.txt` and
`windows-tests.txt` record passing final checks. PowerShell wraps native stderr
as NativeCommandError even for successful cargo/Mesa diagnostics; native exit
status was captured and checked, and the enclosing commands exited with it.

No graphical deadline failure occurred in this iteration. Passing these runs
does not establish a fix for the intermittent upstream swap delays recorded by
earlier reports; no upstream graphics settings or engines changed here.

## Remaining work and evidence limits

Automatic fallback is now available in the opt-in live-name presenter. Next:
attach explicit opening to a native activation gesture, then native cursor
selection and direct backend integration. A session host must choose when to
resume hover/name presentation after dismissal and retain readers while active.
No complete full-name-access, window-management or release checkbox is promoted.

Private-client and WSLg evidence does not certify direct scanout, physical input,
VM integration or hardware. Integrated VM boot/recovery and the named physical
laptop/GPU workstation records remain owed in their delivery phases. Independent
supervisor gates/publication remain pending. Reports arriving during publication
are reconciled next iteration; release verification is not claimed.

No cleanup, shared service/mount/package/session change, WSL restart/helper,
second worker/loop, dev-loop edit, other-repository/checkout change or credential/
identity access. All tracked/new code, tests, example and documentation inspected.
Changelog, roadmap, queue completion/next component and this report's STATE
reference are included in the same change.
