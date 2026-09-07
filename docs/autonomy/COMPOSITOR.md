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
production event scheduling and pointer routing remain subsequent work.
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
exists. `Server::bind` advertises no seat, selection, context or agent protocol.
`Server::bind_keyboard` adds the keyboard seat described below. An output
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

Nested rendering and keyboard routing are implemented as described below. Next
implement pointer routing, popup lifecycle and the direct DRM/seat
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
creation. `Nested::pump` handles parent resize/close; close is terminal. The
keyboard component below adds `pump_keyboard` for input-enabled displays.

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

## Keyboard component (2026-09-07)

`Server::bind_keyboard(runtime, name, XkbConfig)` creates a keyboard-only
`alo-seat`. A keymap compilation failure drops the new display and its private
socket, rather than leaving a session that cannot accept keys. The layout is
explicitly supplied by trusted session configuration; only the developer fixtures
choose `us`. Keyboard settings UI and persistent preferences remain session work.
Clients receive an fd-backed XKB keymap and repeat rate/delay of 25 Hz/600 ms.
Repeat belongs to clients; Winit's repeat events and duplicate transitions do
not generate additional presses. Smithay and XKB remain pinned and unmodified.

`keyboard_focus` accepts only this display's live mapped toplevel roots. Invalid,
foreign and stale targets refuse before changing focus. `keyboard_key` accepts
bounded Linux evdev codes, converts to XKB once and routes only with focus.
There is no unfocused key buffer. Changing or clearing focus releases held keys
through XKB before leave, at the last accepted input timestamp, so depressed
modifiers cannot stick or cross into a new application's enter array. Lock
modifiers retain the seat's normal XKB semantics. Dispatch clears focus after
unmap, role destruction and disconnect; key delivery also rechecks eligibility.

`Nested::pump_keyboard(&mut server)` processes parent activation and key events
in order. Parent focus loss/close clears focus; while active, the first mapped
root receives focus, matching the renderer's current front-to-back creation
order. This is a minimal backend policy pending delivery step 3's window
activation, shortcuts and stacking. Winit supplies XKB-offset codes; the bridge
converts to the common evdev API without double-offsetting. Render-only callers
can still use `Server::bind` and `Nested::pump`. Do not mix render-only pumping
into a keyboard-driven session. Pointer capability is deliberately absent.

These internal Rust interfaces expose no IPC, adapter verb, agent input injection
or context reader. ADR 0001 and the agent/adapters contracts remain unchanged;
surface ownership never grants an agent authority. ADR 0002's native shell
boundary is preserved. Startup errors remain diagnostics that the future session
entry must translate. Splitting keyboard routing and test wire observations into
their own files keeps surface lifecycle and input state independently reviewable.

### Checks actually run

