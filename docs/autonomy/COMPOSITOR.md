# Native compositor development

Item 33 implements the v0.01 Smithay compositor in `docs/features.md`, following
ADR 0002. The first component is `crates/alo-shell`: a reusable Linux Wayland
server library. The second adds nested Wayland/GLES rendering, described below.
It is not yet a session executable or a usable desktop.

## Explicit window raising

`Server::raise_window` moves a live mapped toplevel to the front of the shared
rendering and hit-test order, carrying its popup subtree with it. Other roots
retain their relative order; repeated raising is idempotent. Invalid foreign,
stale, unmapped and non-toplevel targets refuse without changing order. The next
successful frame presents the change. New roles still initially follow existing
roles; mapping alone does not implicitly raise them.

An existing pointer focus is re-hit at its last validated location/time, through
the ordinary routing path so held-button and popup grabs retain their authority.
A cleared pointer focus is not re-entered by raising. Keyboard focus and XDG
activation are separate policy; the nested backend still selects the front root
on its next keyboard event. This trusted native API is not an agent endpoint,
activation protocol or native switching control. WindowRaiseError distinguishes
invalid targets from a pointer refresh failure after stacking changed.
Real socket and GLES evidence: `updates/native-window-raising.md`.

## Cooperative window close requests

`Server::request_window_close` is trusted native shell plumbing for delivery
step 3. It resolves a live mapped toplevel owned by this display and queues one
XDG close event per explicit call. Normal dispatch flushes the event. Success
means queued, not received or closed: the application can ignore it or show its
own save dialog. No resource destruction, focus change, retry or process kill
follows. Unmapped, destroyed, disconnected, foreign and non-toplevel targets
return `WindowCloseError::Unmapped`; a fresh configured remap is eligible again.

This API exposes no agent endpoint or window enumeration capability. The
application-verbs contract's approval/grant boundary remains unchanged, including
its requirement that close asks rather than discards unsaved work. Native close
controls, configurable shortcut dispatch and the application adapter remain
separate unfinished components. The real socket tests exercise wire delivery,
isolation, continued keyboard input/render eligibility and voluntary teardown.
WSLg regression is graphics development evidence only. Report:
`updates/native-window-close-requests.md`.

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
is advertised after `Server::render` first successfully submits a frame with a
positive framebuffer size and valid output metadata.
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

Nested's single `alo-nested` output uses actual framebuffer dimensions, scale 1
and unknown physical dimensions/refresh. Other targets supply their own validated
metadata (see Truthful output metadata below). Successful resize replaces the
old mode without creating another global. Each configured toplevel tree is imported in existing
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


## Native popup chains (2026-09-07)

Opt-in popups now accept mapped popup parents. Shared scene placement accumulates
XDG geometry through ancestors and orders descendants above their parents.
Parent-before-child storage and iterative traversal avoid recursive client depth;
terminal dismissal sends done child-first and cancels descendant pointer grabs.
The trusted FrameTarget contract documents accumulated Popup::location offsets.

Three new real-client tests pass (43 Linux shell tests); WSLg popup/cursor fixture
submits 67 surfaces across frames, including a nested child and both callbacks/
output entries and dismissal leaves. Windows/Linux fmt and focused clippy/tests,
Linux rustdoc and the build pass. Exact commands, intermediate corrections and
local wire/GLES logs: `updates/native-popup-chains.md`.

Next: explicit popup grabs with serial/seat validation and outside-click dismissal.
Repositioning/output constraints, parent leave, direct display/input, production
session, supervisor full publication gates and physical acceptance remain owed.
No WSLg result certifies hardware.

## Pointer-triggered popup grabs (2026-09-07)

Opt-in popups accept the display seat's active pointer press on the parent tree.
The grab ends the initiating implicit drag and consumes its later physical release;
submenus may inherit the current chain's original serial. Keyboard focus follows
the topmost mapped grabbing popup and returns to the surviving parent. Pointer
events remain within the owner's client; outside presses dismiss child-first and
are consumed. Stationary clicks re-hit the scene, including a second button pressed
outside while an implicit drag retains its old target. Unmap, destruction,
disconnect and backend focus loss cancel held input without redirecting releases.

Six new real-client tests pass (49 Linux shell tests). The WSLg fixture now has
`nested_check --grabs`: a real client requests a grab after scripted trusted seat
input, submits its SHM popup through GLES, receives keyboard input, and observes
outside-click dismissal, output leave and parent-focus restoration. Existing
`--popups --cursor` regression also passes. Exact commands and local logs are in
`updates/native-popup-pointer-grabs.md`; focused Windows/Linux fmt/clippy/tests
and warnings-denied Linux rustdoc pass.

This is a pointer-triggered component. Keyboard/release-triggered initiation,
repositioning/output constraints, parent-leave backend support, direct display/input
and production session integration remain. The scripted WSLg check is submission
and protocol evidence, not pixel readback or physical input/display certification.
Supervisor full Windows/Linux/BPF publication gates and hardware acceptance remain owed.

## Keyboard-triggered popup grabs (2026-09-07)

