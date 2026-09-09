# Native window control view

Status: additive trusted Rust view API, 2026-09-08. ADRs 0002 and 0010;
v0.01 window management. No protocol, agent capability or stored format changes.

`WindowControlLayout::new(viewport, origin, enabled, restoring)` constructs an
immutable scale-one view of minimise, maximise/restore and close, in that order.
Three 32 by 32 buttons have four-pixel transparent gaps (104 by 32 overall).
The output is positive and at most 1,000,000 per dimension; each origin coordinate
is between -1,000,000 and 1,000,000. Invalid geometry refuses before allocation
of drawing primitives or any client/graphics effects. Negative origins clip.
The host chooses placement; this API does not reserve a title bar or work area.

`controls()` retains all three buttons even when clipped or disabled. Each exposes
its `Action`, full bounds and supplied enabled state. `Action::said` is the sole
label path; the host must load `alo_shortcuts::shortcut_words` and present those
labels through its native text/accessibility surface. The maximize vocabulary
describes both maximize and restore. This component draws icons, not label text.

`hit(x, y)` uses half-open button and viewport bounds. Non-finite coordinates,
gaps and clipped-away areas have no hit. Disabled buttons still return a hit:
unavailability must not cause a click to reach a client underneath the control.
The hit is presentation information, never permission to execute an operation.

`paint(frame, scheme)` uses the existing alo-appearance tokens, with opaque
grounds and contrasting original glyphs. Disabled controls have both a changed
ground and a separate strike mark. Terracotta is never used. The restoring flag
selects overlapping rectangles rather than the maximize rectangle. All solids
are clipped to the same output used by hit testing; gaps remain untouched.
The caller supplies an active normal-transform scale-one frame matching the
captured viewport, draws the strip after clients and before the pointer, and
owns finish/submission. Paint errors propagate as `RenderError::Submission`;
a partially drawn frame must not be published after failure.

The component owns no server, surface, input seat, callbacks or window authority.
Availability is supplied presentation data and is not derived or cached policy.
Production composition, native label presentation, mapping-lifetime input tokens,
live operation revalidation, pointer press/release ownership and cancellation,
hover/pressed feedback and nested/direct routing remain integration work.
Existing ordinary client keyboard paths are unchanged. This API completes the
control view's layout and painter, not usable controls or window management.

Evidence: `window_controls_tests.rs` checks geometry, malformed input, exact
coverage and non-color disabled distinctions. The `window_controls_check` example
compares complete GLES readbacks to independent literal glyph masks and palette
values. This is a WSLg development fixture, not display submission, live control
interaction, physical input or certified hardware evidence.

## Live presentation snapshots

Additive trusted Rust API, 2026-09-09:
`Server::window_control_snapshot(surface, viewport, origin)` returns a
`WindowControlSnapshot` for the explicit visible mapped toplevel in this display.
`surface()` retains that exact protocol handle; `layout()` exposes the immutable
view, actions, hit geometry and derived availability. `layout().restoring()`
reports the glyph choice. Foreign, hidden, child, popup, unmapped and dead targets
return `WindowControlSnapshotError::Unmapped`; malformed layout returns `Layout`.
There is no focused-window or stacking fallback, and no input seat is required.

Minimise and close are enabled for every eligible root. Maximize/restore uses the
same side-effect-free layout planner as execution, including output availability,
competing move/resize/popup grabs, representable normal geometry and committed
restore limits. `maximize_refusal()` exposes the existing `WindowMaximizeError`
for diagnostics, not user-facing text. No new refusal text vocabulary is added.
The latest requested mode drives the glyph even before acknowledgment or commit:
maximized offers restore, while normal and tiled offer maximize. Restore can remain
available after output retirement; a new maximize requires a submitted output.
The supplied painting viewport does not establish that output availability.

Capture takes `&self`: it sends no configure or close, prunes no state, remembers
no geometry, changes no focus and consumes no input. Availability is true only at
capture time. Refresh for every frame and use the existing `Action::said` labels.
The snapshot has no dispatch method and is **not a mapping-lifetime token**:
unmap/remap can reuse the same protocol handle, and a retained snapshot remains
frozen. Pointer routing must bind presses to a separate mapping lifetime,
cancel stale ownership and revalidate operations at release. These rules prevent
presentation data from becoming cached authority; usable controls remain open.

