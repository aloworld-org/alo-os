# The canvas against its specification

**`docs/design/the-canvas-as-a-workspace.md` arrived on 2026-10-08** and is the
owner's specification of what the canvas is. This is the other half: what this
machine measured on the same day, held against it.

**Everything below was run or read on 2026-10-08**, on the third PC, under
WSLg's Wayland parent, software rendered. Nothing here is inferred from a plan
or recalled from a report. Where a row says *not measured*, it was not measured.

**It is not a verdict on the work.** The specification's own second sentence is
that it describes intended behaviour and claims nothing about what exists. Most
of it was written after most of this code. The value of the table is that two
lanes can now stop guessing which parts disagree.

## What was measured, and how

| instrument | what it did |
|---|---|
| `desktop_check --save-to` | 32 pictures of the desktop, every standing in both schemes and both readings |
| `a_real_application --run foot` | a Wayland terminal nobody here wrote, on the real compositor, photographed |
| the canvas walk | `ALO_NESTED_SUBMODE=canvas-walk`, ten moments, exit 0 |
| reading | named files and line numbers, each with a positive control where a zero was the answer |

## Where what is built already disagrees

### §18 *Light canvas surfaces and clear navy text* — it is black

`crates/alo-shell/src/scene_drawing.rs:125` clears every frame to
`Color32F::new(0.0, 0.0, 0.0, 1.0)` under the comment *Neutral clear, not the
shell's pending token-based visual design.*

So the disagreement is **known to the code and labelled as temporary**, which is
different from a bug. `crates/alo-shell/src/desktop_raster.rs` holds no
`background` and no `wallpaper` — 348 lines, 5 functions, searched with a
positive control on the same file.

**And the surface it stands in place of is built.** This document said in an
earlier draft that ADR 0075's *material* did not exist, on a search for the
ADR's own noun. That was wrong, and the right answer was two files away:

- `crates/alo-appearance/src/shipped.rs:48` — `pub const THE_SURFACE: Token =
  Token::Cream;`
- `crates/alo-appearance/src/background.rs:31` — `pub enum Background`
- `crates/alo-appearance/src/appearance.rs:149` — `pub fn background_on(&self,
  display: &DisplayId) -> Background`

The code calls it `Background`; ADR 0075 calls it *material*; a search for
*material* therefore returns nothing and means nothing. **The surface ships
Cream and has since 2026-10-04**, and `shipped.rs` even records avoiding the
near-miss — `Porcelain` now carries `bg/surface`, which is white, and shipping
that *would have shipped a pure white desktop instead of the design's canvas*.

So §18's *light canvas surfaces* is not unbuilt. It is built, it is Cream, and
one hardcoded black stands in front of it.

### §5 *A shared window component should provide consistent title bands* — there is no band

Measured by photographing `foot` on the compositor. The three window controls
exist — minimise, maximise, close — and **they are drawn directly onto the
application's own pixels**, covering the first part of its first line of output.
There is no title band holding them, no title, and no border.

§8 asks that resizing *preserve a usable title band and controls*. There is
nothing yet for it to preserve.

### §11 *The Dock grows with its contents* — it is handed a literal zero

`crates/alo-shell/src/desktop_raster.rs:197`:

```rust
let dock_picture = crate::dock_raster::picture(
    dock, look, size,
    // Nothing in this crate decides what the Dock holds yet:
    // `alo_dock::Holding` answers that and is not plumbed into a
    // compositor. A bar holding nothing is narrow, which is true
    // rather than a placeholder.
    0,
)?;
```

`dock_raster::picture` takes `holding: usize` and sizes a bar by
`Room::a_bar_holding(holding)`; it draws no icon. `alo_dock::Holding` reaches
`crates/alo-shell/src/` in two comments and no line of code.

**The model is not missing.** `crates/alo-dock/src/holding.rs:48` declares
`OnTheDock { app, pinned, windows, put_aside }`, and `announcing.rs` reads an
icon out to a screen reader. What is missing is the wire between them.

So §11's *clicking an application that already has windows should help the
person return to those windows* has nothing to click.

## Where what is built already agrees

### §3 and §4 — panning and zooming work, measured

`ALO_NESTED_SUBMODE=canvas-walk` walked all ten moments against a real parent
and exited 0, each step read back off the frame it drew:

```
three applications open on the canvas                1366x768, 832 pixels painted
one of them dragged by its name                      1366x768, 832 pixels painted
another resized from its corner                      1366x768, 809 pixels painted
the canvas panned                                    1366x768, 832 pixels painted
zoomed out to show all of it                         1366x768, 2224 pixels painted
zoomed back into one and worked in it                1366x768, 832 pixels painted
panned by the arrow keys, with no pointer            1366x768, 576 pixels painted
show all again, from the keyboard                    1366x768, 2224 pixels painted
zoomed back in from the keyboard                     1366x768, 1088 pixels painted
a frame reached and focused with no pointer          1366x768, 1088 pixels painted
```

That is drag, resize, pan, **Show all**, zoom back in, and the same by keyboard
alone — §3's *keyboard routes for navigating the workspace, finding a window and
returning to it*, and §4's *Show all*. It is the strongest agreement in this
document.

**What it does not show is what any of it looks like.** 832 pixels of 1,049,088
are painted, because its three applications are test rectangles. The machinery
is measured; the appearance is not.

### §1 — an ordinary application does belong to the canvas

> Every application belongs to a Place, including ordinary applications that
> were not built specifically for alo OS.

`foot` was built by somebody else, connected to the compositor's own socket,
drew, and was composited — 83 surfaces over 15 seconds. The *Place* half is not
measured; that an outside application reaches the canvas at all now is.

## What is owed, in the order a measurement could be made

1. **Nothing here measures §17 persistence**, and the specification names the
   trap: *a test that reconstructs objects in memory is not sufficient evidence
   that restart persistence works.* No run on this machine has restarted
   anything.
2. **Nothing here measures §6, §13 or §14** — focus against selection, arranging
   several windows, or alo acting on a scope. They were not attempted, so this
   document says nothing about them.
3. **Nothing here is a panel.** Every picture is a nested surface under another
   compositor, software rendered. `desktop_check`'s own closing line still
   stands: *a virtual output proves the drawing path, not a panel.*