A keyboard-only seat can initiate a popup from its latest delivered held key
press on the parent. Another accepted key event, focus/lifetime loss or successful
grab invalidates that serial; synthetic releases cannot initiate. Active submenu
chains retain their original serial. Pending-grab destruction cannot replay it.
This bounded policy avoids historical serial storage and arbitrary timeout rules.
Three real-client tests pass (52 Linux shell tests). Focused Windows/Linux fmt,
clippy/tests, Linux rustdoc and example build pass. WSLg `--keyboard-grabs` submits
a real popup and verifies callbacks/output membership, keys, dismissal, restored
focus and disconnect; `--grabs` pointer regression also passes. Exact commands and
local wire/graphics evidence: `updates/native-popup-keyboard-grabs.md`.

Release-triggered initiation remains the next component. Repositioning/output
constraints, parent leave, direct display/input, session integration, full
supervisor gates and physical acceptance remain owed. No adapter/agent contract
changes; WSLg scripted submission is not physical input/display certification.


## Keyboard-release popup initiation (2026-09-07)

The keyboard serial policy now retains the latest delivered real key event,
including a matched release. This supersedes the held-press-only policy above.
A newer accepted key event, focus/lifetime loss or successful root initiation
invalidates it. Invalid/duplicate input creates no event; synthetic cleanup
releases never create authority. Submenus retain the active chain serial.
The bounded single-event policy needs neither historical serial storage nor
an arbitrary timeout. This is trusted backend routing, not an agent API.

Three new real-client tests pass, including pending-grab replay, foreign-parent,
unmap/remap, superseded event and synthetic-release refusal. Linux shell total:
55 passing tests. `nested_check --keyboard-release-grabs` submits a real SHM popup
through GLES and checks focus, outside-click dismissal, output leave and cleanup.
The wire trace shows key press serial 3, release serial 4, then popup grab serial 4.
Key-press and pointer-grab graphics regressions also pass. Full commands and local
log paths: `updates/native-popup-keyboard-release-initiation.md`.

Pointer-release initiation is implemented below. Repositioning/output
constraints, parent-leave notifications, direct display/input and production
session integration remain owed. Scripted WSLg submission is not pixel readback,
actual parent input or physical laptop/GPU workstation certification. Supervisor
full independent publication gates have not run for this change.

## Pointer-release popup initiation (2026-09-07)

The latest matched real button release authorizes one root popup under its
recipient's toplevel or popup tree. A subsurface can initiate its parent's menu;
the exact recipient must retain pointer focus. New accepted button events, focus
or lifetime loss and successful initiation invalidate the stored release.
Invalid/duplicate events create no serial; synthetic cancellation never grants
authority. Active submenus inherit the chain serial as before. Pointer presses
retain their existing active implicit-grab policy; keyboard authority is unchanged.

At the last held-button release the backend re-hits the scene to validate the
recipient: Smithay ends the drag but retains pointer focus until the next motion.
This prevents a release outside the old recipient from authorizing a menu, even
if a later motion returns there. While another button remains held, its implicit
drag continues to own delivery. This is local compositor policy, with no engine
patch, arbitrary timeout or historical serial collection.

Three new real-client tests cover subsurface/submenu success and consumption,
pending-grab replay, nine refusal scenarios and independent-client survival.
Linux shell total: 58 tests pass. WSLg `nested_check --pointer-release-grabs`
submits a real SHM popup and checks callback/output enter, keyboard routing,
outside dismissal, output leave and disconnect. Pointer press and keyboard release
regressions pass. Exact commands, initial discovered failure and local wire logs:
`updates/native-popup-pointer-release-initiation.md`.

Repositioning/output constraints, parent-leave notifications, direct display/input
and production session remain unfinished. Scripted WSLg is submission/protocol
evidence, not pixel readback, actual parent input or physical certification.
Supervisor full Windows/Linux/BPF publication gates remain owed.


## Explicit popup repositioning (2026-09-07)

`xdg_popup.reposition` now validates the same bounded positioner operands as
initial placement. Smithay sends token, popup configure and surface configure;
its acknowledged state becomes scene geometry only on `wl_surface.commit`.
Multiple outstanding requests can be acknowledged/committed individually or
superseded; stale acknowledgement refuses that client. Descendants inherit the
committed parent origin in the existing shared rendering/hit-testing traversal.
Unsafe requests still dismiss child-first; dismissed roles cannot reposition.
No extra configure history, upstream patch or agent-facing authority is added.

Three new Unix-socket tests and WSLg popup/child GLES submission pass; failed
submission retains callbacks and descendant input uses moved geometry. Exact
commands, lint corrections and limits are in
`updates/native-popup-repositioning.md`. Output constraints and automatic reactive
placement remain unfinished, along with parent leave, direct display/input and
production session wiring. WSLg evidence is scripted protocol/submission, not
pixel readback, actual parent mouse observation or physical certification.

## Popup output constraints (2026-09-07)

