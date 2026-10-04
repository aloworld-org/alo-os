# The interface in the file, and what this machine actually does

**The design file is the visual reference**, and since 2026-10-04 it is also
**committed**: `docs/design/figma-snapshot/` holds the node tree, so a design
change arrives as a diff instead of as nothing. Read that, not Figma.

`https://www.figma.com/design/nDxyF5Ho9oC4RObjVzwBNJ`, page *03 — alo OS ·
Living canvas* (`70:28`), with `01 · Your canvas` at `70:29`, Foundations at
`8:2` and UI primitives at `9:2`. Every known root is listed in
`figma-snapshot/ROOTS.md`, because the interface lists only the cover.

**The page is named *03*, not *07* — and *07* is a frame.** `78:1225` is
`07 · Move a window`, at depth 2 on this page. So both halves of the wrong
sentence were real: a page of that name and a thing numbered 07. **That is how it
survived being read** — nothing in it was invented, the number was attached to the
wrong noun.

It was in **two** documents, not one: this line, and
`the-canvas-in-numbers.md`, both corrected on 2026-10-04. Fixing one and leaving
the other is the fault `#451` had to be amended for, in the opposite direction.

**And the number cannot identify a frame anyway.** Measured from the snapshot:

```text
frames at depth 2 whose name opens with a number   142
distinct numbers used                               49
numbers used more than once                         15
worst case: 14 is twenty-four frames  — 14 · Proposal rejected, then a
            whole 14 · Setup / … sequence
            10 is eleven frames, 16 and 17 are nine each
```

So a citation reading *frame 12* names five frames and *frame 11* names eight.
**Cite the node id, or cite the family and the number together** — `Minimized
panel / 07 · Collapsed rail` is unambiguous because the family narrows it;
`07` alone is not.

## The inventory, counted on 2026-10-04

Measured from the committed snapshot. **Each class is a rule, so the count can be
re-taken rather than re-judged:**

| | | |
|---|---|---|
| drawn at 1440×960 | a screen or an interaction state | **196** |
| `<symbol>` | a component or component set | 6 |
| `<text>` at page level | an annotation: section heading, title | 12 |
| `<frame>` not at 1440×960 | a long-form document, not a screen | 3 |
| `<section>` | a group, counted by its contents below | 1 |
| | direct children of `70:28` | **218** |

```text
section 351:25693  Dock edges · v0.01 specification  4680x7400
  37 children, of which 17 are drawn at 1440x960
```

**So screens and states on this page: 196 + 17 = 213.**

**The 196, by family.** Counted without truncation, which is not a free remark —
see below.

```text
142  (bare number) · …                    e.g. 01 · Your canvas
 26  Dock NN · …                           Dock 01 · Resting … Dock 26 · alo working
 12  Minimized panel / …                   01 · Before minimize … 12 · Empty handle
  9  Selection / …
  2  Full screen / …      2  Move / …      2  Resize / …
  1  Figma demo index · Everyday shortcuts
```

**142 of the 196 carry only a bare number**, which is the form that cannot identify
a frame: `14` alone is twenty-four of them. The 54 that carry a family —
`Minimized panel / 07`, `Dock 11` — resolve, because the prefix narrows the number.
So the citable half of this page is the smaller half, and that is a fact about the
file rather than a complaint about it.

**This table was nearly wrong, by the same mechanism as the `07` error.** The first
listing of non-numbered children was piped through `head -50`; it would have printed
76 lines, so 40 went unseen — including all 26 `Dock` screens and **all twelve
`Minimized panel` frames**, which are the ones `putting-a-window-aside.md` cites.
They were found by a separate targeted search, so the omission cost nothing; had the
truncated list been trusted, the twelve frames this repository most needs would have
been reported absent. **A number or a list that was truncated is one that was not
read**, and the counts above come from passes with no `head` in them.

