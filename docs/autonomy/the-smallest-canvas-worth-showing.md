# The smallest canvas worth showing

**What this is.** The owner decided on 2026-09-26 that the launch film shows the
canvas **running**, not drawn. So this is the smallest canvas that is a real
interface rather than a prototype: enough to film, and enough to ship to a
developer who will try to break it.

**What it is not.** Places, a World, the minimized shelf, Compact, multiple
selection and arrangement, full-screen with edge-revealed controls, the zoom
menu as designed, ghost previews, a Place that remembers time. Those are v1 and
v1.1 and they are listed in `docs/features.md` with their tiers. **A minimum
that grows is not a minimum**, and every one of them is a thing somebody will
reasonably ask for while this is being built.

**The one sentence it has to earn:** apps float on a surface you can move and
zoom, and zooming out shows you everything at once.

## The hidden cost, said once so nobody meets it late

`crates/alo-shell` carries **twenty-two modules** for the window-control reader.
A canvas changes what *where is this window* means for every one of them, and
*every canvas also answers as a list* is the accessibility half of the canvas
rather than a nicety. Task 7 exists so that the first accessibility review does
not undo the other nine.

---

## Tasks

### 1. A plane that moves under a viewport that does not

**Status:** **Done, 2026-09-27.** `crates/alo-canvas` holds the plane and the
camera; `alo_shell::scene::trees` applies the whole camera at one seam;
`alo_shell::Server::{pan_the_canvas, zoom_the_canvas, look_at_the_canvas}` move
it, and `FrameTarget::look_at` carries it to whichever display draws. Held by
`crates/alo-shell/tests/one_plane_under_one_viewport.rs` — six tests, including
the constraint read out of the source — and measured in pixels by the walk, which
draws the same two windows at 512 painted pixels at life size and 84 at 40 %.

**Owed, and not part of this task's acceptance:** the pointer. `trees` answers in
screen pixels, and the surface-local coordinate a client is sent is derived by
subtracting that origin, which is exact at life size and meaningless at any other
zoom. So `Server::pointer_target` **refuses to hit anything** while the canvas is
zoomed, rather than sending a client a coordinate that is not one. That refusal is
task 2's to remove, and until it is removed a zoomed canvas can be looked at and
not clicked. **Depends on:** nothing.

Two layers, and the separation is the whole architecture: canvas content in one,
viewport controls in the other. The dock and the status area are children of the
viewport, never of the plane, so panning and zooming cannot move them. Every
later task rests on this being right, and it is the thing that is expensive to
retrofit.

- **Acceptance:** the plane translates and scales while the dock and the status
  area stay exactly where they are, held by a test that reads their position
  before and after a pan and a zoom; no control drawn in the viewport layer can
  be covered by a frame.
- **Constraint:** nothing in the viewport layer may read the plane's transform
  to correct itself. If it has to, it is in the wrong layer.

#### What exists for this task, read on 2026-09-27 before starting

- **The arithmetic is done and tested**: `crates/alo-canvas` holds the plane and
  the camera — signed plane units, a zoom in integer thousandths, pointer-centred
  zooming, and the plane-to-surface transform. Thirteen tests, including a screen
  press reaching the same surface coordinate at three zooms and two pan offsets,
  which is most of **task 2**'s acceptance held before task 2 starts. The zoom is
  a ratio rather than a float deliberately: *nearly the right pixel* in a drawing
  program is a wrong pixel, and `f64` gives different answers depending on the
  order the sums were done.
- **One place decides where every client is drawn.** `alo_shell::scene`'s element
  list maps each root to `window_buffer_origin`, so the plane's translation has a
  single seam to go through rather than being threaded everywhere.
- **There is no scaling path in the drawing at all.** Nothing in
  `crates/alo-shell/src/drawing.rs` or the scene takes a scale, and no rescaling
  render element is used anywhere. So *the plane scales* is not wiring — it is a
  new capability in the GLES path, and it is the expensive half of this task
  rather than the pan. Worth knowing before task 2 rests on it, because task 2's
  coordinates are already exact and its **drawing** is not.
- **The dock and the status area are already drawn in a separate layer** from
  clients: `scene_drawing::paint` draws client trees first and the native layers
  above them, and `desktop_raster` lays the dock out from the output's size
  alone. So the *separation* exists in the drawing order today; what does not
  exist is a camera on the plane side, which is what would make the separation
  mean something.

### 2. A frame is where it looks, at any zoom

**Status:** **Done, 2026-09-27.** Two spaces with one conversion between them:
`Pointer::location` stays in screen pixels, which is what a backend reports and
what the arrow and the window-control strip are placed in, and
`Surfaces::on_the_plane` converts once — in `pointer_motion`, in the hit test, and
in `window_press`, which is where a screen press becomes a fact about a frame. The
pan needs no term in that conversion, because dividing the pointer and the origins
by the same zoom cancels it exactly.

Held by `crates/alo-shell/tests/a_frame_is_where_it_looks/mod.rs`: two real
clients read their own `wl_pointer` events off the wire, and every case aims at
**the middle of a 16x16 window** and asserts the client was told `(8, 8)` — at 40
per cent, life size and 250 per cent, and at two pan offsets. The screen
coordinate differs in all six cases and the expected answer never does. Proved to
bite: with the conversion removed, the middle of the window is reported at `3.2`,
which is `8 × 0.4`. The second test presses two frames 400 units apart and
requires the other one to hear nothing, which is the failure a person would
describe as *I clicked that window and the other one answered*.

**Not held, and not this task's to decide:** *focus follows the frame that was
pressed*. Nothing in this shell moves **keyboard** focus on a press at any zoom —
`Server::keyboard_focus` is an explicit host API and no pointer path calls it — so
that clause is about a shell policy this compositor has never had rather than
about the plane's transform. Click-to-focus versus focus-follows-pointer is an
ADR, not a line in a canvas task, and inventing it here would settle a decision
in the wrong place. What is held is that the press reaches the frame that looks
pressed. **Depends on:** 1.

An application opens as a frame on the plane and receives input where a person
points. This is the task that is quietly hard: a click at screen coordinates
must reach the right surface-local coordinates through a transform, and it must
still be right at 40% and at 250%.

- **Acceptance:** a pointer press at a known screen point arrives at the
  expected surface coordinate at three zoom levels and two pan offsets, held by
  a test; focus follows the frame that was pressed.
