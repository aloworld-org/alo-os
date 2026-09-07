# Native compositor development

Item 33 implements the v0.01 Smithay compositor in `docs/features.md`, following
ADR 0002. The first component is `crates/alo-shell`: a reusable Linux Wayland
server library. It is not yet a session executable or a rendered desktop.

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
cannot insert clients without that state. A future backend drives this method
and consumes `mapped_surfaces`; it must provide event scheduling and rendering.
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
exists. No seat, output, selection, context or agent protocol is advertised.
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

Next: consume these surfaces in the nested backend, advertise one output,
render client buffers, and send frame callbacks only after successful submission.
Test graphics initialization/submission failure and client exit while rendering.
Then implement keyboard/pointer routing, popup lifecycle and the direct DRM/seat
backend. Window management, native entry, appearance, agent interaction and image
integration continue in delivery order. This component does not complete item 33.

The supervisor still owes its independent complete Windows/Linux tests, clippy,
rustdoc and pinned BPF publication gates. Booted direct-display execution,
physical keyboard/pointer/display checks and the release's certified-machine
records are still owed; no WSL or VM result can certify them.