WSLg socket and native metadata were rechecked: xkbcommon 1.13.1, Wayland 1.24.0,
EGL 1.5. No packages, shared kernel resources or other checkouts were changed.
Linux uses `/root/alo-os-target` and explicit Cargo PATH for this checkout.

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
git diff --check
```

All passed. Linux: **20 tests**, no ignored tests (one unit, sixteen real-client
tests, three socket tests). Five new keyboard tests cover keymap/repeat,
enter/key/modifiers/leave, two-client isolation, duplicate presses/unmatched
releases, focus clear/reentry, unmap/disconnect/destruction, invalid key codes,
missing keyboard, stale/foreign targets, and invalid keymap socket cleanup.
Windows compiles with Linux code excluded and runs zero Linux tests. Two clippy
passes found development issues: manual saturating arithmetic, then test slicing
and `expect` usage. Corrected the code and tests without suppressing lints;
final affected-target clippy passed. No test failures or gate relaxation.

Additional integration commands, each exit 0:

```powershell
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle input::keyboard_keymap_focus_and_keys_are_isolated_between_clients -- --exact --nocapture
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check
```

Local `.git/alo-keyboard-wire.log` records 34206-byte keymap fd transfer, repeat
25/600, evdev 42/30 presses, releases before leave, zero depressed modifiers and
empty held-key arrays on enter to the next client. Keys are injected at the
trusted backend API in this test; these are actual Unix-socket protocol events,
not a physical keyboard measurement. `.git/alo-keyboard-nested.log` records
320x200 GLES root/child submission (callbacks [367,367] in this run), keyboard-only
seat/keymap/repeat assertions, unmap/remap, refusal and disconnect. Existing Mesa
diagnostics remain. The WSLg parent has its own pointer; its wire events do not
mean our nested server advertises pointer capability.

Still owed: actual parent activation cycling while holding physical keys, direct
seat/display execution, physical layout/modifier/repeat acceptance and certified
machine records. The current fixture does not type through the parent window.
Pointer routing, popups, production scheduling, shortcuts/window management and
delivery steps 3-8 remain unfinished. Item 33 stays unchecked. The supervisor
must independently run all full Windows/Linux/BPF publication gates; this worker
has not staged, committed or pushed.

## Pointer seat core (2026-09-07)

`Server::enable_pointer` adds optional pointer capability to the existing keyboard
seat. `pointer_motion` uses Smithay 0.7.0's desktop surface-tree hit testing,
matching renderer root order and origin at scale one, including subsurface offsets
and committed input regions. The upstream `desktop` feature adds no dependency
or lockfile change. `pointer_button` accepts BTN_LEFT through BTN_TASK and
suppresses duplicate/unmatched transitions. Smithay's implicit grab retains the
pressed surface across motion; `pointer_axis` forwards scroll frames. Coordinates
and scroll values must fit finite Wayland fixed-point values. Invalid inputs
refuse without changing focus; unfocused buttons/scroll are dropped.

`pointer_leave` cancels held buttons and clears focus; dispatch also checks that
the focused surface and its ancestors remain buffered under a live mapped root.
Unmap, child destruction and disconnect cannot transfer a held button to another
application. Cancellation first removes Smithay's pending focus so releasing a
grab cannot restore another recipient. A subsequent motion establishes new focus.
These are trusted backend Rust APIs, not agent verbs or context access. The
accepted ADR 0001/application contracts remain unchanged; ADR 0002's native shell
uses upstream protocol behavior rather than patching the engine.

This completes the selected core, not pointer integration or item 33. The nested
backend still enables only keyboard input. Cursor requests are not rendered yet.
Next: wire parent motion/button/axis/leave and focus loss to these APIs, implement
cursor presentation and verify them through WSLg. Root placement/window activation,
popups, direct input/display, production scheduling and delivery steps 3-8 remain.
Physical mouse/keyboard/display checks on certified machines remain owed; WSLg
and socket injection cannot certify hardware.

### Checks actually run for the pointer core

Ubuntu WSLg socket and pkg-config prerequisites rechecked: xkbcommon 1.13.1,
Wayland server 1.24.0, EGL 1.5. No package installation or shared kernel changes.
This checkout retained its Linux target directory `/root/alo-os-target`.

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle pointer:: -- --nocapture
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check
git diff --check
```

All final commands passed. Linux: **24 tests**, no ignored tests (one unit,
20 real-client tests, three socket tests). Windows passes with zero Linux tests,
not protocol evidence. Initial compilation discovered that the new test file
was also being discovered as a standalone integration target; moving it under
`tests/pointer/mod.rs` corrected that. The shared `empty_input` fixture method
was unused in the graphical example; the example now verifies rendering with
an empty input region. No test assertion failed and no lint was suppressed.

Additional evidence: `.git/alo-pointer-wire.log` records four passing pointer
tests with capability mask 3, child-local enter (2,3), button 272 transitions,
finger source, horizontal -2/vertical 5 scroll, axis stops and frame boundaries.
`.git/alo-pointer-nested.log` records the empty-region request followed by actual
GLES root/child callbacks, unmap/remap, refusal and disconnect (exit 0). Its parent
pointer events belong to WSLg; they do not demonstrate our nested pointer bridge.
Independent supervisor full Windows/Linux/BPF publication gates remain owed.

## Nested pointer bridge (2026-09-07)

`Nested::pump_seat` now routes keyboard plus parent absolute pointer motion,
evdev buttons and scroll through `Server::nested_pointer`. The backend router
accepts input only while active, cancels held buttons on deactivation/close, and
requires fresh motion after reactivation. The keyboard-only API stays available.
The WSLg fixture enables both capabilities and uses the combined event pump.