- **Constraint:** the application is never told about the canvas. It is told its
  size and gets its events, as it would on any compositor.

### 3. Dragging a frame

**Status:** **Done, 2026-09-27.** See below. **Depends on:** 2.

Dragged by its title area, so a press inside the content always belongs to the
application. The frame moves with the pointer at any zoom — a drag of 100 screen
pixels at half zoom moves it 200 plane units.

- **Acceptance:** dragging at three zoom levels moves the frame by the pointer's
  own distance in plane units, and a press inside the content never moves it.

#### Done, except the resize half of its refusal (2026-09-27)

**Status:** **Done, 2026-09-27**, for everything this task's acceptance names.

*Dragging at three zoom levels moves the frame by the pointer's own distance in
plane units* — a drag of 150 screen pixels becomes 375 plane units at 40 per cent,
150 at life size and 60 at 250 per cent, at two pan offsets.

*A press inside the content never moves it* — held in the **same** test, on the
same frame, at the place the drag just left it, because a frame nothing can move
satisfies that clause perfectly and the refusal is worth nothing except beside a
move that has been shown to work. The content press comes second so a refusal
cannot be mistaken for a frame that was already stuck.

The handle is the name, per ADR 0071. A frame's name is
`Server::the_name_of` — the title, then the app id, then
`alo_access::words::AN_APPLICATION`, the phrase the accessibility tree already
reads for a window. `xdg_toplevel.set_title` was never read in this crate;
smithay held it all along. The band is `crate::frame_handle`, **forty-eight** pixels
on the glass rather than on the plane: a band in plane units would shrink to under
three pixels at the furthest zoom, and zoomed out is exactly when a person is
moving frames, so the handle would vanish when it was wanted.

Forty-eight because `docs/design/the-canvas-in-numbers.md` gives it, read off the
design file at Figma node `70:29`. It was thirty-two for a day, derived from the
width of a native window control with a paragraph explaining why thirty-two was
sensible — while the document had carried forty-eight since before that paragraph
existed. `crates/alo-shell/tests/the_frame_in_numbers.rs` now holds the constant to
that document's own table row, which closes the gap the document names about
itself: *nothing yet holds these to anything*. A number a design file already gives
is not a number to derive.

`xdg_toplevel.move` is refused **outright**, not merely when the press was in the
content: that test would depend on mapping a serial back to where its press began,
and an implicit grab is where that mapping stops being trustworthy. ADR 0071
carries the reasoning and the cost.

**Six existing tests were re-rooted rather than deleted**, because a
shell-initiated move never touches the client and their subjects — cancellation,
ownership, geometry tracking, invalid motion, a maximised frame refusing a drag,
minimize cancelling one — are all still real. Two changed meaning and say so: the
one that proved a press on a **subsurface** authorised a move of its root now
proves it cannot, which is the sharpest case of the new rule, and the maximised
case now also presses the band, because a frame in a window mode refuses the
shell's road too.

**Owed, and it is task 4's to land:** `xdg_toplevel.resize` is refused by the same
argument and is **not refused yet**. Resizing from the left edge then the right
composes into a translation, so leaving resize open leaves move reachable in two
gestures — but nothing in the shell can resize yet, so refusing it now would leave
a person unable to resize at all. It ships welded to task 4's edge gesture and
cursor. Until then a client can still move itself that way.

### 4. Resizing, and the application told as it happens

**Status:** **Done, 2026-09-29.** **Depends on:** 2.

**The transaction was already right; what was missing was a door a person could
reach.** `crate::resize_transaction` has configured the client during the drag,
refreshed its limits on every motion and anchored the opposite edge since before
this task — all of it reachable only through `xdg_toplevel.resize`, the road ADR
0071 refuses. So the shell's own bands enter the **same** transaction rather than a
second one: a compositor with two resize implementations is one where a person can
find the difference.

**The numbers are the design file's** — a 6 px edge and a 24 × 24 corner — and the
bands sit **outside** the frame, because ADR 0065 says every click inside belongs
to the application. A corner beats an edge where they overlap.

**The cursor is what makes the decision real**, and it is four shapes for eight
directions with the hotspot in the middle rather than the tip. The first draft was
drawn by hand and a symmetry test caught it: one end was heavier than the other,
which tells somebody an edge moves one way. They are generated now — one half
drawn, the other that half turned half a turn — so the two ends cannot differ.

**And the refusal shipped welded to the gesture**, as ADR 0071 requires. Six tests
drove the transaction through the wire request and were redirected through the
band, because what they assert was never about the road. The one that tested the
wire's own refusals — stale serial, foreign target, invalid edge — now asserts the
pair that matters instead: a client asking gets nothing, **and** the same window
still resizes from the band. A test that checked only the refusal would pass on a
compositor nobody can resize anything on.

From the edges and the corners, with the application told its new size while the
drag is happening rather than at the end, and minimum sizes respected.

- **Acceptance:** each edge and corner resizes; the application receives its
  size during the drag; a resize that would go below the application's minimum
  stops at it rather than being refused silently.

#### What ADR 0071 hands this task (2026-09-27)

The edges and the corners are **this task's**, and ADR 0071 settled the collision
that made that ambiguous: ADR 0065 called the edge a resize handle in one paragraph
and a move handle in its table two paragraphs later. The table was the loose one.

Three things arrive here rather than in task 3:

- **The cursor**, which is the part that makes the decision real rather than an
  assertion about which pixels win: over an edge or a corner the cursor becomes the
  **double-headed arrow for that direction**, and over the name it does not. That is
  why a frame needs no furniture and still says what each part does. Four shapes
  cover eight directions; the hotspot is the middle rather than the tip, unlike
  `default_cursor`'s arrow.
- **The shell's own edge and corner bands**, the same shape as the name's band in
  `crate::frame_handle` — position from the plane, thickness on the glass.
- **Refusing `xdg_toplevel.resize`**, which ADR 0071 decided and task 3 did not
  land. It must arrive **with** the gesture above and never before it: nothing in
  the shell can resize today, so refusing the client road first would leave a
  person unable to resize anything. Until both land together, a client can still
  move itself by resizing from the left edge and then the right, which composes
  into a translation — the hole that decided the refusal in the first place.

Refusing `resize` does not freeze a window's size. That request is the interactive,
pointer-driven one; a client may still commit whatever size it likes.

