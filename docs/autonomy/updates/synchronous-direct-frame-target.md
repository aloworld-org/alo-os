# Synchronous direct frame target

Date: 2026-09-07. Workstream: native desktop compositor and release integration.
Responsible contributor: single desktop development worker in `C:\dev\alo-os`.
Status: ready for integration; supervisor publication gates remain pending.

## Change and decisions

`crates/alo-shell/src/direct_target.rs` adds `DirectTarget`, a `FrameTarget` that
renders real GLES window, popup and client cursor trees into prepared pixels and
uses the existing blocking activation/replacement transactions. It freezes the
owned discovery snapshot, borrows the renderer and session descriptor, and never
dispatches clients. Only committed identities reach the existing presentation
membership and callback path. Initial or replacement refusal preserves callbacks
and previous membership; clean refusals permit a subsequent attempt.

Successful replacement with old-resource cleanup failure still publishes the new
identities. `retirement_error()` exposes that failure, all further submissions
refuse before painting or DRM I/O, and consuming `disable()` reports both the
stored error and any shutdown failure through `DirectShutdownError`. Candidate
cleanup failure also latches shutdown; its original error belongs to the caller.
Disable refusal preserves existing quarantine and no-drop-retry semantics.

The safe synchronous transport avoids the unresolved asynchronous cookie problem
without an upstream patch or unsafe code, consistent with ADR 0002. Full-frame
CPU readback and fresh allocation favor complete ownership over optimization.
No agent verb, context reader, application-adapter contract or release scope
changes. New diagnostic errors are backend data; native entry must translate
them. Use fresh discovery inside `DirectSession::with_device`, exclusive ownership
of an inactive output, a current renderer and an active session through shutdown.
The owner already authorized this implementation; no additional approval needed.

Queue acceptance was recorded before implementation. All reports published at
iteration start were already referenced in STATE; no unreconciled contribution
or Claude-assigned work was taken. Later publications reconcile next iteration.

## Executed verification

Windows PowerShell, exit 0 for each:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Linux shell code/tests are cfg-excluded on Windows. Ubuntu commands were run
through `wsl -d Ubuntu -u root -- env PATH=/root/.cargo/bin:/usr/bin:/bin
CARGO_TARGET_DIR=/root/alo-os-target bash -c '...'` from this checkout. Verified
Rust 1.98.0, WSLg `/run/user/0/wayland-0`, pkg-config wayland-server 1.24.0,
EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2, libinput 1.31.1.
No dependency installation needed. `/dev/dri` is absent.

```sh
cargo test -p alo-shell --lib direct_target --locked
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
```

All final commands exit 0: 97 unit + 65 lifecycle + 3 socket + 2 doctests = 167
passing checks, no failures or ignored checks. Five new tests cover initial and
replacement allocation/upload/test/enable refusal and retry, cleanup latching,
ordered shutdown retaining two errors, no duplicate disable/destruction, graphics
refusal before DRM, and actual Wayland callback/enter/leave wire events over seven
commit/refusal stages. Successful DRM and scene pixels in these unit tests are
injected; the protocol fixture explicitly controls scene eligibility. Final test
strengthening adds repaint counters and a fresh callback after terminal cleanup;
fmt, Linux all-target clippy and the five focused tests were rerun successfully.

Initial unused broad fixture warnings were removed by implementing a minimal
wire client, and private-doc/conditional clippy findings were fixed without lint
exemptions. `.git/direct-target-linux.log` retains the full initial passing command
output, but its PowerShell stderr wrapper returned an inconsistent status; the
entire command group was repeated directly with `set -e`, confirmed exit 0.

Live WSLg checks, with `XDG_RUNTIME_DIR=/run/user/0 WAYLAND_DISPLAY=wayland-0`:

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
__EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-direct-target-egl.json timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
```

First two exit 0. Offscreen verifies the existing 1,056-pixel real SHM scene and
now calls `Server::render` with public `DirectTarget`: `/dev/null` refuses with
ENOTTY (25), no premature callbacks/membership, clean shutdown. Existing scene
size, disconnect and truncated-SHM import refusals still pass. Nested regression
submits 125 client surfaces, including popup and cursor checks. Invalid EGL exits
1 with invalid EGL Display, as required. Logs:
`.git/direct-target-{gles,nested,refusal}.log`. Mesa fallback diagnostics persist;
no successful KMS commit or hardware measurement occurred.

## Integration and remaining evidence

Updated CHANGELOG, ROADMAP, QUEUE, STATE and COMPOSITOR in this same change.
Proposed publication: `feat(shell): connect synchronous direct frame presentation`.
Tracked and new source/document diffs reviewed; `git diff --check` passes.
No staging, commit, push, other checkout changes, supervisor/gate edits, worker
launch, shared kernel/BPF/cgroup/service changes or physical installation.

Next executable component: render the compositor-owned default cursor in direct
scenes (scale-one shape/hotspot, clipping, hidden/client-cursor switching and pixel
tests), then wire pause/retirement, direct input and session entry. Shared output
metadata still uses the nested placeholder name/model and unknown refresh/physical
size; direct session integration must supply truthful output metadata. Real DRM
renderer creation and successful scanout require a DRM-equipped login/VM; scheduling
is still the caller's responsibility. Async cookie transport, GPU context-loss/
draw/readback faults, parent-leave/libseat/unmap-panic limits remain outstanding.
Supervisor full Windows/Linux workspace test/lint/rustdoc/BPF gates remain owed.
All physical business-laptop and >=24 GB GPU-workstation display/input/session/
suspend-resume and remaining v0.01 acceptance records remain owed. Compositor and
release stay unchecked; WSLg and injected DRM are not hardware certification.
