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

## Mapping-bound name reading state

2026-09-09 additive trusted API: `Server::begin_window_control_reader` prepares
the complete page set for an explicitly named action on the published strip.
`WindowControlReaderStyle` fixes preferred size, scheme and text scale for that
reading session; the host must dismiss/reopen when vocabulary or style changes.
Disabled names remain eligible. Missing/hidden/non-strip selection or competing
input yields None; geometry, placement, text and allocation errors retain the
page preparer's atomic refusal. Opening does not change native/client focus.

`read_window_control_page` validates the server's live publication identity before
selecting a zero-based page. The borrowed result carries the page, unabridged Said
and one-based position/total numbers. Out-of-range selection does not wrap or
change the previous index. Stale identity or observed competing input permanently
dismisses the reader even if the requested index is invalid. Explicit `dismiss`
is permanent. No pages can be extracted from the opaque reader without validation.

Each published strip lifetime has a unique identity, shared only by identical
live refreshes. Removal/republication, different root, changed viewport/origin,
hide/remap, changed maximize/restore intent and backend retirement invalidate
old readers. Observing competing input also renews that identity. Another server
cannot accept the handle, and refusal there permanently closes it. Ordinary
typing does not navigate, steal keyboard focus or execute an operation.

This completes reader state and checked selection, not a submitted reader frame.
Borrowed pages remain frozen snapshots: discard them across host events. No input
ownership or scene-publication authority is granted. Externalized position wording,
keyboard/pointer navigation and transactional reader rendering/retirement are the
next component; ordinary full-label rendering still refuses incomplete names.
Tests use real private Wayland clients with raster comparisons and lifecycle
changes; they do not prove parent key delivery, on-screen reader UI or scanout.
Evidence: `docs/autonomy/updates/mapping-bound-native-name-readers.md`.

## Externalized reader navigation model

2026-09-09 additive trusted API: `WindowControlReaderPage::chrome(&Strings)`
returns the complete position, previous, next and dismiss wording as four
independent `Said` values, plus `can_previous`/`can_next` boundary state. Disabled
navigation retains its name. All wording is prepared or none is returned.
Missing/unfilled vocabulary refuses with `WindowControlLabelError::Vocabulary`;
blank or over-4096-byte wording refuses with `Text`; invalid one-based position
or a total outside 1..=128 refuses with `Geometry`. These are diagnostic errors,
not person-facing refusal sentences. No shaping or fit is promised by this model.

Hosts register `window_control_reader_words::declare_reader_words` before loading
translations. Registration is atomic on key collision and composes with the
existing shortcut vocabulary. The four additive format-1 translation keys are:

| Key | Source wording |
| --- | --- |
| `shell.name-page` | `Page {page} of {total}` |
| `shell.name-previous` | `Previous page` |
| `shell.name-next` | `Next page` |
| `shell.name-dismiss` | `Done reading` |

Position uses bounded decimal integers without grouping (1 through 128). The
translator owns the order of both gaps. Each string retains its own source or
translation provenance: translating position does not mark an untranslated
button translated. Dismiss refers only to the reader, never CloseWindow.

`Server::navigate_window_control_reader` accepts `Previous` or `Next`, validates
the live reader before considering boundaries, and returns the borrowed page.
There is no wraparound; unavailable navigation preserves selection. A stale
request at a boundary still permanently retires the reader. Explicit dismissal
continues to use `WindowControlReader::dismiss`. No window operation, client
focus change, configured shortcut or keyboard/pointer ownership is introduced.

These are frozen host snapshots, to discard across host events. Public chrome
data does not prove a live reader or successful frame submission. The live
server navigation check remains mandatory; never dispatch from old availability
booleans. Chrome layout/rasterization, input routing and transactional reader
composition/submission/publication are still required before usable full-name
access. Existing complete-label transactions keep their strict refusal behavior.
Evidence: `docs/autonomy/updates/externalized-native-reader-navigation.md`.


## Bounded reader chrome preparation

2026-09-09 additive trusted API: `WindowControlReaderChrome::prepare` consumes the
worded model and takes the explicit shaper, strip layout, prepared page,
`LabelGeometry`, scheme and text scale. It returns
`PreparedWindowControlReaderChrome` only when all four complete wording rows fit.
`rows()` is an immutable four-element array in position/previous/next/dismiss
order; `available()` retains previous/next availability as frozen data.