### 5. Pan

**Status:** **Done, 2026-09-29**, all three roads — **and that line was false for two
days.** It is true now: the keyboard route landed 2026-09-30 in
`crates/alo-shell/src/canvas_arrow_pan.rs`, held by four tests at the foot of
`crates/alo-shell/tests/panning_the_plane/mod.rs`. **Depends on:** 1.

**How a Done line came to be written about a road that did not exist**, kept
because the mechanism is more useful than the correction. Two roads were recorded
as owed — two-finger scroll and Space-and-drag — both were closed, and the count
went *wheel + trackpad + space-drag = three*. But *wheel and trackpad over empty
canvas* is **one** road in this task's own sentence, and the third is the keyboard
one, which had never been written down as owed and so was never counted. **The
arithmetic was checked against the list of what was missing rather than against
the acceptance sentence.** Auditing the ledger, not the thing.

It survived a second audit too. *Does pan have a keyboard route* was asked of
`alo_shortcuts::Action`, which holds the canvas's other keyboard actions — and the
answer could not have been there: `docs/design/the-shortcuts-and-the-edges.md`
gives panning the **bare arrow keys**, and an `alo_shortcuts::Chord` requires
Super, Ctrl or Alt. Searching a space the answer is structurally excluded from,
and reading the empty result as *nothing exists* rather than *wrong space*.

Found by going to build task 10, whose acceptance says *then the same by keyboard
alone*, and checking which keyboard forms existed before assembling a walk that
claims to use them.

**One of the two that were owed was never missing.** *Two-finger scroll* was
recorded as still to do; `crate::libinput_scroll` already turns `ScrollFinger`
into an ordinary `AxisFrame` and `crate::canvas_pan` never asks what the source
was, so a touchpad had panned all along and nothing held it. Now
`a_two_finger_scroll_pans_exactly_as_a_wheel_does` asserts it against the wheel's
own answer rather than a number written in the test, so the two cannot drift.

**Space and drag needed something that was not a gesture road at all.** It is the
one moment a person holds a key with nothing focused, and `Server::keyboard_key`
returned before the call that advances XKB in exactly that state — so the shell
could not tell Space was down. Fixed for Ctrl+wheel in task 6 and spent twice.
Space is deliberately not an `alo_shortcuts::Chord`, which requires Super, Ctrl or
Alt: this is not a shortcut to rebind, it is the gesture every canvas application
has, and offering to rebind it would be offering to rebind how somebody holds a
sheet of paper.

Wheel and trackpad over empty canvas, a keyboard route, and a drag gesture that
does not fight editing inside a frame. **Scroll over a frame scrolls that
frame's content; scroll over empty canvas pans.**

- **Acceptance:** each road is exercised and the rule above is held by a test
  that scrolls over both and asserts which moved.

#### The wheel and the rule are held; the other two roads are not (2026-09-27)

**Held**, by `crates/alo-shell/tests/panning_the_plane/mod.rs` — four tests, at
three zooms. `crate::canvas_pan` asks one question in one place and the hit test
already answers it: *over a frame* is *the pointer has a client focus*, so no
second opinion about where frames are was needed. Each case asserts what moved
**and** what did not, which is what the acceptance's *asserts which moved* is
worth: a scroll that panned the canvas and also reached the application passes a
test that only watches the camera, and a person reading a long document would have
the canvas slide out from under it. Proved to bite in both directions — make the
pan unconditional and the frame's own scroll is stolen; remove it and the plane
never moves.

Two smaller facts fell out and are held too: a trackpad's tenths are **kept**
between events rather than truncated away, because the plane is measured in whole
units and dropping a fraction per event is a canvas that never moves for a slow
scroll; and a scroll off the edge of the plane is refused and keeps **nothing**, so
turning round does not spend a remainder as a jump.

**A finding, not an edit: the keyboard route needs another crate.** A chord reaches
a native operation through `alo_shortcuts::Action`, and that enum has no canvas
action in it — no pan, no zoom, no *Show all*. It is a vocabulary a person edits in
Settings, so it is a public surface (CLAUDE.md: config keys are contracts) in a
crate this plan does not own, and its names need entries in `alo-strings` for all
24 languages. **Task 6 needs the same thing** — *zoom has a keyboard route* is in
its acceptance. So one addition to `alo-shortcuts` unblocks both, and neither task
5 nor task 6 can be marked done until it lands.

**Also not held:** space-and-drag, which ADR 0065 names beside the wheel, and the
trackpad's two-finger form. Both are gestures rather than arithmetic.

**Nothing here is accelerated.** A scroll pans by its own value in screen pixels,
one for one. A wheel notch is ten of those by convention and a trackpad sends
something nearer a pixel, so the two feel different — a device-feel question that
wants a person and a real trackpad, not a multiplier chosen in a source file.

**One rough edge, named rather than papered over.** *Nothing focused* is what this
reads as *the arrow is on the canvas*, and the two are not quite the same thing: it
is also true in the moment after the pointer has left the output, so a scroll
arriving then pans. `alo-shell` keeps no separate record of whether a pointer is
present — focus is the only signal — and adding one is a change to pointer lifetime
rather than to the canvas, so it is written here instead of invented in
`pointer_axis`.

Two existing tests in this crate encoded the old contract and were updated rather
than relaxed: `pointer::pointer_scroll_and_invalid_input_preserve_focus` now
requires the scroll to pan **and** the client to still hear nothing, which is more
than it asked before, and `direct_pointer`'s wire-lifetime test puts the camera back
after the scroll that now pans, because every assertion in it is about a
surface-local coordinate and none of it is about the canvas.

### 6. Zoom, and *Show all*

**Status:** **Done, 2026-09-28**, for everything this task's acceptance names.
**Depends on:** 1, 5.

**The pinch is done too, 2026-09-28**, and it was a protocol rather than a line
beside the wheel: `zwp_pointer_gestures_v1` is advertised now, so a pinch also
reaches an application that wants one. On the plane it zooms **continuously** —
the scale a touchpad reports is measured against the moment the fingers went
down, so the canvas is that starting zoom times that scale, all the way through
the gesture. `crate::canvas_pinch`.

`alo-desktops`' recogniser already answered `Intent::Zoom` and the shell already
refused it; that refusal stands, under a new reason. It answers **once, at the end
of the gesture**, and a canvas has to move while the fingers do — so carrying it
out as well would spend one pinch twice. Its test is renamed from
*because nothing here can* to
`a_zoom_is_not_carried_out_because_the_canvas_already_did_it`.

