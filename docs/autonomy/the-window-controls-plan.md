# The window controls — the plan

The owner's specification of 2026-11-09, recorded with every measurement in
[`docs/design/the-window-controls.md`](../design/the-window-controls.md). Four
stages, each landed as its own change that works on its own; **a choice that
does not work yet is not drawn**, so each stage adds to the window panel only
what it makes work.

## Tasks

### 1. The band, its three controls and the window panel's placement

The title and the application's name always on a 54-tall band; Window
actions, Minimise and Close revealed as the pointer approaches, with the fade
and no wait; *Always show window controls* in Settings; the panel with
Maximise / Restore, Left and Right with their preview, Full screen / Exit full
screen; right-click and double-click on the band; the keyboard and a screen
reader into the band and the panel; Minimise travelling to the side panel; the
full-screen top-edge reveal. Landed in parts; the design note's *What is built*
says which.

### 2. Move to… and Undo placement

Move a window to another Place, or another display when more than one is
connected, on the one canvas. Undo a committed placement for **that window**,
from a history of its own, never the Place's series.

### 3. Help with this window

Open an alo request with the window identified, within the person's existing
permissions; hidden while AI is off; the teal working band with a plain
description and Stop.

### 4. Capture & export

The visible window, a selected area, or the full document where the
application offers one; PNG, JPEG or a clearly labelled image PDF; copy, or
save to the folder shown with the normal collision handling; Open and Show in
folder.

## What stage 1 has built, as it lands

- **The full-screen top-edge reveal.** A full-screen window's band stays above
  the display, where nothing draws or presses it; the pointer in the top 12
  units across the window, or the keyboard on the band or in its panel
  (`Alt+F3`), hangs it over the window's own top, and it stays while the
  pointer is anywhere in it or the keyboard holds it, then goes with no wait.
  The window is never moved or resized for it
  (`crates/alo-shell/src/the_band_over_full_screen.rs`).
