# The canvas, walked

**2026-09-30, the Mac lane.** Task 10 of
`docs/autonomy/the-smallest-canvas-worth-showing.md`, and the last task in that
plan: *one walk — open three applications, drag one, resize another, pan, zoom out
to Show all, zoom back into one and work in it, then the same by keyboard alone. A
raster at each step and the exact sequence recorded as a table the test reads out
of the published report.*

## What the raster is evidence of, said before the table rather than after it

**What this compositor drew, and not what a display showed.** The parent is
`weston --backend=headless` and the renderer is llvmpipe, inside a Lima VM on an
Apple M3. Every step submits a frame through that parent's own EGL and reads back
the pixels the frame was painted into before the buffer is swapped away. **No
panel has shown any of it.** That is the task's own constraint and it is repeated
here rather than left to be inferred.

**And the pixel counts are weaker evidence than they look.** A 16×16 window moved
across the plane paints exactly as many pixels where it lands as where it left, so
*832 painted* before and after a drag says nothing about the drag. Three of the
first four rows below report 832 for that reason. What carries the behaviour is the
assertion beside each step — the frame's own origin moved, the press was not
delivered to a client, a second resize was refused while one was held — and the
raster's job is the narrower one it can actually do: **this step drew something
rather than nothing.** A step whose surface refused, or was laid out off the
output, comes back as an empty picture and the fixture says so.

## The walk, step by step

| # | Step | What the step proves beyond drawing |
|---|---|---|
| 1 | three applications open on the canvas | three clients mapped and placed apart, so *Show all* has an extent to fit |
| 2 | one of them dragged by its name | the frame's own origin moved, and the press on the shell's band reached no client |
| 3 | another resized from its corner | a press on the corner band began a transaction — a second resize was refused while it was held, and accepted again once the button was up |
| 4 | the canvas panned | the camera moved and the frames did not |
| 5 | zoomed out to show all of it | every frame's two corners land inside the 1366×768 viewport, none clipped |
| 6 | zoomed back into one and worked in it | a key reached the focused application at life size, which is the refusal task 1 recorded and task 2 removed |
| — | *dragging and resizing by keyboard* | **no keyboard form exists in v0.5 and none is promised** — see below |
| 7 | panned by the arrow keys, with no pointer | the arrow was not handed to any application and the camera moved |
| 8 | show all again, from the keyboard | the shipped chord fits every frame, read from `alo_shortcuts` defaults rather than written here |
| 9 | zoomed back in from the keyboard | the shipped zoom chord moved the camera |
| 10 | a frame reached and focused with no pointer | `NextWindow` selected a frame **and** a key then arrived in it |

Ten steps. The order is `EVERY_MOMENT` in
`crates/alo-shell/examples/support/the_canvas_walk_moments.rs`, which the fixture
labels its rasters from and which
`crates/alo-shell/tests/the_canvas_walked.rs` holds against this table. A step
renamed or reordered in one place and not the other fails that test, which is what
the acceptance's *read from the report rather than a copy* asks for.

## Two of the six steps have no keyboard half, and that is scope rather than a gap

*Then the same by keyboard alone* cannot mean all six, and the walk does not
pretend it does.

**Dragging and resizing a frame have no keyboard form in v0.5, and none is
promised anywhere.** ADR 0065's *every one of them has a keyboard form* attaches to
its own list — zoom, pan, fit the Place, fill the screen with what is selected,
work inside a frame. `docs/features.md:422` attaches *each with a keyboard form* to
the same three, and that line is **[v1]**.
`docs/design/the-shortcuts-and-the-edges.md` does give `Alt + F7` and `Alt + F8`
for moving and resizing a window precisely — under **Working with windows**, not
under *Moving around the canvas* — and no `alo_shortcuts::Action` exists for
either.

So the keyboard half of this walk is the canvas's own movement: pan, *Show all*,
zoom, and reaching a frame with no pointer at all. Building `Alt + F7` and
`Alt + F8` to make the walk symmetrical would have been inventing v0.5 scope to
satisfy one sentence's reading, which CLAUDE.md's *scope is gated* forbids and
which is how a minimum stops being one. **The row is in the table above, in the
place a person would look for it, saying what is absent and why.**

## Three things the walk found while being written

**A wait that does not dispatch is a deadlock.** The first version blocked on a
channel until three clients had mapped. Nothing was then answering their
roundtrips, so they sat in a handshake until their sockets reset, and the only
thing printed was a client's panic about a reset socket.
`crate::the_walk_check` already carries the sentence — *the only thread that
answers a roundtrip is this one* — one screen from where the mistake was made.

**A refusal that returns early takes its reason with it.** The same version
propagated its error straight out, so any refusal left three clients waiting the
full two-minute deadline and the server's actual reason was never printed. The
walk now runs its steps inside a closure and releases the clients whatever
happens.

**Show all fits; it does not zoom out.** A check was written demanding that *Show
all* leave the camera further out than it found it, because the plan's sentence
says *zoom out to Show all*. It zoomed **in**, to 168 per cent, and it was right
to: three small frames clustered on a large output are fitted by magnifying them.
The phrasing had been encoded and the behaviour had not. The check is now task 6's
own — every frame's corners inside the viewport, none clipped.

## What this closes

The plan is finished. **Ten tasks published 2026-09-26 and ten at the close, none
added and none dropped** — though two of the nine *Done* lines were found false
while this task was being built, both by going to check what task 10's acceptance
would rest on:

- **Task 5, Pan** said *all three roads* and had two. The keyboard route did not
  exist; it landed 2026-09-30 in `crates/alo-shell/src/canvas_arrow_pan.rs`.
- **Task 8, A frame is never lost** claimed an acceptance naming *the dock or the
  status area* and holds only the dock. Its status now reads **Open**. ADR 0076
  took the status area's location away and handed the question to this plan, which
  never answered it — **that answer is still owed**, and the entry in
  `docs/autonomy/evidence-a-person-can-work-on-it-all-day.md` belongs to the lane that holds that file.