**And the rough edge found while building it is fixed rather than named: the shell
sees a modifier with nothing focused.** `Server::keyboard_key` returned before
`KeyboardHandle::input`, which is what advances xkb, whenever `current_focus()`
was `None` — so Ctrl+wheel over an empty canvas panned. A compositor has to know
what is held to answer for its own gestures; only *forwarding* depends on focus.
The key now always reaches xkb and reaches a client only when one is focused.

That split *pressed* into two sets and both had to be kept: `pressed_keys` is what
is physically down, `Keyboard::forwarded` is what a client was told about, and the
two questions that meant the second — *is an application holding a chord*, *is this
key owned by ordinary client routing* — now read it. The release on a focus change
narrowed with it, or somebody holding Ctrl over empty canvas and then focusing a
window would have had it released underneath their own finger.

**Still not done, and still gestures:** task 5's space-and-drag and the trackpad
two-finger pan.

Pointer-centred, so the point under the pointer stays under it. A documented
modifier with the wheel, a pinch, and a keyboard route. **Show all** fits every
frame on screen, and it is the moment the film exists for.

- **Acceptance:** the point under the pointer is unchanged by a zoom, within a
  pixel; *Show all* leaves every frame inside the viewport with no frame
  clipped; zoom has a keyboard route.
- **Constraint:** the extent *Show all* fits is the frames' own, computed from
  them. A plane with declared bounds that the frames can sit outside is how the
  design file's own fit would have been wrong.

### 7. Every canvas answers as a list

**Status:** **Done, 2026-09-29**, all three thirds. **Depends on:** 2.

The accessibility half. The frames are enumerable in a stable order with their
names, reachable and focusable by keyboard alone, and what a reader is told does
not depend on where a frame happens to sit.

- **Acceptance:** with no pointer at all, every frame can be reached, focused
  and named; the order is stable across a pan and a zoom; the existing reader
  modules answer about a frame on the plane as they do about a window today.
- **Constraint:** this is not a second interface. It is the same frames, said in
  order.

#### Reached and ordered, and blocked on a frame having a name (2026-09-27)

**Held**, by `crates/alo-shell/tests/every_frame_answers_as_a_list/mod.rs`, and the
constraint held it down to almost nothing new: the shipped `NextWindow` and
`PreviousWindow` chords are the road, `switch_window`'s ring is the order, and the
cases prove **a canvas cannot change either**. Focus is read as each client's own
`wl_keyboard.enter` rather than asked back of the compositor — *focused* means the
application was told, and a compositor that moved its own idea of focus silently is
the first failure a screen reader meets. Three frames are reached with **no pointer
enabled at all**.

One correction to an overclaim worth keeping: a stability test across a pan and a
zoom catches **less** than it looks. Both are a translation and a positive scale, so
a plain sort by where frames sit is monotonic under them and would pass untouched.
What that case catches is banded reading orders, where a zoom changes which frames
share a row, and anything measured from the viewport. The failure a person meets is
a different one — an enumeration that reaches **only what is on the screen** — so it
has its own case: a frame half a million units away is still reached and focused.
Proved to bite by skipping frames past the viewport in `switch_window`, which fails
that case alone.

#### Named, and no longer blocked (2026-09-29)

**Done.** `crate::frame_name` reads `xdg_toplevel.set_title` and `app_id`, and
`ReadAloudTree::with_the_frames_open` hangs one node per open frame under the
desktop's *the windows open*, which had been a list with nothing in it — a reader
was told this machine has a list of windows and never what was in it.

**A frame's name is the only name in that tree a translator never sees.** Every
other one is an `alo_access::words::Word` put through the person's language; a
frame's is whatever the application called itself, already in whatever language it
chose. The one case that is ours to word is an application that set neither a title
nor a class, and that is `alo_access::words::AN_APPLICATION` — the phrase the tree
already used for a window nobody named.

Three tests: all three ways a name can arrive; that a reader hears the windows in
the same order the keyboard walks them, which is task 7's *not a second interface*
constraint made checkable; and that a window which closes stops being read, so a
reader is never offered one that is not there.

**What is still owed here is not task 7's.** `ReadAloudTree` has no production
caller — only tests build it — so this is a tree that answers correctly and that
nothing serves yet. That, and the four EN 301 549 clauses downgraded on 2026-09-29,
are task 8 of `docs/autonomy/access-and-language-plan.md`.

#### The blocker as it stood (2026-09-27)

**Blocked, and it is a missing capability shared with task 3: a frame has no
name.** `xdg_toplevel.set_title` is not read anywhere in `alo-shell` — the only
labels it draws are the three window controls' own — so *every frame can be reached,
focused and **named*** cannot be finished. The same missing name is task 3's handle
(ADR 0065: *the name is the handle*) and ADR 0065's *frames carry a name, shown when
the canvas is far out*. **One capability unblocks the second half of task 3 and the
last third of task 7**, and it is the first thing to build next: read the toplevel's
title and app_id, hold them per frame, fall back where an application gives neither
(`window_control_name_fallback` already answers that question for controls), and say
it where a reader asks.

### 8. A frame is never lost

**Status:** **Open.** Dock occlusion and drag-extent protections pass their current
tests. **That is evidence of progress, not completion** — the owner's words of
2026-09-30, and the reason this line does not say *partly*. **Depends on:** 3, 6,
and on the minimized-window panel, which is the third PC's and in progress.

**This line said "Done for the dock… open for the status area" for about an
hour, and the supervisor refused it in nought seconds.**
`tools/kernel-loop/src/plan.rs` reads the first word of a status, saw *done*,
found no `**Done, <date>.**` mark and said the loop would take the task up again.
It was right twice over: the line was malformed *and* the task is genuinely
unfinished, so being taken up again is the correct outcome rather than the fault.
A field with two states was the wrong place to encode *partly*, and the blunt
answer is the true one — **this task is not done.**

**The measurement that decided the shape of it.** *Show all* fits the frames' own
extent and the widest extent it can fit is bounded by `Zoom::FURTHEST_OUT`: on a
1280×720 output, 25,600 × 14,400 plane units. The plane reaches ±1,000,000 —
**seventy-eight times further across than the fit can hold.** So *a frame dragged
far away is still found by Show all* was not something the canvas had; it is
something a rule has to keep, and the first draft of this task missed it because
the acceptance sentence reads like a property rather than a promise.

