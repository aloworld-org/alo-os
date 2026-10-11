# 0103 — The Dock, More apps and the minimised panel are three connected surfaces

- **Status:** accepted
- **Date:** 2026-10-11
- **Decided by:** the owner
- **Supersedes two rules by name:**
  - that **More apps shows only the applications that overflowed** — stated in
    [0102](0102-the-minimised-window-panel-and-the-docks-overflow-are-two-surfaces.md)
    and in `docs/features.md`;
  - that **an unpinned application with only minimised windows stays in the
    Dock** — never written down anywhere, which is why it needs naming here: it
    was the behaviour of `alo_dock::Holding::showing`, which adds an icon for
    every application that has a window and asks nothing about that window's
    state.
- **Leaves standing** everything else in 0102: the two surfaces hold different
  things, the popover opens clear of the panel's column, and the panel never
  receives applications because the Dock is full.

## Why a third surface, rather than two

0102 separated the panel from the overflow and was right about the separation.
What it got wrong is what the overflow *is*. It treated the popover as a
spillover — the tail of a list too long for its container — which makes it a
surface whose existence depends on arithmetic, appearing and vanishing as a
person resizes a display or opens a window.

It is now **an application overview**: a predictable route to the applications a
person has, open or pinned, that is there whether or not anything overflowed.
That changes three things at once, which is why one ADR carries all of them.

1. **The control is permanent.** It was already drawn permanently and last, by
   measurement off `Edge=Top, State=Rest`, which showed `More apps and canvas
   options` present with nothing overflowing. That measurement was right and the
   reason recorded for it — *the design draws it* — was the weaker half of the
   real reason, which is this clause.
2. **Its contents are not the overflow's.** Pinned applications first in their
   saved order, then every other currently open application — **including those
   whose windows are all minimised**, which by rule 3 are not in the Dock at
   all. Each application exactly once. Installed-but-neither-pinned-nor-open
   applications are **not** here; they are reached through Apps or the alo Bar.
3. **It is a grid with a search field**, not a list of names. Labelled icons,
   names beneath them, search filtering across both groups with stable ordering
   preserved and a clear empty result. The field stays still while the grid
   scrolls.

## The Dock's membership rule, which is new

| application state | in the Dock? |
|---|---|
| pinned, no open windows | **yes** |
| pinned, one or more non-minimised windows | **yes** |
| pinned, all windows minimised | **yes**, because it is pinned |
| unpinned, at least one non-minimised window | **yes** |
| unpinned, all windows minimised | **no** — its windows are in the panel |
| unpinned, no windows | **no** |

**"Non-minimised" is not "visible".** A window on another Place, outside the
camera's view, or behind another window is non-minimised and keeps its
application in the Dock. The question is the window's *state*, never where the
camera happens to be pointing — and that distinction is the one a canvas makes
easy to get wrong, which is why it is written into the rule rather than left to
be inferred.

Each application appears **once**, whatever its window count. Pinned
applications keep their chosen positions when opened; others follow in stable
opening order, and **focusing an application must not reorder the Dock**.

## What a click does, in the Dock and in More apps alike

- **no open windows** → launch it;
- **one open window** → focus it, restoring it first if it was minimised;
- **several open windows** → open that application's window picker, minimised
  windows included and their state clearly shown; choosing one focuses or
  restores that window.

Focusing a window elsewhere **travels to it** — to its existing Place and
location. It is never silently relocated. Clicking an application that is
already focused does not minimise it and does not make a second window; *New
window* stays an explicit action where an application supports one.

## What minimising does

Minimise acts on **one window**, never on an application. It removes that window
from the canvas presentation, preserves its Place, position, dimensions and
restoration context, adds an item to the panel, and **recalculates its
application's Dock eligibility** by the table above. It does not close the
window or discard its work, and it is not *Compact* — a compacted window is
still on the canvas.

So: an unpinned application with two windows keeps its icon when one is
minimised and loses it when the second is; both windows stay separately in the
panel; restoring either brings the icon back; and a pinned application keeps its
icon throughout.

## The panel, restated so that nothing is inferred

It holds **individual minimised windows and nothing else** — never an
application launcher, never a Dock overflow. Two windows of one application are
two items, never collapsed behind one icon. Each expanded card carries the
window's title, its application identity, a recognisable preview where that is
permitted, and the designed privacy placeholder where it is not. Selecting a
card restores **that** window at its saved geometry and Place, travels to it
where needed, and removes its item — never every window of the application.

Its three presentations all stand: expanded previews, collapsed rail, empty
handle. It is fixed while the canvas pans and zooms; further windows scroll
inside its list while the expansion control stays put.

The normalised measurements, which are logical pixels and are converted at each
display's scale:

| what | measure |
|---|---|
| expanded reference panel | 244 × 604 |
| preview card | 216 × 106, 10 gap |
| collapsed target | 48 × 48, 8 gap, 8 outer padding |
| collapsed rail height | `64 + 56 × count`, until the height forces internal scrolling |
| shared-edge handle | one centred 48 × 48 control inside 64 × 64 |

**The empty presentation follows its own design**, and is deliberately *not*
`64 + 56 × 0`. The handle holds no overflowed restore controls; expanding it
opens the designed inward preview lane.

## What this costs, and the honest state of it

The **drawing** of the Dock, of a list-shaped popover and of the panel's three
presentations exists. Almost none of the **behaviour** above does:

- `Holding::showing` asks nothing about window state, so the membership table is
  not implemented — this is the superseded rule, and it is the one change that
  alters what a person sees on a Dock that works today;
- no application's icon can be clicked to any effect —
  `WhatAPressOnTheDockDoes::GoTo` is returned and acted on by nothing, so none
  of §2 exists;
- there is no window picker, no search field, no grid, and no keyboard route
  into any of it;
- `ThePins::what_to_pin` can answer, and nothing produces the `Discovery` it
  needs, so the default pins have never reached a screen;
- Escape dismisses nothing.

**That list is the work, and it is not a disclaimer.** It is recorded here
because a reader a month from now will otherwise find three documents describing
behaviour and a machine with none of it, which is the fault
`docs/autonomy/STATE.md` was created to stop.

## Figma is an input here, not the record

The owner's note is explicit: *the existing list-style overflow reference is
historical input, not the final design for this revised grid*, and the Figma
file is to be updated before the new visual layout is treated as settled. So
this ADR governs the behaviour, the measurements above govern the panel, and the
popover's **visual** layout is open until the design file carries the grid.

Nothing is to be measured off hidden legacy layers. The canonical references are
listed in `docs/design/the-docks-visual-specification.md`.