The capacity uses the existing 9..=2048 by 9..=512 geometry limits. It must be
wholly on the shared page/strip viewport, overlapping neither the page nor any
strip control. Each row uses its natural wrapped height with four-pixel padding
and four-pixel gaps between rows; unused capacity remains unpainted. The sum of
row allocations cannot exceed 1,048,576 opaque RGBA pixels (4 MiB). All wording
is checked again, including the final row, because the input model has public
fields. Invalid geometry/text/vocabulary/glyphs retain the label error; mismatched
or overlapping placement returns `Placement`; exhausted height or any clipped
ink returns `LineTooLarge`. Refusal yields no partial prepared chrome.

Preparation preserves each original Said and its entire displayed text, including
source guillemets supplied by `Strings::shown(Showing::InDevelopment)`. It does
not invent a source marker or change `Showing::ToAPerson`. The bundled Inter,
14px/20px metrics multiplied by the person's scale, wrapping and scheme tokens
are shared with complete control labels. No shrinking, ellipsis or host-font scan.

`paint` draws every opaque row into a matching scale-one frame using the existing
label painter. The host must discard a failed frame. These are wording rasters,
not interactive button state: unavailable navigation retains its complete name
and availability for the later interaction renderer. The prepared data retains
no live mapping and grants no pointer/keyboard ownership. The host must still
revalidate the page, compose and submit one reader transaction, publish overlay
input ownership only on success, and retire on lifecycle changes. Native input
navigation, interaction feedback and integrated reader publication remain owed.
Existing full-label scene validation and normal keyboard routing are unchanged.

Evidence: `docs/autonomy/updates/bounded-native-reader-chrome-rendering.md`.


## Mapping-bound reader key transactions

2026-09-09 additive trusted host primitive: `WindowControlReaderKeys::route`
accepts a server, optional live reader, evdev code, KeyState and optional
`ReaderKeyCommand` (Previous, Next, Dismiss). Opening a reader does not install
a keyboard filter. The host must offer commands only after its reader frame has
successfully published, route every owned release even when inactive, and call
`cancel` on removal, failed submission, focus/seat/session loss or binding changes.
Backend event-pump wiring and reader-frame publication remain subsequent work.

A valid new press captures the exact reading session, page visit and command.
A release executes at most once after live mapping validation. Two readers of the
same strip are distinct; leaving a page and returning creates a new visit, so an
old press cannot act on that later visit. Boundary requests consume without
wrapping. Dismiss closes only the reader. Repeats consume without execution or
rearming; another command held concurrently is consumed but never armed.
Cancellation retains ownership until each release drains. Releases use the
captured command even if the host's current semantic mapping has changed.

Only valid evdev codes 1..=767 acquire ownership, bounding retained storage.
Missing keyboard seats and keys already held by ordinary client routing refuse
new acquisition. None commands and unknown releases return Forward; the host
routes those through ordinary keyboard delivery once. Consumed, Changed and
Dismissed must never be forwarded. Changed requests fresh reader composition;
Dismissed requests pixel removal. No window operation, XKB state change, client
focus change, configured binding or new user-facing string is introduced.

This completes explicit keyboard transaction ownership, not automatically active
reader keyboard UI. Pointer ownership, interaction feedback, transactional
reader composition/publication/retirement and backend mapping remain required.
Evidence: `docs/autonomy/updates/mapping-bound-reader-key-transactions.md`.

## Mapping-bound reader pointer transactions

2026-09-09 additive trusted host primitive: `WindowControlReaderPointer` accepts
semantic `ReaderPointerHit::Content` or `Command(ReaderKeyCommand)` hits from the
host's successfully published reader frame. None means outside/inactive. This
component does not perform geometry hit testing, paint feedback, publish a reader
or install backend routing. Existing ReaderKeyCommand/ReaderKeyRoute enums are
shared to keep previous/next/dismiss and routing outcomes identical across input.

