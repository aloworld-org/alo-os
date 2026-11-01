# ADR 0144 — An application's scalable icon is drawn by `resvg`

**Status:** **accepted, 2026-11-01**, under the approval
[ADR 0101](0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md)
gave: *select and integrate a suitable maintained Rust SVG rasteriser through
the normal dependency and ADR process, after checking whether the existing
graphics stack already provides it.* The choice of library is this lane's, and
the owner may reverse it.

## What it changes for the person

On a booted machine on 2026-11-01 (image of `c4e6cee0`) the Dock showed Dolphin
as the letter "o" while Firefox beside it had its own icon. Dolphin's Flatpak
exports one icon, `share/icons/hicolor/scalable/apps/org.kde.dolphin.svg`, and
nothing on the machine could draw an SVG. The desktop's own log said so:
*9 applications draw their own icon; … 5 have only a scalable one*. With this,
those applications show their own icons.

## The check ADR 0101 asked for first

[ADR 0142](0142-small-vector-artwork-is-drawn-by-the-rasteriser-the-text-already-uses.md)
made it for alo's own artwork and chose `alo-vector-shapes`, a strict reader of
a small part of SVG drawn with `zeno`, which the text stack already brings. It
named this day: *an SVG this crate refuses cannot be drawn until the crate is
widened, or `resvg` is brought in … the day the design exports a gradient or a
transform is the day to revisit this.*

An application's icon is not alo's artwork. It is whatever the application's
authors drew, and theme icons routinely use gradients, transforms, masks,
`<rect>`, `<circle>` and CSS styles — everything `alo-vector-shapes` refuses
by design. Widening that crate to all of static SVG would be writing an SVG
renderer, and its strictness is the point of it for the design's icons.

## The decision

**`alo-desktop` draws an application's SVG icon with `resvg` 0.45.1**, built
with `default-features = false`: no text, no system fonts, no embedded raster
images. It is given no resources directory, so an icon cannot make the desktop
open another file. A file over 4 MiB is not read. The picture is drawn at twice
the Dock's logical size, centred, and handed to the shell as straight RGBA like
a decoded PNG.

`alo-icons` now answers with the SVG's path (`WhatWasFound::AScalablePicture`)
instead of only saying one exists. A PNG at least as large as asked still
wins; an SVG wins over a PNG too small, because it is drawn at the size asked.

**`alo-shell` does not change.** The compositor takes no new dependency: the
desktop reads the machine and hands the shell pixels, as it already did.

## What was weighed

- **Widening `alo-vector-shapes`** — rejected above.
- **Asking the icon theme for a PNG of the same name** — the Flatpak exports
  none, and the system theme's icon of another name is not the application's.
- **`librsvg`** — C, with a system library at run time; this workspace's
  daemons link no system library (`crates/alo-image/tests/no_daemon_links_a_system_library.rs`).

## What it adds

`resvg`, `usvg`, `tiny-skia` and their pure-Rust dependencies, to
`alo-desktop` only. No build script runs C code and no system library is
linked. **Checked by building the workspace, not by building the image**: no
image was built for this change.

## How it is checked

On every build: `crates/alo-desktop/src/a_scalable_icon_drawn_tests.rs` draws a
gradient, a half-transparent fill and a non-square icon, and refuses rubbish
and an oversized file; `the_artwork_each_application_has_tests.rs` lays a tree
out as Dolphin's Flatpak export is and finds the icon drawn; `alo-icons`'s
tests hold the new preference order. **Not checked on a booted machine**: that
Dolphin's real icon draws there. The next walk is where.
