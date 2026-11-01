# The alo Bar when it is open

## Round 2: the launcher the owner approved on 2026-11-01

**This section supersedes the tables below where they disagree.** `561:275`
(1440 × 960) and `561:370` (1280 × 800), *Find, open, or ask · the launcher*,
read off the live file on 2026-11-01.

| | 1440 | 1280 | the rule |
|---|---|---|---|
| panel | 680 at (380, 208), **360 tall** | 604 at (338, 174), **357 tall** | 47.2 % of the width, top at 21.7 %; **its height holds what it shows**, up to three fifths of the display, and the list scrolls from there |
| air inside | 26 | 23 | the panel's width over 26 |
| heading | 40, title Manrope SemiBold 22 / 30 at +5 | the same | a measure |
| Close | 48 × 40, `bg/cool`, corner 12, a 16 cross in `text/primary` | the same | a measure |
| field | 56, `bg/cool`, corner 12, a **2 `text/primary` outline**; search mark 20 at (18, 18) in `text/secondary`; typed text Regular 16 / 25 at 50 | the same | a measure |
| line under it | 12 below, SemiBold 13 / 18, `text/muted`, **on the panel's white** | the same | |
| an application's row | 12 below, 56 tall, corner 12; chosen: the accent at **12 %**; icon tile 32 at (12, 12), corner 8; name SemiBold 14 / 22 at 56; *· kind* Regular 14 `text/muted` 6 after; chosen: *Open* SemiBold 11 and an Enter key cap 26 × 18 ending 14 from the row's end | the same | rows 4 apart |
| hairline | 4 below the last row, 1, `border/default` | the same | |
| the row that asks | 4 below, **64** tall; alo's mark on an `accent/soft` tile 32 at (12, 16); *Ask alo "…"* SemiBold 14 and *· a new task* Regular 14 at (56, 12); a cloud 14 and where the answer would come from, SemiBold 11 `text/muted`, at (56, 36) | the same | |
| keys | 12 below, 28: ↑ ↓ caps *choose*, ↵ *open*, Esc *close*; caps white, a 1 `border/strong` line, corner 6 | the same | |
| under the keys | 15 | 15 | a measure, not a share |
| backdrop | `bg/canvas` at **45 %** over the whole screen, blur 4 | the same | |
| panel | white, 1 `border/default`, corner 18, `Elevation/Floating` | the same | |

**What was wrong, and is put right.** The grey band: every line on the panel was
inked on the field's grey and pasted onto the white panel, so the line under
the field sat in a grey rectangle. Words are now shaped on the ground they are
drawn on. **No heading, no Close, no backdrop, no key hints, and no way to
choose a row with the arrows** — all five are drawn and reached now.

**Readings taken, each for the owner to overrule:**

- **The blur is not drawn**, only the dimming. Nothing in this shell reads back
  what is behind a surface to blur it; that is a renderer change of its own.
- **Where the answer would come from is `alo-models`' own sentence** — the one a
  task window shows (*by openai, which has not said where it runs*), not the
  frame's *Answered by …*. Two sentences about where a person's words go is the
  duplication law 1 cannot afford; changing the shared sentence changes every
  surface at once, which is the owner's call.
- **A row's kind** is *Application*, followed by what the application says it is
  (its entry's `GenericName`, in its own words): *Application · File Manager*
  where the frame draws *Application · Files*.
- **An application this machine has no icon for** shows the cool tile empty, as
  the Dock's fallback ground, with the name beside it.
- **A press anywhere on the dimmed screen closes the launcher** as Escape does,
  and reaches nothing behind it; a press on a row does what Enter does on it.
- **No screen reader is told the launcher's rows yet**: `alo_access::Surface`
  has no surface for it, as it had none before. That is its own change.

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