Six real-client tests in `tests/window_controls/mod.rs` cover no-effect reads,
typing isolation, output/limits/geometry/busy refusal, pending mode changes and
target lifetime. The nested offscreen fixture paints twelve complete snapshot-derived
light/dark frames across five maximize/restore boundaries and a reused restored
boundary after minimization, alongside its unchanged client scene pixel assertions.
Exact executed results and limits belong to
`docs/autonomy/updates/live-window-control-snapshots.md`.

## Mapping-bound primary-button transactions

Additive trusted Rust API, 2026-09-09. `Server::press_window_control` takes an
explicit root, the currently painted viewport/origin and output-local position.
It captures live layout and a private visibility identity, never authorizing
from a retained presentation snapshot. A true result owns the primary press,
including disabled hits and duplicates. A duplicate cannot replace the target or
rearm a cancelled press. Gaps, clipped positions and non-finite coordinates return
false. Invalid roots/layouts return `WindowControlPressError::Snapshot`; existing
client held buttons, popup grabs or interactive move/resize return `Busy` on hits.
Refusals take no new ownership and do not disturb the existing client operation.

`cancel_window_control` disarms execution but retains ownership of the matching
release. `release_window_control` takes the current painted viewport/origin and
release position, removes ownership before validation/execution and returns:

- `Unowned`: no native press exists; ordinary routing may continue.
- `Cancelled`: native release consumed with no execution; never forward it.
- `Executed(Action)`: exactly one existing trusted operation was queued.
- `WindowControlReleaseError`: native release consumed; the existing live
  maximize/minimize/close refusal is preserved. Never forward or retry it.

Release requires the original visible mapping, unchanged layout, matching hit,
enabled state at press and (for maximize/restore) matching captured intent.
Changed output/limits or competing operations are revalidated by the existing
transaction methods. A disabled press stays disarmed when output becomes available.
Leaving the hit area at release, invalid coordinates, explicit cancellation,
unmap/remap, hide/reveal and death cancel without selecting another window.
A private `Arc` identity changes inside the actual unmap/visibility transition;
it cannot wrap like a counter or survive a hide/reveal within one dispatch.
Ordinary buffer updates preserve it. No native press emits client pointer events,
changes keyboard focus, raises a window or grants any agent authority.

This completes the trusted transaction component. The production nested/direct
host must still intercept primary events before client routing, supply current
paint geometry, consume owned events and invoke cancellation on pointer leave,
input reset, removed controls and seat/session loss. Other buttons and normal
keyboard input retain their existing routes. The component is callable without
an input seat for isolated fixtures; it is not automatic event interception or
an on-screen interaction claim. Motion feedback, native labels and composed
production controls remain subsequent components.

Real-client tests live in `tests/window_controls/input.rs`. The offscreen GLES
fixture additionally minimizes via a native transaction, verifies the full hidden
scene, restores and verifies the full preserved client scene. Executed checks and
limits: `docs/autonomy/updates/mapping-bound-window-control-transactions.md`.

## Motion and input-loss cancellation

`Server::window_control_motion(viewport, origin, position)` observes each native
gesture motion using the current painted geometry. It returns true while the
transaction owns motion, including after cancellation; the host must withhold
that event from client routing. False means no native owner. It never changes
client focus, ordinary seat position, keyboard input or window state. Primary
press/release interception remains a host responsibility.

Motion outside the original action, including gaps, clipping and non-finite
coordinates, permanently disarms the press. Changed geometry, mapping identity
or maximize/restore intent also disarms it. Returning to the original state or
receiving a duplicate press does not rearm it. Motion and release share one
identity/geometry/intent predicate; live operation refusal remains at release.
An unavailable operation still produces its existing typed release error.

`pointer_leave` now cancels native execution before clearing client input, even
if pointer capability is missing and cleanup returns an error. Nested/direct
pointer deactivation inherits this behavior. `clear_input` also cancels native
execution on whole-seat reset. Cancellation retains matching release ownership;
hosts must continue consuming that release and must explicitly cancel removed UI.
This component adds cancellation hooks, not automatic motion/button interception,
hover feedback, rendered labels or composed production controls.

Evidence and limits: `docs/autonomy/updates/native-control-motion-cancellation.md`.

## Combined native and client pointer routing

