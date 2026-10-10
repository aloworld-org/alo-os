# The Dock's visual specification, measured from the canonical Figma

**Read this before changing anything the Dock draws.** `the-alo-dock.md` carries
the Dock's behaviour and the owner's geometry rulings; this file carries what it
**looks like**, measured node by node from the design file on 2026-10-10.

**Why it is a separate file.** The owner's words of that day, after this lane
showed four screenshots of a working overflow and called the design done:

> This screenshot is useful as an overflow test, but it does not match the
> approved Dock visually. Keep it as a diagnostic fixture, not evidence that the
> design is complete. … The agreed 76px horizontal height and 48×48 targets
> establish geometry; they do not replace the rest of the visual design.
> Likewise, the overflow row minimum is not a complete panel specification.

Geometry and appearance had been collapsed into one record, and a ruling about a
target size was read as settling what a person sees. They are two questions.

## Provenance

| | |
|---|---|
| file | `nDxyF5Ho9oC4RObjVzwBNJ` — *alo OS* |
| measured | 2026-10-10, from the Mac lane, through the Figma MCP server |
| method | `get_metadata` for the tree, `get_design_context` for fills, radii, borders and effects, `get_screenshot` for the render |

**The Mac has Figma access after all.** `docs/design/figma-snapshot/variables.toml`
says *the development PC and the Mac have no Figma access at all*, written when
that was true. It is not any more, and the sentence is corrected there.

**And the committed snapshot cannot answer this.**
`docs/design/figma-snapshot/70-28.xml` holds five attributes per node — `id`,
`name`, `x`, `y`, `width`, `height` — and **nothing else**. No fill, no radius,
no stroke, no effect. Measured: of 21 072 attribute occurrences in that file,
every one is one of those five, plus 791 `hidden`. So every visual question in
this document had to be asked of the live file, and a diff of that snapshot can
never show a colour or a corner changing.

## There are two Dock designs in the file, and they disagree

This is the first thing to know and it is not a detail.

| | `Canvas / Dock + alo Bar` | `Dock edges / Orientation` |
|---|---|---|
| component | **`77:107`** | **`348:23940`** (set) |
| measured from | `321:17430`, the instance in *Dock 01 · Resting* | **`348:23874`**, `Edge=Top, State=Overflow` |
| where it is used | the bottom edge | left, right and top |
| in the section | *03 — alo OS · Living canvas* | ***Dock edges · v0.01 specification*** |
| slot | **52 × 48** | **44 × 44** |
| icon inside it | **24 × 24** | **28 × 28** |
| gap between slots | 8 | 8 |
| corner radius | **16** | `radius/lg` = **18** |
| shadow | `0 4px 16px -2px rgba(16,42,67,.12)` | `Overlay / Glass`: `0 8px 24px #07131F14` |
| backdrop blur | none | **18** |
| padding | `space/3` = 12 | `space/3` = 12 |
| gap between groups | `space/3` = 12 | `space/2` = 8 |
| after the apps | the composer, *Show all* (88 × 44), *···* (44 × 44) | a divider, the composer, *History* (44 × 44), *More* (44 × 44) |

**Neither is 48 × 48**, which is what the owner ruled on 2026-10-10 for the
overflow control's target. The ruling is a target — what a hand must be able to
hit — and these are slots and artwork. A 44 slot cannot hold a 48 target, so one
of the two has to give, and that is the owner's to say rather than this lane's.

**The edge set is the one labelled `v0.01`**, which is this release, and its own
component description is the clearest sentence in the file:

> Viewport overlay. Top edge. 44×44 logical px targets; 28px icon minimum. Open
> indicators distinguish pinned from open. Overflow opens inward. Side composer
> stays horizontal. No automatic scaling by text size.

## The surface

Both Docks are a white rounded panel with a hairline border and a soft shadow.

| | value | token |
|---|---|---|
| fill | `#FFFFFF` | `bg/surface` |
| border | **1** solid `#E7EBEF` | `border/default` |
| radius | 16 (bottom) / **18** (edges) | `radius/lg` |
| shadow | `0 8px 24px rgba(7,19,31,0.08)` | effect style `Overlay / Glass` |
| blur behind it | **18** | the same style |

