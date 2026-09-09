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

## Mapping-bound host presentation

Additive trusted Rust API, 2026-09-09. `Server::present_window_controls` records
one explicitly composed strip, with its surface, visibility identity, geometry
and maximize/restore intent. It does not paint or select a window. The host calls
it after successful composition of that mapping, and passes None on removal or
failed submission. A retained surface handle alone does not prove a current frame.
An invalid or foreign candidate also retires the old presentation before refusal.
Replacing the target, viewport, origin, visibility identity or observed intent
clears native label focus and disarms held execution. Repeating the same live
frame preserves focus and a held gesture. This is one server-owned state, not a
transferable token, saved stack index or client-keyboard-focus fallback.

`presented_window_controls(position)` returns fresh feedback, and
`presented_window_control_label(hover, size)` selects current native focus or fresh
hover through the existing selector. Both revalidate the mapping and observed
intent, retiring stale presentation. Hide/reveal and unmap/remap retire identity
even when both happen between host observations. Returned snapshots/labels are
still frozen; discard previous rendering on None/error. Hover is never cached.

`focus_window_control(action)` explicitly chooses a visible strip action solely
for label presentation. Disabled controls remain eligible. None, non-strip actions,
fully clipped controls and held/competing input clear old focus and return false.
Client keyboard focus, wire events and operations are unchanged. Observing held
or competing input through these APIs clears focus permanently; the host must
observe input/ownership changes, not defer them until after a grab has ended.

`route_presented_window_control_pointer(position, event, time)` dismisses native
label focus, validates the published target and calls the existing combined router
exactly once. It caches no position and adds no hit area. Missing presentation
uses ordinary client fallback. Retired native presses keep their matching release
ownership, so returning to a target never rearms execution or leaks the release.
Do not mix this lifecycle with independently supplied low-level painted targets.

`retire_window_controls()` is idempotent and cancels execution while preserving
release ownership. Existing `pointer_leave` (including nested/direct pointer
deactivation) and `clear_input` call it before capability-dependent cleanup;
missing pointer capability cannot retain native focus. Hosts must also retire on
their own output/submission loss paths. No backend starts composing a strip or
routing its normal event pump through these APIs automatically in this component.

Nested composition/event pumping, native keyboard navigation, label overlay hit
policy and readable full-text presentation for clipped names remain integration
work, followed by direct composition. The existing font/vocabulary/tokens and
ADRs 0002/0010 remain unchanged. Tests: `tests/window_controls/presentation.rs`;
ten full-frame GLES label lifecycle checks in the existing nested fixture.
Evidence and limits: `docs/autonomy/updates/mapping-bound-native-control-presentation.md`.

## Nested parent-event integration

Additive trusted Rust adapter, 2026-09-09: `NestedControlInput` owns only the last
validated parent motion, separately from the client seat position that native
gestures deliberately freeze. Keep one adapter with one backend/server lifetime.
`route(server, active, event)` accepts the existing `NestedPointerEvent` and routes
motion/buttons exactly once through the current published presentation. No window
is inferred from focus. `position()` supplies current hover coordinates, not
authority. Native out-and-back motion permanently cancels execution.

Deactivation/close forgets position and retires controls. Buttons and scroll wait
for fresh motion after reactivation. An owned primary release is still consumed
while inactive or before new motion, so a cancelled gesture cannot leak or block
the next genuine gesture. Invalid motion uses the ordinary fixed-point coordinate
limits, retires presentation and forgets position before refusal. Scroll clears
native label focus and retains the existing seat route; secondary buttons and
ordinary keyboard routing remain independent.

`Nested::pump_seat` now uses this adapter automatically. It retains the first input
failure, stops routing subsequent events in that pump, resets input, and returns
the error without fallback/retry. `RenderError::WindowControl` retains native
routing refusal details. The graphics-only and keyboard-only pumps do not install
native pointer routing. The legacy explicit `Server::nested_pointer` is unchanged.

This installs event routing, not strip/label composition. The host must still
publish only successfully composed controls and retire failed/removed frames.
Overlay hit policy, full clipped-name access, native navigation, cursor integration
during native grabs and direct-backend composition remain work. Smithay 0.7's
missing parent cursor-leave notifications remain an existing backend limitation.
Four real-client adapter tests and eight complete GLES label frames plus live
minimization pass; synthetic adapter events do not prove actual parent-event
delivery, on-screen interaction, direct scanout or hardware acceptance. Report:
`docs/autonomy/updates/nested-native-control-event-routing.md`.

## Native scene composition

Additive trusted Rust APIs, 2026-09-09: `WindowControlScene` borrows a fresh strip,
optional prepared label and existing appearance scheme. `render_control_scanout`
prepares owned pixels; `Nested::submit_control_scene` submits through the same GLES
painter. Existing no-control entry points delegate with None and retain their
behavior. No native scene is cached between frames.

