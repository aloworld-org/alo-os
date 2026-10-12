# The external window edge

**Status: the owner's specification, with every figure measured off the design
file rather than read off a picture.** Recorded before implementation, which is
`CLAUDE.md`'s first standing rule.

**Date:** 2026-10-09.

**The design:** `Canvas / External window edge`, node `398:26305` of
`nDxyF5Ho9oC4RObjVzwBNJ`, seven variants. Read live on 2026-10-09 and committed
at `figma-snapshot/398-26305.xml`, **which is authoritative for this component
while `70-28.xml` is stale for it** — the page export does not contain it at
all.

**Freshness is not claimed from node ids.** The owner's direction: *node IDs
alone do not establish freshness: existing 398:\* nodes were edited, and new
visual elements were added under 402:\*.* The capture evidences itself by
being read from the live file on 2026-10-09, and by containing `402:26312`,
`402:26313` and `402:26318` — the compact-size elements themselves.

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

## The decoration decision

The owner's ruling of 2026-10-09, which settles what is drawn for each kind of
window:

| window | what appears |
|---|---|
| the application draws its own header | **keep that header**, and add alo's external movement edge and window menu — **no duplicate window buttons** |
| the shell supplies decorations | the external edge carries the title, the movement and the controls — **and `Canvas / Window title` is not also drawn inside** |
| a borderless application | its content is preserved, and it gets the external movement edge and access to shell actions |

An application with incomplete controls still reaches every shell action
through the window menu and the keyboard.

### The internal 48px band is superseded, and this is what that means

**For ordinary shell-decorated windows the internal `Canvas / Window title`
band is replaced by this edge.** Two bands on one window is the duplicate
surface this ruling exists to prevent.

**That it is still in Figma does not authorise a second band.** A component
left in a design file is not an instruction to draw it, and this is written
down because the next person to find `Canvas / Window title` — 1,814 instances,
48 tall — will reasonably assume it is current.

**Shared implementation pieces are kept where useful; the duplicate surface
leaves the rendering path.** So `crate::window_name_band`'s arithmetic may
survive as arithmetic — a band's rectangle, a reachable handle — while nothing
draws a second band on a window that has this edge.

**And the top overlay keeps *Return to canvas*.** The owner's ruling of
2026-10-09: moving window controls onto this edge *does not remove this
navigation action*. What leaves the top overlay is the duplicated window
controls — minimise, maximise and close — because a copy of them there would be
the duplicate surface this replacement exists to prevent. The overlay's own
84-pixel region stays as an **overlay envelope and not a reserved desktop
band**; `the-regions-a-pointer-can-be-in.md` carries that ruling in full.

**What this does not touch:** `canvas_never_lost`'s `A_USABLE_HANDLE` of
44 × 24. That is the minimum reachable grab area, it is about not losing a
window, and this edge's 44-high interaction region accommodates it rather than
replacing it.

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

## A title a person can read, however long it is

**The owner's ruling of 2026-10-10**, which closes a gap this edge created. The
title is truncated with an ellipsis when it does not fit, and until this ruling
the whole string survived **only for a screen reader** —
`EdgePicture::the_whole_title`. A sighted person with a narrow window could not
read their own document's name, and nothing on the screen said there was more.

That gap had a predecessor and it is worth naming, because it is the reason this
ruling is not simply *make the edge wider*. `window_control_reader_*` — 2,039
lines, retired on 2026-10-10 — was a **paged name reader**: *Page 2 of 3*,
*Previous page*, *Done reading*, built so a person could read a control name too
long for a small tile. It had **no production caller, ever**, measured with a
positive control. So the problem was recognised, answered at length, and never
reached anybody. Deleting it took nothing away from a person and left the
question open.

> **A sighted person must be able to read the complete title.**
>
> * The window edge uses grapheme-safe truncation with an ellipsis, measured in
>   the actual Manrope font.
> * Hovering the title or focusing it with the keyboard reveals the full title
>   in a wrapping tooltip, positioned within the screen and clear of the window
>   controls.
> * The tooltip must not interfere with dragging or button interaction.
> * The window menu also exposes the full, selectable title, providing a
>   persistent way to read and copy it with pointer, keyboard or touch.
> * Screen readers always receive the complete title.

### Three ways to the same string, and why it is three and not one

Each clause answers a different person, which is why none of them is redundant:

| | what it is for |
|---|---|
| the **ellipsis** | somebody who only needs to know there is more |
| the **tooltip** | somebody reading it now, with a pointer or the keyboard |
| the **window menu** | somebody who needs it to stay still — to read slowly, to copy it, or to reach it by touch |

**The tooltip alone would have failed two people.** It is transient by nature,
so it cannot be read slowly, cannot be copied, and on a touch screen there is no
hover to produce it. *Pointer, keyboard or touch* in the ruling is what makes
the menu's half non-negotiable: a tooltip is the convenient road and the menu is
the one that always exists.

### What this costs, said plainly

**The window menu does not exist.** `crates/alo-shell/tests/support/wm_capabilities.rs`
uses `WindowMenu` as its example of a capability this compositor deliberately
does **not** advertise, and asserts the set is exact without it. So
`OnTheEdge::Menu` has been a control with no action and no word —
`window_edge_reading`'s `what_it_does` answers `None` for it, with a test that
fails the day somebody gives it one.

This ruling is that day. A menu with a selectable title is the first thing the
window menu is **for**, rather than a surface built and then furnished.

### The three already true, and the measurement behind each

- **Grapheme-safe truncation** — `window_edge_title::fitted` cuts on
  `grapheme_indices(true)`, so a combining accent is never separated from its
  letter and an emoji sequence is never split. Held by
  `a_combining_accent_is_never_cut_from_its_letter` and
  `an_emoji_sequence_is_never_split`.
- **Measured in the actual Manrope** — `painted_text::how_wide` shapes through
  the bundled face rather than estimating, and
  `the_face_alo_asks_for_is_the_face_that_shapes` asserts the face that answers
  is the face asked for. The font matters to the arithmetic: a title fitted
  against Inter and drawn in Manrope is a title that overruns.
- **Screen readers get the whole string** — a window's node in the tree is named
  from `FrameName`, which is the application's own untruncated string. The
  truncation happens in the picture and never in the tree.

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

**And a long title reached three ways**, which the owner's ruling of 2026-10-10
asks for by name: *verify long-title access through pointer, keyboard and the
window menu before marking them complete.* Three roads, three checks, and a
tooltip that appears on hover is evidence for exactly one of them.

> Do not mark this complete merely because the drawing component exists.

## What this supersedes — settled 2026-10-09

An earlier version of this document said the internal 48px band and this edge
were *different surfaces and both numbers are current*, and left whether the
band survives as an open question for the owner.

**The owner has answered: it does not, for ordinary shell-decorated windows.**
See *The decoration decision* above. 32-in-44 is the live measurement; 48 is
the superseded internal band; and the duplicate surface comes out of the
rendering path while any useful arithmetic may stay.