#### Where the status area goes — settled by the owner, 2026-09-30

ADR 0076 took *at the far end of the dock* off that promise and handed the question
**to this plan**, and the plan carried it unanswered. The owner has answered it:

**The status area is fixed at the top-right of the viewport**, separate from the
canvas, the Dock and the minimized-window panel. **The right panel starts below
it.** Its contents may change — clock, battery, network, volume, brightness — and
**the shell always knows its current bounds**, which is what makes it something a
frame can be kept out of rather than a promise with no geometry.

It is a viewport control, so task 1's separation already says the rest: a child of
the viewport and never of the plane, so panning and zooming cannot move it.

#### The rule this task holds is stronger than *not entirely covered*

Also the owner's, 2026-09-30, and it replaces *entirely under the dock*:

**A frame must retain a usable portion of its name band outside every fixed
control** — the Dock, the status area, and the **expanded** minimized-window panel.
**One exposed pixel is technically reachable and practically lost.** So the
protection preserves at least a **44 × 24 logical-pixel drag target**, scaled with
the accessibility settings — `alo_access::TurnedOn::larger_text` answers the scale,
and a person who has made everything larger has made this larger too.

The three controls are a set rather than a list to extend by hand: a fourth fixed
control added later must join it, because *outside every fixed control* is the
promise and *outside the three we thought of* is not.

#### Output changes are a dependency, not an afterthought

A position valid on a large display becomes unreachable after switching to a
smaller one, after the interface scale goes up, or after the right panel expands.
**Reachability is rechecked when those bounds change.** Where recovery needs a frame
moved, **the move is shown and its previous position recorded** — a frame that
silently relocated itself is a person's arrangement edited without them.

**Only two ways a frame can actually be lost, once panning exists.** A frame above
the viewport, behind another, or off to one side is reached by panning. What
panning cannot undo is being further out than the fit reaches, and being entirely
under the dock — the latter because ADR 0071 makes the name the only handle, so a
name under the dock is a frame nothing can pick up. *Entirely* is the condition: a
band half out is one a pointer can still land on.

**It stops a drag rather than undoing one.** A drag crossing either line keeps the
last position that did not, which is what an edge feels like. A compositor that
accepted the drag and took the window back afterwards would be one whose windows
fight the person holding them.

Nothing may be placed, dragged or restored where a person cannot get it back:
not off the plane's reachable area, not behind a viewport control, not at a zoom
where it cannot be seen.

- **Acceptance**, and all four are owed before this closes — the owner's list of
  2026-09-30:
  - ***Show all* can include every frame within the supported zoom range.**
  - **Fixed controls cannot cover every usable drag handle** — the Dock, the status
    area and the expanded minimized-window panel, at least 44 × 24 logical pixels
    of name band left reachable, scaled with the accessibility settings.
  - **Display, scale, Dock-position and panel-state changes preserve recovery**, and
    a recovery that moves a frame shows the move and records where it was.
  - **Keyboard users can find and move a frame without reaching its name band.**
- **What is held today, corrected 2026-10-05:** the first, **the second entire**,
  and **the first half of the third**.

  *This read "the first, and the Dock's share of the second" until today, and it
  was behind the code by two clauses.* **The second is complete:** every fixed
  control is held and compared, not just the Dock's band —
  `FixedControlsDrawn` carries the band, the panel's reserved column, the status
  area and the top controls, and `direct_desktop.rs` fills all four from the
  frame that laid them out, so a fifth control cannot join the set without being
  wired from the draw. **And the first half of the third is built:** that same
  draw records the controls and, when they differ from the frame before, brings
  back the frames they now hide — `bring_back_frames_the_moved_controls_hide`
  has a production caller. *No reachability is rechecked when bounds change*,
  which this line used to say, stopped being true when that landed.

- **What is left of the third is one sentence reaching a person, and it is not
  blocked on what this plan said it was.** The record exists —
  `Recovery::BroughtBack` carries where the frame was — and the telling does
  not.

  *Until 2026-10-05 this plan argued the telling needed
  `alo_notifying::arriving::from_alo_os`, that alo OS has no production
  notification anywhere in the tree, and that the notifications portal's
  `[v0.5]` tier therefore **forbade** building it. That argument was wrong, and
  it was corrected a day earlier in the code rather than here* —
  `canvas_fixed_controls.rs`'s own note says it named the wrong road, and
  `docs/features.md:497` settles it in the promise's own words: **when the
  machine moves a window, the person is told** is `[v0.01]`, and it says
  outright that it is *not the notifications portal and not calm
  notifications — both stay where they are, because they are the ordinary
  notification system and the canvas needs one sentence about one frame.*

  So the tier does not forbid it; the tier **asks for it**. One sentence about
  one frame is this crate's own kind of surface, like the egress indicator, and
  `egress_status_place::Place::of_the_other_end` already records where such a
  thing goes.

  **What is genuinely undecided is narrower, and is the only part worth
  carrying forward:** how *where it was* reaches a person. `alo_canvas::At` is
  `{ x, y }`, no crate on the canvas side carries any vocabulary, and a
  coordinate read aloud is not something anybody can act on. That is a question
  about words, not about tiers or portals.

  *Kept as a correction rather than deleted, because the mechanism is the
  reusable part: a blocker was argued from a tier, the argument was repaired in
  a code comment a day later, and the plan went on carrying the original for a
  further day. A reason that moves needs to move in both places, and the one
  nobody re-reads is the plan.*

