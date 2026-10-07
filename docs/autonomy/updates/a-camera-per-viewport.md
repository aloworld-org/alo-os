# A camera per viewport

**Mac lane, 2026-10-06.** `docs/autonomy/more-than-one-display-plan.md` task 7,
the last task in the plan.

## What was wrong

Every display was handed the session's one camera. Tasks 4 and 5 had made the
*furniture* per display — a frame each, a dock each at its own scale — but
`render_frame` ended with `target.look_at(self.surfaces.camera)`, one value
for all of them. **Two displays were two views of the same thing.** Whatever
else had been made per display, the result was mirroring.

## What changed

`Surfaces.cameras` is a map keyed by display name, as `presentations` and the
per-display fixed controls already are. Each display is drawn through its
own, and a display nobody has panned looks at the origin at life size — which
is what every display looked at before any of them had a camera.

All **six** assignment sites go through one writer, `Server::set_the_camera`,
so *which display did this change* is answered in one place. `the_camera()`
answers for the display the person is working on: the one under their
pointer, or the main screen, because a keyboard command has no position and
the main screen is where the arrangement says a new window opens.

### The circularity, and the way out

`scene::trees` needs a camera to say where a surface sits **on a screen**, and
the camera belongs to the screen that surface is on. Asked in that order it
cannot be answered.

It is not circular, because a surface's place on the **plane** does not depend
on any camera. So the screen is chosen from the plane position and the camera
follows. The popup path therefore takes no camera argument at all any more.

## Three defects, all found by tests that already existed

**None of them was visible in the diff**, and that is the part worth keeping.

**One: the forbidden shape, rebuilt.** The first version paired each display's
rectangle with a *copy* of its camera, written once a frame. That is exactly
what this task's constraint forbids — the duplicate collapsed on 2026-10-04,
one camera with a second home kept in step by hand — and it had a comment
defending it. Four integration tests caught it, because a copy written per
frame is **stale before the first frame**: a pointer pressed before anything
was drawn converted through a camera nobody was looking through. The
rectangle now carries the display's **name** and the camera is read from its
one home at the moment it is needed. A rectangle may be copied per frame; it
is what the arrangement says and does not change under a pan. A camera may
not.

**Two: a key written in one place and read in another.** With no arrangement
there is no display to name, so the writer used the empty name while
`render_frame` read the display's actual name. The pan went somewhere nothing
looked. `one_plane_under_one_viewport::a_panned_session_hands_its_camera_to_the_display`
caught it, and its assertion message — *the session panned and the display was
never told* — was written long before today.

**Three: a guard I blinded.** `the_camera_has_one_home.rs` scans for types
holding a camera field and asserts both that none is unexpected **and that the
count has not dropped**, with a note that *fewer is the dangerous direction:
it means the scan stopped finding them.* Renaming `camera` to `cameras` made
its scanner miss the one real home, taking it from four to three. Without that
second assertion the rename would have silently disarmed the guard protecting
this task's whole constraint.

The scanner now matches the camera **type** rather than the field's name,
which is strictly broader than what it replaced and so cannot have stopped
catching anything. The tempting fix — adding `cameras: ` beside `camera: ` —
would have patched this instance and left the same blind spot for the next
rename.

## What is tested, and where it runs again

Five tests in `a_camera_per_viewport_tests.rs`, on every build:

- two displays hold two cameras, and panning one leaves the other where it was
- a write lands on one display only
- a display nobody has panned looks at the origin at life size
- one display reads back what was written
- with no pointer, the main screen is the one being worked on

Plus the three guards above, which now protect the constraint rather than
merely appearing to.

Gated on one tree: `FMT=0`, **workspace-wide** `CLIPPY=0`, `SUITE-EXIT=0`,
651 lib tests.

## What this does not do

- **`On the machine.` is not ticked**, and this is the ninth task in a row.
  Two displays have never been lit. This plan said so in its first paragraph
  before any of it was written: *its code halves can close here and none of
  its machine halves can.*
- **Hotplug is still not detected.** Displays are discovered once, at session
  start.
- **The desktop raster is per display; the canvas arrangement is one.** That
  is the acceptance — *neither display is a second desktop with its own
  arrangement* — and it holds because frames live on the plane, which no
  camera moves.
