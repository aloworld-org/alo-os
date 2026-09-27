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

**Status:** ready. **Depends on:** 1.

#### What task 1 left for this one, written on 2026-09-27

The refusal above is where to start: `Server::pointer_target` returns `None` at
any zoom but life size. Removing it is not one division, which is why task 1 did
not do it. `crate::scene::trees` answers in screen pixels and Smithay derives a
client's surface-local coordinate by subtracting the focus origin from the
pointer's own location, so the pointer's units have to be decided rather than
converted at the last moment — and `Surfaces::pointer.location`, the window
controls and the resize gesture all read that same stored location and are all in
the **viewport** layer, where a zoom must not reach. Two spaces, named, with one
conversion between them; not a scale factor sprinkled at each reader.

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

**Status:** ready. **Depends on:** 2.

Dragged by its title area, so a press inside the content always belongs to the
application. The frame moves with the pointer at any zoom — a drag of 100 screen
pixels at half zoom moves it 200 plane units.

- **Acceptance:** dragging at three zoom levels moves the frame by the pointer's
  own distance in plane units, and a press inside the content never moves it.

### 4. Resizing, and the application told as it happens

**Status:** ready. **Depends on:** 2.

From the edges and the corners, with the application told its new size while the
drag is happening rather than at the end, and minimum sizes respected.

- **Acceptance:** each edge and corner resizes; the application receives its
  size during the drag; a resize that would go below the application's minimum
  stops at it rather than being refused silently.

### 5. Pan

**Status:** ready. **Depends on:** 1.

Wheel and trackpad over empty canvas, a keyboard route, and a drag gesture that
does not fight editing inside a frame. **Scroll over a frame scrolls that
frame's content; scroll over empty canvas pans.**

- **Acceptance:** each road is exercised and the rule above is held by a test
  that scrolls over both and asserts which moved.

### 6. Zoom, and *Show all*

**Status:** ready. **Depends on:** 1, 5.

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