`motion` validates the current reader and pointer seat, computes enabled hover,
and returns whether motion and axes must be withheld from clients. Content and
disabled navigation consume but have no command feedback. Existing client-held
buttons and competing popup/move/resize/control input refuse new ownership. An
owned reader gesture consumes even outside the reader until its releases drain.

`button` accepts only BTN_LEFT through BTN_TASK (eight retained slots). Only the
primary button on an enabled command arms. It captures the exact reader, page
visit and semantic command; matching release executes once. Leaving that command,
page away-and-back, replacement, stale/foreign reader, cancellation or another
button disarms permanently. Returning and duplicate presses cannot rearm. Content,
disabled rows and other mouse buttons consume without executing. Unknown releases
and invalid codes return Forward. Changed requests fresh composition; Dismissed
requests reader removal, never an application close or focus change.

`feedback` revalidates semantic hovered/pressed commands against the live reader;
stale feedback clears all armed commands. It is state for a future painter, not
rendering evidence. `cancel` clears hover and execution but retains every matching
release obligation. Hosts must cancel on failed/removed frames, pointer leave,
focus/seat/session loss, geometry changes and competing keyboard interaction;
keep routing owned releases while inactive. Offer hits only after successful
reader publication, re-hit before axes/buttons, and never forward consumed input.
This host integration and coordinated key/pointer activation remain unfinished.

Private-client tests cover semantic transactions, all eight button release slots,
ordinary typing, existing client grabs and explicit host pointer routing. They do
not prove parent pointer delivery, rendered interaction feedback or on-screen
reader navigation. Evidence:
`docs/autonomy/updates/mapping-bound-reader-pointer-transactions.md`.

## Native reader hit geometry and feedback composition

2026-09-10 additive trusted API: `WindowControlReaderInteraction::new` borrows
one prepared page, complete chrome and strip layout. It checks the original
page bounds, viewport and three strip rectangles, then reserves two pixels
outside each previous/next/dismiss wording row. Every expanded box must remain
on output and disjoint from the page, other rows and all strip controls. Position
wording retains its original bounds. Invalid placement returns `ControlScene`
before any rendering. Existing chrome preparation remains available without the
additional gutter requirement; an interaction view needs the extra capacity.

`hit((x, y))` uses those exact scale-one output-local rectangles, including opaque
gutters. Page and position return `Content`; previous/next/dismiss return semantic
commands even when unavailable, so the existing live pointer transaction can
consume without acting. Fractional coordinates are not rounded or clamped.
Left/top edges are inclusive, right/bottom exclusive; nonfinite values, unused
capacity and transparent gaps return None. Hit geometry alone confers no authority.

`paint(frame, feedback)` first rejects disabled hover/press and a pressed command
that differs from hover. It then paints the complete page and wording, clears
opaque gutters and paints feedback outside every text pixel. Enabled rows have
a one-pixel underline, hover a one-pixel outline, and armed press a two-pixel
outline; unavailable rows retain full wording without a command affordance.
These shape differences use the original chrome scheme's Cream/Navy or
Charcoal/Cream tokens; no terracotta, shrinking, clipping or recolouring of text.
The bounded gutter painter allocates at most 24 solid primitives, no text copies.
Bad feedback leaves the frame untouched; a renderer failure requires discarding
the entire frame, as with existing label painting.

This is frozen geometry and composition, not live page identity or publication.
Matching bounds do not prove matching content, vocabulary or page visit. Hosts
must prepare chrome from the current validated reader page, refresh semantic
feedback, compose and submit transactionally, expose hits only after success,
and retire/cancel ownership on failure or lifecycle changes. Transactional reader
submission/publication/retirement and coordinated backend key/pointer activation
remain the next component. No agent surface, focus change or configured shortcut.

Tests cover exact painted coverage, fractional/nonfinite edges, geometry and
feedback refusals, all text scales, source marking and live private-client
navigation/retirement with normal typing. The GLES example has four explicit
acceptance phases: no arguments for unchanged chrome, then `--idle`, `--hover`
and `--pressed` for interaction. Each phase retains a 30-second deadline and all
page/translation/scheme/scale cases. Every interaction pixel and hit is compared;
idle also verifies that refused feedback leaves the framebuffer untouched.
Exact commands/results and evidence limits:
`docs/autonomy/updates/native-reader-hit-geometry-and-feedback.md`.

