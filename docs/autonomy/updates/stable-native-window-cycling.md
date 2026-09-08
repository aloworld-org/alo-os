# Stable native window cycling

Date: 2026-09-08. Workstream: native desktop compositor/window management.
Responsible contributor: the single desktop development worker in C:\dev\alo-os,
also the integration owner. Status: ready for integration after recorded checks.

## Change and decisions

`crates/alo-shell/src/window_switch.rs` adds trusted Server::switch_window with
typed forward/backward selection and explicit empty/activation errors. The server
owns a ring of mapped roots, refreshed at dispatch boundaries and before selection.
Newly observed mappings append, survivors retain order, and observed unmaps or
disconnects remove roots. Remapping after removal appends again. Raising does not
reorder this ring: otherwise repeated selection can oscillate between two windows
and strand the third. Transitions within a single dispatch are observed at its end.

The keyboard helper resolves actual focused toplevel ownership, including grabbed
popups. No focus selects the first/last root; traversal wraps; one root selects
itself and preserves its popup grab. Activation shares existing XDG state, raising,
held-key release and popup dismissal paths. Missing keyboards refuse before ring
or focus mutation, including empty displays. Empty selection refuses without
changing focus. Activation errors retain their partial focus/raise-change detail.
Success returns the chosen root and queues configures, not client-render evidence.

ADR 0002 and the existing v0.01 window-switching requirement authorize this native
Rust component. No unsafe exception, engine patch, new release scope or approval
is needed. Rechecked pinned Smithay EGLDisplay::new: it remains unsafe under the
existing lint restriction, so DELIVERY's first independent window-management
component is selected. No alternative engine policy is introduced. ADR 0010's
colour/label rules remain unchanged; this component has no visual controls or
user-facing strings. Public rustdoc and COMPOSITOR.md document the boundary.
There is no agent endpoint/context reader; application focus still requires its
grant/proposal/single-approval contract. Native shortcut dispatch remains next.

Acceptance was recorded in QUEUE before coding. The source, tests and graphics
fixture each have separate files. Four new real-socket tests exercise three-client
forward/backward traversal and wrapping despite raises; explicit focus anchoring;
no-focus first/last selection; empty/missing-keyboard refusal; removal, remapping
and disconnect; singleton idempotence; popup-root anchoring and grab dismissal;
synthetic held-key release, empty keyboard-enter keys and fresh-key routing.

## Verification

Ubuntu WSL2 Rust 1.98.0 (88d9e12ae 2026-08-18); prerequisites checked with:

```sh
rustc --version
pkg-config --modversion wayland-client egl glesv2 xkbcommon libudev libinput gbm libseat
test -S /mnt/wslg/runtime-dir/wayland-0
```

Versions: 1.24.0, 1.5, 3.2, 1.13.1, 259, 1.31.1, 26.0.8-1ubuntu0.3, 0.9.2.
WSLg present; /dev/dri absent. No packages, shared kernel/BPF state, services or
other checkout changes. Registry signature inspection used grep after discovering
rg is absent inside Ubuntu (host repository searches used rg).

From C:\dev\alo-os on Windows, passed:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
```

Windows runs zero Linux-only shell runtime cases. Linux commands from
/mnt/c/dev/alo-os with PATH=/root/.cargo/bin:/usr/bin:/bin and isolated
CARGO_TARGET_DIR=/root/alo-os-target:

```sh
cargo test -p alo-shell --locked window_switch
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
cargo build -p alo-shell --examples --locked
cargo fmt --all --check
```

Focused tests: four passed. Full shell tests: 138 unit, 95 client lifecycle,
three socket ownership and three compile-fail doctests passed, none failed or
ignored. The full run includes the later added wire assertions for released keys
and empty enter arrays. First clippy attempt found a missing private method
comment; added documentation without suppression, then affected all-target clippy
passed. No runtime test failed. Rustdoc and examples passed with the final code.

After rebuilding examples, with XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir and
WAYLAND_DISPLAY=wayland-0, both passed (exit zero):

```sh
timeout 30s /root/alo-os-target/debug/examples/nested_check --offscreen
timeout 30s /root/alo-os-target/debug/examples/nested_check --popups --cursor
```

New window_switch_check.rs extends the eight-stage offscreen fixture: forward
selection exposes the second client's magenta pixel, backward selection restores
the first client's red pixel, and returned roots are checked. The existing client
wire assertions still pass. Existing raising/activation/popup/cursor golden pixels
and truncated-SHM refusal passed. Nested regression submitted 115 surfaces.
Expected Mesa fallback diagnostics and deliberately invalid-client errors occurred
alongside success. These checks demonstrate real GLES and Wayland clients, not
direct scanout, physical keyboard use, GPU acceleration or hardware certification.

Full independent Windows/Linux workspace/rustdoc/BPF publication gates belong to
the supervisor and were not run by this worker. Source/test/docs diff reviewed;
final whitespace verification is recorded with the integration entry in STATE.

## Reconciliation and remaining acceptance

At iteration start, compared all published report filenames against STATE:
none unreferenced. Claude retains security/model-choice ownership. No source
reports were rewritten. Reports arriving during publication reconcile next time.
CHANGELOG, ROADMAP, QUEUE and STATE are updated in this change; no feature or
release checkbox is completed.

This completes stable cycling plumbing. Next: person-configured shortcut dispatch
for cycling and close, with consumed press/release isolation in nested/direct
input. Rendered controls, application grouping, adapter wiring, move/resize/
minimise/maximise/tile, launcher/dock, clipboard and later delivery work remain.
Safe standalone GLES, full DRM/seat entry, populated input/hotplug, GPU context
loss/failed-disable recovery and certified physical laptop/GPU workstation
records remain owed. Hardware access is required for those records; WSLg cannot
supply them. No staging, commit, push, other repository edit, worker/loop launch,
tools/dev-loop change or physical installation; supervisor owns publication.