Composition order is client roots/popups, native strip, label, then client cursor
or compositor arrow. Cursor imports remain alive through frame finish. Native
pixels add no Wayland identities or callbacks; drawn client identity order is
preserved, including clients behind opaque native pixels. Preparation still sends
no callbacks or output membership. Errors return no prepared/submitted frame.

`RenderError::ControlScene` refuses output/layout/label viewport mismatch, any
clipped label, and any label covering a control. Validation precedes client import
(and offscreen allocation). A failed candidate is never silently replaced with
cached native content. These are composition refusals, not an alternate full-text
reader: the host must provide a larger label or another full-text presentation.

This rendering boundary owns no window authority or input. The host must obtain
fresh mapping-bound presentation without dispatch during submission, publish only
after successful submission and retire on failure/removal. It must establish label
overlay input policy before interactive use. Ordinary keyboard input is unchanged.
Nested host transactions, overlay policy, full clipped-name access, navigation and
native cursor selection remain integration work; direct installation also remains.
No new vocabulary, agent surface, palette or ADR. Evidence and precise limits:
`docs/autonomy/updates/native-control-scene-composition.md`.

## Transactional native strip submission

Additive trusted Rust API, 2026-09-09: `WindowControlFrame` supplies an explicit
root, origin, current pointer position and scheme. `Server::render_window_controls`
takes the viewport from its actual `FrameTarget`, captures fresh layout/feedback,
and submits clients, popups, strip and cursor through the existing output and
callback transaction without intervening dispatch. Only success publishes the
mapping-bound strip. `Nested::render_window_controls` supplies its own actual
parent pointer position; pump input before rendering.

`FrameTarget::submit_controls` must compose the supplied scene or refuse. Its
default refuses Some with `RenderError::ControlsUnsupported`, and forwards None
to the existing popup/cursor submission. Nested implements the shared GLES path.
`ControlSnapshot` preserves live-target/geometry refusal details; a backend that
omits the selected root returns `ControlTargetOmitted` before output/callback
publication. The backend remains trusted to report the surfaces it actually drew.

Any validation/submission error retires previous native authority, clears native
focus and cancels held execution while keeping ownership of the release. Pending
callbacks remain queued. None, including ordinary `Server::render`, explicitly
removes controls and retires their authority even if that removal frame fails.
Unchanged successful presentation preserves valid gestures and native label focus;
replacement/relayout uses the existing retirement policy. Frame callbacks and
output membership retain their original meaning: backend submission, not physical
presentation. The first output submission may enable maximize on the next freshly
captured frame; input operations always revalidate current policy.

This transaction composes strips only. It does not silently install opaque labels
without overlay input policy. The labeled transaction below adds fresh labels and
their pointer policy; alternate full-text access, native navigation,
cursor selection during native gestures and direct integration remain unfinished.
There is no new agent surface, vocabulary, font, palette or policy decision.
Tests: `tests/window_controls/frame.rs`; actual EGL submission integration:
`nested_check --controls`. Report and precise evidence limits:
`docs/autonomy/updates/transactional-native-control-submission.md`.

## Transactional native label composition

Additive trusted Rust API, 2026-09-09: `WindowControlLabelFrame` supplies the
session vocabulary, reusable private native shaper, requested box and existing
text scale. `Server::render_labeled_window_controls` selects and shapes a fresh
label in the strip transaction, before submission and without client dispatch.
Only identical live mapping/geometry may reuse explicit native label focus;
otherwise current hover selects the name, including disabled controls. The nested
entry point supplies actual parent position. Neither entry point infers client
keyboard focus or chooses a window. Pump input before rendering.

Clients/popups, strip, complete label and cursor share one backend submission.
Only success publishes label bounds with strip authority. Missing selection,
held native/overlay/client input and None dismiss labels in the next frame.
`ControlLabel` preserves selection/shaping errors; missing vocabulary, malformed
geometry, clipping or overlap refuse before backend submission. Output/callback
publication is unchanged. Failure retires native authority and preserves pending
callbacks. Strip-only and ordinary rendering remove label exclusion as well.

Published opaque bounds exclude pointer motion, buttons and scroll from clients.
The published pointer router consumes covered events, clears client pointer focus
without changing keyboard focus, and retains bounds across an input batch until
replacement/removal/retirement. Thus moving into a label and pressing before the
next frame cannot click through it. Existing client grabs and window/popup grabs
retain priority; a label never steals their release. Button presses over labels
are separately owned for every button code, including secondary/chorded buttons.
Their matching releases remain consumed through label dismissal, failed frames,
strip replacement, mapping retirement and backend leave/reset. While held, they
suppress labels, native focus acquisition and strip hover feedback. No operation
executes from a label. Fresh motion restores client routing after exclusion ends.

