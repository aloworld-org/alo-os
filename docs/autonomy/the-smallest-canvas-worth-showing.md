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

**Status:** ready. **Depends on:** 2.

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

**Status:** ready. **Depends on:** 1.

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

**Status:** ready. **Depends on:** 2.

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

**Status:** ready. **Depends on:** 3, 6.

Nothing may be placed, dragged or restored where a person cannot get it back:
not off the plane's reachable area, not behind a viewport control, not at a zoom
where it cannot be seen.

- **Acceptance:** a frame dragged far away is still found by *Show all*; a frame
  cannot be left entirely under the dock or the status area; a test asserts each.

### 9. The canvas is where they left it

**Status:** ready. **Depends on:** 1, 3, 6.

Frame positions and the camera survive a session ending and starting again.

- **Acceptance:** positions and camera are restored exactly; a frame whose
  application did not come back is absent rather than drawn empty.

### 10. The walk, which is also the film's sequence

**Status:** blocked — on 1 to 9. **Depends on:** 1–9.

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
