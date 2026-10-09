# The external window edge

**Status: the owner's specification, with every figure measured off the design
file rather than read off a picture.** Recorded before implementation, which is
`CLAUDE.md`'s first standing rule.

**Date:** 2026-10-09.

**The design:** `Canvas / External window edge`, node `398:26305` of
`nDxyF5Ho9oC4RObjVzwBNJ`, seven variants. Read live on 2026-10-09 and committed
at `figma-snapshot/398-26305.xml` — **it is not in `70-28.xml`**, whose export
of 2026-10-07 stops at node ids `394:*`, so the component was drawn after it.

**It supersedes the earlier 56px strip proposal**, by the owner's direction.

## What it is, and the one thing it must never be

The edge belongs to **alo's shell**, sits **outside** the application's content,
and is attached to the window.

- never insert controls into an application's own toolbar;
- never crop, cover or recolour application content;
- it moves with its window;
- it is included in placement and reachability — `where-a-new-window-opens.md`'s
  handle is on this edge;
- **revealing or concealing it must not resize or shift the application.**

**Decoration ownership is read from the application's protocol state, never
guessed.** Not from a screenshot, not from the application's name, not from
what its titlebar looks like. `crates/alo-shell/src/surfaces.rs` already answers
every `xdg_decoration` request with `Mode::ServerSide`; what an application then
does is a fact to be read, not inferred.

## Two cases

| the application | alo's external edge carries |
|---|---|
| supplies its own decorations | a movement affordance and a window menu — **no duplicate minimise/maximise/close** |
| delegates decorations to the shell | a movement affordance, the window title, and alo's window controls |

An application with incomplete controls still reaches every shell action
through the window menu and the keyboard.

## The measurements

All logical units. Node ids are the design's own.

| | |
|---|---|
| external interaction region | **44** high |
| visible strip | **32** high, at **y = 12–44** inside the region |
| each control target | **44 × 44**, non-overlapping |
| control artwork | **14 × 14** |
| resting grip | **24 × 2** |

**Horizontal layout, measured from `398:26245` and `398:26232`:**

| | shell-owned | application-owned |
|---|---|---|
| drag region | x 8, width 444 | x 8, width 540 |
| controls | Minimise 456, Maximise 504, Close 552 | Menu 552 |

So the rule, derived from both and holding for each: **controls are 44 wide,
4 apart, the last ending 4 from the trailing edge; the drag region starts at 8
and runs to 4 before the first control.** `600` is the component's specimen
width and **not a window width** — the edge adapts to each window's real
bounds, controls stay at the trailing edge, and every remaining unit goes to
dragging and the title.

**At rest, both owners are geometrically identical** (`398:26228` and
`398:26241`): no visible strip at all, a drag region at y 28 height 12, and the
24 × 2 grip centred in it — absolute centre x 302 of 600, y 33.

**Artwork is centred in the visible strip, not in the target.** x 15 of 44 is
centred horizontally; y 21 is 9 below the strip's top at y 12, and
`(32 − 14) / 2 = 9`.

**Dragging is Reveal's geometry, node for node** (`398:26290`). What changes is
appearance.

Apply display scaling **once**. Keep drawing and hit testing consistent with
the canvas transform. At overview scales where targets become too small, use
the canvas selection interaction rather than pretending tiny controls are
usable.

## Appearance, from the file's own variables

Read with `get_variable_defs` on each variant.

| role | variable | value |
|---|---|---|
| surface | `bg/surface` | `#ffffff` |
| border | `border/default` | `#e7ebef` |
| title and controls | `text/primary` | `#102a43` — navy |
| secondary | `text/muted` | `#596b78` |
| hovered control | `bg/cool` | `#eef2f4` |
| Close on hover | `status/danger` | `#b42318` |
| typography | `Label/Medium` | Manrope SemiBold 13, line height 18, letter spacing 0.1 |
| corners | `radius/sm`, `radius/md` | 8, 12 |

**`status/danger` and `bg/cool` resolve on the Close-hover variant and on no
other**, which is the measurement behind *danger-coloured Close icon on hover*
and *subtle highlight only on the hovered control*.

**The keyboard-focus variant resolves no variable the Reveal state does not**,
so the navy focus outline is `text/primary` rather than a colour of its own.

Button backgrounds are transparent at rest. **No teal** for ordinary window
controls — teal is alo's, per `the-canvas-as-a-workspace.md` §6. No permanent
square tiles, no bouncing, no hover enlargement.

**Maximise/restore and full screen stay distinct.** `docs/features.md` separates
them deliberately, having conflated them once. Tooltips and accessible names
describe the action actually performed.

## Reveal

Reveal when the pointer enters the interaction region, or keyboard navigation
asks for its controls. Keep it revealed while:

- the pointer is in the region or on its controls;
- keyboard focus is inside it;
- its window menu is open;
- a drag or related interaction is in progress.

**A continuous pointer path, and no timed grace period.** A person must never
have to move quickly to reach a control.

**Hover must never move, restore, hide or close the application**, and must
never change which windows exist or are visible. A full-screen window uses the
established viewport-edge reveal rather than reserving an external margin.

## Interaction, through the production host

- dragging the movement region moves the window;
- buttons and menus do not start a drag;
- Minimise puts the window aside through the existing mechanism;
- Maximise/restore preserves ordinary geometry;
- Close uses the application's normal close request and its unsaved-work
  handling;
- keyboard actions remain available in every decoration mode;
- borderless applications stay movable and recoverable.

**One authoritative geometry calculation** serves rendering, pointer targets
and reachability. Host-owned camera and input state stays in the host and is
passed in explicitly — the same rule `where-a-new-window-opens.md` carries, and
the same reason: a second copy is a second answer.

## What completion requires

Real applications in all three states — shell decorations, application-owned
decorations, borderless — checked for:

no duplicate title-bar controls; no application content obscured; **no content
shift between resting and revealed**; pointer travel onto every control without
it disappearing; keyboard focus, menu interaction and dragging all holding the
reveal; correct alignment after moving, resizing, zooming and changing display
scale; reachable handles around the Dock and other fixed controls; and correct
minimise, maximise/restore and close.

**The Figma prototype demonstrates hover transitions only.** It is not evidence
that dragging or window actions work.

> Do not mark this complete merely because the drawing component exists.

## What this supersedes

`crate::window_name_band`'s `BAND_IS_TALL` is **48**, from `Canvas / Window
title` — the band *inside* a shell-drawn window. This edge is **outside** the
window and its strip is 32 in a 44 region. **They are different surfaces and
both numbers are current**; anything reading one for the other is wrong.
Whether the internal band survives alongside this edge is the owner's and is
not decided here.
