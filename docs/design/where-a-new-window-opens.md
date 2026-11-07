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

## The host's side of it, added 2026-10-09

The owner assigned the integration to the lane that owns the host:

> **Window band:** the host controls visibility from pointer position, keyboard
> focus and active interactions. Keep it visible while the person moves onto
> its controls or drags the window. **Hover must not change which windows exist
> or are visible.**
>
> **Placement:** at a new window's first map, the host supplies the current
> Place, camera/view bounds, reading direction, active-window geometry and
> fixed-control bounds. **Pass these into the placement calculation explicitly;
> do not duplicate the host's state inside the window store.**
>
> **Apply default placement once.** Restoring a window uses saved geometry;
> later redraws or remapping must not unexpectedly reposition it.
>
> Preserve the agreed 16-unit neighbouring gap and 48-unit overlap offset,
> subject to reachability.

And the bar for calling it finished:

> Completion requires a **production demonstration**: open two windows,
> exercise placement, reveal and use the band, and verify that restoring
> preserves geometry. **An exported function with no caller is not evidence of
> working behaviour.**

## One question this leaves open, named rather than answered

**Does a remap keep its place?**

`crate::window_placement::reset` clears a window's placement when it **unmaps**,
on the rule that *placement belongs to one mapping lifetime, not the reusable
protocol object*. So a client that unmaps and remaps the same surface arrives
unplaced and is placed afresh.

The contract says *later redraws or remapping must not unexpectedly reposition
it*. A redraw does not — `place_unless_already_placed` holds that. **A remap
after an unmap does**, because the placement was cleared in between.

Which is right depends on what a remap means to the person: a window that
flickered, or a new window. The two readings give different behaviour and the
same code, so it is recorded here rather than settled quietly.

## Where this is implemented

Nowhere yet, on 2026-10-09. `new_toplevel` in `crates/alo-shell/src/surfaces.rs`
assigns a window its Place and no position, so every window is placed at the
origin and the second one opens exactly on top of the first — **alo burying a
window with nobody choosing it**, which is the one thing the promise forbids.

`crate::window_placement::set` is the primitive that moves one. What has no
caller is anything that chooses a position **when a window maps**.

## alo's own windows count as windows — 2026-11-01

**The round-2 frames the owner approved on 2026-11-01** (`557:144`, `557:296`,
and the rules drawn on `557:456`: *room beside: 16 apart; no room: in front, 48
down and across; many in a row: restart inside the view*) draw alo's terminal
and an alo task window among the applications, and a new window opening
**beside** them.

Until that day the placer was handed the applications' windows alone. alo's
terminal and its task windows are kept in the display's pixels rather than on
the plane, and nothing passed them in — so with alo's terminal open and nothing
else, a new application opened in the middle of the view, over the terminal.
`crate::a_new_window_is_placed` now moves both onto the plane with the camera,
as it does a fixed control, and counts them among the open windows. The window
*beside which* a new one opens is the frontmost of whichever kind the person
went to last (`crate::which_window_is_in_front`), so a person typing in alo's
terminal gets the next window beside the terminal.

**And never exactly on top.** Two roads could still arrive at a corner a window
already had — the restart at the view's corner and the step from the middle —
and a window exactly over another hides all of it, its title strip included.
`where_a_window_opens` now steps any such answer on by the same 48 until it is
on nobody's corner. **This makes an existing promise true** — rule 5's *losing
access to a window is not [permitted]* — rather than adding one.

## A native permission dialog follows its caller — 2026-11-04

A system dialog can be a different Wayland client from the application whose
button opened it. Its exported XDG parent handle makes that relationship
explicit. Like another window of the focused application, a newly mapped
dialog with that focused parent comes in front and takes the keyboard. A
background parent, a guessed handle, or a revoked export cannot interrupt the
person. This extends rule 8's person-initiated opening to native permission
dialogs; it does not activate an existing window when its parent later changes.

---