`route_presented_window_control_axis(position, frame)` validates coordinates and
axis values, retires stale presentation, dismisses native focus and consumes
covered/overlay-owned scroll. Its boolean reports client delivery. The nested
adapter uses it and drains owned releases even inactive or without fresh motion.
Hosts composing these labels must use the published pointer/axis routes, never
mix them with independent low-level painted targets or direct client forwarding.
Failure is a rendering failure, not permission to keep operating an old frame.

Five real-client tests are in `tests/window_controls/label_frame.rs`. Actual
nested EGL label submission/dismissal/refusal checks extend `nested_check --controls`.
Existing offscreen full-scene readback continues to cover painter ordering.
This completes the nested label transaction and overlay pointer policy. Alternate
full-text access on constrained outputs, native navigation/cursor selection and
direct integration remain unfinished; clipped labels currently refuse rather
than silently losing words. No agent surface, vocabulary, palette, fonts or
accepted ADR changes. Evidence: `docs/autonomy/updates/transactional-native-label-composition.md`.

## Adaptive full-name expansion

2026-09-09: `WindowControlLabels::prepare_expanded` retains a fitting requested
box. If it clips text/output or covers a control, it tries output-contained space
below, then above the strip, keeping a four-pixel gap. Alternative boxes use the
available width/height, capped at 2048 by 512. At most three rasters are prepared;
there is no search loop, font shrinking, ellipsis or discarded source/fallback
text. Selection action, geometry and availability must belong to the supplied
layout; transient hover feedback is irrelevant to naming. Viewports must agree.
Invalid geometry, vocabulary and unsupported glyphs still refuse immediately.

The existing labeled Server/Nested transaction now uses this preparation, so a
9px preferred box can produce a full-width name. If no complete candidate fits,
`ControlScene` still refuses before backend submission. The low-level `prepare`
and scene validation remain strict and unchanged. Only successful submission
publishes the expanded rectangle for pointer exclusion. Failed expansion or
submission retires native authority and preserves callbacks; held overlay input
still dismisses the label and drains releases. Ordinary client typing is intact.

Automatic expansion is implemented and graphical recovery checks pass. Full-text
access remains incomplete for constrained
output: names exceeding both available boxes still need a bounded paged reader.
That remaining component precedes native navigation/cursor selection and direct
integration. No public agent interface, vocabulary, palette, font or ADR changes.
Evidence: `docs/autonomy/updates/adaptive-native-control-labels.md` and its
follow-up `docs/autonomy/updates/repairing-an-unfinished-desktop-task.md`.

## Bounded native-name page preparation

2026-09-09 additive trusted Rust API: `WindowControlLabels::prepare_pages` takes
an explicit selection, matching layout, output-contained reader box, vocabulary,
scheme and text scale. It shapes the complete name once using the same private
fonts and shaping rules as ordinary labels, then partitions whole visual lines
into immutable `WindowControlLabelPages`. Words, shaping clusters, line order,
source/translation provenance and text scale remain intact. It does not split
the source into independently shaped substrings. Explicit line breaks and wrapped
lines participate in the same partition, including blank visual lines.

`said()` retains the unabridged name. `pages()` exposes a nonempty ordered slice;
each `WindowControlLabelPage` exposes its contiguous visual-line range, bounds,
opaque RGBA pixels and scale-one painter. All ranges together cover every shaped
line exactly once. A one-page name has the same raster as ordinary preparation
at the same geometry. A page is a distinct type: it cannot be supplied as a
complete label to `WindowControlScene`. The painter reuses the ordinary label's
scanline drawing, without granting input authority or installing navigation.

The reader box retains 9..=2048 by 9..=512 limits and four-pixel padding, must
belong to the layout viewport and must not cover any control. The selection's
action, bounds and availability must match a control; transient feedback does
not change the name. Each page must contain complete lines and every nontransparent
ink pixel. Too-small line/ink space returns `LineTooLarge`, foreign/off-output/
overlapping placement returns `Placement`, and existing geometry, text, vocabulary
or missing-glyph errors are preserved under `Label`. Shaping checks all glyphs,
including later pages. More than 128 pages or 4,194,304 aggregate RGBA pixels
returns `Budget` before page raster allocation. These limits bound the prepared
reader to 16 MiB of pixels, independently of its 4096-byte wording limit.
All refusals are atomic: no prefix of the page set escapes on failure.

This completes page preparation/rendering only. Live mapping-bound navigation,
page position wording, keyboard/pointer ownership, submission/publication and
retirement still need a reader transaction before full-name access is usable.
Ordinary label transactions still refuse names that cannot expand completely;
they do not silently replace a name with its first page. Existing client typing
and release ownership are unchanged. Native navigation/cursor selection and
direct integration remain later components. No new agent surface, vocabulary,
palette, font or ADR. Component evidence and exact limits:
`docs/autonomy/updates/paged-native-control-label-rendering.md`.