## Transactional native reader frames

2026-09-10 additive trusted host API: `Server::render_window_control_reader`
accepts a live reader, actual `FrameTarget`, `WindowControlReaderFrame` and time.
The frame supplies navigation vocabulary, a reusable shaper, explicit chrome
capacity and the semantic pointer owner. The original reader's scheme and text
scale apply to page, strip, chrome and feedback. The server validates the exact
reader/page visit against its published strip, prepares all navigation wording,
refreshes feedback and composes one borrowed `WindowControlReaderScene` without
dispatching client requests. Callers cannot construct that scene from unrelated
page and chrome rasters. Reopen the reading session on vocabulary/style changes.

`FrameTarget::submit_reader` defaults to `ControlsUnsupported`; supporting only
strips cannot silently count as reader submission. Supporting targets must paint
the complete scene above client/popup trees and below either cursor, validate
the actual extent before drawing, and report only successfully submitted surfaces.
`Nested` implements this through its existing GLES swap boundary and shared
native scene painter. Direct targets currently refuse readers. The existing
complete-label and offscreen control APIs retain their signatures and behavior.

Only successful submission including the strip's root publishes reader identity,
page-visit identity and the exact opaque hit rectangles. Client callbacks and
output publication use the existing transaction. Unsupported targets, omitted
roots, invalid geometry/vocabulary, stale readers and rendering failures preserve
pending callbacks, clear native publication, cancel pointer execution while
retaining releases, and permanently dismiss the failed reader. Recovery requires
an explicitly reopened reader. Failed submission does not prove removal of old
pixels; the host must request a successful removal/recovery frame.

`window_control_reader_presented` and `presented_window_control_reader_hit`
revalidate the supplied live reader and exact page visit on every query. Mismatch
conservatively clears the old publication, including querying another reader;
away-and-back page navigation cannot revive it. Identical successful refreshes
preserve pointer feedback/presses; replacement identity, page visit or hit geometry
cancels them before composition. Ordinary strip/label rendering clears reader
publication, while ordinary rendering and strip retirement clear both. Hit data
retains fractional/nonfinite handling and covers only painted opaque areas.

This is an explicit host composition/publication component, not an installed
input filter. Before exposing the reader in a person's session, the host still
must coordinate pointer/key routing with published identity, cancel both owners
on removal/focus/seat/session loss or competing input, and drain owned releases
while inactive. Parent key mapping/navigation, native reader selection and direct
backend integration remain required. No agent verb, client focus change, new
palette, hardcoded production wording or engine patch is introduced.

Four real-client transaction tests cover exact publication, callbacks, refresh,
navigation, removal, refusal and ordinary typing. `nested_check --reader` adds
actual WSLg submission in both schemes with semantic pointer navigation and
dismissal. It does not synthesize parent navigation events or certify scanout.
`--trace` optionally reports event-loop timing without changing any deadline.
Exact results, preserved failures and evidence limits:
`docs/autonomy/updates/transactional-native-reader-frames.md`.

## Publication-bound reader input coordination

2026-09-10 additive trusted API: `WindowControlReaderInput` owns the key and
pointer transactions for one backend/server lifetime, including inactive periods.
Pass its `frame_pointer()` to reader frame preparation. `key` accepts an explicit
host semantic mapping (None for ordinary typing); `pointer` derives its own hit
from the current publication and actual backend position. It never accepts a
caller-provided semantic hit. Motion also determines axis interception. Forward
events exactly once only on `Forward`; `Changed` requests a fresh reader frame,
and `Dismissed` requests removal. This API performs no client delivery itself.

Every event revalidates the live reader and a private continuous-publication
identity. Identical successful refreshes preserve that identity. Changed reader,
page visit or geometry, even geometry changed away and back between input events,
creates a new identity and cancels both devices before routing. Removal/refusal
leaves no authority. `synchronize` handles activation changes without an input
event; false activation retires publication and requires a fresh frame on return.
Hosts still perform ordinary seat/focus teardown on loss. `cancel` disarms both
owners on key-mapping changes while retaining release obligations.

