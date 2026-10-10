# The Dock has an implementation contract, with exact numbers, and nobody had read it

- **Contributor:** the Mac lane
- **Date:** 2026-10-09
- **Measured from:** the live Figma file `nDxyF5Ho9oC4RObjVzwBNJ`, node
  **`348:28451`**, *Implementation contract*, 4520 × 324, eight text nodes. Also
  present in the committed snapshot `docs/design/figma-snapshot/70-28.xml` since
  **2026-10-07**.

## Why this is a report and not a change

**The numbers were already there and the lane building the dock did not know.**
Two lanes spent an afternoon measuring a 14-pixel bar, inferring a design from a
token, and arguing about whether 14 contradicted 44 — while a node in the
committed snapshot named the glyph, the target, the margins, the lane
coordinates and the reveal rule outright.

Nothing here is a new decision. It is a pointer, so the next person measures
instead of inferring.

## What the contract says, quoted from the live file

> **Dock size is content-driven, with 24 logical px margins after reserved
> regions. Icons may reduce from 32 to 28; targets remain 44×44. At the limit,
> overflow keeps all applications reachable. Never shrink text or rotate the
> composer.**

> **Side Dock + shelf on the same edge: shelf owns y84–180 of the edge; Dock owns
> y192–960. Dock occupies the outer lane (x24–92 left, x1348–1416 right). The
> expanded shelf opens inward at x116 or x1072. The 12px gap is a hover-safe
> travel corridor, not a click blocker.**

> **Reveal is latched while pointer is in the assigned strip, actual surface or
> its travel corridor; also while keyboard focus, a child menu or drag belongs to
> it. No hide delay. Only active surface regions intercept clicks; hover-only
> regions do not.**

Five more text nodes cover the top dock's rows, keyboard order, accessibility,
top controls as overlays, and what the prototype does and does not stand for.

## Held against `crates/alo-dock/src/measures.rs`

**The number that matters agrees exactly.**

| | design | code |
|---|---|---|
| **pitch**, one icon to the next | `44` target + `12` corridor = **56** | `ICON 48` + `GAP 8` = **56** |
| **target** | `44 × 44` | **44** — `alo_appearance::targets::ENHANCED_TARGET` |
| **glyph** | `32`, reducible to `28` | **absent** |
| **icon cell** | — | `ICON` = **48**, *"the side of an application's icon in the dock"* |
| **margin** | `24` after reserved regions | `MARGIN` = **8**, *"between what the dock holds and each of the dock's two faces"* |

**Three quantities, not one, and the contract uses all three in one sentence.**
A glyph sits inside a target sits inside a cell: `32 ≤ 44 ≤ 48`. They nest, and
§8 of the dock specification says outright that *larger targets are valid*. So
`ICON = 48` against *targets remain 44×44* is **not** a contradiction, and the
two margins are measuring different things — the bar's internal padding and the
inset from reserved regions.

**This was nearly published as a discrepancy twice.** The third PC first
compared a 14-pixel drawn extent against the 44 target and asked rather than
committing; this lane supplied the arithmetic. The second time they handed over
the contract's numbers without a conclusion for the same reason. **Both times the
thing that prevented a false finding was handing numbers to whoever owned the
file.**

## What is genuinely missing, with a number attached

**A glyph measure.** The contract says 32, reducible to 28, and `measures.rs`
has none — because nothing draws a glyph. That is the fourth layer of the dock's
emptiness, now specified rather than open:

1. `desktop_raster` passes a literal `0` for what the dock holds — and its own
   comment says *true rather than a placeholder*, which is **correct**;
2. nothing outside `alo-dock` ever builds a `Holding` — the only `.pin()` calls
   are its own tests;
3. a `Holding` needs a `Windows`, *every window open on this machine in the order
   last used*, and nothing tracks that;
4. **no glyph measure, and nothing produces an icon's picture** — the crate that
   knows what is installed, `alo-applications`, has no icon concept in 17 files.

So the 16-pixel bar — `MARGIN + MARGIN`, with nothing between — is the honest
width of a dock holding nothing on a machine where nothing is pinned, nothing is
open and nothing is installed. **A consequence, not a defect.**

## One measurement the design and the code disagree on, slightly

The contract allocates the side dock the lane `x24–92`, **68 logical pixels**.
The code's cross-axis is `MARGIN + ICON + MARGIN` = **64**. The drawn frame
`348:25480 Dock / Left` measures **70 × 391**.

Three numbers within six pixels of each other, and the code fits inside the
allocated lane. **Not reported as a fault**, because *dock size is
content-driven* and a lane is an allocation rather than a width. Recorded so
that whoever draws the icons knows the lane is 68 and the code assumes 64.

## What this says about the board

**Nothing in this repository counts a capability that is built and has no
caller**, which is why the board reads *code written* for a dock whose
interaction contract, measures, overflow, previews, reveal rule, labels and
accessible names are all written and tested, and which a person sees as a sliver.

Three such gaps were found on 2026-10-08 — the night light, the desktop's
surface, and the dock's contents. Two are now wired. **That shape of gap is worth
counting**, and this report is the third naming of it rather than the first.