**And the 144-versus-199 reconciliation, which is arithmetic once the definition
is fixed.** This document said *144 top-level frames* on 2026-09-26. Today the
same measure — direct `<frame>` children of `70:28` — is **199**, so 55 were
added. Those 199 split exactly:

```text
199 top-level frames = 196 screens at 1440x960 + 3 long-form documents
```

The 17 screens inside the section were **never** in either count: a section's
children are not direct children of the page, so a frame count misses them in
both directions. **One of them is the node the owner named as a discovery check**,
`351:25693`, which is the section itself.

**A frame is design evidence and not authorisation.** `CLAUDE.md` binds building
to `docs/features.md` with a tier, inside the current release. Fifty-five new
frames are fifty-five things to read, not a licence.

This document exists because a drawing and a working machine are different
claims, and the gap between them is the only honest measure of how much is left.
**Nothing here is called implemented because it is drawn.**

## What the file covers

Read from the page's own structure rather than from a summary of it: the canvas
and its eleven windows, focus, move and resize per window, full screen with its
bottom controls and right panel, the minimized shelf collapsed and expanded,
restoring, closing, the zoom menu, canvas options, History, *Find, open, or
ask*, first start, **No AI**, proposals kept and rejected, per-application
control sets for documents, browser and Blender, *Keep controls visible*,
**reduced transparency**, selection with arrange, group, *Give to alo*, alo
working, paused and stopped, *window access removed*, and a dark canvas.

That is a more complete interaction design than any written specification of it,
and the states below are named as they are named there.

## What the shell supports today

`crates/alo-shell` draws, and has tests for: the lock screen, sign-in, the
desktop, settings, the record window, approvals, recovery, notifications, the
dock, the status area, screen division, capture, and a large window-control
surface including its screen-reader path.

It does **not** contain a canvas, a viewport/plane separation, Places, a World,
a minimized panel, an alo Bar, window multi-selection, or Compact. Those are not
half-built: there is no module for any of them.

## The discrepancies, as tasks

Each row is a task somebody can take. **Milestone** is where it belongs, not
where anyone wishes it were.

**Re-measured on 2026-10-04, and this table understated the machine by six rows.**
It was written on 2026-09-26 and last touched on 2026-09-27, and said *no canvas*,
*nothing to be fixed against*, *no panel* and *no zoom* — all four of which exist
and are reached through the production path. **Two canvas documents cite this table
as their arbiter**, so a stale cell here is a stale premise there, which is why it
is corrected rather than noted.

**What was corrected and what was not, because the two columns have different
authorities.** *What the shell does* is a claim about this repository and is
measured here. *What the file shows* is a claim about the Figma file — and **this
lane has no Figma access**: no `mcp__claude_ai_Figma__*` tool in its session, no
token, and `WebFetch` answers 403 on a private file. So **not one cell in that
column has been touched**, and if the design has changed since 2026-09-26 this
document would not show it. The laptop lane can read the file; this one cannot, and
a measurement only one lane can take is a measurement nobody can check.

**And the Milestone column is superseded for every canvas row.** Six rows read
`v1`; [ADR 0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md)
put every agreed canvas capability into one current milestone on 2026-10-04, so they
read **Complete canvas**. The owner's own words on what that means are the reason
this table exists: *a model, unused function, **Figma state** or isolated test is
progress — not completion.*

