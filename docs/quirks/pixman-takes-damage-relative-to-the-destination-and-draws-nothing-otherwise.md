# Pixman takes damage relative to the destination, and draws nothing otherwise

**What it is.** `smithay` 0.7.0's pixman backend —
`src/backend/renderer/pixman/mod.rs`, `Frame::render_texture_from_to`.

**What version.** smithay `=0.7.0`, pinned in `crates/alo-shell/Cargo.toml`.
Measured 2026-10-11.

## What actually happens

The `damage` argument is **relative to the destination rectangle's own
location**, not in the frame's coordinates. Around line 557:

```rust
let damage_boxes = damage.iter().copied().map(|mut rect| {
    rect.loc += dst_loc;        // <-- the destination's location, added
    ...
```

and the result is intersected with a clip region that is already bounded by the
destination. So damage given in the frame's coordinates lands at **twice the
offset**, the intersection is empty, and the call

- returns `Ok(())`,
- draws **absolutely nothing**,
- and reports no error of any kind.

The signature gives no hint: every other rectangle in that call —
`dst`, `opaque_regions` — is in the frame's coordinates. Only `damage` is not.

## How it was found, and why it took a morning

The Dock's surface — a rounded panel with a hairline and a shadow — became an
RGBA texture on 2026-10-11. The **GLES path worked first time**, which is what
made this expensive: the picture on screen was right, so the code looked right,
and only the software renderer was blank.

What caught it was `the_dock_reaches_the_bytes_a_card_is_handed`, which draws
through `SoftwarePainter` and asserts the frame handed to the card is not one
flat colour. Three wrong hypotheses came first — the raster hanging off the
screen's edge, the opaque-regions argument, an unsupported import format — and
each was cheap to test and wrong. The answer was only visible by reading the
backend's own source.

**An instrumented `eprintln!` is what turned it around**: it showed the texture
reaching the painter (`washed=1`) with a sane destination, which ruled out every
hypothesis about the caller at once and left only the call itself.

## What we do about it

`crates/alo-shell/src/painted.rs`'s `wash` passes
`&[Rectangle::from_size(area.size)]` — the destination's own extent, at the
origin — and says why in a comment at the call.

## The trap this sat in for a year

`crates/alo-shell/src/lock_texture.rs` has made this call since long before the
Dock did, passing the **frame's** coordinates, and has always been correct — **by
accident.** A lock frame's destination is the whole output at the origin, where
relative and absolute are the same rectangle. It was also typed concretely to
`GlesRenderer`, so it never reached pixman at all.

So the one existing caller could not demonstrate the bug, and a second caller
written by reading the first inherits it. **Measured 2026-10-11 by the desktop
lane across the whole of `crates/alo-shell/src`: `render_texture_from_to` had
exactly one caller, `lock_texture.rs:26`.** The Dock is the second, and the first
to draw a texture anywhere other than the origin.

## Where it will bite next, and who owns that

Two places, both known before they land rather than after:

- **`task/third-pc/the-desktop-draws-applications`** makes the software painter
  draw application windows — the first time client surfaces at arbitrary offsets
  go down the pixman path. They go through `WaylandSurfaceRenderElement` and
  smithay's own damage tracking rather than a raw `render_texture_from_to`, so
  they **may** be unaffected; that branch owes a window drawn at a non-zero
  offset on the pixman path, **checked for pixels rather than for `Ok`**. The
  desktop lane holds that.
- The same branch makes `lock_texture` generic over the renderer, which is when
  a lock frame first reaches pixman. Its destination is still the origin, so it
  stays correct — but it stops being correct *by accident* and starts being
  correct *because the destination is the origin*, and that sentence belongs in
  the file before somebody moves the destination.

**The failure shape is what makes this worth a page rather than a comment**: not
a wrong pixel or a partial draw, but a silent, total, successful nothing.
