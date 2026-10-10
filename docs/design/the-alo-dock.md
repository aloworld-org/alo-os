# The alo Dock

The Dock is the fixed place a person goes to reach their applications and the
windows they already have open. **Windows keep their positions; the canvas moves
to them.** The Dock does not move when the canvas does.

This note is the owner's specification, recorded so that what gets built can be
checked against something. Where it says *this note's reading*, that is a lane's
inference and not the owner's words — the distinction matters because an
inference can be wrong and an instruction cannot.

The designs: [Dock · resting](https://www.figma.com/design/nDxyF5Ho9oC4RObjVzwBNJ?node-id=321-17306)
and [the interaction rules](https://www.figma.com/design/nDxyF5Ho9oC4RObjVzwBNJ?node-id=331-20993).

## The rule everything else follows from

> **Click an app to return to where you last used it. Choose a preview when you
> want a different window.**

Check a proposed behaviour against that sentence. If the answer would surprise
somebody who believes it, the answer is wrong.

## What one click does

| The application | What a click does |
|---|---|
| nothing open | open a window near what the person is looking at, and focus it |
| one window open | focus it; move the view only if it is not already shown |
| several open | focus the one used last |
| the one used last was put aside | bring it back where it was, then travel to it |
| already in view | bring it forward; the canvas does not move |

**A click never creates a window by surprise**, and clicking again does not cycle,
does not open another, and is not quietly repurposed as *back*.

This is built: `crates/alo-dock/src/clicking.rs`.

**One case fell out of taking two rules seriously at once** and is in neither the
specification nor the designs: a window that was put aside *whose place is already
on screen*. *Restore then travel* and *if it is visible, do not pan* are both
rules, so the answer is to bring it back and move nothing. Travelling no distance
is still motion, and motion that says nothing is what a person turns off.

## Choosing another window

Hovering an icon shows the application's name, then named previews of its windows
— the designs show a Browser with **Launch website**, **Research** and
**Preview**. The one used last is first and is marked as such. Keyboard focus
opens the same list; so does a long press. One list, three doors.

A preview can carry a small map of the canvas showing where that window sits and
where the person is looking now.

**This is not the duplicate the owner refused.** The right panel holds windows
that were put aside, across every application, permanently. The Dock's list is
per application, appears on demand, and is how a person reaches a particular
window. Different questions.

## Travelling, and coming back

After travelling, a temporary **Back to previous view** control returns the exact
pan and zoom — not merely the previously focused application. It goes away after
the next deliberate action.

**Peek** gives a readable look at another window without moving the canvas at
all. Release or Escape ends it; clicking the preview travels properly. The
keyboard path must reach the same result as holding a pointer.

## Windows put aside

The Dock still shows the application is open. The **right panel owns the
previews** of windows put aside and can collapse to icons. Choosing one restores
that window at its own place; if something now overlaps it, the restored window
comes forward and **nothing is rearranged**.

## Making a new window

Dragging an icon onto the canvas shows the outline of the window that would be
made, where it would be. Dropping makes it; Escape cancels. **The result is
visible before it happens.** There is also **New window** in the icon's menu, so
dragging is not the only way.

## Opening a file in an application

Dragging a file over an icon makes the Dock name the action — *Open in Blender* —
**before** the drop. A selected file offers the same action without dragging, for
anybody who cannot or would rather not drag.

**Hovering a file over an icon never opens or sends anything.** The intended act
is explicit first.

## The icon's menu

**New window, Show windows, Pin to Dock, Minimize, Full screen, Quit.** On a
selected window it also offers **Record this window** and **Give this window to
alo**; recording shows that it is recording and offers Stop, and alo working in a
window marks that window, names alo, and offers Stop. **Neither becomes a
permanent button in the Dock**, and the Dock does not become a dashboard for the
agent.

## What this file does not carry

**It carries behaviour and the owner's geometry rulings. It does not carry what
the Dock looks like** — the surface, its corner radius, its border, its shadow,
the running indicators, the artwork in a slot, or how the overflow list is
styled. Those are measured in
[the Dock's visual specification](the-docks-visual-specification.md), node by
node, from the canonical Figma.

The split was made on 2026-10-10 after this lane showed a working overflow and
treated it as evidence the design was finished. The owner:

> The agreed 76px horizontal height and 48×48 targets establish geometry; they do
> not replace the rest of the visual design. Likewise, the overflow row minimum is
> not a complete panel specification.

## Full screen, pinning, and too many applications

True full screen covers the Dock. Moving to the bottom edge reveals it over the
content, and **the pointer can travel onto the revealed Dock and click without it
disappearing on the way** — which is a real constraint on how the reveal is
built, not a detail.

*Read "moving to the bottom edge" as the edge under the bar, not the whole width
of it.* [The regions a pointer can be in](the-regions-a-pointer-can-be-in.md)
carries the measured areas — the Dock's reaches 24 either side of the bar and
runs down through the gap beneath it, and stops well short of the panel at the
right — together with the owner's rule for where two surfaces meet at a corner.
This sentence was written when the Dock was the only surface with an edge, and
read alone it invites a strip the full width of the screen, which would claim a
corner the panel owns.

**Pinned** means the application stays available when it is closed. **Open** means
it has a window. Those are two states and they are told apart without relying on
colour. Icons keep a usable size; the rest go into a named overflow list.

## Keyboard and screen reader

Arrow keys move between applications and previews, Enter focuses, the menu key
opens the actions, Escape closes whatever Dock surface is open. Everything the
Dock shows on hover is reachable by keyboard focus.

A screen reader announces the application's name, how many windows it has,
whether one is focused or was put aside, and where a chosen window sits on the
canvas — *"Browser, three windows open, one minimised"*.

## Two things the designs say that the words do not

**The status area is at the top right.** [ADR
0076](../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
took the clock, battery, network and volume off the Dock and recorded them in
`docs/autonomy/evidence-a-person-can-work-on-it-all-day.md` as **owed a location**. The resting design shows
them at the top right of the display — `17:42 · 86% · Private · wifi · volume ·
brightness` — which is that location. **This note does not treat the design as the
decision**: the evidence entry still says owed, and it is the owner's or the
shell plan's to close. It is recorded here so the next reader does not repeat the
search.

**The Dock is a floating rounded bar, centred, with room beneath it.** It is not
a band across the whole width of the screen. What `alo-shell` draws today *is*
such a band, flush to the bottom edge, which is what the code has always done and
what the removal in ADR 0076 preserved. **That is now known to be wrong** and is
work, not a defect in what landed.

The resting design also shows three things the words do not mention: a **find,
open, or ask** field inside the Dock, a **Show all** control, and a **History**
control that sits outside the Dock at the bottom right.

## What is owed elsewhere

**The order windows were used in must survive a sign-out.** `leaving.toml`
restores what was open; if the use-order is not restored with it, the first click
on every application the next morning goes somewhere arbitrary — which breaks the
rule on the morning a person most relies on it. That belongs to whoever owns
`leaving.toml`, not to a crate about a dock. `crates/alo-dock/src/windows.rs`
says so at the place it would go.

## The sizes, decided where the file disagreed with itself

**Settled by the owner on 2026-10-09**, and recorded here because
`CLAUDE.md` gives this directory *what the owner decided where it is silent or
disagrees with itself* — and it disagreed: `Dock 01 · Resting` draws a **24**
glyph while the side docks' implementation contract says *icons may reduce from
32 to 28*.

| | |
|---|---|
| application icon | **32 × 32** |
| clickable area | **48 × 48** |
| gap between clickable areas | **8** |
| the Dock's outer padding | **8** |
| horizontal Dock height, vertical Dock width | **64** |
| utility glyphs — search, overflow | 20 to 24, in the same 48 target |

**The owner gave these as proposed dimensions rather than as measurements of the
Figma file**, and said so, which is why they are written here as a decision and
not as a reading.

**No automatic shrinking, and no hover enlargement.** This revises the
contract's *may reduce from 32 to 28*, in the owner's words: *smaller artwork
alone does not create more usable space.* When the Dock runs out of edge it uses
overflow. A 28-pixel glyph in a 48-pixel target buys four pixels and costs
legibility for the person who can least spare it, and §10 already forbids
magnification that pushes neighbours sideways — *the person should be able to
aim once and click.*

**Icon size becomes a person's setting later**, with **`[v0.5]` The dock's size,
and whether it hides when a window needs the room**. Until then it is fixed,
which is a number nobody has been offered rather than a choice withdrawn.

### The overflow, and the three measurements it supersedes

**The owner's ruling of 2026-10-10**, asked for because the design file gives
three different sizes for one control and a row pitch below the floor every
other control in alo OS is held to:

> 1. Overflow occupies one application slot: 48 × 48 logical pixels. Centre the
>    overflow glyph inside it and use the same slot spacing as neighbouring
>    applications on all four edges. This supersedes the conflicting 72 × 44
>    annotation and 44 × 44 component measurement. Keep its accessible name and
>    tooltip: "Show more open apps."
> 2. Overflow rows have a minimum 44px hit-target height. Remove the 39px pitch;
>    do not create an exception. Allow rows to grow for larger text. Let the
>    panel grow beyond 313px when space permits, then scroll its list within the
>    available screen space. Keyboard focus must scroll the focused row into
>    view.

#### What was measured, and why none of it could be built as drawn

Taken off `docs/design/figma-snapshot/70-28.xml` on 2026-10-10, so that the next
reader does not repeat the search:

| node | measured | what it is |
|---|---|---|
| `Show more open apps` | 72 × 44 | an annotation, in `Dock 01 · Resting` only — the bottom edge |
| `Dock edges / Button / More` | **44 × 44** | the component, in the same family as every application button |
| `Browser · focus hit area` | 52 × 48 | an annotation of an application's region, in the same frame as the first |
| `More apps / opens inward` | 248 × 313 | the panel, identical on all four edges |
| its rows | 6 names, pitch **39** | text 210 × 22.1, from y 54 at 39 apart |
| `Choose Browser in overflow` | 248 × **46** | a row's target, annotated — 7 more than the pitch, so the drawn targets overlap |

Three facts make the first row unbuildable as drawn. It is **44 tall where an
application's annotation in the same frame is 48**, so taking it literally makes
the overflow control shorter than the icons beside it. It is **72 wide in a bar
whose slot pitch is 56**, so it cannot sit in the row without its own
arithmetic. And it exists **only for the bottom edge**, while the component set
— where the More button is 44 × 44, exactly like Docs, Browser and Files — is
what the side and top docks are built from.

The row pitch fails on its own terms: **39 is below
`alo_appearance::targets::ENHANCED_TARGET`**, the 44 that WCAG 2.5.5 names and
that every control in this product is built to, and the design file's own
implementation contract says *targets remain 44 × 44* three lines away.

#### What is built

- The control is **one slot**: `alo_dock::places::WhatIsHere::TheOverflow`, laid
  out by the same loop and the same `MARGIN`, `ICON` and `GAP` as an
  application, on all four edges. There is nothing special to assert about it,
  which is the point.
- It is **last, and only when there is something behind it.** `alo_dock::fit`
  keeps one slot back when it has to put anything aside, so a bar that overflows
  shows one application fewer than one that just fits; a bar with room for
  everything has no control at all rather than a hidden one.
- Its mark is **U+2026**, at `measures::A_UTILITY_GLYPH` — 24, the top of the
  owner's *20 to 24* — centred in the 48 slot. A mark rather than a word,
  because a word would be a different width in every language inside a slot that
  is one size everywhere.
- Its **name** is `alo_dock::words::SHOW_MORE_OPEN_APPS`, translated, and is both
  what a screen reader speaks and what the tooltip shows. One string, because
  they answer the same question.

**The 313 is not a height any more.** It was six rows at a pitch that is gone;
the panel is as tall as its contents, up to the room on the screen, and scrolls
beyond that. Its **width**, 248, is measured and is the same on every edge, so it
stays a number.

#### The list itself

`alo_dock::overflow` is the layout, and every number in it is either the owner's
rule or a measurement with its provenance:

| | |
|---|---|
| a row's height | `max(44, a line of text + 8 either side)` — the floor is `alo_appearance::targets::ENHANCED_TARGET`, not a 44 written again |
| the panel's width | **248**, measured, the same on all four edges |
| above the first row | 12, the heading's line of text, 12, and a 1-pixel rule |
| the panel's height | the heading and as many rows as the room allows — no ceiling of its own |
| when the room runs out | the list scrolls, **by whole rows** |

**A row is the larger of the floor and what the text needs, never a choice
between them.** A row pinned at 44 loses its name at 200%, the size EN 301 549
requires a layout to survive; a row that only followed the text is 37 at 100% and
below the floor. The maximum of the two is both clauses at once.

**It scrolls by whole rows because *keyboard focus must scroll the focused row
into view* has to be exact.** A list scrolled by pixels can leave a focused row
half off the top, which is the same fault as a target below 44: the person who
most needs the keyboard is the one who cannot see where they are. And the scroll
is **the least it can be**, so arrowing one past the bottom moves the list by one
rather than jumping the focused row to the top and losing a person their place.

**What is not built: nothing presses the Dock.** Measured 2026-10-10 across the
2531 `.rs` files outside `alo-dock`: `alo_dock::clicking` has **no caller**,
`alo_dock::menu` has none, and `alo_dock::Places` is named twice, both in
`dock_raster` and both for drawing — so `Places::at`, the hit test this crate
exists for, is never called. The Dock is drawn and is not pressable at all: not
the overflow control, not an application's icon, not the menu. That is a gap
under every Dock interaction rather than under the overflow alone, and it is the
next thing.

### An application with no artwork shows its first letter

**The owner's ruling of 2026-10-10**, asked for because this file said what an
icon *is* — 32 × 32 inside a 48 target — and nowhere what is drawn when there is
no artwork to put there:

> The fallback should be on the first letter of the application.

**It is not a hypothetical case; today it is every case.** Measured the same
day: `alo-applications` names an icon in **none of its seventeen files**, so
there is no artwork for any application on this machine. A Dock that drew only
real icons would draw nothing, which is the band a person sees now.

**Three things follow, and they are decisions rather than readings:**

- **The letter is the application's**, from `alo_dock::AppId`, not the window's
  title. Two windows of one application are one icon, so an icon that took its
  letter from a title would change when a person switched tabs.
- **The first letter is the first *grapheme*, not the first `char`.** A name
  beginning in Devanagari, Thai, Hangul or an emoji has a first `char` that is
  half a letter, and `"नमस्ते".chars().next()` is a fragment no reader
  recognises. This is the i18n law reaching a place that is not a translated
  string: the text is the application's own, and cutting it wrongly breaks it
  for exactly the languages with the least software already.
- **It is not uppercased.** Case is locale-dependent — Turkish `i` uppercases to
  `İ`, and a lowercase script like Georgian or Devanagari has no upper case at
  all — so raising it would be this shell making a typographic decision about
  somebody else's alphabet. The letter is shown as the application wrote it.

**What this does not decide**: what replaces the letter when artwork arrives, and
whether a person may choose an icon themselves. Both are later and neither is
promised.

**Four of the six were already the code's numbers**, written by somebody who had
not seen this list: `ICON = 48` was always the *clickable area*, with `GAP = 8`,
`MARGIN = 8` and `Room::a_dock_of_icons()` asserting **64**. What was missing was
the artwork's own size, now `measures::GLYPH`.

**They are held at compile time.** `measures.rs` carries `const` assertions
rather than tests, because these are relationships between constants: setting
the glyph back to 28 fails the **build**, naming the decision. Measured by doing
it.

## Not settled

- **A window on another display.** The canvas travels to a window; if the window
  is on another screen, travelling is not what happens and focus crossing screens
  is. The spirit is clear and the mechanism is not.
- **A full-screen window and its icon.** Reaching the Dock at all means the
  bottom-edge reveal. Whether a click then leaves full screen or switches to it
  is undecided. `clicking.rs` currently brings it forward without travel, which
  is this note's reading rather than a decision.
- **What is announced after a click** — the window that took focus, and whether
  the travel itself is worth saying.

## The four edges, measured off the frames on 2026-10-10

The owner restored the edge choice on 2026-09-30 and the design for it arrived as
**Dock edges · v0.01 specification** (`node-id=351-25693`). These are read off the
**frames**, not off the captions, because `CLAUDE.md` says so in as many words — *a
screenshot is not a specification; take the numbers off the frames and record them*
— and because on this page the two disagree.

| What | Frame | Measured |
|---|---|---|
| Side Dock lane, left | `Dock / Left` | `x=24`, **70** wide → x24–94 |
| Side Dock lane, right | `Dock / Right` | `x=1346`, **70** wide → x1346–1416 |
| Top Dock row | `Dock / Top` | `y=16`, **74** tall → y16–90 |
| Expanded shelf, inward lane | `Shelf / Shared edge · inward lane` | **244** × 604, at x116 left / x1072 right |
| Shelf's upper edge handle | `Shelf / Upper edge handle` | **64 × 64** at y96, x24 left / x1352 right |
| Shared edge, the split | — | shelf owns y84–180; Dock owns y192–960 |

The side lane is **symmetric**: 1416 is 1440 − 24, and both lanes are 70, so the 24
is a margin from the screen's edge and the 70 is the lane itself.

### Where the page disagrees with itself, so nobody re-derives it

Three figures appear twice on this page and differ. The frames are taken as the
answer in each case; the prose is recorded so a reader meeting it knows it was seen
and not missed.

| | Frames | Build contract (`348:28451`) | Section caption |
|---|---|---|---|
| side lane | **70** | x24–94 → 70 ✓ | x24–92 → 68 ✗ |
| right lane | **70** | x1346–1416 → 70 ✓ | x1348–1416 → 68 ✗ |
| top row | **74** | y16–92 → 76 ✗ | — |

The caption is two pixels narrow on both side lanes and the contract is two tall on
the top row — a consistent two, in opposite directions, which reads like a stroke
counted in one place and not the other. **It is not resolved here**, because two
pixels is exactly the size of a thing worth asking about rather than guessing: a
reader who needs it exact should ask the owner, and a reader who needs it buildable
should use the frame.

### The lane is not what this crate derives, and that is the finding

`alo_dock::Room::a_dock_of_icons` is `MARGIN + ICON + MARGIN` — 8 + 48 + 8 = **64**.
The frames say **70**. Six pixels, three a side, and nothing in `measures` accounts
for them.

**So the lane is recorded as its own measured figure rather than reconciled.**
Deriving 70 out of the existing three would mean changing `MARGIN` for every edge to
make one edge come out right, which is fitting the measures to a frame — and the
bottom Dock's own numbers are held to this file elsewhere and would move with it.
What the six pixels are *for* is a question for the owner or for a closer reading of
the frame's contents; what they are is 70.
