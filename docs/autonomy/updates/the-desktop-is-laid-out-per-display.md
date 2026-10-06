# The desktop is laid out for each display it is drawn on

**Mac lane, 2026-10-06.** `docs/autonomy/more-than-one-display-plan.md` task 5,
the layout half. Task 4 made every display draw; this makes each one draw
**its own** desktop rather than a copy of the first display's.

## What was wrong, measured

`Desk::present` took **one** size — the loop's own target
(`direct_desktop.rs:579`) — laid the dock, status area and put-aside panel out
from it once, and handed that single picture to every display. A second
monitor of a different shape showed furniture built for its neighbour.

And the scale was worse, because it was a literal. `DesktopFrame::display_scale`
has existed since 2026-10-02 and its only production value is
`crates/alo-desktop/src/main.rs:526`, which hands `100` — one physical pixel per
logical one, true of this laptop and of nothing dense. So a dense display drew
its desktop at half the room it owns, and `alo-desktop` could not have fixed
it: `TheDesktop::now()` returns one frame and does not know which display it
is for. The field's own documentation says where the number belongs — *only
the caller knows which display this is.*

## What changed

**Each display is laid out from its own size and its own scale.** `present`
now lays out a picture per display before any of them is drawn, and each
`Layered` paints its own.

**The scale is looked up, never computed.** `Server::the_scale_of_display`
asks the arrangement the session holds — the person's own, read by
`alo-displays` from their file — and that is the whole of it. A shell that
worked a scale out from pixels and millimetres would be deciding how large
this person's interface is, in a drawing crate, which both this plan's header
and `the-shell-plan.md`'s forbid in as many words. **A hundred when the
arrangement does not know a display**, which is exactly what every display got
before this existed, so a session with no arrangement lays out as it did.

**`Layered` stopped being generic over its display.** One session's displays
are not one type — the loop holds its own target and `Desk` holds the rest —
and a generic wrapper makes a list of them impossible to write. The displays
differ; what is painted above them does not.

**Settings is not laid out per display.** It is one window, and a window is on
one display; nothing maps a surface to a display until task 7. It stays on the
display the loop itself holds. Drawing it on each would be two Settings
windows with one of them unreachable — wrong in a way no test here would have
caught.

## What was checked and deliberately not changed

`Server::the_fixed_controls_were_drawn` takes an `alo_appearance::TextScale`,
**not** a display scale, and it stays that way. The owner ruled on 2026-10-01
that the 44 × 24 handle floor is logical, scaled by the person's text size and
nothing else, and asked that any implication of a second conversion be
removed. `desktop_raster_tests::the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`
measures it. Handing the display scale there looked like the obvious
completion of this task and would have re-introduced the exact fault that
ruling exists to prevent — a handle protecting 88 × 48 at 200 per cent where
the promise is 44 × 24.

## What is tested, and where it runs again

Three tests in `the_session_holds_its_screens_tests.rs`, run by
`cargo test -p alo-shell` on every build:

| What it holds | Test |
| --- | --- |
| Each display answers with its own scale | `each_display_answers_with_its_own_scale` |
| No arrangement is one to one | `a_session_with_no_arrangement_is_one_to_one` |
| An unarranged display does not borrow a neighbour's | `a_display_the_arrangement_does_not_know_is_one_to_one` |

The first builds a 24-inch 4K beside a 27-inch 1080p — about 185 dpi against
about 82 — because `screens_testing`'s two fixtures are both about 165 dpi, and
**two displays that happen to agree cannot show that each was asked
separately.**

## What this does not do

- **`On the machine.` is not ticked.** One laptop, no second display. Seventh
  task running.
- **The fixed-control bounds are still the first display's.** That is the
  acceptance clause this task did not meet, and it is now task **5b**, split
  out rather than left to make this one read as closeable. It needs something
  that does not exist yet — nothing maps a surface to a display — which task 6
  is blocked on and task 7 needs too.
- **A window moved between displays** is not yet drawn at the right size on
  both, for the same missing map.
