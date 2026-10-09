# Where a new window opens

**Status: the owner's specification of intended behaviour.** The design
contract for window placement, recorded **before implementation** at the
owner's direction. `CLAUDE.md` says this folder records *what the owner decided
where [the design file] is silent*, and on placement it was silent: the design
file shows a person's own arranged workspace, and arrangements are not an
algorithm.

**Date:** 2026-10-09.

**Scope.** Where alo puts a window when alo is the one choosing. It decides
nothing a person may do afterwards — see `the-canvas-as-a-workspace.md` §1 and
§7, and `CLAUDE.md`'s fifth law.

---

## The rule

**1. Explicit placement wins.** If the person drops a window at a location or
chooses an arrangement, use that location.

**2. Restoring is different from opening.** An existing window returns to its
saved Place and geometry. Travel to it; do not treat it as a newly created
window.

**3. A new window opens on the current Place, within the current view.** Prefer
free space beside the active window, with a small consistent gap. Try the right
side first in left-to-right layouts and mirror that preference for
right-to-left layouts.

**4. Do not move or resize existing windows to make room.** Do not
automatically zoom out or impose a grid.

**5. If no suitable space exists, overlap deliberately.** Open the new window in
front, slightly offset from the active window. Keep its title band and controls
reachable. **Overlap is permitted; losing access to a window is not.**

**6. With no active window, prefer a free area near the centre of the current
view.** Use the application's requested initial size, constrained by the usable
display area and its supported minimum size.

**7. Repeated openings must not drift off-screen.** When another offset would
make the new window inaccessible, restart the placement within the usable view.

**8. A person-initiated opening activates the new window.** Background
applications and alo must not steal focus or move the camera without an
authorised interaction that calls for it.

Fixed controls must not completely cover the new window's usable movement
handle. Show all, the Dock's window picker and keyboard navigation must keep
every window recoverable.

---

## What this settles, and what it does not

**Overlap is not forbidden.** `docs/features.md` promises *nothing is stacked —
no window is buried behind another **where the person cannot find it***, and on
2026-10-09 a lane read that to the dash and was about to build a rule
preventing one window from covering another. Rule 5 is the owner's own reading:
**alo may overlap on purpose when there is no room, and may never leave a
window unreachable.** The thing forbidden is loss, not occlusion.

**The design file's coordinates are not the algorithm.** `Dock 19 · New window
placement` and the other frames in the design show eleven windows at positions
like `x=56 y=44`, `x=736 y=8`, `x=2080 y=440`, with horizontal gaps of 32 and
64 and plenty of neither. Those are a person's arrangement, drawn to illustrate
a workspace in use. **The Figma examples illustrate the rule; their manually
arranged coordinates are not the placement algorithm.** Anybody deriving
offsets from them is reading an arrangement as a specification.

## The three examples an implementation owes

Named here so that an implementation is checked against situations rather than
against a restatement of its own code.

### Room beside the active window

One window open, space to its right in the current view. The new window opens
to the right of it, one consistent gap away, within the view. In a
right-to-left layout the same case opens to the **left** — rule 3's mirror,
which is a different outcome from the same rule rather than a second rule.

### Overlap when space is limited

The view is full: no free space beside the active window large enough for the
new one. By rule 4 nothing already open moves or resizes, and the view does not
zoom out. By rule 5 the new window opens **in front of** the active one,
slightly offset, with **its title band and controls reachable** — which is
`crate::window_name_band` and `canvas_never_lost`'s `A_USABLE_HANDLE`, 44 × 24
of band that must stay uncovered.

### Repeated openings

Several windows opened one after another, each offset from the last. By rule 7
the offsets must not walk a window out of the usable view: when the next offset
would put one somewhere inaccessible, placement **restarts within the view**
rather than continuing off the edge. A person opening ten terminals must end
with ten reachable terminals.

## Where this is implemented

Nowhere yet, on 2026-10-09. `new_toplevel` in `crates/alo-shell/src/surfaces.rs`
assigns a window its Place and no position, so every window is placed at the
origin and the second one opens exactly on top of the first — **alo burying a
window with nobody choosing it**, which is the one thing the promise forbids.

`crate::window_placement::set` is the primitive that moves one. What has no
caller is anything that chooses a position **when a window maps**.
