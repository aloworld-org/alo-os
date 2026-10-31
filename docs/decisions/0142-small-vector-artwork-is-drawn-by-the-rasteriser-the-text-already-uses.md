# ADR 0142 — Small vector artwork is drawn by the rasteriser the text already uses

**Status:** **accepted, 2026-10-30**, under the approval
[ADR 0101](0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md)
gave: *select and integrate a suitable maintained Rust SVG rasteriser through
the normal dependency and ADR process, after checking whether the existing
graphics stack already provides it.* The owner's decision of 2026-10-30 is the
direction — take the reply box's icons from the design file as SVG, keep them
in the repository, and add a small Rust library that draws vector shapes. **The
choice of library below is this lane's**, and the owner may reverse it.

## What it changes for the person

The task window's reply box drew three empty grey boxes and an empty navy
circle. With this it draws the design's Attach paperclip, the arrow beside the
model's name, the Permissions shield, and a Send arrow — or, while alo is
working, a solid Stop square — sharp at every display scale.

## The check ADR 0101 asked for first: the stack already has one

`cosmic-text`, which draws every letter the shell shows, draws glyph outlines
through `swash`, and `swash` rasterises them with **`zeno` 0.3**. `zeno` is a
pure-Rust path rasteriser built to SVG's path specification: SVG path data,
non-zero and even-odd fills, strokes with SVG's caps and joins, transforms, and
256-level anti-aliasing. It was already in `Cargo.lock` and already compiled
into `alo-compositor`.

## The decision

**A new crate, `alo-vector-shapes`, reads a strict part of SVG and draws it
with `zeno`.** It reads one `<svg>` with a `viewBox` from zero, `<g>` groups,
and `<path>` elements filled or stroked in one colour — all the design file
exports for these icons — and **refuses everything else by name**, so a file
that needs more is refused rather than drawn with part of it missing. It answers
coverage for a box of any size in pixels; the shell inks that in its palette's
colour, at the display's scale.

## What was weighed against it

**`resvg` with `tiny-skia`**, the obvious pure-Rust choice, draws all of static
SVG. Against it, for this use:

- **It adds about a dozen crates** to the compositor (`usvg`, `tiny-skia`,
  `tiny-skia-path`, `svgtypes`, `roxmltree`, `simplecss`, `kurbo`, `strict-num`,
  `arrayref`, `bytemuck`, `data-url`, …), and with its default features a second
  font stack and image decoders. `zeno` adds **none**: it is the same crate,
  at the same version, that `swash` already brings.
- **Its breadth is not wanted here.** Five icons of one or two paths each use
  none of what `resvg` adds. A crate that draws everything also draws a file the
  design did not intend without saying so, which is the opposite of what this
  repository asks of a check.

**What this costs**: an SVG this crate refuses cannot be drawn until the crate
is widened, or `resvg` is brought in. That is the right failure — loud, at the
test that reads every committed icon — and the day the design exports a
gradient or a transform is the day to revisit this.

## What it adds to the image

Nothing new to fetch or run. The image's compositor stage (`image/Containerfile`,
`FROM ${THE_BASE} AS shell`) builds `alo-shell` and `alo-desktop` with a minimal
Rust toolchain and the system's C libraries. `alo-vector-shapes` is pure Rust,
has no build script and no C code, and its one dependency is already compiled in
that stage for `cosmic-text`. **Checked by building the workspace, not by
building the image**: no image was built for this change.

## Where the artwork lives

`crates/alo-shell/icons/the-reply-box/`, five SVG files byte for byte as the
Figma connector's `download_assets` returned them on 2026-10-30, with the design
node each came from in `where-they-came-from.toml`.
`crates/alo-shell/src/the_reply_boxs_icons_tests.rs` reads every file, holds it
to that record and to the design's colours, on every build.
