# Native compositor development

Item 33 implements the v0.01 Smithay compositor in `docs/features.md`, following
ADR 0002. The first component is `crates/alo-shell`: a reusable Linux Wayland
server library. The second adds nested Wayland/GLES rendering, described below.
It is not yet a session executable or a usable desktop.

## Protocol core

`Server::bind(runtime, name)` creates `runtime/name/wayland`. The runtime must
be an absolute, real directory owned by the effective user with mode 0700.
The name is a bounded ASCII component; a fresh mode-0700 child directory is
created exclusively. Existing directories, files or sockets are refused rather
than adopted or replaced. The absolute returned socket path can be passed to a
session application's `WAYLAND_DISPLAY`; the library does not change environment
variables or launch a process. Drop closes the display and removes its socket,
lock and empty directory. Failed binds clean up their fresh directory too.
Unexpected directory contents are never recursively removed.

`Server::dispatch` accepts up to 16 clients per call, dispatches pending Wayland
requests, removes dead toplevel handles and flushes events. Each client gets its
own Smithay compositor transaction state. The display is private, so callers
cannot insert clients without that state. A backend drives this method and
consumes `mapped_surfaces`; the nested driver now supplies rendering, while
production event scheduling and input routing remain subsequent work.
The acceptance budget bounds that loop only, not all client request processing.

The advertised protocols are core compositor/subcompositor, SHM and XDG shell.
Toplevels get their first configure after the initial empty commit. A buffer
cannot make a toplevel eligible for rendering until a configure is acknowledged.
Null-buffer unmap releases the buffer and resets XDG role state, including old
acknowledgements; remapping requires a fresh empty commit/configure/ack sequence.
Smithay 0.7.0's buffer helper handles buffer ownership. Its XDG commit hook resets
the initial-configure flag on unmap but retains configured/acknowledgement state,
so this compositor resets the public role state as well. This is compositor
policy using upstream APIs, not an engine patch. Protocol errors disconnect only
the offending client. Dead roots are filtered immediately and pruned on dispatch.

Popups are explicitly dismissed with `popup_done` until popup placement/input
exists. No seat, selection, context or agent protocol is advertised. An output
is advertised when `Server::render` first receives a positive framebuffer size.
The `SeatHandler` implementation is required by Smithay's XDG dispatch types;
it creates no seat or input device. Frame callbacks are not completed by this
core: the rendering backend must complete them after submission, never claim
that an unrendered frame was presented.

These choices follow ADRs 0001/0002 and preserve `app-adapters.md` and
`daemon-protocol.md`: applications' Wayland surfaces do not grant an agent access
to context or control. The Rust API is internal shell plumbing, not a new
adapter/agent contract. Typed startup errors are developer diagnostics; session
entry still owes translated user-facing failure messages.

## Evidence collected 2026-09-07

Ubuntu/WSL setup and dependencies remain those in `GRAPHICS.md`. Rechecked native
metadata: Wayland 1.24.0, EGL 1.5, xkbcommon 1.13.1. WSLg's socket was present.
Builds use this checkout's `/root/alo-os-target`, not another contributor's target.
No cgroups, BPF pins or shared system services were modified in this component.

Commands actually run from `C:\dev\alo-os` (Ubuntu working directory
`/mnt/c/dev/alo-os`):

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
git diff --check
```

All passed. Linux has **eleven integration tests**, no ignored tests:

- Initial empty commit, configure/ack, map, buffer release on unmap, fresh
  configure/ack and remap, followed by orderly role/surface destruction.
- Abrupt client disconnect and a replacement client; two simultaneously mapped
  clients with one disconnecting while the other's buffer remains owned.
- Server teardown closes live clients and removes the display socket.
- Premature buffer, configure received without acknowledgement, invalid serial,
  stale serial after unmap, and remap without a new handshake are refused.
  A valid client remains usable after another client's protocol errors.
- Socket collision, invalid names, symlink/non-private runtime, occupied file
  preservation, normal teardown/rebind and overlong socket bind cleanup.

Windows clippy/tests pass with the Linux-only library and tests excluded; that
is build portability, not protocol execution evidence. Linux rustdoc passes
with warnings denied. Initial compilation required mapping Wayland display
initialization errors and supplying Smithay's required seat trait. The first
test run refused its temporary runtime directories: the fixture now explicitly
sets 0700 rather than assuming tempfile supplies it. Production checks were
not relaxed. Subsequent finalized tests passed.

Additional integration evidence:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle configure_map_unmap_remap_and_orderly_destroy -- --exact --nocapture
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target timeout 30s /root/alo-os-target/debug/alo-graphics-check render
```

Both exited 0. The first was captured locally in `.git/alo-shell-wire.log`.
It records SHM `create_pool` transferring a file descriptor for 1024 bytes,
16x16 ARGB buffer creation, configure serial 1 acknowledged before attachment,
null attachment and `wl_buffer.release`, configure serial 2 acknowledged before
reattachment, and ordered role/surface destruction with buffer release.
The client and server speak through a real Unix socket in separate threads;
this is not a physical machine, separate-process crash or presentation test.

The second command submits the existing graphics probe's 320x200 GLES frame
through WSLg. Mesa still reports the device-selection diagnostics in GRAPHICS.md.
It confirms the graphical prerequisite remains usable; **it does not render
the new server's clients** and does not establish GPU acceleration.

## Remaining work and acceptance