Reasons: physical parent pixels match our physical-sized framebuffer at nested
logical scale one, so no second host-scale multiplication is applied. Wheel
v120 units use an initial 15-pixel step per 120 units while retaining the v120
protocol value; pixel deltas retain their upstream sign and amount. Nonfinite
or out-of-range deltas refuse before integer conversion. Translation stays
separate from graphics ownership. ADR 0002's pinned native engine remains
unpatched; ADR 0001 and daemon/application contracts gain no agent-facing input
or context API. All new public Rust items have documentation.

Limitation found in pinned source: Smithay 0.7 drops CursorEntered/CursorLeft.
Focus loss is observable, pointer leave without focus loss is not. See quirks.md.
The selected bridge is complete; parent-leave notification support and client
cursor rendering are the next backend component, not a completed compositor.
Physical parent focus/input cycles, direct display/input, popups, production
scheduling and certified-machine acceptance are still owed. No WSLg certification.

### Checks actually run

Ubuntu WSLg socket verified; pkg-config reports xkbcommon 1.13.1, Wayland server
1.24.0, EGL 1.5. No dependency installation or shared kernel changes needed.
The checkout uses /root/alo-os-target, separate from the Claude checkout.

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle pointer::nested_bridge -- --nocapture
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check
git diff --check
```

Final checks passed: Linux 28 tests (3 unit, 22 client, 3 socket), no ignored
tests; Windows zero Linux tests, intentionally not protocol evidence. Initial
compilation found a protocol axis variant spelling; first clippy found missing
private documentation and two test unwraps. These were corrected without lint
suppression or changing test assertions; all executed assertions passed.

Additional evidence: local .git/alo-nested-pointer-wire.log records two bridge
tests passing, capability 3, button 272 press, axis_value120(0,120), synthetic
release and leave, then fresh enter at (4,5). The test supplies trusted normalized
backend events over the real client protocol; it does not inject physical input.
.git/alo-nested-pointer-graphics.log records capability 3, root/child callbacks
[402,402], six submitted client surfaces and unmap/remap/refusal/disconnect,
exit 0. This is graphical integration regression, not actual parent pointer
movement evidence. Full independent Windows/Linux/BPF publication gates belong
to the supervisor and have not been run by this worker.

## Client cursor presentation (2026-09-07)

`Server::cursor` snapshots the cursor accepted by Smithay's focus/serial/role
checks. `FrameTarget::submit_scene` is additive: existing implementations compile
but explicitly refuse a non-default cursor until they implement presentation.
`Nested` draws cursor surface trees first in front-to-back order, at pointer
position minus hotspot, hides the host cursor for surface/hidden requests, and
restores it for the default. A removed buffer draws nothing; a destroyed surface
or lost focus returns the default. Output membership and callbacks include visible
cursor surfaces only after successful submission, using the existing coordinator.

Reasons: reuse pinned Smithay protocol validation and GLES surface-tree import,
without patching an engine or adding dependencies (ADR 0002). Hotspot subtraction
and tree offsets use floating point, clipping offscreen geometry before integer
conversion so full-range i32 hotspots cannot overflow renderer rectangles. No
agent verb, application adapter, daemon IPC or context reader changes (ADR 0001
and `docs/contracts/app-adapters.md`). Public backend additions have rustdoc;
default cursor styling is supplied by the parent in this nested backend.

### Checks actually run

Ubuntu `/run/user/0/wayland-0` socket checked; pkg-config xkbcommon 1.13.1,
Wayland server 1.24.0 and EGL 1.5. No prerequisite installation or shared-kernel
changes. This checkout alone uses `/root/alo-os-target`.

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy -p alo-shell --all-targets --locked -- -D warnings
cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo fmt --all --check
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo clippy -p alo-shell --all-targets --locked -- -D warnings
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo test -p alo-shell --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target RUSTDOCFLAGS=-Dwarnings cargo doc -p alo-shell --no-deps --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target cargo build -p alo-shell --example nested_check --locked
wsl -d Ubuntu -- env PATH=/root/.cargo/bin:/usr/bin:/bin CARGO_TARGET_DIR=/root/alo-os-target WAYLAND_DEBUG=1 timeout 30s cargo test -p alo-shell --locked --test client_lifecycle cursor:: -- --nocapture
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check --cursor
wsl -d Ubuntu -- env WAYLAND_DEBUG=1 timeout 30s /root/alo-os-target/debug/examples/nested_check
git diff --check
```

