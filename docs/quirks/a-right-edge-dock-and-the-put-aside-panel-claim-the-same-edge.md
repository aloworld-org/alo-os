# A right-edge Dock and the put-aside panel claim the same edge

**Date:** 2026-10-11. **Status:** open, and not caused by either surface.

## What was measured

On a 1366×768 display, left-to-right reading, `Dock::set_edge(Edge::Right)`:

| surface | rectangle |
|---|---|
| the Dock's band | x 1290–1366, full height |
| the panel's reserved column | x 1254–1366, full height |

The band sits **entirely inside** the column. Both rectangles are correct by
their own derivations — the band is the Dock's thickness in from its edge, and
the column is `ICON + 2 × MARGIN + 2 × (3 × MARGIN)` in from the display's — and
neither surface asks the other anything.

Taken from `alo-shell`'s `the_overflow_list_is_a_bounded_popover_anchored_to_the_bar`,
which prints both rectangles when it fails, and reproducible from
`crates/alo-shell/src/desktop_raster_tests.rs` with no hardware.

## Why it is written down rather than fixed

Nothing has decided what should happen. Three answers are all defensible and
each is a different product:

- the panel mirrors to the free edge when the Dock takes its own;
- the Dock refuses the edge the panel holds;
- both stay and the panel's column starts where the Dock's band ends.

The third is the smallest change and the only one that is purely geometric. The
first two are choices about what a person is offered, so they are
`docs/features.md` questions and not this file's.

## What was done in the meantime

Only the thing that could be done honestly: the **precedence** for the Dock's
overflow list was settled, in
[ADR 0102](../decisions/0102-the-minimised-window-panel-and-the-docks-overflow-are-two-surfaces.md).
On an edge where the band is inside the column, the list cannot be both flush to
its control and clear of the panel, so it is clear of the panel and stops one
`GAP` short of the column — and the test asserts it stopped **there**, not merely
somewhere, so this quirk cannot drift into a list floating on the canvas.

The band's own overlap with the column is untouched. A press in the overlap is
whatever the existing order of hit tests makes it, which nobody has measured
either; `crate::a_click_brings_a_window_back` claims only the rail, so a press
in the shared strip reaches the Dock, and that is an accident rather than a
decision.