- **The fourth is unstarted *and* unscoped, and those are
  different things.** *Keyboard users can find and move a frame without
  reaching its name band* **cannot be held by any test, because there is no
  keyboard road to move a frame at all** — ADR 0065's *every one of them has a
  keyboard form* covers zoom, pan, fit, fill and work-inside, and
  `docs/features.md`'s *each with a keyboard form* attaches it to the same three, on
  the **`[v0.01]`** promise *Frames, dragged and resized like a design canvas*.

  *This read `docs/features.md:422` … at `[v1]` until 2026-10-03, and was wrong twice:
  the phrase is at 443, and the promise moved to `[v0.01]` on 2026-09-30 when the owner
  put the full canvas experience into the current release.* **The conclusion is
  unchanged and that is the point of recording it** — the argument is about *what the
  phrase attaches to*, which is fit, fill and work-inside and not dragging, at whatever
  tier. A citation can be wrong in its line and its tier while the claim it supports
  stands, and a reader who checks the line and finds transcripts has no way to tell
  which.
  Satisfying it means **new v0.5 capability**, which `docs/features.md` gates and
  which is the owner's to grant.

  Said this plainly on purpose. *An acceptance condition naming a road that does
  not exist reads as almost-done when it is not started* — and this plan has
  already produced that fault twice, in task 5's *all three roads* and in task 8's
  own dropped half. An unticked box invites somebody to think it is nearly paid.

  **Half-built and unscoped differ in who can close them.** The third is this
  plan's to build, and since 2026-10-05 the question is only how *where it was*
  is worded — not whether the tier allows it. The fourth is the owner's to grant, and the reason is worth keeping in
  general form — the laptop lane's, 2026-09-30: **an acceptance condition cannot
  quietly promote a promise from v1 to v0.5. If it could, the roadmap would be
  editable by anybody writing a test they cannot pass yet.**

### 9. The canvas is where they left it

**Status:** **Open, reopened by the owner 2026-10-03.** **Depends on:** 1, 3, 6.

> **The owner's words: *do not weaken the promise.*** The in-memory arrangement work is
> complete; restart persistence is not. This status is corrected rather than the promise
> reduced, and everything the task said on 2026-09-29 is kept below because it is still true
> of the part that was built.

**What was `Done` and what was not, measured by the panel lane and confirmed here on
2026-10-03.** The arrangement is assembled in memory and used in production —
`alo-shell/src/canvas_remembered.rs` holds an `Arrangement`, puts a frame back where it was,
and can produce `the_arrangement_now()`. What does not happen is any of it reaching a disk:

```text
Arrangement::written() / Arrangement::read()   no callers anywhere
the_arrangement_now()                          4 callers, all in one test file
WhereTheyLeftIt (the production type)          constructed only in tests
any file path for a canvas arrangement         none exists
```

So **a person's layout does not survive a restart on a real machine**, and this read `Done`
for four days. *A caution for whoever checks this: `alo-displays` has its own `arrangement`
— the physical layout of monitors — and that one **is** properly kept and tested in
`alo-displays/src/keeping.rs`. Searching for "arrangement is the arrangement kept" lands
there and answers yes about the wrong thing. Two different things, one name.*

**What the implementation must do, by the owner's ruling of 2026-10-03:**

- Persist **Place identity, frame identity and geometry, each window's presentation state,
  and each Place's camera**.
- **Preserve normal geometry** when a window is minimised, compacted or full screen — so a
  window comes back the shape it really is, not the shape it was last seen in.
- **Connect saving and loading to the actual production session lifecycle**, which is the
  integration that was missing and the reason this was not done.
- **Save after meaningful layout changes**, not only on a clean shutdown — a machine that
  loses power is the case this promise is for.
- **Versioned, atomic writes** in the existing private per-user state location, handling
  missing or damaged state safely.
- **Restore frames so they stay reachable** when displays or scaling change — task 8's own
  question, which this task already offers a remembered place to.
- **Restoring a layout must not silently restart agent work or restore expired permissions.**

**Call it canvas layout persistence** in documentation and tests, kept clearly separate from
monitor arrangement persistence.

- **Acceptance, by the owner's ruling:** arrange windows across several Places, change their
  cameras, end the session, **start a fresh process**, and observe those layouts restored
  **through the production path**. Include **interrupted-write** and **changed-display**
  tests — not only serialization round trips.
- **Its own change:** the owner ruled that persistence gets its own integration change and
  its own restart evidence, separately from the grouping and Stop work.

**Built on 2026-10-03, and here is what a person gets.** Arrange windows, move the camera,
end the session, begin another, and the windows are where they were left — through the
production path, not a round trip in memory.

```text
alo-kept           a public atomic text write, so a crate with its own validated
                   format rents the sibling-sync-readback-rename discipline
                   rather than writing a second copy of it
alo-arranging      each window carries its geometry AND how it was showing;
                   a keeping module names canvas-layout.toml, and a layout
                   reaches the disk only if the bytes read back as that layout
alo-shell          asks the desktop for the layout once, offers every arrived
                   frame its remembered place before the frame is drawn, and
                   tells the desktop when the arrangement has actually changed
alo-desktop        reads and writes the file, because the shell shows and never
                   measures and a file is a reading like the battery
```

**Two halves were not built, and neither was hidden behind a passing test. One is built now; the other needs a machine.**

**One: the ordinary geometry of a window that was not ordinary. Built 2026-10-04.** The
ruling asks that a window which was maximised, filling the screen or in a share comes back
*at its ordinary size*. The state survived and **the size it would return to did not**: the
shell's ordinary geometry is in **output** coordinates while an arrangement is in **plane**
coordinates, and converting needs the camera as it was when the mode was entered, which
nothing keeps.

**Built where this note said it would be, and without needing that camera.**
`window_mode_plan` captures the frame's plane rectangle directly, from the buffer origin
that is already in plane units — `scene::trees` applies the camera when it *draws*, so a
rectangle taken this way is comparable at any zoom. `canvas_remembered` reads it back through
`where_it_was_as_its_ordinary_self`, and the arithmetic is `canvas_show_all::a_rectangle`'s,
shared rather than repeated.

**Captured each time the frame leaves `Normal`, not once ever** — which this note did not
say and which matters: un-maximise a window, drag it across the canvas, maximise it again,
and the rectangle it should come back to is where the person just put it. A capture-once rule
would have restored it to where it sat before the drag.

**What it was doing instead, measured by reverting the fix:** a maximised window was written
down as `((300, 200), (900, 600))` — the maximised rectangle — where its ordinary self was
`((300, 200), (16, 16))`. The origin is identical in both, so a check on position alone would
have passed throughout. `AWindowWas`'s field has promised *always the ordinary geometry,
never the geometry it was last drawn at* since it was written, and its only producer was
handing it the geometry it was last drawn at.

*The first version of that test passed with the fix reverted.* `set_maximized` over the
protocol does not move a frame's plane rectangle on its own — the fixture commits a fixed
16×16 buffer — so the remembered and live rectangles were the same number and the test could
not tell the fix from its absence. It now forces the difference with a viewport destination
and **asserts that difference exists** before concluding anything from it.