Final commands passed. Linux: 31 tests (3 unit, 25 client, 3 socket), no ignored
tests. Windows: zero Linux tests, intentionally not protocol evidence. Three
new cursor tests exercise hotspot/movement, hidden requests, stale serial and
unfocused-client isolation, role conflict disconnect, destruction/unmap/disconnect
cleanup, explicit leave, unsupported-target refusal and failed-swap callback
retention. Initial lint checks caught explicit test expect/panic calls; cursor
requests now use the existing shared test fixture's assertion convention, with
no lint configuration changes. All executed test assertions passed.

Additional local logs: `.git/alo-cursor-wire.log` records all three cursor tests
passing and the actual cursor requests/refusals. `.git/alo-cursor-graphics.log`
records exit 0, a real 16x16 ARGB cursor at scripted pointer (26,35), hotspot
(2,3), callback after GLES submission and output leave on cursor unmap. The fixture
first uses (i32::MIN,i32::MAX) and withholds that offscreen callback before moving
the same cursor onscreen. It counted 28 submitted client surfaces across frames.
`.git/alo-cursor-regression.log`: normal seat/rendering fixture exit 0, six submitted
surfaces, unmap/remap/refusal/disconnect. Mesa prints the already documented WSLg
driver-probing diagnostics before successful EGL rendering.

This completes client cursor presentation as a component, not item 33. The cursor
fixture uses trusted scripted motion rather than physical parent input. Actual
parent cursor visibility/input observation, parent-leave notification support,
popups, direct display/input and certified-machine records remain owed. Full
independent Windows/Linux/BPF publication gates belong to the supervisor. No WSLg
test certifies hardware; delivery steps 3-8 and every remaining v0.01 item remain.

## Native popup protocol lifetimes (2026-09-07)

The opt-in `Server::enable_popup_protocol` handshake exposes configured popup
buffer snapshots through `popup_surfaces`. Initial placement is relative to the
parent's XDG window geometry. Unmap, parent loss and unsupported repositioning
dismiss terminally. The default nested backend retains explicit dismissal until
popup rendering/input is implemented; no unrendered popup receives a callback.

Five new real-client tests pass, bringing focused Linux coverage to 36 tests.
Windows/Linux fmt, focused tests and all-target clippy, Linux rustdoc and example
build pass. The WSLg `nested_check --popups` fixture and `--cursor` regression
both exit 0. Exact commands, local wire/graphics logs, implementation decisions,
intermediate compile corrections and remaining evidence are recorded in
`updates/native-popup-protocol-lifetimes.md`.

Parent-leave notifications remain a backend task: Smithay 0.7 drops CursorLeft
and provides no raw event-loop hook. No engine patch or unsafe-code exception
was introduced. Next is popup presentation/hit testing; chains, grabs, output
constraints/repositioning, direct display, full supervisor gates and physical
acceptance remain owed. This component does not complete the compositor.

## Native popup presentation (2026-09-07)

`Nested` now consumes the opt-in popup snapshots. A shared scene builder places
each parent's newest popup trees above that parent, below preceding windows and
below the cursor. `Popup::location` aligns committed XDG window geometry origins,
clamping geometry to tree bounds. Bounds and offset arithmetic use floating point
before output clipping, including extreme client child positions. Pointer routing
uses the same scene and retains implicit drag recipients until release or cleanup.

The additive `FrameTarget::submit_popups` default refuses live popups on older
targets; output membership and callbacks change only after successful submission.
This is an internal trusted backend contract, not an agent/adapter capability.
No change to the application-adapter contract, engine patch or new dependency.
ADRs 0001/0002 remain the authority for the boundary and native implementation.

Four new real-client tests pass: geometry/stacking/regions/isolation; popup and
subsurface order and live unmap cleanup; drag cancellation across parent loss,
dismissal, role destruction and disconnect; failed/unsupported presentation and
callback/output retention. Linux shell total: 40 tests. WSLg submits popup buffers,
withholds offscreen callbacks and observes output leave on dismissal. Exact
commands, local logs and intermediate corrections are retained in
`updates/native-popup-presentation.md`.

Popup protocol remains explicit opt-in. Nested chains, explicit grabs,
repositioning/output constraints, parent-leave notification backend, direct
display/input, production session entry and physical acceptance remain unfinished.
The supervisor's complete Windows/Linux/BPF publication gates are still owed.
WSLg checks exercise development graphics, never certified hardware.
