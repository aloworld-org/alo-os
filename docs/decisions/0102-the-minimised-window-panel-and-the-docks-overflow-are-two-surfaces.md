# ADR 0102 — The minimised-window panel and the Dock's overflow are two surfaces

**Status:** **accepted, 2026-10-11**, by the owner.

- **Supersedes:** nothing. Narrows
  [0101](0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md),
  which settled what the Dock looks like and said nothing about what opens from it.

## Why this exists

Two surfaces sit at the right-hand side of the screen, and until 2026-10-11
nothing had ever drawn them in one frame. They are different in contents, in
lifetime and in what selecting an item does — and they had come to look like one
right-hand column, which is what prompted this ruling.

The cause was neither surface's contents. **It was arithmetic.** The overflow
list hangs off the far end of the Dock's bar; a bar wide enough to overflow has
its far end near the display's far end; and the panel is at the display's far
end. So the popover opened straight over the panel's reserved column. Measured
on a 1366×768 display with a bottom Dock: the list at x 1055–1303, the panel's
column at 1254–1366 — 49 pixels of overlap, with correct and entirely separate
data behind each.

## The ruling, in the owner's words

> **Right panel — minimised windows**
>
> - Contains only windows the person has minimised.
> - Each item represents one window, with its title, application identity and
>   preview where permitted.
> - Several minimised windows from one application remain separate items.
> - Selecting an item restores that window in its original Place and position,
>   travelling to it when necessary.
> - Remains fixed while the canvas pans or zooms, with the designed expanded,
>   collapsed and empty presentations.
> - Never receives applications merely because the Dock is full.
>
> **Dock overflow — additional applications**
>
> - Opens only when the person activates the Dock's "…" control.
> - Appears as a compact, bounded popover anchored to that control, opening
>   inward and clear of the minimised-window panel.
> - Lists applications that do not fit in the Dock.
> - Dismisses on Escape, outside interaction or completed selection.
> - Must not become a persistent right-hand column.

## What it costs in code

**One rectangle, asked for once.** The column the panel reserves is now
`alo_shell::panel_raster::the_column_it_reserves`, taking the look, the display
size and the edge — and no `Panel`, because the column does not depend on what
is in it. Both `panel_raster::picture` and the Dock's overflow list read it.
Deriving it in two files would be two files that agree today.

The list is then clamped into the span clear of that column, **with the Dock's
own `GAP` between them** rather than flush. Flush satisfies *not overlapping*
and fails the clause it is there for: a 248-wide list hard against a 112-wide
column reads as one column.

**The edge is derived, never stored.** The panel mirrors to the left when a
person reads right to left — the owner's ruling of 2026-09-30 — and the Dock
reads the same `look.reading()` the panel does, so the two cannot disagree about
which side the column is on.

## Where a right-edge Dock contradicts this, and what was done about it

A right-edge Dock on a left-to-right display puts its band at x 1290–1366 while
the panel reserves 1254–1366. **The band is inside the column**: two fixed
surfaces competing for one edge, which nobody has settled and which this ADR
does not settle either.

What was settled is the precedence. On such an edge the list cannot be both
flush to its control and clear of the panel, and *clear of the minimised-window
panel* is the ruling — so the list stops one gap short of the column, and
`the_overflow_list_is_a_bounded_popover_anchored_to_the_bar` asserts it stopped
**there** rather than merely somewhere. The underlying collision is named in
`docs/quirks.md` and is open.

## What this does not yet do

- **Escape does not dismiss the list.** An outside press does, through
  `crate::pointer`. There is no keyboard route: intercepting Escape needs the
  seat, and taking it unconditionally would steal Escape from every client. Open
  and named here rather than left to be discovered.
- **No row can be selected.** The list's rows are drawn and are not pressable,
  so *dismisses on completed selection* has nothing to complete. That is the
  same missing piece as pressing an application on the bar itself —
  `WhatAPressOnTheDockDoes::GoTo` is returned and acted on by nothing — and both
  belong to the same change.

## The guarantees that are tests

Each of these runs on every build, in `alo-shell`'s library suite:

- `the_overflow_list_opens_clear_of_the_put_aside_panels_column` — all four Dock
  edges × both reading directions, against the rectangles **the frame drew**
  rather than a recomputed column.
- `the_overflow_list_is_a_bounded_popover_anchored_to_the_bar` — bounded on both
  axes, and a measured distance from its control.
- `the_panel_holds_windows_and_not_applications` (in `alo-put-aside`) — three
  windows of one application are three previews.
- `there_is_one_slot_for_each_window_put_aside` — and the drawn panel gives one
  slot per window.

Both of the first two were mutation-tested: restoring the old clamp and removing
the gap each fail them, with the rectangles printed.
