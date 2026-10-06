# A popup is constrained to the screen it is on

**Mac lane, 2026-10-06.** `docs/autonomy/more-than-one-display-plan.md` task 6.

## What was wrong

A popup's constraint rectangle was built as `(-origin.x, -origin.y)` with the
output's **size** — a screen assumed to begin where the desk does. That is
true of one display and of no second one.

A parent 80 units into a display beginning at 1920 was told its screen's left
edge lay 2000 units to its left, which is the far edge of the **other**
monitor. So a menu near the inner edge did not flip against anything: it had,
as far as the arithmetic was concerned, another whole screen of room, and it
opened off the display it belongs to.

The size itself came from `popups.output_size`, one global set by whichever
display rendered the most recent frame.

## What changed

**The bound is the rectangle of the screen the parent is on**, corner
included. The `Server` writes every display's desk rectangle into `Popups`
once a frame — told rather than asked, exactly as `output_size` already was,
because `Surfaces` has no `Screens` and giving it one would make every
surface's placement depend on the arrangement. The camera made this same move
from field to argument on 2026-10-04, for the same reason.

**A popup whose display goes is dismissed.** Wired into the *display has
gone* path before the display leaves the desk — afterwards there is no
rectangle left to ask which popups were on it. A session with no arrangement
has one display, so when it goes every popup goes with it; that case was
**wrong in the first draft** and is the reason the end-to-end test exists.

## What is tested, and where it runs again

Eight tests, all on every build.

**Seven on the arithmetic**, against two functions extracted so they could be
checked rather than only reached through a client:

- the main screen is bound **exactly** as before — the safeguard for every
  single-display machine
- a second display is bound to its own corner, right edge included
- a zoom divides the corner as well as the size; dividing one and not the
  other misplaces the edge by exactly the zoom
- a display **left** of the main screen works — `Position` says across is
  negative there
- the parent's own display is chosen; off the desk falls back to the first;
  no screens is no answer

**One driving a real client** through the whole road
(`tests/popups/when_a_display_goes.rs`): a mapped application opens a popup,
the display retires, and the client is asserted to have received exactly one
`popup_done` with nothing left on the server.

That test was written the hard way on purpose. Two cheaper options were
available — state the gap, or widen the public API so a test could reach
`the_display_left` — and both would have shipped the defect described above,
because the real road exercises the no-arrangement case and neither of them
would have. Its two assertions *before* the retirement are load-bearing:
without them it would pass against a compositor that dismissed every popup
the moment it opened.

Gated on one tree: `FMT=0`, **workspace-wide** `CLIPPY=0`, `SUITE-EXIT=0`,
646 lib tests.

## What this does not do

- **`On the machine.` is not ticked.** One laptop, no second display. Ninth
  task running.
- **A true unplug is still not detected.** Nothing watches for a display
  arriving or leaving mid-session; discovery runs once at session start. The
  dismissal is wired to the ways a display can currently go — the session's
  output retiring, and a display whose area changes. Hotplug is a gap this
  plan does not close, and it is named in the plan rather than left for
  whoever first plugs a monitor in.
- **Two displays still mirror.** Every display is handed the same surfaces at
  the same camera. Extending is task 7.