Any key press cancels pointer execution; any pointer press cancels key execution.
Motion alone does not cancel a key. Navigation/dismissal cancels both immediately.
Every owned release must reach this object, including with no reader or inactive
input. Client-owned keys/buttons cannot be acquired. Unknown events retain their
ordinary path, subject to the backend's existing activation/coordinate validation.
Absent/nonfinite pointer positions hit nothing and cannot execute; do not infer
the actual parent position from the client seat while native input is consumed.

This completes the reusable ordered-input coordinator, not its attachment to the
nested/direct event pumps, key mapping, reader selection or cursor selection.
Four private-client tests cover continuous publication, navigation, typing,
refusal, client grabs, cross-device cancellation and loss/removal release draining.
The WSLg reader fixture now routes its explicit host gestures through this API;
it still does not synthesize parent events or certify physical presentation.
For developer diagnosis, setting `ALO_NESTED_TRACE_SUBMISSION` enables stderr
timing for nested bind, paint and upstream submission; it does not change
submission behavior or fixture deadlines.
Evidence: `docs/autonomy/updates/publication-bound-reader-input.md`.

## Nested parent reader routing

2026-09-10 additive trusted API: `Nested::pump_reader_seat(server, reader)`
attaches the publication coordinator to ordered Winit activation, keyboard,
absolute motion, button and axis events. Pass the live reader explicitly; None
handles removal and still drains owned releases. `pump_seat` uses the same owner
with no reader. Continue a full-seat pump after pointer acquisition; the
keyboard-only pump does not deliver pointer events. Render-only `pump` remains
input-free and is not an alternative for an active input session.

`NestedControlInput::route_reader` is the exact pointer/activation adapter used
by that pump; `reader_key` maps evdev PageUp (104), PageDown (109), and Escape (1)
to previous, next and dismiss. These local reader navigation keys leave normal
text and other keys on the existing client path and are not configurable desktop
shortcuts. Repeats, boundaries, client-owned keys and stale publications keep
the existing transaction rules. Only Forward reaches ordinary delivery once.
The pump supplies keyboard focus before key delivery and drops inactive ordinary
keys. No reader opens automatically and no navigation key closes an application.

Actual finite parent position is retained even when reader motion is consumed;
buttons and axes use it, never the frozen client position. Axes revalidate the
current hit before interception. Malformed coordinates/buttons/axes refuse and
cancel, without retry or forwarding. Input loss/close/errors clear actual
position and retire authority. `cancel` retains owned key/button releases; the
pump no longer replaces input state after error. Reactivation requires fresh
motion and fresh publication. Hosts must continue forwarding all ordered events.

`Nested::render_reader` takes `NestedReaderFrame` (strings, shaper and chrome
geometry), and supplies the same backend-owned pointer feedback to the existing
server frame transaction. No event dispatch occurs during this borrow. Inspect
the reader after pumping, draw its current page or remove its pixels; failure
and dismissal never authorize input against an old scene. Reader selection,
cursor selection and direct backend integration remain separate work.

Four private-client tests exercise the installed adapter. WSLg checks include
two backend-owned reader submissions and actual reader-pump calls, in addition
to the existing twelve explicitly host-driven frames. They do not synthesize
parent key/button events or read back those submitted frames. Exact evidence:
`docs/autonomy/updates/nested-reader-event-routing.md`.

## Native full-name keyboard activation

2026-09-10 additive trusted API: `Nested::pump_reader_session` borrows a
`NestedReaderSession` with a retained optional reader, strings, text shaper,
appearance and chrome capacity. F1 (evdev 59) opens an explicitly native-focused
control's complete name on release, including disabled controls. Hover alone
does not acquire F1. With no native focus, an existing reader or any ordinary
client key held, application routing retains the key. This local name-help
gesture is separate from configurable desktop command dispatch.

`NestedControlInput::reader_session_key` is the exact ordered key adapter.
Preparation checks every page before returning from press; errors cancel input
and retain the consumed release. Repeats never reopen. Opening returns `Changed`
and installs an unpublished page-zero reader in the host's slot. Render it before
navigation can acquire authority. Existing PageUp/PageDown/Escape routing remains;
the host removes dismissed readers and their pixels after pumping.