Nested rendering is now implemented as described below. Next implement
keyboard/pointer routing, popup lifecycle and the direct DRM/seat
backend. Window management, native entry, appearance, agent interaction and image
integration continue in delivery order. This component does not complete item 33.

The supervisor still owes its independent complete Windows/Linux tests, clippy,
rustdoc and pinned BPF publication gates. Booted direct-display execution,
physical keyboard/pointer/display checks and the release's certified-machine
records are still owed; no WSL or VM result can certify them.

## Nested rendering component (2026-09-07)

`Nested::new(title, size)` creates one Wayland-parented GLES window on the main
thread. The session must supply its translated title; the explicit developer
fixture uses a diagnostic title. Invalid dimensions, missing/unreachable
Wayland configuration and failed graphics initialization return errors. X11
is refused after checking the actual display handle. Winit permits one event
loop per process: initialization failure ends the driver, rather than retrying
creation. `Nested::pump` handles parent resize/close; close is terminal. Input
events remain unconnected and no seat is advertised to applications.

Drive `pump`, `Server::dispatch`, then `Server::render(&mut nested, time)`.
Dispatch flushes pending protocol events before a potentially blocking swap;
the next dispatch flushes newly completed callbacks. The bounded example sleeps
between frames; it is an integration driver, not production event scheduling.
The caller supplies monotonic milliseconds wrapping at 32 bits. The public
Rust `FrameTarget` trait is trusted compositor plumbing, not an agent or adapter
surface. It permits deterministic failed-submission tests without requiring
graphics in the standard test suite. Existing agent/adapter contracts do not
change, and rendering does not grant context access (ADRs 0001/0002).

The single `alo-nested` output uses actual framebuffer dimensions, scale 1 and
unknown physical dimensions/refresh. Resize replaces the old mode without
creating another global. Each configured toplevel tree is imported in existing
creation order at the origin, preserving child offsets and tree stacking.
Window placement, focus and stacking policy belong to the next delivery steps.
Only elements intersecting the output enter its membership and callback set.
Offscreen children remain pending; they are not reported as submitted.

`drawing.rs` uses Smithay's fallible per-surface import API because its tree
convenience helper logs import failures and continues. Import, drawing, finish
and swap errors propagate; output membership and frame callbacks advance only
after successful submission, without intervening client dispatch. This is a
frame pacing notification, not proof of physical presentation. The backend
clears to neutral black pending the token-based shell design; it introduces no
second alo palette. Smithay 0.7.0 is unmodified. Its nested backend supplies the
parent synchronization; no direct scanout or GPU acceleration is certified.

### Checks actually run

Native prerequisite metadata rechecked: Wayland 1.24.0, EGL 1.5, GBM
26.0.8-1ubuntu0.3, xkbcommon 1.13.1, libinput 1.31.1, libseat 0.9.2; WSLg socket
present. Rust uses the explicit `/root/.cargo/bin` PATH from GRAPHICS.md;
the login shell's default PATH did not contain rustc. No dependency installation,
shared kernel mutation or change to another checkout was needed.

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
git diff --check
```

Linux: **15 passing tests** (one unit, eleven client lifecycle/output, three
socket ownership), no ignored tests. New checks cover invalid initial dimensions,
empty framebuffer refusal, output mode/resize and enter/leave, exactly-once
callback completion, failed-swap callback retention, unrendered surfaces and
synchronized children remaining part of their toplevel. Windows compiles with
Linux code excluded and executes zero Linux tests; it is not graphics evidence.

Additional explicit graphical integration, after building the example:

```powershell
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check
wsl -d Ubuntu -- env __EGL_VENDOR_LIBRARY_FILENAMES=/nonexistent/alo-egl-vendor.json timeout 30s /root/alo-os-target/debug/examples/nested_check
wsl -d Ubuntu -- env -u WAYLAND_DISPLAY timeout 30s /root/alo-os-target/debug/examples/nested_check
```

The first exited **0**; local `.git/alo-nested-wire.log` records two callbacks at
509 ms for a 16x16 ARGB root and child at (24,32), no callback/enter for the child
at (1000,1000), and six submitted client-surface instances across frames.
It also verifies both output leaves on unmap, buffer release, a fresh remap,
orderly destruction, premature-buffer refusal and abrupt disconnect while the
same GLES backend continues running. Counts/timestamps vary with scheduling.
This is real buffer import/draw/submission and wire evidence, not pixel readback
or physical presentation evidence. Mesa's existing loader diagnostics remain.

The failure invocations each exited **1**, respectively reporting
`Egl(DisplayNotSupported)` and `WAYLAND_DISPLAY is missing`, with no successful
submission. Logs remain local in `.git/alo-nested-invalid-egl.log` and
`.git/alo-nested-no-display.log`. The swap-failure test uses an injected backend
error; an actual parent-loss/swap fault and physical display remain machine
evidence owed, not claims supplied by that injection.

Development corrections: the callback event is non-exhaustive and required an
`if let`; clippy required a collapsed mode-event condition. One test fixture
attempted to construct a negative Smithay `Size`, which panics before reaching
our API; a zero-size case now tests the actual empty-output boundary. The first
child-surface graphical check committed the unmapped parent once too many and
discarded its already-received configure in the fixture. Removing that extra
commit restored the intended fresh handshake; protocol requirements were not
relaxed. Its local failure trace is retained separately. Final checks passed.

Item 33 remains unchecked. Keyboard/pointer, popups, production scheduling,
direct display and the usable desktop remain unfinished. Full supervisor
Windows/Linux/BPF gates and physical certified-machine evidence are still owed.