Initial and explicit popup placement now uses the last positive target extent
supplied to Server::render. Client flip/slide/resize flags are applied by pinned
Smithay after translating output bounds through committed parent window geometry,
including nested popup origins. The separate popup_placement module bounds every
operand before upstream i32 arithmetic. Extreme origins/targets refuse safely.
Requests and acknowledgements alone still do not move the scene. Failed frames
retain callbacks; empty targets do not overwrite the last valid output extent.
No output means unconstrained protocol-fixture placement. Impossible fits or
absent adjustment permissions can leave a popup clipped.

Four new socket tests, final 65-test Linux shell suite and WSLg combined
popup/cursor submission pass. The real GLES fixture submits constrained buffers
at (304,13), then (0,0), with callback/output membership and unmap cleanup. Exact
commands, evidence and reconciliation are in
`updates/native-popup-output-constraints.md`.

Automatic reactive placement on output/parent changes remains the next component.
Parent leave, direct display/input and production session remain unfinished;
scripted WSLg is not pixel readback, actual parent input or hardware acceptance.
Supervisor independent full publication gates and physical laptop/GPU workstation
records remain owed. No compositor or release box is ticked.

## Reactive popup placement (2026-09-07)

Mapped reactive popups now reconstrain after a valid output or committed ancestor
geometry change. The scheduler in `popup_reactive.rs` uses the existing bounded
placement helper and Smithay's latest server state to avoid duplicate configures
while acknowledgements are outstanding. Both committed and latest requested
positioners must permit reactivity. Initial buffers and newly granted permission
wait for their acknowledged commit; explicit withdrawal is honored immediately.
Nested placement uses committed ancestors. Rendering and input remain atomic on
the popup's acknowledged commit, and unsafe placement dismisses child-first.

Five new socket tests pass (70 Linux shell tests total). Final WSLg combined
popup/cursor fixture submits reactive SHM buffers at (304,13), then (304,15) after
committed parent window-geometry change, with callbacks and output leave. Final
affected Windows/Linux checks pass. Exact commands, local logs, initial lint and
fixture-count corrections: `updates/native-popup-reactive-placement.md`.

Parent leave, direct display/input and production session are unfinished. No
parent-size prediction is implemented; committed parent geometry is authoritative.
Scripted WSLg does not prove pixel readback, actual parent input or physical
acceptance. Supervisor full publication gates and hardware records remain owed.

## Direct-display resource discovery (2026-09-07)

`discover_output` borrows the session's descriptor and queries DRM resources,
connectors (without force-probing) and encoders through pinned drm-rs 0.14.1,
the version already used by Smithay. `drm_inventory.rs` owns ioctl transport;
`direct_output.rs` owns single-output policy. No unsafe code, master-acquisition
ioctl, client capability update or modeset is added. Internal eDP/LVDS/DSI panels
win if usable, followed by stable connector ID. Writeback and unknown/disconnected
ports refuse; advertised preferred progressive timings win, falling back to the
first supported advertised mode. The smallest compatible CRTC ID wins, without
claiming that it is free. Exact mode timings survive selection.

The result is a snapshot, not a reservation. Session ownership, hotplug
revalidation and atomic test/commit must precede scanout. Interlaced, doublescan,
stereo, multiscanned and malformed modes are outside this initial backend's
supported selection; alternate valid modes remain eligible. Query errors preserve
the kernel reason and refuse the entire discovery instead of partial selection.

Seven new tests pass, including real `/dev/null` ioctl ENOTTY and descriptor
survival; the shell suite passes 77 Linux tests. The developer executable
`direct_output_check /dev/dri/cardN` diagnoses a session-owned development card.
Opening a primary device may implicitly make its first opener DRM master; the
library itself only borrows an existing fd. No DRM node exists on this WSL host,
so only absent-card and non-DRM executable refusals were measured. Do not infer
successful resource queries, scanout or hardware acceptance from those checks.
WSLg popup/cursor regression submitted 115 surfaces and exited 0. Exact commands,
logs, report reconciliation and remaining acceptance:
`updates/direct-display-resource-discovery.md`.

Next is session-mediated device ownership/pause/resume, followed by atomic
modesetting, page flips and direct input. Parent-leave backend support and
production entry remain unfinished. Compositor/release boxes remain unchecked;
full supervisor gates and physical laptop/GPU workstation records are still owed.

## Direct-display session lifetime (2026-09-07)

`DirectSession` now owns a pinned Smithay libseat notifier and one lazily acquired
device. Poll drains the seat-to-channel handoff; a pause retires the descriptor,
activation allows fresh acquisition, and open/close/reported dispatch failures
require a new session. Explicit shutdown reports device-close errors before the
notifier disappears. The library supplies scoped borrows only; callers must not
retain descriptors or scanout resources across polls. No raw-open fallback or VT
switch is provided, and no agent or adapter contract changes.

Nine new tests pass, including real descriptor EOF and calloop forwarding; full
Linux shell suite passes 86 tests. An explicitly unavailable seatd socket refuses
with ENOENT/exit 1. WSLg popup/cursor regression submits 115 surfaces, exit 0.
Windows/Linux fmt and affected clippy/tests, Linux rustdoc and example build pass.
Commands, local logs and exact evidence: `updates/direct-display-session-lifetime.md`.

