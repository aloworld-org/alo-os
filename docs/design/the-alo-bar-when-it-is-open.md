# The alo Bar when it is open

**Read off `alo Bar / Open` (`81:890`), inside `09 · Find, open, or ask`
(`78:1671`), on 2026-10-20.** Every number here was taken off the frame; none
was matched by eye.

The frame is 1440 × 960, which is the design's reference size and **not a
screen anybody has**. So the table below gives each number twice: as the frame
draws it, and as the rule a layout follows — `CLAUDE.md`: *a figure taken from a
frame becomes a proportion or a named rule, never a constant that holds only at
the size somebody drew.*

## What it is

A panel **over the canvas**, not the Dock's field growing upward. The Dock
stays where it is; this opens in front of it with a backdrop behind
(`Search backdrop`, a sibling frame).

## The numbers, and the rule each becomes

| | at 1440 × 960 | the rule |
|---|---|---|
| panel | 680 × 390 at (380, 208) | **47.2% of width**, centred across; top at **21.7% of height** |
| inset, all round | 25 | panel width ÷ 27.2 — one measure, used on every side |
| heading | 22 px, at y 30 | **5.6% of panel height** |
| close control | 48 × 40, radius 12, at the heading's right | a target, flush to the inset |
| field | 632 × 56, radius 12, at y 77 | panel width less two insets; **a target's height** |
| what is typed | 16 px, inset 16 into the field | |
| the line beneath | 13 px, at y 145 | one line, a **gap of 12** under the field |
| a row | 632 × 56, radius 12 | the field's width and height exactly |
| row to row | 68 | a row plus **12** |
| first row | y 175 | a **gap of 30** under the line |

**The rows are 632 × 56 — the same box as the field.** That is the design
saying a row is a thing you act on, drawn at the size of the thing you type
into, and it is why a row's height is a named target rather than a number of
its own.

## The one number that is not a size

**Four rows are drawn and the panel is 390 tall.** The fourth sits at y 379, so
its bottom is 435 — past the panel. That is deliberate: **the list scrolls**,
and the design shows it clipped rather than pretending a result count fits.
Nothing should cap the number of results to what is visible.

## What a row says

`Name · where it is` — *Launch strategy · Docs*, *Launch website · Browser*,
*Opening scene · Blender*. **The second half is the kind of thing**, so an
application and a document are told apart at a glance. The separator is a
middle dot with two spaces each side, the same on every row.

The line under the field is *On this computer · 3 open windows* — **where the
results came from, and how many.**

## What this means for the installer's screens

Nothing. They are a separate surface with their own frames
(`530:30834`) and their own derivation. This file is the canvas's Bar.

## What is not here

**Which rows a search finds, and in what order.** That is
`alo_shell::the_bar_finds`, which decides and says, and has no geometry in it
at all.

**The keyboard.** How a row is reached, what Enter does on it and what Escape
closes are `docs/design/the-shortcuts-and-the-edges.md`'s subject, and the
frame shows no focus state — so the ring is `a_focus_ring`'s rule, as it is
everywhere else on the Dock.
