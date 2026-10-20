# The canvas in numbers

**Read from the design file on 2026-09-27**, page *03 — alo OS · Living canvas*,
frame `70:29` of `nDxyF5Ho9oC4RObjVzwBNJ`. Values, not a description of a
picture, so that whoever builds the canvas builds what was drawn and whoever
rebuilds it later gets the same answer.

**This is a reading, not a source.** `docs/design/palette.toml` is the source
for the six colours the system is built out of, and a test holds the Rust
constants to it. Nothing yet holds *these* to anything, which is the gap named
at the end.

## The frame

| | |
|---|---|
| **Name band** | **48 px tall**, the full width of the frame, at its top |
| **Resize edge** | a **6 px** band down the right side and along the bottom |
| **Resize corner** | **24 × 24** at the bottom right |
| Corner radius | `radius/md` = **12** for a frame, `radius/sm` = **8** for controls inside it |
| Surface | `bg/surface` **#FFFFFF** |
| Border | `border/default` **#E7EBEF** |

The name band is what a frame is dragged by, and the 6 px edge and the corner
are what it is resized by — with the double-headed arrow over the second and
not the first ([the shortcuts and the edges](the-shortcuts-and-the-edges.md)).

## Colour, as the canvas uses it

| Token | Value | For |
|---|---|---|
| `text/primary` | **#102A43** | primary text — **navy, the approved colour** |
| `text/secondary` | #274C68 | secondary text |
| `accent/default` | **#0F6B72** | alo — deep teal, and never alone |
| `accent/soft` | #E8F4F2 | the ground a proposal sits on |
| `bg/surface` | #FFFFFF | a frame's own surface |
| `bg/subtle` | #F2F4F6 | a quieter surface inside one |
| `border/default` | #E7EBEF | a frame's edge |
| `navy/950` | #07131F | the darkest ink |
| `white` | #FFFFFF | |

## Spacing and radius

`space/2` = 8 · `space/3` = 12 · `space/4` = 16 · `space/6` = 24 ·
`space/8` = 32 · `radius/sm` = 8 · `radius/md` = 12

## Type

All **Manrope**, with **Geist Mono** for anything the machine says in code.

**And the faces are carried, not borrowed** — which is the fact a builder needs
and which was recorded only in a Rust doc comment until 2026-10-20.

| | |
|---|---|
| where they are | `crates/alo-shell/fonts/` — `Manrope.ttf`, `Manrope-SemiBold.ttf`, `GeistMono.ttf`, and `fonts/README.md` with the licence, the upstream revision and the hash |
| how they are loaded | `include_bytes!`, into the binary. **Never `/usr/share/fonts`**, so what a surface draws in does not depend on what a base image happens to ship |
| who asks for them | `alo_shell::painted_text::alos_own_text` — `Family::Name("Manrope")` with `Weight(600)`, in one place so a measurement and a drawing cannot disagree about the face they meant |
| **why `Inter.ttf` is still there** | as the fallback, deliberately. A script Manrope has no glyphs for falls through to the rest of the font database; narrowing that would mean a person's language stops rendering. **The shell does not draw in Inter** — it asks for Manrope by name and keeps Inter behind it |

**Settled 2026-10-09, and it had been wrong before that**: this crate bundled
only Inter and asked for `Family::SansSerif`, so nothing in the product drew the
typeface its own design uses. The weight matters as much as the family —
`Manrope-SemiBold.ttf` is bundled beside `Manrope.ttf` rather than left to
synthesis, because cosmic-text drops a family **silently** when no loaded face
matches the weight asked for, and a heading that renders Regular where the
design says SemiBold is a hierarchy quietly wrong rather than a visible fault.

### The boot environment draws in the same face, and it costs the initramfs nothing

`alo-installing` is the one program in the boot environment, and ADR 0128 gives
it screens of its own to draw — a network picker, a disk picker, and the review
before a disk is erased. Those are alo's own surfaces and take alo's own face.

Because the faces are **embedded rather than read from the file system**, the
environment needs no font added to `image/installing/alo-installing.conf` and
depends on nothing the base ships. `Manrope-SemiBold.ttf` is 145 KB inside a
binary that is already copied in whole.

**Two things the environment cannot take from this page**, and they are written
here so nobody assumes otherwise:

- **the mode is the firmware's.** `simpledrm` presents whatever UEFI left, which
  is not 1440×960 and is not ours to choose, and there is no compositor in the
  environment to normalise it. Every number on this page is a proportion or a
  logical measure for that surface, exactly as `CLAUDE.md` requires of every
  display;
- **nothing here has been drawn there yet.** The kernel carries the surface
  (`CONFIG_DRM_SIMPLEDRM=y`, built in) and the font is in the binary, but no
  machine has put a pixel on a screen from the environment. That is a
  measurement nobody has taken, not a thing this page describes.

| Role | Size / line | Weight | Tracking |
|---|---|---|---|
| Display/Medium | 36 / 44 | 600 | −0.8 |
| Heading/Large | 28 / 36 | 600 | −0.4 |
| Heading/Medium | 22 / 30 | 600 | −0.2 |
| Body/Large | 16 / 25 | 400 | 0 |
| Body/Medium | 14 / 22 | 400 | 0 |
| Label/Medium | 13 / 18 | 600 | +0.1 |
| Label/Small | 11 / 16 | 600 | +0.3 |
| System/Mono | 12 / 18 | 500 | 0 |

**Neither typeface is in the image.** Both are openly licensed and shippable,
and packaging them is part of the image work rather than something the
compositor can assume.

## Two things this reading found

**The canvas page and Foundations disagree about primary text.** The canvas uses
**#102A43**, which is navy and is what the owner's direction specifies. The
Foundations page uses **#07131F**. The canvas is the one that is right, and
[#140](the-interface-in-the-file.md) recorded the Foundations value
as a fault. This is the evidence for which way to fix it.

**Six of these colours are not in `palette.toml`.** `text/secondary`,
`accent/soft`, `bg/subtle`, `border/default`, `navy/950` and `#FFFFFF` exist in
the design and nowhere in the source the shell reads. `palette.toml` says
plainly that spacing, type and radii are deliberately not palette — but these
six *are* colours, and a colour the shell draws that no source names is exactly
the drift that file exists to prevent.

**What would close it:** these values in a source the Rust reads, and a test
holding the two together, as `a_palette_with_one_source.rs` already does for the
six. Until then this document is a reading somebody has to keep current by hand,
and it will go stale the way every hand-kept copy in this repository has.
