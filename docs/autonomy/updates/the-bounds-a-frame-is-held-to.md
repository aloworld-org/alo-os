# The bounds a frame is held to are its own display's

**Mac lane, 2026-10-06.** `docs/autonomy/more-than-one-display-plan.md` task
5b, split out of task 5 earlier the same day because task 5's own text
predicted it would be two.

## What was wrong

`Server` held **one** `FixedControls` for the session: one dock band, one
panel column, one indicator band, one handle floor. With one display that is
right and is what shipped. With two it is whichever display drew last, applied
to every frame — so a frame on the second display was held clear of the
**first** display's dock, a rectangle nothing occupies where that frame is,
and left free to sit under the dock that is actually over it.

**The worst of it was a hoist.** `frames_the_controls_now_hide` read the
controls once, above its loop over every mapped surface. That is the line that
made every frame answer to one display's furniture, and it reads as an
optimisation.

## What changed

**One store per display**, keyed by display name as `presentations` already
are, and every consumer resolves which one:

| who asks | what it resolves from |
|---|---|
| `frames_the_controls_now_hide` | each frame, **inside** the loop |
| `somewhere_this_frame_can_be_reached` | the frame it was given, found first |
| the drag path | the window it already holds |
| `what_each_surface_said_at` | the point |
| `at_most_one_surface_claims_any_point` | **every** display, true only if all hold |

That last one is a change of meaning, deliberately: the promise is about a
point, a point is on one display, so two displays are two statements rather
than one. `None` still means *nothing drawn anywhere yet*, which is the
distinction this repository keeps losing and which was checked rather than
assumed.

**One display answers exactly as before.** Where the session has drawn one
display there is one entry and it is returned whatever the arrangement says,
so every caller that never knew about displays keeps its answer and a machine
nobody has arranged behaves as it did. The arrangement is consulted only when
there is a choice to make.

## The ground underneath, which 6 and 7 also need

`Server::the_display_a_window_is_on` answers *which display is this frame on*,
by **greatest overlap** of desk rectangles — the rule a person would give if
asked while looking at the screen. Ties go to the main screen, then to the
arrangement's own order, so the answer never depends on iteration order.

**No new coordinate system was introduced, and none was needed.**
`alo_displays::Position` already says a display's corner is *negative to the
left of the main screen*, so the compositor's single coordinate space already
**is** the desk with the main screen at its origin. What was missing was
anybody asking the arrangement which of its rectangles a point falls in. That
is the whole reason the file is short.

A window on no display at all answers `None` rather than the main screen: a
window dragged off the desk is where the person put it, and answering *the
main screen* would move it in the only sense the question has.

## What is tested, and where it runs again

Eight tests in `which_display_a_window_is_on_tests.rs`, run by
`cargo test -p alo-shell` on every build:

- a frame is held to the controls of its own display — **the acceptance**
- one display answers as it always did
- a window past the first display is on the second
- a straddling window belongs where most of it is
- a window touching an edge shows nothing on it
- a window off the desk is on no display
- a session with no arrangement answers nothing
- a window on the first display is on the first

Gated on one tree: `FMT=0`, **workspace-wide** `CLIPPY=0`, `SUITE-EXIT=0`.

## What this does not do

- **`On the machine.` is not ticked.** One laptop, no second display. Eighth
  task running.
- **The handle floor is still text-scaled and nothing else**, by the owner's
  ruling of 2026-10-01. This task changed *which* rectangles are asked about,
  never how they are measured — which is what its own plan entry warned
  against, after task 5 nearly got it wrong.
- **Two displays still mirror.** Every display is handed the same surfaces at
  the same camera; no display offset reaches the draw. Extending is task 7,
  and this task does not pretend otherwise.
