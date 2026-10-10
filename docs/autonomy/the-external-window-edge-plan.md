# The external window edge: what is done and what is left

**What this is.** The remaining work on the owner's external window edge, with
who owns each piece and what would prove it finished. Written on 2026-10-09
because the task spans two lanes and several sessions, and a plan that lives in
chat is one the next session cannot read.

**The contracts it serves:** `docs/design/the-external-window-edge.md` and the
owner's rulings of 2026-10-09 on decoration ownership, on the title, and on the
84px overlay envelope.

**The bar, in the owner's words:** *do not mark this complete merely because
the drawing component exists*, and *completion means the new edge works on real
windows and the old controls have no remaining production rendering or input
path.*

## Done

| | |
|---|---|
| 1 | the geometry — region, strip, grip, controls, drag, title, reproducing the design's own coordinates |
| 2 | both decoration cases — app-owned gets a menu and no duplicate buttons |
| 3 | the exact dimensions, measured off `398:26305` rather than read off a picture |
| 4 | the appearance — every colour an existing role, nothing drawn behind a control at rest |
| 5 | the title — Manrope bundled and drawing, grapheme-aware truncation, the whole title kept for a reader |
| 6 | the picture — logical units to output pixels once, refusing an edge that would fall off the output |
| 7 | **drawn on real applications**, on the real compositor |
| 8 | both design documents carry the 84px overlay-envelope ruling |

**Measured, and it changes the shape of the removal:** no production binary
draws the old controls at all. Searched `alo-desktop`, the compositor binary,
`direct_desktop` and `booting` with a positive control — nothing. **No shipped
window has ever had chrome**, so the owner's *never ship with duplicate
controls* is satisfied by construction and the removal is of a surface only
fixtures ever drew.

## Left, in the order it has to happen

### A. Accessibility, ported before anything is deleted — **done 2026-10-10**

The owner: *port accessibility before deleting its implementation. Preserve
accessible names, roles, available actions, focus behaviour and state
announcements. Screen-reader and keyboard users must reach the new edge without
first hovering.*

**The reuse is already there and it is better than a port.**
`alo_access::Control::for_action` builds a control whose name *is* the action's
own word — *one string rather than two that must agree*, which that file notes
satisfies EN 301 549 clause 11.2.5.3 by construction. And
`alo_shortcuts::Action` already has `CloseWindow`, `MinimiseWindow` and
`MaximiseWindow` with words.

So the edge's controls map onto actions that exist, and the accessible names
come free and correct in every language.

**And the clause failed on the first attempt, which is why it is in the file.**
`what_a_reader_is_told` began by reading the drawn controls off an `Edge`, and a
concealed edge correctly has none — so a keyboard user could reach nothing until
a pointer had been near a control. The test caught it. It takes `Decorations`
and no geometry now: **revealing is what a person sees and is never what a
person can reach.**

#### What the port actually was, because this plan described it wrongly

This document said the implementation to port was *~1,300 lines of reader chrome
and labels*, meaning `window_control_reader_*`. **That is not a screen-reader
tree.** Read whole, it is a *visible paged name reader* — `Page {page} of
{total}`, *Previous page*, *Next page*, *Done reading* — for a control name too
long to fit a small tile. 2,039 lines, and none of it is the accessibility tree.

The accessibility tree is `access_nodes` → `access_bus` → `access_serving`, and
**the thing actually missing there was not a port at all**: the list of open
windows held names and nothing under them. A reader was told a window was open
and never that it could be closed. So:

| | |
|---|---|
| `window_edge_reading` | which controls a window *has*, from `Decorations` alone |
| `window_edge_who_draws` | who draws a frame's header — **one** answer, asked by both the drawing and the reading, so they cannot drift |
| `access_nodes` | each open window now carries its controls as children |
| the live bus | `the_served_tree_follows_the_windows_that_open` reads all three back off a real at-spi2 bus, with the count asserted before the contents so an empty list cannot pass |