**Two: the process boundary.** The acceptance says *start a fresh process*. The restart
evidence runs two sessions that share **nothing but the file on the disk** — the layout
leaves one through `the_arrangement_now` and the keeping module, and arrives in the other
through `at_sign_in` and the put-back — which is every step of the road except the fork. A
genuinely fresh process needs a machine that boots to the compositor, which is the installer
plan's ground. The test is named for a *session* rather than a *process* so that its name
does not claim the half it does not do.

**A third thing was the owner's to decide and is now decided.** `Arrangement::read`
refuses a file whose version is not this one — strict equality, not a floor. Raised by the
panel lane reading this change, and the distinction was theirs: **a file written by an older
version of our own format is not wrong, it is old**, and this one check treats them
identically. ADR 0038's *a file that is there and wrong is refused whole* is about wrongness.

**Settled by the owner on 2026-10-04, in these words: _old rules that were modified are
wrong._** So an older version is not merely old — it was written under a rule that has since
been modified, and reading it means interpreting bytes under rules they were not written to.
`FORMAT`'s own note already forbade exactly that (*a later version may refuse this one
outright; what it may not do is read it and be wrong about what it means*), so the ruling
confirms a stance that was written down and that the panel lane was arguing against rather
than into a gap. **The check stays strict**, and the argument and the ruling are both kept
beside the constant so the next reader does not re-open it.

**And one measurement in that constant's note had gone stale, which is what makes the ruling
bite.** It said *nothing in this repository writes this file to disk … so there are no files
in the world to break*. True on 2026-09-30, false since 2026-10-03: `keeping::keep` writes
`canvas-layout.toml` and `alo-desktop` calls it. **Version 3 files exist on real machines**,
so a bump to 4 discards a person's remembered arrangement — once, visibly, with the reason
said and the next change replacing the file. Bounded, and no longer free.

Today the cost is nothing, because no such file exists anywhere — `written` has had no
production caller until this change. **The first release that ships one makes every later
format bump discard a person's arrangement:** windows back at the plane's origin, the camera
reset, nothing said, and from their side nothing went wrong.

There are three answers and choosing between them is a product decision rather than a lane's:

1. **Refuse**, as now — a format change costs everyone their layout, once, silently.
2. **Read what still parses** — keep the Places and windows an older shape can still be
   understood as, and lose the rest.
3. **Migrate** — carry an older file forward on read, which means keeping every shape this
   crate has ever written.

Nothing here is built for 2 or 3, and the check is not changed, because the answer decides
what a person loses. **It is cheap now and stops being cheap the day v0.01 reaches somebody
with a `canvas-layout.toml` on their disk.**

**And the interrupted-write test the owner asked for by name is here**, as is the
changed-display half's honest position: a damaged or half-written file leaves the previous
layout readable and names what was wrong without stopping the session, and reachability when
a display changes is task 8's own question, which a remembered place is already offered to.

*Everything below was written when this read `Done`, and is kept because it describes the
half that is built:*

`crates/alo-arranging` owns the file and **decides nothing about a canvas**: every
value read back is rebuilt by `At::checked`, `Size::checked` and `Zoom::of`, the
same road a live canvas takes, so a file hand-edited to a place off the plane is
refused by the crate that owns that rule. There is deliberately no `Deserialize`
for a `Camera`, because a position read back unchecked is a position nothing
validated.

**A window is remembered by what it is, not by which one it was.** A `wl_surface`
does not survive a session, so the key is `app_id`. That has a consequence worth
stating rather than discovering: **two windows of one application share one
remembered place**, and the first to open claims it. A title would have been the
alternative and is worse — a person renaming a document would find the window
somewhere else tomorrow.

**A remembered place is offered to task 8's own question before it is used.** A
file written on a wide display can hold a place that is off a narrow one, and a
person signing in on the smaller machine would get a window they could not reach.

**An application that did not come back is absent rather than drawn empty**, and
that needed no code: the arrangement is a list of places waiting to be claimed, and
one nothing claims is never used.

Frame positions and the camera survive a session ending and starting again.

- **Acceptance:** positions and camera are restored exactly; a frame whose
  application did not come back is absent rather than drawn empty.

### 10. The walk, which is also the film's sequence

**Status:** **Done, 2026-09-30.** **Depends on:** 1–9.

*This said **this closes the plan** until 2026-10-01. It does not close it any
more: the owner restored the Dock's edge choice to v0.01 and directed the work
into this plan, which is task 11 below. The closing claim is kept rather than
deleted, because a reader who remembers this plan as finished needs to see what
reopened it.*

Two halves, as `the-shell-plan.md` task 14 established for the shell's own
walk. The **raster** half is
`crates/alo-shell/examples/support/the_canvas_walk_check.rs`, run as the
`canvas-walk` sub-mode of `the_nested_fixtures` — ten steps against a real
headless-weston parent, each read back off the frame it drew. The **sequence** half
is `crates/alo-shell/tests/the_canvas_walked.rs`, which holds the walk's own order
against the table in `docs/autonomy/updates/the-canvas-walked.md` and runs on any
machine.

**Four keyboard steps follow six pointer steps rather than six.** Dragging and
resizing a frame have no keyboard form in v0.5 and none is promised: ADR 0065's
*every one of them has a keyboard form* attaches to zoom, pan, fit, fill and work-
inside, and `docs/features.md`'s *each with a keyboard form* attaches it to the same
three at
**[v1]**. `docs/design/the-shortcuts-and-the-edges.md` gives `Alt + F7` and
`Alt + F8` under *Working with windows*, not under *Moving around the canvas*, and
no `alo_shortcuts::Action` exists for either. The table names the absence in the
row a person would look for it in, and a test requires that row to be there —
because this plan's own history is of acceptances half-met where the missing half
was true but unstated.

**What the walk found while being written**, each recorded in the report: a wait
that does not dispatch is a deadlock; a refusal that returns early takes its reason
with it; and **Show all fits rather than zooms out** — it took the camera *in*, to
168 per cent, because three small frames clustered on a large output are fitted by
magnifying them. The first check demanded *further out*, having encoded the
acceptance's phrasing instead of the behaviour.

One walk: open three applications, drag one, resize another, pan, zoom out to
*Show all*, zoom back into one and work in it, then the same by keyboard alone.
A raster at each step and the exact sequence recorded as a table the test reads
out of the published report.

