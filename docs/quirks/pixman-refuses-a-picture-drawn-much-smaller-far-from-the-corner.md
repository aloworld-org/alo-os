# Pixman refuses a picture drawn much smaller, far from the screen's corner

- **Version:** smithay 0.7.0 (`renderer_pixman`), pixman crate 0.2.1, over the
  system's libpixman.
- **Whose:** smithay's (how it builds the transform) and pixman's (its 16.16
  fixed-point transform). alo OS configures both and patches neither.
- **Date:** 2026-11-12, after a boot walk where putting Firefox aside ended the
  desktop twice.

## What this means for a person

On a machine with no graphics card, a window drawn much smaller than itself
near the right-hand or bottom edge of the screen cannot be drawn by the
software renderer. **The window put aside is the case that meets it**: its
miniature is a whole window drawn some forty pixels wide in the panel at the
right-hand edge. Until 2026-11-12 the refusal ended the desktop and every
window the person had open went with it. Now that window's square shows its
application's mark on a blank card, as a private window's does
(`crates/alo-shell/src/a_picture_the_renderer_refuses.rs`).

## What happens

`PixmanFrame::render_texture_from_to` draws a scaled picture through a
`pixman::Transform` built as *translate by minus the destination's corner, then
scale by source ÷ destination*. Pixman's transform entries are 16.16 fixed
point and cannot reach 32 768; `pixman_transform_scale` fails when an entry
would, the crate's `Transform::scale` answers `None`, and smithay returns
`PixmanError::Unsupported` — **"The requested operation is not supported"**,
with no word of which picture or why.

So a picture is refused when **the destination's corner × (source ÷
destination) ≥ 32 768** on either axis. A 1 280-wide window drawn about 42 wide
at the panel's x near 1 224 is about 37 000, refused; Dolphin's 760 at the same
place is about 23 000, drawn; the same 1 280 near x = 35 is about 1 100, drawn.

## What was measured

**Runs on every test run**:
`a_picture_the_renderer_refuses_tests::pixman_refuses_a_wide_window_drawn_small_at_the_right_hand_edge`
lays the panel out on a 1 280 × 800 screen with the panel's own code and draws
each case with pixman. It prints the miniature's place as `MEASURED:`.

**What it does not show**: Firefox's own buffer size on the machine where the
fault was seen. That the walk's Firefox crossed the line is inferred from its
window being larger than the screen and Dolphin's 760 × 586 not crossing it; a
booted machine is what confirms it.

## Where else it can bite

Any picture drawn smaller than itself far from the corner: a window on the
canvas when the person has zoomed far out, or a surface an application scales
down through `wp_viewporter`. The painter (`crate::scene_drawing`) no longer
ends a frame on any client picture it is refused; it notes the refusal and
draws the rest.