## Amended 2026-11-07 — when there is no room, the view can glide to empty space

**The owner, 2026-11-07:** *"when the user opens the window of the application it
should not open on top of the other, it should smooth move the canvas and open on
the empty space. Also this can be put in the settings so the user chooses."*
Asked which a new machine starts with: **glide to empty space**. Asked where it
looks first: *"Nearest space and the new window should be in focus."*

So rule 5 becomes a choice, in Settings → Windows, **When there's no room for a
new window**:

- **Glide to empty space** — the default. The view moves smoothly to the
  **nearest** free space on the current Place, from what the person is looking
  at, in any direction; the new window opens there **without covering any
  window**, and **it has the focus**. Nothing else moves or resizes (rule 4
  stands), and the view does not zoom.
- **Open in front, slightly offset** — rule 5 as written on 2026-10-09, unchanged.

Rules 1–4 and 6–8 are untouched: room in the current view is still used first,
beside the active window. A person-initiated opening is the *authorised
interaction* rule 8 asks for, so this glide is allowed; a window an application
or alo opens in the background still never moves the camera. **The glide
follows the person's reduced-motion setting** — with it on, the view jumps
rather than glides — and its length is a duration, never a speed the person has
to match. A person may still put one window over another themselves at either
setting.

### Built 2026-11-07, and what this lane decided where the owner was silent

**Where it is.** `crate::a_new_window_is_placed` decides,
`crate::the_nearest_empty_space` finds the space, `crate::the_view_glides`
moves the view, and the choice is `alo_appearance::WhenThereIsNoRoom`, kept in
`appearance.toml` and chosen in Settings → Appearance, in a section under
*Windows* (`the-settings-window.md`, question 8).

**When it happens — all three, or not at all:**

- rules 3 to 7 found **no room in view**: their answer lies over another
  window, or is not wholly in the view (for a window larger than the view, its
  handle is not in it). Room in view is used exactly as before;
- the person kept **Glide to empty space**;
- **the person opened the window** — the same answer rule 8 uses to give it the
  focus (`crate::a_window_the_person_opened`). A window an application or alo
  opened in the background keeps rule 5, in front and offset, and the view
  does not move.

**Decided by this lane, each the owner's to change:**

1. ***Nearest* is how far the view has to move**, not how far the window is
   from the middle of the screen. Ties go to the space nearest the middle of
   the view, then to the reading direction's side (rule 3's preference,
   mirrored), then to the higher.
2. **The 16 gap is kept on every side**, from every window, and between the new
   window and the edge of the view it glides to.
3. **Never further than *Show all* reaches.** A Place so full that no space is
   left within what *Show all* fits at its furthest out gets rule 5 instead —
   in front, offset — rather than a window that could not be brought back.
4. **The glide lasts 300 ms however far it goes**, eased in and out. The design
   names no duration; 300 ms is what common desktop guidelines give a
   transition that moves the whole view.
5. **The person's hand wins.** A pan, a zoom, *Show all* or going to another
   Place ends a glide where it is.
6. **Reduced motion is read from the person's own `access.toml` at sign-in**
   (`alo_access::Setting::ReducedMotion`), because that is where it is kept.
   **Nothing in Settings turns it on yet**, so on a machine today the glide
   always glides; the first-start sheet's *More options* is the designed road
   (`the-first-start.md`, row 3).

**alo's own windows glide too — the owner, 2026-11-07: *"Yes, the same"*.**
Asked whether the terminal, Settings, the record and task windows follow the
same rule, the owner said yes the same day. So each of them asks the one rule
in `crate::room_in_a_full_view`, with the same choice: rules 3 to 7 first,
and with no room and *Glide to empty space* kept, the nearest empty space,
covering nothing, and a glide (or a jump, with less motion asked for). alo's
own windows open only when the person opens them, so each asks every time. A
task window's space includes **the edge above it**, which carries its Close.
The record now counts among the windows a new one must not cover, as Settings
and the terminal already did.