- **Acceptance:** the walk produces a raster at every step, and the table is read
  from the report rather than a copy, so a step that changes without the table
  fails.
- **Constraint:** the report says what the raster is evidence of — what this
  compositor drew, and not what a display showed — unless it was run on a
  machine with one.

### 11. The Dock at any edge a person chooses

**Status:** **Open, and no longer blocked — the designs arrived.** **Owner:** the
`alo-dock` lane — this PC, lane B, under the settings plan's row in
[the lane table](a-new-machine-becomes-a-lane.md). **Written 2026-10-01 by the
owner's direction; unblocked 2026-10-08.**

**What changed.** This task said *blocked on design … which do not exist*, and
that was true when it was written on 2026-10-01. **It stopped being true without
anybody noticing**, which is what a blocker recorded as prose does. The
committed snapshot `docs/design/figma-snapshot/70-28.xml`, refreshed 2026-10-07,
holds **fourteen frames** for the three edges:

| edge | resting | overflow | full screen · concealed | full screen · revealed | shared edge | composer open |
|---|---|---|---|---|---|---|
| Left | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Right | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Top | ✓ | ✓ | ✓ | ✓ | ✓ | — |

**Six variants each, not five.** An earlier version of this table listed five and
left out *full screen · concealed*, because it was built from the frames that
embed a `Dock / Left` instance rather than from the set itself. Corrected by
enumerating the parent.

**And the absence for Top is proven rather than likely.** The parent is a Figma
**section**, `351:25693`, *Dock edges · v0.01 specification*, with **37 direct
children**: a title, a subtitle, seventeen captions, seventeen frames and an
*Implementation contract*. Enumerated in full, so *Top has no composer variant*
is a complete count rather than a search that found nothing.

None carries a `hidden` flag, and the side variants are **70 wide and 391 tall**
— a band laid out for a vertical edge, not a horizontal one rotated, which is
the thing *both orientations* exists to require.

**The hardest half of the blocker is answered by name.** This task said *the side
variants must carry a usable alo Bar interaction of their own rather than the
horizontal composer rotated*, and **`Dock Left / Composer open` and `Dock Right /
Composer open` are drawn.** The owner's specification says the same thing in
words — §11, *the alo composer opens horizontally into available space rather
than becoming a rotated text field*.

**And the behaviour each variant has to keep is now specified** in
`docs/design/the-canvas-as-a-workspace.md`, which did not exist when this was
written: §11 for overflow and for returning a person to their windows, §10 for
the reveal path and its *continuous pointer path … moving towards an icon must
not make it disappear*, §12 for the shared edge — *the panel retains its
designated handle and opens its previews inward*.

**Two questions remain for the owner, and neither blocks a start:**

1. **Is `Full screen · revealed` also the hover-revealed case?** This task asked
   for both *hover-revealed — the band revealed over a window that covers it*
   and *full-screen*. The bottom edge has `Dock 02 · Hover target` and `Dock 11 ·
   Full screen edge` as separate frames; the three new edges have one frame
   each. Either the two cases are one at these edges, or one is still owed.
2. **Does Top need a `Composer open`?** The other two have one because a vertical
   edge cannot hold a horizontal composer. A top dock is already horizontal, so
   it may need nothing — but that is the owner's to say rather than this lane's.

`docs/features.md` promises it at **[v0.01]**: *the bottom edge by default, and
the person may choose bottom, left, right or top*, working **in both
orientations rather than being a horizontal bar someone turned sideways**.
[ADR 0076](../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
withdrew that promise on 2026-09-29 and the owner reversed the withdrawal within
a day; bottom is now the default rather than the only.

**Measured before it was written, and it is a removal rather than a gap.**
`alo-dock` has no edge at all. `crates/alo-dock/src/changes.rs` says in its own
words that *there is no `edge` field and no `displays` field*, and two of its
tests assert that a `dock.toml` naming an edge loads with the edge **ignored**
rather than refused, so a file from an earlier release still reads. `along.rs`,
which held the two orientations, is deleted. **So a reader checking whether this
is built finds tests passing, and they pass because it is not** — those two
tests are what changes when this is done.

- **Acceptance:** a person may put the Dock on the bottom, top, left or right,
  and it is laid out **for** that edge rather than rotated into it — a vertical
  band is not a horizontal one turned sideways. Labels, the overflow area and
  the reveal path all behave at every edge, and `alo_dock::revealing`'s regions
  are read from the surface's own edge rather than assumed to be the bottom.
  Where the Dock and the panel of put-aside windows would claim the same edge,
  the collision is resolved by a rule in the code and tested at every pair, not
  left to whichever is laid out second. A `dock.toml` that names an edge is
  **honoured** rather than ignored, and the two tests asserting the opposite are
  replaced in the same change.
- **Constraint:** every figure is derived from `alo_dock::measures` and nothing
  from the design file is copied into code — the source check
  `no_figure_from_the_design_reaches_the_code` in `alo-shell` already forbids
  it for the panel and this is held to the same rule. The reveal keeps **no
  timers**, which `docs/features.md` promises at v0.01 for the reason that a
  timer makes the behaviour depend on how fast somebody can move. Per-display
  placement stays at **[v0.5]** and is not in this task.
- **Nothing is owed before this can start.** The designs arrived; see below.

#### The designs this task waited on, and what arrived

**This section said the left, right and top variants do not exist.** That was
measured and true on 2026-10-01. The snapshot refreshed on 2026-10-07 has all
three, so what follows is kept as the list of what each variant needed, with
what arrived against it:

- **resting** — the band at that edge with nothing hovered
- **overflow** — more to show than the edge has room for
- **hover-revealed** — the band revealed over a window that covers it
- **full-screen** — what a window filling the screen leaves of it
- **shared-edge** — the Dock and the put-aside panel claiming one edge

**The side variants must carry a usable alo Bar interaction of their own rather
than the horizontal composer rotated.** A composer laid out along a short
vertical edge is not the same control, and rotating it is the thing the *both
orientations* clause in the promise exists to forbid.

**That reasoning stands and is now satisfied rather than withdrawn.** Building
the other three from the bottom variant would have been this lane deciding a
design, which the fifth law gives to the design rather than to the code. The
three were drawn instead, which is the right way for a blocker like this to end.