| # | What the file shows | What the shell does | Milestone |
|---|---|---|---|
| 1 | deep teal for alo, navy for text and human controls | deep teal for the agent; navy is text already, human controls unchecked | **v0.5** |
| 2 | the four-corner open-square mark beside alo | a mark exists beside the agent; it is not that shape | **v0.5** |
| 3 | network, volume and brightness in the status area | shipped — clock, battery, network, volume | **done** |
| 4 | one `Canvas / Window title` component | one title component, after the file's own cleanup | **done** |
| 5 | a movable, resizable canvas of windows | **built** — `alo-canvas` holds the plane, the camera, `Place` and `World`; frames are dragged and resized through the production path | **Complete canvas** |
| 6 | viewport controls fixed while the canvas moves | **built** — four fixed controls, and a frame keeps 44 × 24 of its name clear of every one of them; compared against each other on a real draw | **Complete canvas** |
| 7 | the minimized panel, collapsed and expanded | **built** — it conceals until reached, reveals, groups by Place, and gives a window back on a click; 15 integration tests in `alo-put-aside` | **Complete canvas** |
| 8 | selection, arrange, group, *Give to alo*, Stop | **still no cross-frame selection**, which is the half this row is about. *Give it to alo* and Stop are now `[v0.01]` | **Complete canvas** |
| 9 | full screen with edge-revealed controls | **partly** — the panel's edge reveal is built and the top controls give way to a full-screen window; whether the *controls* appear on reaching the edge is task 6 of `putting-a-window-aside.md` and not this lane's to claim | **Complete canvas** |
| 10 | the zoom menu — *Zoom out* and *Show all* to the overview, *Zoom in* to the closer canvas | **built** — `Zoom::of`, `FURTHEST_OUT`, `LIFE_SIZE` and `canvas_show_all.rs`, on three roads a person has | **Complete canvas** |
| 11 | a dark canvas | light and dark schemes exist in `alo-appearance` | **v0.5 tokens, v1 canvas** |

## Conflicts, unresolved

**Compact is in no frame — false since some time before 2026-10-04, and still
true of what it was protecting.** The word now appears twice: `40 · Notification
/ Compact` at page level, and `Notification / Working · compact` inside it. Both
are **notifications**, and `9:2` confirms it — the notification component carries
`Layout=Compact` as a variant beside `Layout=Regular`, across five kinds.

**So the claim is wrong as a statement about the word and right as a statement
about windows.** *Compacting a frame to a live tile on the canvas* — the thing
`docs/features.md` promises — is drawn nowhere. Correcting this one to *the word
appears* would have lost the finding; correcting it to *Compact is designed now*
would have been false. The owner's direction of
2026-09-26 keeps **two distinct actions** — Compact leaves a live tile on the
canvas, Minimize puts a preview in the fixed right panel — and
`docs/features.md` still carries the older, stronger sentence, *nothing is
swallowed into a bar*, which the two-action model replaces. The features line is
updated in the same change as this document; **the Compact state remains
undesigned**, and that is the conflict: an approved behaviour with no drawing.

**One dark frame out of 144 — false since some time before 2026-10-04.** There
are two at page level, `34 · Your canvas / Dark` and `42 · Notification / Dark`,
and `9:2` shows the notification component drawn in `Theme=Dark` for **every**
one of its five kinds, in both layouts and both focus states.

So the sentence that followed it is wrong too: a person meeting a notification at
night now meets a drawn one. What remains true is the shape of the gap —
**sign-in, settings, approvals, the shelf and full screen are still drawn once, in
light.** A scheme is still not a screen, and the count that mattered was never the
number of dark frames but which surfaces have one.

**Foundations does not match the approved palette.** `text-primary` is
`#07131F`, not navy `#102A43`, though navy is the approved colour for primary
text and human controls and sits in the file as an unused swatch. Cool surface
`#EEF2F4` is absent. Only `status-positive` exists — no warning and no
destructive, which the direction asks to keep separate. The swatch frame is
still **named** `Orange` although its label now reads *Deep teal*, and a
generator that reads layer names will read the old word.

**The variables are named as CSS custom properties** — `var(--alo-color-…)`.
There is no CSS in alo OS and there will not be
([palette.toml](palette.toml) says so at the top); the source is that file and
the target is `alo_appearance::Token`. The names cost nothing to keep and will
mislead every implementer who reads them.

**The alo mark is a diamond** in the canvas frames and a circle on the cover.
The approved mark is the four-corner open square.

## What this document is not

It is not permission to build anything. Rows marked **v1** are v1 because
`docs/features.md` tiers them there and the release in progress has a gate to
pass. A drawing does not move a milestone.
