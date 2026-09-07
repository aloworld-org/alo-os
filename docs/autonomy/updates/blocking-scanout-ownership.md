# Blocking scanout ownership

- Date: 2026-09-07
- Workstream: native desktop compositor, delivery step 2
- Contributor: single desktop worker and shared-progress integration owner
- Status: ready for integration; supervisor owns gates and publication

## Change and acceptance

`crates/alo-shell/src/scanout.rs` introduces `DisplayResources::activate` and
`ActiveScanout::disable`. Initialized resources transfer into an active owner
only after TEST_ONLY and a blocking atomic enable succeed. The frozen request
snapshot also supplies disable routing. Disable atomically detaches connector
and primary plane, clears the mode and deactivates the CRTC before destruction.
Both activation refusal paths preserve errno and every resource cleanup failure.
Disable refusal quarantines kernel handles without destruction or retry; the
session descriptor and duplicates must then be retired. Explicit disable reports
errors; drop provides best-effort retirement. A lifetime doctest prevents resources
escaping their descriptor. No agent or adapter public surface changes.

`scanout_check` is an explicit session-mediated enable/disable diagnostic. It
always attempts session shutdown after the scoped operation, retaining operation
and shutdown errors. It never opens a DRM card outside libseat. It immediately
disables the black buffer; it is not a usable desktop or a display measurement.

Selected in QUEUE before coding. Blocking commits complete a useful static
scanout lifetime before nonblocking presentation; no pending event is needed to
retire these resources. This follows ADR 0002 and the kernel's
[atomic commit lifecycle](https://www.kernel.org/doc/html/latest/gpu/drm-kms-helpers.html)
without engine patches. The caller must own the output exclusively. Disable does
not restore another compositor's configuration. Six new unit tests cover exact
enable/disable requests, fixed flags, live retention, explicit/drop retirement,
test/enable refusal plus cleanup failures, failed-disable quarantine, cleanup after
successful disable, and actual kernel ioctl refusal (several paths share a test).

## Verification actually executed

Windows, C:\dev\alo-os:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
git diff --check
```

Passed. Windows excludes Linux shell tests. Final Rust formatting was rerun after
adding request assertions and the lifetime doctest; Linux final fmt also passed.

Ubuntu WSL, same checkout, PATH=/root/.cargo/bin:/usr/bin:/bin,
CARGO_TARGET_DIR=/root/alo-os-target:

```sh
cargo test -p alo-shell --lib --locked
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
cargo fmt --all --check
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo test -p alo-shell --lib real_blocking_enable --locked -- --nocapture
```

Passed: initial 52 unit tests; final suite 52 unit + 64 client lifecycle + 3 socket
+ 2 lifetime doctests = **121 checks**, zero failures/ignored tests. Clippy passed
before and after final request assertions. Real blocking enable/disable requests
through the production transport both refuse ENOTTY (25), retaining caller fd.
Local logs: `.git/alo-blocking-scanout-{unit,clippy,linux,ioctl,docs,examples}.log`.
PowerShell's outer stderr redirection labeled successful Cargo stderr as a native
error and returned a wrapper exit 1; the chained Linux checks all reached successful
example completion. Final fmt/docs/example and focused ioctl commands were repeated
with redirection inside Bash and an explicit WSL exit-code assertion, all exit 0.
No test failure, lint exemption or test weakening.

Additional integration from Windows (exit codes explicitly checked):

```powershell
wsl -d Ubuntu -- env LIBSEAT_BACKEND=seatd SEATD_SOCK=/run/alo-scanout-absent-seat.sock timeout 15s /root/alo-os-target/debug/examples/scanout_check /dev/dri/card0 --enable-and-disable
wsl -d Ubuntu -- timeout 15s /root/alo-os-target/debug/examples/scanout_check /dev/null --unknown
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

Unavailable seat refuses connect with ENOENT/exit 1; unknown option refuses usage
before seat access, exit 1. WSLg exits 0 with **125 client surfaces** submitted,
popup/reactive/cursor callbacks, unmap/remap, isolated refusal and disconnect passing.
Logs: `.git/alo-blocking-scanout-{seat,argument,wslg}.log`. No physical input or pixel
readback measured. Verified the WSLg socket and pkg-config metadata for Wayland,
EGL, GBM, libseat, libinput and xkbcommon; Rust 1.98.0. `/dev/dri` remains absent.
No packages needed. WSL emitted existing unknown .wslconfig-key warnings; no host
configuration was changed. Initial root-level QUEUE/STATE reads were corrected to
docs/autonomy. These diagnostics are not evidence of DRM success.

## Reconciliation and remaining work

At iteration start one published report was not referenced in STATE:
`docs/autonomy/updates/end-to-end-network-enforcement.md`. Read the audit and its
counting-test source. File verbs enter Bounding; ordinary provider requests do not,
and destination registration is absent. The report adds evidence, not enforcement.
Its Linux workspace/BPF gates remain contributor-reported and were not rerun here.
Claude's requested boundary/destination-lifetime decision remains pending, including
DNS and connection reuse. UDP, inherited sockets, loopback proxies and physical
enforcement remain unverified. The desktop worker did not take that workstream.

CHANGELOG, ROADMAP, QUEUE and STATE consolidated by the integration owner, preserving
the network enforcement gap. COMPOSITOR records the additive shell lifecycle.
Source, tests, example and full tracked/new-file diff reviewed; diff check passes.
No staging, commit, push, worker launch, tools/dev-loop changes, other checkout
access or shared kernel/BPF/cgroup/service mutation. Reports arriving during
publication reconcile next iteration.

Next executable component: nonblocking framebuffer submission with matching
page-flip completion and old-buffer retirement, including stale/foreign events and
commit refusal. Renderer/session pause ordering, direct input, production entry,
parent-leave and libseat disable-order limitations remain. Successful DRM allocation,
mapping, test, enable and disable need a DRM-equipped development login/VM. Certified
business laptop and >=24 GB GPU workstation physical display/input, session switching,
suspend/resume and all hardware checklist evidence remain owed. Full supervisor
Windows/Linux/BPF test/lint/rustdoc publication gates have not run for this change.
Compositor and release remain unchecked; the full v0.01 scope is preserved.