`Server::route_window_control_pointer` accepts a current `PaintedWindowControls`
view (explicit root, viewport and origin), output-local position, motion/button
event and monotonic timestamp. None denotes absent controls. The router consumes
owned primary transitions and motion, or calls the existing ordinary client
route itself. Its `Client`, `Consumed` and `Released` results must never be
forwarded a second time. Typed input/press/release errors also prohibit automatic
fallback or retry; a release error has already consumed native ownership.

Replacement or removal of the painted root disarms a held press on any routed
event, including duplicate presses and releases at identical geometry. Restoring
the original target cannot rearm it. Existing mapping, geometry, intent and live
operation validation remain in the transaction component. Disabled hits consume
their gestures. Client-held buttons and popup/interactive grabs refuse new native
acquisition; ordinary unmatched releases retain their existing semantics.

Other buttons keep ordinary routing. Unowned motion updates client seat position
and focus; owned motion updates neither. The host supplies every motion and uses
the existing keyboard/axis routes and leave/reset hooks. Cancel immediately if
presentation disappears between events. Continue routing the matching primary
release even after loss of input. This additive API is trusted shell input, never
an agent verb. It does not automatically select, paint or install controls in the
nested/direct backends. Native labels, feedback, cursor/presentation integration
and production composition remain work to complete usable controls.

Real-client routing tests: `tests/window_controls/routing.rs`. Graphical fixture:
`examples/support/window_minimize_check.rs`. Exact executed evidence and limits:
`docs/autonomy/updates/native-control-pointer-routing.md`.

## Live pointer feedback

Additive trusted Rust presentation API, 2026-09-09:
`Server::window_control_feedback(surface, viewport, origin, position)` captures
the existing live snapshot plus `WindowControl::feedback()`: `Idle`, `Hovered`
or `Pressed`. Position is output-local; None means no eligible pointer after
leave or input loss. The same clipped hit geometry, live availability planner
and private press mapping/geometry/intent predicate are reused. There is no
focus/stacking fallback. Invalid targets/layouts retain the snapshot refusals.

Without a native press, an enabled hit is hovered unless client buttons or a
popup/move/resize grab owns input. With a held press, only its original still
armed and available hit may appear pressed. All other controls remain idle;
cancelled/disabled/foreign/stale gestures cannot turn into hover or transfer
their pressed appearance. A disabled-to-enabled transition cannot arm a held
disabled press. Live unavailability removes pressed feedback without consuming
ownership or changing the eventual typed release refusal.

This method takes `&self`. Reads send no wire events, consume no input and do
not cancel, rearm or execute anything. Route every pointer event first, use the
existing leave/reset hooks and cancel removed presentation immediately. Reading
an outside position is not a substitute for routing that motion. Capture again
for every frame; old snapshots intentionally retain their old appearance.

The immutable view also has `with_pointer_feedback(position, pressed)` for
synthetic presentation and isolated painter fixtures. It clears previous
feedback, uses the shared clipped hit test and leaves disabled hits idle. It
owns no server/transaction and cannot grant execution authority; live hosts
should use the server capture API. Bounds, labels and default idle paint remain
unchanged. This extends the view rather than changing any agent/protocol format.

Hover uses the existing disabled-ground token with a contrasting one-pixel
border inset one pixel. An armed press swaps the ordinary ground/ink and uses
a two-pixel border at the same inset. Border thickness distinguishes the states
without hue alone; disabled strikes remain distinct and unchanged. All drawing
clips to the shared viewport and leaves gaps transparent. No terracotta, new
palette or hardcoded user-facing string is introduced (ADRs 0002/0010).

Two unit and three real-client feedback tests cover geometry, state, refusal,
frozen snapshots and ordinary input isolation. The standalone GLES fixture
checks 896 complete frames; the nested fixture additionally paints live hovered,
pressed, explicitly cancelled and out-and-back cancelled snapshots in both
schemes. These are offscreen development checks. Native externalized label
presentation and production nested/direct composition remain unfinished;
this component completes feedback, not usable controls. Exact checks and limits:
`docs/autonomy/updates/native-control-pointer-feedback.md`.

## Prepared native control labels