This is device discovery lifetime, not running scanout. See `../quirks.md` for
upstream disable-before-notify and internal panic limitations. Next: atomic DRM
property/capability discovery and test-only validation, then renderer lifecycle,
scanout/page flips and direct input. Successful device acquisition/reacquisition,
real display/input, parent leave, session entry and physical acceptance remain.
Supervisor full publication gates remain owed; compositor/release boxes unchanged.

## Atomic display property discovery (2026-09-07)

`discover_atomic_output` is the next borrowed-seat-descriptor discovery layer:
atomic/universal-plane negotiation, primary-plane selection and validation of
standard connector/CRTC/plane property schemas. It returns handles and advertised
formats without allocating buffers or changing scanout. Use within
`DirectSession::with_device`; capabilities persist on the open file description,
and snapshots must be discarded on pause/hotplug. Kernel TEST_ONLY remains next.

Eight new tests pass (94 Linux shell tests total); Windows/Linux affected clippy,
tests and fmt plus Linux rustdoc/examples pass. Real `/dev/null` capability ioctl
refuses ENOTTY; WSL card is absent (ENOENT). WSLg popup/cursor regression submits
115 client surfaces. Exact commands, logs, decisions, raw-parser limits and owed
successful DRM/hardware evidence: `updates/atomic-display-property-discovery.md`.
Full supervisor publication gates remain pending. No production compositor or
hardware checkbox is completed by this component.

## Direct-display resource ownership (2026-09-07)

`DisplayResources::allocate` prepares one unbound XRGB8888 dumb framebuffer and
an exact advertised mode blob inside the session descriptor's lifetime. Explicit
`release` preserves all cleanup failures; partial allocation unwinds in reverse
order. These handles are for subsequent TEST_ONLY use, never active scanout.
`atomic_output_check /dev/dri/cardN --allocate` exercises discovery, allocation
and explicit cleanup in a developer login. Production must use `DirectSession`.

Eight new unit cases plus a compile-fail lifetime doctest pass; Linux shell total
is 102 tests plus one doctest. Windows/Linux affected fmt/clippy/tests, Linux
rustdoc/examples and WSLg 115-surface regression pass. Real CREATE_DUMB on a
non-DRM fd refuses ENOTTY without closing it; absent card and invalid diagnostic
arguments refuse. Exact commands, logs and upstream allocation-wrapper limits:
`updates/direct-display-resource-ownership.md` and `../quirks.md`.

Next is full-mode atomic TEST_ONLY construction/refusal/cleanup. Successful DRM
allocation and destruction, mapping/drawing, scanout/page flips, renderer pause
ordering, direct input, production entry and all physical acceptance remain.
No release/compositor checkbox changes; supervisor full publication gates pending.

## Atomic display configuration validation (2026-09-07)

`DisplayResources::allocate` now freezes a full-mode atomic plan before allocation,
refusing incomplete/aliased required property maps and colliding object IDs.
`test_and_release` consumes the candidate, submits TEST_ONLY | ALLOW_MODESET on
its allocation descriptor and explicitly releases every resource on both results.
Kernel refusal and every cleanup error survive. No active commit, rendering,
reservation or retry. Caller must supply fresh discovery on the same session fd;
public snapshot metadata is trusted and kernel compatibility is tested at ioctl.

`atomic_output_check /dev/dri/cardN --test-only` provides the developer diagnostic;
production remains scoped to DirectSession. Six new tests pass; Linux shell total
109 checks including doctest. Windows/Linux affected fmt/clippy/tests, Linux docs/
examples, real atomic ENOTTY and WSLg 115-surface regression pass. Exact commands:
`updates/atomic-display-configuration-validation.md`. Successful DRM validation
needs a DRM-equipped login/VM. Scanout ownership/page flips, renderer pause ordering,
direct input, production entry, parent-leave and all physical acceptance remain.
Supervisor full publication gates still owed; compositor/release stay unchecked.

## Scanout-buffer initialization (2026-09-07)

`DisplayResources::allocate` now initializes every byte of the unbound XRGB8888
allocation before framebuffer registration. `scanout_buffer.rs` validates the
layout and mapped capacity, clears visible pixels, stride padding and allocation
tail, and scopes access to a callback. `resource_device.rs` supplies the pinned
drm-rs shared writable mapping; its drop unmaps before registration or unwind.
Allocation therefore requires an RDWR descriptor, already supplied by DirectSession;
the developer diagnostic requests that access for --allocate/--test-only too.
Black is deterministic memory initialization, not a change to the shell palette.
Mapping failure retains errno and any buffer-destruction failure. The upstream
munmap panic and pre-slice hidden-capacity assumptions are recorded in quirks;
this is not complete production fault containment. Agent/adapter contracts unchanged.

Five new tests cover padded bytes/tail, short mappings without partial writes,
invalid geometry/format/overflow, map refusal plus cleanup failure, and actual
MAP_DUMB ENOTTY with caller descriptor survival. Exact checks and integration
evidence: `updates/scanout-buffer-initialization.md`. Successful DRM memory access
and physical evidence remain unavailable here. Active commits/page-flip retirement,
renderer pause ordering, direct input and production entry remain unfinished;
the compositor and full v0.01 release remain unchecked.