## The running indicators, which are the thing that was missing entirely

Under each application's slot, flush to its bottom edge, in `text/primary`
`#102A43`. **Three states, told apart by width and not by colour** — which is
what `the-alo-dock.md` promises and had nothing behind it:

| the application | mark | measured | node |
|---|---|---|---|
| focused | a **bar**, 12 × 3, centred | left 16, top 40 in a 44 slot | `348:23880` *Active window indicator* |
| open, not focused | a **dot**, 4 × 3, centred | left 20, top 40 | `348:23886` *Open window indicator* |
| pinned, not open | **nothing** | — | — |

Both sit at `top: 40` in a 44-tall slot and are 3 high, so both end flush with
the slot's bottom edge, and both are centred across it.

## The composer — the *alo Bar* half of *Dock + alo Bar*

`349:25790` on the edge Dock, `I321:17430;74:53` on the bottom one. Identical:

| | |
|---|---|
| size | **344 × 48** |
| fill | `#F5F6F7` |
| radius | `radius/md` = **12** |
| padding | 12, gap 12 |
| prompt | *Find, open, or ask…* — Manrope Regular **16/25**, `text/secondary` `#274C68`, 262 wide |
| shortcut | *alo* — Manrope SemiBold **11/16**, letter-spacing 0.3, 38 wide |

## The controls after it

| | size | radius | content |
|---|---|---|---|
| *Show all* (bottom only) | 88 × 44 | 12 | Manrope SemiBold **13/18**, tracking 0.1, `text/primary` |
| *···* / *More* | 44 × 44 | 12 | the same type, or a 28 × 28 glyph on the edge Dock (`348:23636`) |
| *History* (edge Dock) | 44 × 44 | — | a 28 × 28 glyph (`348:23633`) |
| the divider before the composer | **1 × 28** | — | `border/default` |

## The icons are drawn artwork, and there are seventeen of them

`Dock edges / Button / <name>`, each a 44 × 44 button holding one **28 × 28**
SVG at `left: 8, top: 8`. Docs `348:23590`, Browser `348:23594`, Blender
`348:23598`, Files `348:23602`, Mail `348:23606`, Calendar `348:23609`, Notes
`348:23612`, Terminal `348:23615`, Settings `348:23618`, Music `348:23621`,
Photos `348:23624`, Tasks `348:23627`, Search `348:23630`, History `348:23633`,
More `348:23636`, Calculator `349:25793`, Slides `349:25799`, Recorder
`349:25805`.

Each carries the same description: *44 logical px hit target. Name exposed to
assistive technology; tooltip on hover or keyboard focus.*

**They are line drawings, one weight, one colour.** The first letter this
repository draws today is a fallback the owner ruled for applications that have
**no** artwork; it is not what an application with artwork shows.

## What the compositor can draw today, measured

`crates/alo-shell/src/painted.rs` is the whole of it: `Solid`, an
`[u8; 3]` colour over an axis-aligned `Rectangle`, and `Inked`, a bitmap of
opaque pixels drawn as runs of those rectangles. `solid_run` calls
`Frame::draw_solid` with `Color32F::new(r, g, b, **1.0**)`.

**Every pixel this compositor draws is an opaque rectangle.** So:

| the design asks for | can it be drawn today | what it needs |
|---|---|---|
| the white surface | **yes** | — |
| the 1-pixel border | **yes** | four rectangles |
| the running indicators | **yes** | one rectangle each |
| the composer's field and its text | **yes** | — |
| the divider | **yes** | — |
| *Show all* and *···* as text | **yes** | — |
| the overflow list's surface | **yes** | — |
| **corner radius** | **no** | alpha, or an aliased stair-step nobody would accept |
| **the shadow** | **no** | alpha |
| **the backdrop blur** | **no** | reading back the framebuffer |
| **the 28 × 28 icons** | **not yet** | an SVG rasteriser — a new dependency, so an ADR |

The icons are the one in that bottom group with a cheap road: text is already
rasterised onto a known flat ground and blitted opaque, and an icon on the
Dock's own white is the same problem. What is missing is only something to turn
an SVG into pixels.

**Alpha is the common blocker for radius and shadow**, and it is one change in
one function rather than three features. That is the order to do them in.