Every explicit native focus request renews a private selection identity. Focus
away-and-back, strip retirement/republication, stale mapping, competing pointer
events or keys, deactivation and backend errors disarm opening. Releases remain
owned across cancellation and switching back to older pump modes. Identical strip
frame refreshes preserve a gesture. No window command or client focus change is
caused by name opening. Before changing vocabulary/appearance/capacity, retire
controls, remove the old reader, then republish and refocus with the new values.

The host must establish native focus; focus navigation, cursor selection and
direct-backend integration remain separate components. Four private-client tests
and two WSLg gesture-to-reader EGL submissions pass. Controls and reader graphical
acceptance pass in recovery with unchanged deadlines. The earlier intermittent
swap delay remains unresolved; independent publication gates are pending.
Evidence and exact limits: `docs/autonomy/updates/native-full-name-keyboard-activation.md`.

## Live native reader selection

2026-09-10 additive trusted API: `Server::open_presented_window_control_reader`
opens the name selected by the live strip's explicit native focus, otherwise a
fresh hover hit. It never infers selection from application keyboard focus or a
retained action. Disabled names remain readable. No selection, stale publication,
off-strip hover or competing input returns None. Invalid wording, page capacity,
chrome capacity or interaction gutters returns the existing typed render error.

Before returning, every page's complete position/previous/next/dismiss wording
and interaction geometry is prepared and checked at the original text scale.
This includes later page numbers that need more space. Temporary chrome rasters
are dropped after each check; the existing 128-page reader budget remains.
Success returns page zero with no input or submission authority. Preparation
errors do not dismiss an already open reader or execute a window command.

`Nested::open_control_reader` supplies its actual parent pointer position to the
same selector. Pump first and call on an explicit host opening request. Keep the
returned reader across navigation, use the same strings and chrome capacity for
`render_reader`, and route through `pump_reader_seat` only under the existing
successful-publication rules. Vocabulary or appearance changes require reopening.
Rendering still independently validates every frame; this preflight is not proof
of graphics submission. This API does not choose an activation key/gesture or
automatically fall back from an oversized tooltip during ordinary strip drawing.

Four private-client checks cover focus precedence, disabled hover, complete page
traversal, typing, absent/stale/busy refusal, malformed capacity, missing wording,
preservation of an existing reader, and capacity that fits page one but fails at
page ten. WSLg selection-to-submission evidence uses explicit native focus, not
synthetic parent input. User activation, native cursor selection and direct
integration remain. Evidence: `docs/autonomy/updates/live-native-reader-selection.md`.

## Automatic complete-name presentation

2026-09-10 additive trusted API: `Server::render_presented_window_control_name`
uses the live published strip, native focus or fresh hover and the supplied reader
style. It prefers the complete preferred/expanded label; exhausted label capacity
automatically opens, preflights and submits a paged reader. Invalid geometry,
wording, fonts and glyphs remain errors, not overflow signals. Existing
`prepare_expanded` behavior is unchanged; its private capacity-aware preparation
distinguishes no fitting box from invalid preparation. Text scale and provenance
are unchanged. Missing publication or a different target size refuses.

The return value is `(submitted_surface_count, optional_reader)`. None means a
complete label or a bare strip with no eligible name was submitted. Some contains
page zero of an already successfully submitted reader. Every page's navigation
capacity has been checked; subsequent frames still validate independently. Failure
cancels pointer feedback, retires native authority and preserves owned releases
and pending callbacks through the existing transactions. No window command runs.

Pump before calling. Use this presenter only while no reading session is retained;
keep a returned reader across navigation, redraw through the existing reader API
and drain owned input across dismissal. The host decides when fresh hover/focus
should resume name presentation after dismissal; calling the opener every frame
would reset navigation. Vocabulary and appearance changes require reopening.
The existing low-level labeled-strip API retains its refusal behavior.

`Nested::render_control_name` supplies actual parent position and the same pointer
owner used by `pump_reader_seat`/`render_reader`. This completes automatic fallback
in the opt-in live-name presenter, not an installed desktop session policy or
activation gesture. Direct backend attachment and native cursor selection remain.
Private-client and nested EGL evidence and exact limits are recorded in
`docs/autonomy/updates/automatic-native-name-fallback.md`.