**The seam named for the other lane:** `who_draws_a_frame` answers
`TheShellDraws` for every frame, because `XdgDecorationHandler` gives every
client that asks `Mode::ServerSide` in all three of its methods — that is this
compositor's only answer, not an assumption. Telling apart a client that **never
binds** `xdg_decoration` needs the toplevel's protocol state read per frame, and
the Mac said on 2026-10-09 they *will read `Decorations` from protocol state
only*. **That function's body is what changes, and nothing else.**

**Still not answered:** the window menu has no `Action`, so it has no word, so a
reader is told about nothing of alo's on an application-decorated window.
`the_window_menu_is_the_one_control_with_no_action_yet` records that and will
fail when somebody adds one. It is not acceptable as a permanent answer.

**Found while reading, and it belongs to C:** `alo_access::Surface::WindowControls`
still declares the old tiles as a surface of this machine, with its own controls
read aloud and two tests holding their names. That is in `alo-access`, which is
a public contract — so retiring it is versioning, not deletion.

### B. Reveal and input — **the Mac**, started

§5 and §6 of the contract. Taken on 2026-10-09 with the decoration reading
verified independently, and the `target` versus `highlight` pair acknowledged.

**And one fault belongs here rather than with the removal:**
`which_surface_claims_a_point` gives the **whole 84-tall rectangle** to the top
controls whenever the band is reserved, while nothing is drawn in it — so an
application beneath is blocked by controls that do not exist. That is the
owner's *hidden controls must not block applications beneath them*, and it is
pointer-region work.

### C. The removal — **this lane, after A and B**

~30 production files, one call site outside the family. Three parts, and they
are not one job:

| | |
|---|---|
| the old tile drawing | replaced; removable once nothing imports it |
| the old input and hit regions | the Mac's replacement must land first |
| ~1,300 lines of reader chrome and labels | **A must land first** — this is the implementation A ports |

**Keep and adapt the tests that protect behaviour.** The guarantees worth
carrying across: non-overlapping targets, clipping at the viewport, nothing
drawn over client pixels, and the accessible names.

### D. The last documents — **done 2026-10-10**

The owner: *update documentation and design references so nobody builds the
retired controls again.*

| | |
|---|---|
| `the-regions-a-pointer-can-be-in.md` | the 84px overlay-envelope ruling |
| `the-external-window-edge.md` | the contract itself, and *Return to canvas* staying |
| `contracts/native-window-controls.md` | **superseded**, with a notice at the top saying by what |
| `design/the-canvas-as-a-workspace.md` | *a shared window component* now names the component |

**The contract was deprecated rather than rewritten or deleted**, because
`docs/contracts/` is a public surface and CLAUDE.md's rule is that it changes
additively and a break requires versioning and deprecation. Deleting the page
would make every link to it dead and tell nobody why. Its labelling rule —
ADR 0089, one string rather than two that agree — is **not** superseded; the new
edge reuses it unchanged.

**Deliberately left alone:** `docs/autonomy/updates/*` are task reports and
record what was true when they were written; rewriting history to agree with the
present is how a repository stops being evidence of anything. `QUEUE.md` and
`STATE.md` have one writer, who is not this lane. And
`contracts/native-window-dividing.md`'s *nothing here reserves a title bar or a
dock* was already right and is more right now.

## What would prove it finished

Real applications in all three states — shell-decorated, application-owned,
borderless — with screenshots of resting and revealed, and:

no duplicate title-bar controls; no application content obscured; **no content
shift between resting and revealed**; pointer travel onto every control without
it disappearing; keyboard focus, menu interaction and dragging all holding the
reveal; correct alignment after moving, resizing, zooming and changing display
scale; reachable handles around the Dock; and correct minimise, maximise and
close.

**The Figma prototype demonstrates hover transitions only.** It is not evidence
that dragging or window actions work.
