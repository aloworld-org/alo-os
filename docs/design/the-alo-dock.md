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
