# Cookie-preserving page-flip event reading

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Contributor: single desktop development worker / release-progress integration owner
- Status: ready for integration; one event-reading component, not complete presentation

## Change and acceptance

`crates/alo-shell/src/drm_events.rs` adds `read_display_events`, `DisplayEvent` and
`FlipComplete`, exported from `lib.rs`. A bounded nonblocking borrowed-fd read
preserves full user_data, explicit CRTC ID, wrapping sequence and kernel timestamp.
Native-endian field copies require no alignment or unsafe code. Unknown event types
never become completions; extended flip tails are skipped. Invalid lengths, short
payloads, zero CRTC and invalid microseconds refuse the entire consumed batch.
WouldBlock, EOF and kernel errno remain distinguishable. The reader never changes
flags, closes the fd or retries; blocking descriptors refuse before consuming data.
User-readable description: direct-display reads now retain the identity needed to
match frames and refuse malformed completion data without returning a partial batch.

Selected this complete prerequisite in QUEUE before implementation. Local source
inspection found drm 0.14.1 discards flip user_data and drm-ffi 0.9.1's atomic helper
submits zero user_data. A future pending owner cannot safely use that wrapper for
cookie matching. This reader uses the documented native ABI, without patching an
engine or adding a dependency. ADRs 0001/0002 and agent/adapter contracts are unchanged.
The [kernel UAPI](https://www.kernel.org/doc/html/v6.12/gpu/drm-uapi.html) supports
the framing/event semantics; pinned drm-ffi layout is independently checked in a test.
Explicit CRTC IDs are required; there is no legacy-cookie-as-CRTC fallback.

`drm_events_tests.rs` has ten tests: full cookie/timestamp/sequence preservation,
batched unknown events, extended and unaligned records, every short flip length,
bad framing, invalid timestamps/CRTC, real descriptor reads and full 4096-byte batch,
WouldBlock/EOF, blocking refusal with unchanged flags and unread data, malformed
batch consumption without partial completion, actual EBADF and caller-fd survival,
and pinned kernel ABI size/offsets. Several cases share one test function.

## Executed checks

Windows PowerShell, C:\dev\alo-os:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Pass. Formatting ran after source changes; final check/clippy/tests repeated on the
final Rust tree. Linux shell tests are cfg-excluded on Windows (zero executed).

Ubuntu WSL, same checkout, invoked with
`wsl -d Ubuntu -u root -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target bash -c 'cd /mnt/c/dev/alo-os && ...'`:

```sh
cargo test -p alo-shell --lib drm_events --locked
cargo test -p alo-shell --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Initial focused eight tests passed, then ten tests passed within the full suite.
First clippy found indexing/expect usage, including tests; fixed with checked access
and Result propagation, no exemptions. Final affected clippy followed by full tests,
fmt, rustdoc and example build all exit 0. Final Linux result: **62 unit + 64 client
lifecycle + 3 socket + 2 lifetime doctests = 131 checks**, zero failures/ignored.
No test failure or gate weakening. WSLg regression exits 0, reporting **133 client
surface submissions**, including popup/cursor callbacks, unmap/remap, isolated
protocol refusal and disconnect. Mesa emits its known ZINK diagnostic before
successful GLES submission. This is not pixel readback or physical input evidence.

Local logs: `.git/alo-drm-events-focused.log`, `alo-drm-events-linux.log` (initial),
`alo-drm-events-clippy.log` (initial refusal), `alo-drm-events-clippy-final.log`,
`alo-drm-events-linux-final.log`, `alo-drm-events-docs.log`,
`alo-drm-events-examples.log`, `alo-drm-events-wslg.log` (all under `.git/`).
Redirection was inside Bash; WSL process exit was explicitly propagated.

Prerequisites verified: Rust 1.98.0, WSLg socket exists; pkg-config reports
wayland-server 1.24.0, EGL 1.5, GLES 3.2, GBM 26.0.8-1ubuntu0.3, libseat 0.9.2,
libinput 1.31.1. No packages needed. `/dev/dri` is absent. Initial root-level
QUEUE/STATE reads and Windows wildcard rg paths were corrected; local upstream
source lookup was corrected to single-word patterns after shell quoting lost spaces.

## Reconciliation and remaining evidence

At iteration start the sole unreferenced published report was
`docs/autonomy/updates/network-request-boundary-approval.md`. Reconciled its explicit
owner approval and retained all limits: Claude owns the scoped ADR and implementation;
explicit DNS and request-limited connections must preserve provider/region policy,
credentials, indicator, records, local models and no silent fallback. No blanket
network exception, broader grants, host-wide changes or physical installation.
Kernel audit changes remain a separate decision. That report only inspected prose
and ran diff checking; it provides no runtime enforcement evidence. The production
provider-request gap and retries/redirects/UDP/inherited-socket/proxy coverage remain
open. No Claude workstream was taken and no contributor checkout was accessed.

All four shared documents and COMPOSITOR updated. This report is referenced in STATE.
Reports arriving during publication are reconciled next iteration. No release claim.

Next: cookie-bearing nonblocking atomic submission, pending-buffer ownership and
matching cookie/CRTC/session before old-buffer release, with stale/foreign events,
commit refusal and teardown tests. The reader is not connected to a pending scanout
owner yet. The caller must exclusively own reads/flags; malformed batches cannot
authorize release. Unsupported events larger than 4096 may fail at the kernel read.
Rendered frames, session pause ordering, direct input, production entry and existing
parent-leave/libseat limits remain. Successful DRM event delivery requires a
DRM-equipped development login/VM. WSL socket fixtures are synthetic ABI integration,
not DRM or hardware certification. Certified business laptop and >=24 GB GPU
workstation display/input, suspend/resume, session switching and all physical
checklist records remain owed.

Source/tests and tracked/new-file diff reviewed; diff check passed. No staging,
commit, push, dev-loop modification, worker launch, shared kernel/BPF/cgroup/service
change or physical installation. Full Windows/Linux/BPF supervisor publication
gates have not run for this change; they remain the supervisor's responsibility.