Additive trusted Rust rendering API, 2026-09-09. `WindowControlLabels::new()`
loads bundled Inter, the design brief's operating typeface, from an explicitly
populated private font database. Pinned cosmic-text performs advanced shaping,
wrapping and rasterization; its fontconfig feature is disabled. No host-font scan,
network access, client context or new user-facing vocabulary is involved.
`from_fonts` accepts a primary face and explicit fallback fonts; an empty database
or any unparseable entry returns `Font`. Unsupported shaped glyphs return
`MissingGlyph`, even when the offending text would lie below the visible box.
The bundled face supports the European-script fixtures; this is not a claim of
universal script coverage. Additional fonts must be supplied for other scripts.

`prepare(control, strings, geometry, scheme, scale)` uses the existing
`control.action().said(strings)` for every action, including disabled controls.
The maximize action still names maximize/restore together. The host must register
`shortcut_words`; missing vocabulary/unfilled text refuses with `Vocabulary`.
Empty text and text above 4096 UTF-8 bytes refuse with `Text`. No ellipsis or
second English fallback is invented. `said()` retains the complete text and its
translation/source provenance for the host's full-name and fallback presentation.

`LabelGeometry` supplies explicit output, origin and box size. Viewport and origin
use the strip's limits; width is 9..=2048 and height 9..=512, with at most 1,048,576
opaque RGBA pixels. Invalid geometry refuses before shaping/allocation. Base text
is 14px with 20px line height, multiplied by the existing 75..300% `TextScale`.
Four pixels of nominal padding accommodate glyph bearings and accents. Text wraps
at words, then glyphs; actual ink clips at box edges, not at the advance boundary.
`clipped()` reports text/line-height overflow or any output-clipped box. Full words
remain available via `said()`; the host must arrange a readable full-text surface
when clipping occurs. This component does not claim accessibility conformance.

`WindowControlLabel::paint(frame)` draws coalesced opaque scanline spans in the
existing light (cream/navy) or dark (charcoal/cream) tokens, clipping every span
to the viewport. The frame must match that viewport at scale one and normal
transform. Paint after controls and before the pointer; on submission error,
discard the partial frame. `pixels()` exposes the bounded prepared RGBA image
for native composition and deterministic readback verification.

Preparation and painting own no client/mapping or input authority. Disabled names
remain readable; label boxes do not change strip hit geometry. Host placement,
hover/focus selection, dismissal, input ownership for overlaid labels and actual
nested/direct production composition remain the next integration component.
No automatic tooltip, window decoration or new shortcut is installed by this API.
Existing client typing/routing code is unchanged. Component evidence:
`docs/autonomy/updates/native-control-label-rendering.md`.

## Live native label selection and placement

Additive trusted Rust API, 2026-09-09:
`Server::window_control_label_target(painted, selection, size)` returns an optional
`WindowControlLabelTarget` containing the current control and `LabelGeometry`.
It takes `&self`, sends no wire event and retains no surface, focus or input token.
`Pointer(x, y)` uses the strip's clipped hit test, including disabled controls;
`Focus(action)` selects an explicitly current native control, never the client's
keyboard focus. Non-strip actions and fully clipped controls select nothing.
`Dismissed`, absent/hidden/unmapped/foreign targets, any held native press (even
cancelled) and competing popup/client-button/move/resize ownership select nothing.

The host refreshes on every frame and discards the old label on None or error.
It supplies Dismissed on leave, focus loss, dismissal or input backend deactivation,
and must never carry native focus across target replacement or mapping retirement.
This API intentionally has no persistent focus cache to confuse with authority.
Focus event dispatch, overlay input ownership, a full-text surface when the shaper
reports clipping, and production nested/direct composition remain host integration.
The label itself adds no pointer hit region and does not forward or consume input.

Size must be 9..=2048 by 9..=512. A selected label on an output smaller than 9px
refuses with Geometry. Otherwise shrink the box to the output, align to the visible
control's left edge, prefer a four-pixel gap below, then above, then clamp vertically.
Horizontal placement clamps to the output. On small outputs the box can overlap
the strip; the host must handle overlay composition/input before shipping it.
Text preparation still reports clipping and preserves full vocabulary/provenance.
The existing font, tokens and action strings remain authoritative (ADRs 0002/0010).

Tests: `tests/window_controls/labels.rs`; live GLES hide/reveal readback:
`examples/support/window_control_label_check.rs`. Exact evidence and limits:
`docs/autonomy/updates/live-native-control-label-presentation.md`.