## Blocking scanout ownership (2026-09-07)

`DisplayResources::activate` consumes the initialized candidate, submits TEST_ONLY,
then enables the frozen full-mode request synchronously. `ActiveScanout::disable`
detaches the connector and primary plane, clears MODE_ID and deactivates the CRTC
in one blocking atomic request. Successful disable precedes all resource destruction.
Drop attempts the same retirement; use explicit disable to observe errors. Failed
disable quarantines resources, without destruction or retry, until the session
descriptor and its duplicates are retired. The descriptor borrow remains enforced.

This is a complete initial static scanout lifetime, using ALLOW_MODESET without
NONBLOCK or PAGE_FLIP_EVENT. Blocking retirement follows the kernel's
[atomic commit lifecycle](https://www.kernel.org/doc/html/latest/gpu/drm-kms-helpers.html).
It is an incremental component under ADR 0002, not a replacement for nonblocking
rendered frames. The caller must exclusively own the output and keep its session
active; the component does not restore a prior compositor's configuration or solve
libseat disable-before-notify. No new agent verbs or adapter contract changes.

`scanout_check /dev/dri/cardN --enable-and-disable` provides a session-mediated
developer diagnostic: enable initialized black, immediately disable, release and
shut down the seat descriptor on both success and failure, retaining both errors.
It has no direct-open fallback. The existing allocation/test-only diagnostic remains
available without active scanout. Do not run the active diagnostic on a live desktop
whose KMS output belongs to another compositor.

Six new unit tests and one lifetime doctest pass; 121 Linux shell checks in total.
Real enable/disable atomic ioctls refuse ENOTTY with fd survival; absent-seat and
usage refusals pass, and WSLg submits 125 client surfaces. Exact commands and limits:
`updates/blocking-scanout-ownership.md`. No successful DRM enable/disable or pixel
readback on this host. Next: nonblocking framebuffer submission and page-flip
matching/retirement; then rendering/session pause ordering, direct input and
production entry. Parent-leave and all physical acceptance remain outstanding.
Supervisor full publication gates have not run for this change.

## Cookie-preserving page-flip event reading (2026-09-07)

`read_display_events` reads one bounded batch from an already nonblocking borrowed
session descriptor. `DisplayEvent::FlipComplete` retains the full 64-bit user_data,
explicit nonzero CRTC, wrapping vblank sequence and kernel timestamp. The clock
depends on DRM_CAP_TIMESTAMP_MONOTONIC; the timestamp is not an inter-frame duration.
Unknown event types stay non-completions. Future flip tails are skipped using the
header length; short headers/payloads, impossible lengths, invalid microseconds and
legacy zero CRTC IDs refuse the entire batch. Parsing uses native-endian byte copies
and no unsafe casts, and accepts unaligned records.

The caller exclusively owns reads and flags. The reader never changes flags,
closes the descriptor, waits or retries. WouldBlock means readiness loss; EOF is
UnexpectedEof; kernel errno is preserved. The 4096-byte limit bounds each read;
unsupported larger events may cause a kernel error. A failed/malformed read cannot
authorize resource release. Synchronous disable or device quarantine remains the
retirement path until pending-commit matching is implemented.

This prerequisite follows the kernel's [event ABI and atomic flags](https://www.kernel.org/doc/html/v6.12/gpu/drm-uapi.html).
Pinned drm 0.14.1 drops page-flip user_data, and drm-ffi 0.9.1's atomic helper
submits zero user_data. Neither engine is patched: this additive Rust reader
preserves the ABI data needed by the next submission component. It does not itself
submit a flip, match stale/foreign events, or retire buffers. The next component
must preserve a unique commit cookie at submission and match cookie, CRTC and
session before releasing the old allocation. Sequence alone is insufficient.
ADRs 0001/0002 and agent/application-adapter contracts are unchanged.

Ten tests include pinned UAPI layout, multi-event/extended/unaligned batches,
truncation and corrupt metadata, actual descriptor reads, a full 128-event batch,
WouldBlock/EOF, blocking-fd refusal without consuming data and EBADF with fd survival.
These are synthetic event bytes through real Linux descriptors, not kernel DRM
completion evidence. Exact checks and limits: `updates/page-flip-event-reading.md`.
Nonblocking submission/retirement, rendered frames, session pause ordering, direct
input, production entry, parent-leave and libseat limits and physical acceptance
remain outstanding; compositor and release stay unchecked.

## Session-scoped page-flip completion gate (2026-09-07)

`FlipGate` now supplies the independent identity layer following the event reader.
It permits one pending submission, reserves a process-unique nonzero cookie before
calling transport, burns failed cookies and preserves submission errno. Allocation
never wraps into reused identities, including across gates for the same CRTC.
Only an exact pending cookie/CRTC match completes, once; sequence and timestamp
are not identities. Recreate the gate on session reacquisition and never reuse
inherited event streams after a process restart.

`read_completion` decodes an entire bounded batch before matching. Malformed reads,
EOF and WouldBlock cannot clear pending state. Other events are consumed, limiting
this API to one gate with exclusive commit/read ownership on a session descriptor.
The caller must supply real kernel events and truthful transport acceptance; this
is not an authority boundary against callers fabricating events.

This does not own or destroy buffers, change blocking ActiveScanout, or issue a
cookie-bearing ioctl. Dropping it never authorizes retirement. The future owner
must keep old/new resources alive until matching completion or synchronous disable,
keep the displayed buffer alive afterwards, and quarantine after disable failure.
Pinned drm-ffi's safe atomic helper sends zero user_data; its raw ioctl is unsafe
and workspace policy forbids it. Next investigate an unpatched safe upstream API;
no unsafe exemption or engine patch is introduced here (an engine patch needs ADR).

Eight happy/refusal tests pass, including concurrent allocation, exhaustion and real
Linux descriptor integration with synthetic ABI bytes. Full Linux shell: 139 checks;
affected fmt/clippy/tests, Linux rustdoc/examples and WSLg regression pass. Evidence:
`updates/session-scoped-flip-completion.md`. Successful DRM commits, buffer retirement,
rendered frames, pause ordering, direct input, production entry, parent-leave/libseat
limitations and physical acceptance remain open. Supervisor full gates remain owed.

## Unbound scanout frame upload (2026-09-07)

`XrgbFrame::new(size, stride, pixels)` validates a borrowed full-frame source:
nonzero dimensions, checked arithmetic, four-byte-aligned stride at least width * 4,
and exact stride * height byte length. Pixels are top-to-bottom DRM XRGB8888 B,G,R,X
bytes, with no conversion or scaling. `DisplayResources::with_frame` consumes an
unbound candidate and returns it only after successful mapping/copy/unmap. Upload
checks exact destination dimensions, XRGB format, pitch and mapping length, copies
visible rows across independent strides and clears all destination padding/tail.
Source padding is never copied. Short mappings refuse before writing any bytes.

On refusal, release all unbound resources once and preserve original plus cleanup
errors. No commit occurs until activation. ActiveScanout exposes no writable mapping;
the upload API cannot be used to modify its displayed memory. The existing upstream
munmap panic limitation still applies. This is a CPU upload boundary; the renderer
must supply the documented format/orientation. No GLES readback or direct FrameTarget
is connected yet. No agent/adapter contract or accepted ADR changes.

Six new tests include pixel/stride refusal and allocation/upload/scanout lifecycle
integration with the existing fault-injection transport. Full Linux shell 145 checks
and WSLg regression pass; neither proves a successful DRM upload. Exact evidence:
`updates/unbound-scanout-frame-upload.md`. Next: renderer conversion/readback with
pixel/orientation checks. Safe cookie-bearing atomic transport, pending retirement,
session pause ordering, direct input, production entry and physical acceptance
remain open. Pinned and inspected upstream drm-ffi atomic helpers leave user_data
zero; no unsafe exemption, dependency change or upstream patch is introduced.

## GLES scanout readback (2026-09-07)

`readback_xrgb(renderer, target, order)` now reads a complete bound GLES target
through Smithay ExportMem into owned `ScanoutPixels`. `frame()` borrows an
XrgbFrame suitable for the existing unbound `DisplayResources::with_frame` upload.
The caller finishes drawing first and owns the same renderer/context as the target.
Read-only GL mapping synchronizes before CPU access. No frame submission, callback,
active-buffer mapping or agent capture surface is added by this API.

The pinned engine computes export and map byte counts in signed i32; dimensions
are checked before export, with a maximum i32::MAX bytes. Export requests ABGR8888
(RGBA/UNSIGNED_BYTE); mapping extent, ABGR/XBGR metadata and the pinned GLES
inversion flag are checked before mapping. Exact packed byte length is required
before conversion. Graphics errors retain GlesError and allocation is fallible.
Source alpha is discarded after composition; B,G,R,0 is emitted with no colour
conversion. A caller explicitly names whether the first GL row is top or bottom:
Smithay Normal and Flipped180 require opposite orders, despite identical mapping
inversion metadata. No other rotation is inferred.

The new `readback_check` example verifies six exact pixels in a 3x2 real GLES
offscreen renderbuffer under WSLg in both orientations. Six new unit/integration
tests cover conversion, odd-width packing, metadata/length/overflow refusal and
conversion through padded upload and blocking scanout lifetime. That latter test
uses fake DRM transport. 151 Linux shell checks, affected Windows/Linux lint/fmt/
tests, Linux rustdoc/examples, invalid-EGL refusal and a 130-client-surface nested
regression pass. Exact commands and limits: `updates/gles-scanout-readback.md`.

Next: offscreen window/popup/cursor scene rendering into ScanoutPixels with real
client pixel checks and import/draw/readback refusal without premature callbacks.
Direct FrameTarget, cookie-bearing atomic transport, retirement, pause/input and
production entry remain unfinished. No DRM node exists on this host; successful
scanout and physical display/input remain unverified. Supervisor full publication
gates and all physical acceptance are still owed. No compositor/release tick.


## Offscreen scene preparation (2026-09-07)

`render_scanout` renders current window, popup and cursor trees through the same
fallible GLES painter used by `Nested`. Positive extent and signed export limits
are checked before allocating an ABGR offscreen renderbuffer. Normal rendering
pairs with explicit TopToBottom readback. Imports remain alive through readback;
the returned `PreparedScanout` owns immutable CPU pixels and drawn identities,
independent of the temporary framebuffer. It is not a `FrameTarget`: preparing or
dropping it sends no callback and publishes no output membership. The trusted
backend must upload/submit successfully before returning those identities, with
no intervening client dispatch. `RenderError::Readback` preserves export failures.
This adds no adapter/agent protocol or screenshot/context capability.

Default and hidden cursors draw no pixels; the direct backend still needs a default
arrow. Full allocation/readback each frame favors simple complete-frame ownership;
performance optimization and actual direct-display presentation are outstanding.
Two new tests and explicit `nested_check --offscreen` verify 1,056 exact scene
pixels, geometry/stacking/clipping, callback preservation on preparation and failed
transport, fixture-only successful submission, disconnect and truncated-SHM import
refusal. All 153 Linux shell checks and affected lint/doc/build checks pass; WSLg
nested regression submits 137 client surfaces. Exact commands and limits:
`updates/offscreen-scene-rendering.md`. No actual GPU context-loss/draw/readback
fault or successful DRM scanout is claimed. Next: connect preparation to consuming
upload and blocking scanout ownership with callback/cleanup tests. Direct target,
cookie transport, retirement, pause/input/session wiring, existing parent-leave/
libseat limits, supervisor full gates and physical acceptance remain open.

## Synchronous direct frame target (2026-09-07)

`DirectTarget` connects the shared GLES scene painter to blocking activation and
replacement, implementing every FrameTarget scene entry point. Construct with a
current borrowed renderer, borrowed session fd and fresh owned AtomicOutput inside
`DirectSession::with_device`; exclusively own an inactive output and keep the seat
active through consuming `disable`. It never dispatches clients. `Server::render`
publishes the exact committed surface identities through existing membership and
callback machinery; refusal preserves callbacks and previous membership.

After a successful replacement with retirement failure, the new identities still
complete callbacks. Inspect `retirement_error()` and stop: all subsequent submits
are latched off before painting or DRM I/O. `disable` preserves both that stored
failure and any shutdown error in `DirectShutdownError::errors`. A failed candidate
with cleanup errors also latches off; retain its returned RenderError::Scanout.
Disable failure quarantines resources and must be followed by retiring all device
descriptors. Drop is only an unreportable safety net, never normal session teardown.

This supplies synchronous frame submission, not a scheduling loop or physical
presentation timestamps. The safe blocking transport follows ADR 0002 without an
engine patch. Public Rust additions change no agent/application-adapter protocol.
Default cursor pixels, direct renderer creation, truthful output metadata (the
shared output still advertises nested placeholders), pause ordering, direct input
and session entry remain unfinished. Full-frame CPU readback/allocation is not a
performance claim. Async cookies, GPU context-loss faults and physical scanout
remain unverified. Five new tests include real callback/output wire assertions;
the WSLg public-target check verifies actual non-DRM refusal with real GLES scenes.
Exact commands, 167-check results and limits:
`updates/synchronous-direct-frame-target.md`. Supervisor full gates remain owed.

## Synchronous scene replacement (2026-09-07)

`ActiveScene::replace` consumes a prepared scene using the original descriptor,
mode, formats and frozen atomic route. It validates size before I/O, allocates and
uploads an unbound buffer, then performs blocking TEST_ONLY and enable. The old
allocation remains alive through both commits. After successful enable, its disable
is disarmed before reverse-order cleanup, so retirement cannot blank the new scene.
The safe transport uses neither NONBLOCK nor PAGE_FLIP_EVENT; no cookie is required.
This follows ADR 0002 without an engine patch or unsafe exception.

Outer `Err(ResourceError)` preserves the old active allocation and identities.
Outer `Ok(SceneReplacement)` means the new scene is active; `retirement_error`
separately reports any old-resource destruction failures. All releases are attempted
once. Any cleanup failure blocks further replacement: explicitly disable the current
scene, then retire the session device. Failed disable quarantines the active resources
without retrying on drop. A clean submission refusal permits a later replacement.

No method dispatches clients or sends callbacks. A caller may notify the returned
active identities only after successful submission, without intervening dispatch.
Continuous direct FrameTarget scheduling/callback integration is still unfinished.
Tests distinguish three framebuffer/blob allocations, verify ordering, pixels,
refusal/retry, cleanup/quarantine and real Wayland resource identities. Successful
DRM transport is fault-injected; WSLg regression is not physical scanout evidence.
Exact checks and limits: `updates/synchronous-scene-replacement.md`.

## Prepared scene activation (2026-09-07)

`PreparedScanout::activate` consumes the scene and validates its full physical
extent against the freshly discovered mode before any DRM allocation. The shared
allocation/upload/TEST_ONLY/blocking-enable transaction returns `ActiveScene`
only on success. This owner retains the drawn identities and immutable scanout
allocation together. `disable` orders detachment before destruction and preserves
cleanup errors; failed disable quarantines resources until descriptor retirement.
No client dispatch, output membership or callback completion occurs in this API.
The caller must own an inactive output exclusively, keep its session active,
and render/activate without intervening client dispatch. Replacing an already
active scene is not supported by this component.

This follows ADR 0002 using the existing safe, pinned DRM transport. A blocking
initial modeset needs no event cookie; it does not bypass the unresolved safe
cookie-bearing asynchronous transport. ADR 0001 and application-adapter surfaces
are unchanged. Native session entry must translate diagnostics. The new public
Rust surface is additive; `ScanoutPixels::size` exposes its validated extent.

Evidence and exact checks: `updates/prepared-scene-activation.md`. Tests use
fault-injected DRM for successful upload/enable/retirement. The WSLg fixture
uses actual GLES client scenes and `/dev/null` for kernel ioctl refusal; it is
not successful DRM scanout. Continuous direct FrameTarget, default cursor,
replacement/retirement, pause/input/session wiring, actual graphics context-loss
faults and physical display/input acceptance remain open. The pinned unmap-panic
limitation still applies. No compositor or release completion is claimed.

## Compositor-owned default cursor (2026-09-07)

`Cursor::Arrow { location }` is the positioned scale-one snapshot used whenever
an enabled pointer has no authorized client cursor, including empty desktop,
focus loss, destruction and disconnect. `Default` remains the legacy unpositioned
snapshot for a server without a pointer seat or an explicit caller: nested uses
its parent arrow, offscreen/direct draws nothing. Cursor-aware targets must
support Arrow explicitly; the default FrameTarget method refuses it. This
supersedes the earlier statements that the default cursor is always host-owned.

`default_cursor.rs` owns an original 12x18 black-outline/white-interior shape.
Its tip hotspot is (0,0), fractional coordinates floor, and each opaque pixel
clips to the output before integer conversion. NaN/infinity refuse; finite
offscreen positions are valid but invisible. The shared GLES painter draws the
arrow after every client tree; it adds no client identities or callbacks. Hidden
and client-surface requests suppress the arrow. The nested host cursor is hidden
only after successful submission. Black/white provides neutral contrast without
requiring the pending shell palette or a theme filesystem dependency.

Four new automated tests and all 171 Linux shell checks pass. The expanded WSLg
`nested_check --offscreen` compares every pixel to an independent row-span golden
shape across seven placements, checks layering and switching on real SHM scenes,
and retains existing callback and non-DRM/import refusal checks. The nested
popup/cursor regression also passes (115 client surfaces). Exact environment,
commands, initial findings and limits:
`updates/compositor-owned-default-cursor.md`. Physical parent-cursor observation,
direct display/input/session acceptance and scaling beyond the current scale-one
output model are not established. Truthful output metadata, pause/retirement,
direct session wiring and hardware records remain; no compositor/release tick.

## Truthful output metadata (2026-09-08)

FrameTarget::metadata describes a scale-one output. Nested supplies alo-nested;
legacy targets supply alo-virtual with unknown physical size/refresh. DirectTarget
uses alo-drm-<connector-id>, kernel connector dimensions and rounded millihertz
from supported progressive timings. Manufacturer/model and subpixel order remain
unknown; no EDID parser or hardware identity claim. Invalid strings, dimensions,
refresh, unsupported timings and conversion overflow refuse before submission.

First successful submission creates the global and freezes identity/physical
properties. Identity replacement requires successful explicit output retirement. Successful
resize replaces the mode on that global; failure preserves advertised metadata,
callbacks and membership. Reactive popup negotiation still uses a valid desired
extent even when submit fails; metadata refusal does not accept a new extent.

The owner authorized recovery after the worker's repeated-failure halt. The
offscreen client now completes the newly advertised output's bind roundtrip
before asserting all four surface enters; refusal stages additionally assert no
output global or metadata events. All original callback/pixel assertions remain.
Normal offscreen checks pass twice; nested metadata/popup/cursor regression passes
with 115 client surfaces. Invalid EGL still refuses with exit 1. Exact checks,
publication gate results and limitations: updates/truthful-output-metadata.md.
Neither these fixtures nor automated tests establish physical scanout or release
completion. Output retirement/pause and direct session/input wiring remain.

## Explicit output retirement (2026-09-08)

`Server::retire_output` checks the trusted target's identity, retires the backend,
then sends surface leaves and withdraws its global. Failed retirement preserves
advertised state and pending callbacks. Direct targets stop submitting after any
retirement attempt and never retry a failed disable. Success clears popup output
constraints and permits a fresh output identity in the same server.

Disabled globals retain inert binding data until display teardown so a queued
client bind does not become a disconnect. This retains one global per retired
lifetime; early reclamation is not implemented. Automatic seat pause, device
failure recovery and direct input/session wiring are not part of this operation.

The real Wayland/injected DRM fixture verifies success and failed disable,
callback retention, late binding, replacement identity and no repeated disable.
Connector IDs are independently configured in its exact transport oracle; alias
and wrong-target refusal checks remain mandatory. WSLg graphical regressions
pass. Publication evidence and limits: `updates/explicit-output-retirement.md`.
Physical retirement/scanout and certified-machine acceptance remain outstanding.
