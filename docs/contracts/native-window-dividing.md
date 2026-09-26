# Native display dividing and desktops

Status: additive trusted Rust session API, 2026-09-26. ADR 0002; v0.5 shell
scope. No protocol, agent verb, application-adapter or stored-format change.
This replaces [`native-window-tiling.md`](native-window-tiling.md), which is
withdrawn: the half it described is gone rather than deprecated.

## The one layout decider

`alo-shell` decides no layout. Where a share is, whether a split is allowed, how
wide each side ends up and what happens when a window closes are all
`alo-dividing`'s answers; which desktop a switch reaches is `alo-desktops`'. The
`Server` holds an instance of each per display and turns their answers into
configures and pixels. That is the whole of the contract's shape, and it is
checked rather than asserted: `alo-shell` declares exactly one crate that
decides where a window goes, read from the manifests.

## What a session holds

`Server::display_arrived(display, named, area, promises, open)` begins the
desktops on a display and restores the division it had when it last left.
`named` is the screen's own written form — the key `alo-dividing` remembers a
division by, so the same screen returning finds what it left and a different one
does not inherit it. `open` answers which window each remembered share is held
by **now**: a share whose application is not running is left out and the rest of
the tree collapses onto what remains, so coming back with one of two
applications gives that one the whole display rather than half and a hole. The
restored division therefore carries today's window numbers and never yesterday's.
Refuses `NotADisplay::AlreadyThere` for a display already here; plugging the same
display in twice would take the first's desktops away.

`Server::display_left(display, named, who)` remembers the division under
`named`, drops it from the live set and hands back what was on the display.
Nothing is closed and nothing is moved — that is `alo-desktops`' rule and this
does not extend it. `who` names the application each window belongs to, which is
what makes the arrangement outlive the window numbers. Refuses
`NotADisplay::Unknown` for a display that was not here.

`Server::dividing(display)` is how this display is divided now, or `None` where
nothing has divided it. `None` is a display undivided, never an empty tree.
`Server::divide(display, division)` replaces it with one `alo-dividing`
decided — a whole `Division`, not an edit to one, because what a division *is*
is that crate's to say. `Server::desktops_on(display)` and
`Server::switch_desktop(display, switch)` are the same shape for desktops:
`alo-desktops` decides which desktop a switch reaches, including whether it
wraps and whether there is one, and its refusal is carried whole.
`Server::has_no_display()` distinguishes a session between the sign-in screen
ending and its first frame from one with an empty display.

A window's number is handed out once and never reused, even after the window
closes. A reused number is a tree still holding a share for a window that is
gone and a different window walking into it. Numbers survive an unmap and
remap — a window that flickers is still the window in somebody's layout — which
is why this is not the fresh-per-mapping visibility identity the compositor also
keeps. Once a frame, the division is told which windows are open; those that
closed lose their shares and the tree collapses.

## Putting a window on a side

`Server::divide_focused_with_next(focused, side)` divides the focused window's
share with the window next in switch order, the focused one on `side`. The side
is `alo_dividing::keyboard::side_for`'s answer to a chord, and every side that
crate names is one this takes — a top and a bottom are shares of a tree exactly
as a left and a right are, which half of an output never could be.

Each window's minimum size is read from its committed XDG state and handed over,
so `alo-dividing` can refuse a split that would squeeze either window below what
it asked for. A window that has stated no minimum is handed over as one with
none, not as one of no size.

Refusals are `NotDivided`: `Dividing(Refused)` carries `alo-dividing`'s own
answer whole and never a sentence assembled here; `NoDisplay` for a session with
nothing to divide; `MoreThanOneDisplay` because which display a window is on is
`alo-displays`' answer and nothing here asks it yet — refused by name rather
than answered with a guess about the first screen.

**A chord with one window open refuses.** A division divides *between* windows,
so a single window has nothing to share with and `alo-dividing` says so. This is
a deliberate loss against the withdrawn tiling API, which would put the only
window on half a display with nothing beside it.

A successful division reconfigures **every** window on that display, not the two
that changed: moving a boundary moves what its neighbours sit against, and
configuring only the touched pair would leave a layout right in the tree and
wrong on the screen. A window the division holds that the compositor no longer
has is passed over; a client's or a mode's refusal to take its share leaves the
layout standing and the next chord asks again. A refused division leaves the
tree exactly as it was, which is `alo-dividing`'s guarantee rather than this
API's.

## What this does not do

No timeout, no forced client resizing, no scaling of a nonconforming buffer and
no engine patch. Divisions are held in `alo-dividing`'s logical units and
multiplied by a display's scale exactly once, at the drawing boundary, so a
share is never stored in pixels. Nothing here reserves a title bar or a dock
work area.

Evidence: `../autonomy/updates/the-division-and-the-desktops-a-session-holds.md`.
