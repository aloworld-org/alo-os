# The interface in the file, and what this machine actually does

**The design file is the visual reference.**
`https://www.figma.com/design/nDxyF5Ho9oC4RObjVzwBNJ`, page *07 — alo OS ·
Living canvas* (`70:28`), read on 2026-09-26: **144 top-level frames**, with
`01 · Your canvas` at `70:29`, Foundations at `8:2` and Components at `9:2`.

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

| # | What the file shows | What the shell does | Milestone |
|---|---|---|---|
| 1 | deep teal for alo, navy for text and human controls | terracotta for the agent; navy is text already | **v0.5** |
| 2 | the four-corner open-square mark beside alo | a mark exists beside the agent; it is not that shape | **v0.5** |
| 3 | network, volume and brightness in the status area | shipped — clock, battery, network, volume | **done** |
| 4 | one `Canvas / Window title` component | one title component, after the file's own cleanup | **done** |
| 5 | a movable, resizable canvas of windows | no canvas | **v1** |
| 6 | viewport controls fixed while the canvas moves | nothing to be fixed against | **v1** |
| 7 | the minimized panel, collapsed and expanded | no panel | **v1** |
| 8 | selection, arrange, group, *Give to alo*, Stop | no multi-selection | **v1** |
| 9 | full screen with edge-revealed controls | full screen without the canvas's edges | **v1** |
| 10 | the zoom menu — *Zoom out* and *Show all* to the overview, *Zoom in* to the closer canvas | no zoom | **v1** |
| 11 | a dark canvas | light and dark schemes exist in `alo-appearance` | **v0.5 tokens, v1 canvas** |

## Conflicts, unresolved

**Compact is in no frame.** The file draws Minimize and the shelf; the word
*Compact* does not appear anywhere on the page. The owner's direction of
2026-09-26 keeps **two distinct actions** — Compact leaves a live tile on the
canvas, Minimize puts a preview in the fixed right panel — and
`docs/features.md` still carries the older, stronger sentence, *nothing is
swallowed into a bar*, which the two-action model replaces. The features line is
updated in the same change as this document; **the Compact state remains
undesigned**, and that is the conflict: an approved behaviour with no drawing.

**One dark frame out of 144.** `34 · Your canvas / Dark` is the only one. A
scheme is not a screen, and every surface a person meets at night — sign-in,
settings, approvals, the shelf, full screen — is drawn once, in light.

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
