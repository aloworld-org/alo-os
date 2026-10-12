//! Native pixels this crate draws itself: flat shapes and inked text, and
//! painting them into a frame one run of equal pixels at a time.
//!
//! Shared by every native surface that is laid out in owned pixels rather than
//! imported from a client — the sign-in screen and the egress indicator — so
//! that the two cannot paint the same picture two different ways.

use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle},
};

use crate::RenderError;

/// One flat rectangle of one colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Solid {
    /// Where, in output pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// Red, green, blue.
    pub(crate) colour: [u8; 3],
}

/// Text inked onto its own ground, row-major and opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Inked {
    /// Where, in output pixels; always inside the output.
    pub(crate) area: Rectangle<i32, Physical>,
    /// `area.size.w * area.size.h` pixels.
    pub(crate) pixels: Vec<[u8; 3]>,
}

/// A raster with an alpha channel, composited over whatever is behind it.
///
/// **What [`Solid`] cannot be.** A `Solid` is one opaque colour over an
/// axis-aligned rectangle, which is all a flat band or a line of text needs and
/// is why that is what this file draws. A rounded corner and a shadow are
/// **partial coverage**, and no number of rectangles makes one: the pixel at a
/// corner is some of the panel and some of what is behind it.
///
/// So this carries straight `RGBA` — four bytes a pixel, row-major, the memory
/// order `Fourcc::Abgr8888` wants — and is uploaded as a texture by whoever
/// holds the renderer. It is **not** painted by [`paint`] below, because
/// importing a texture needs the `Renderer` and `paint` only has a `Frame`:
/// `crate::scene_drawing` imports these before it opens its frame, which is the
/// same order it already uses for every client surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Washed {
    /// Where, in output pixels.
    pub(crate) area: Rectangle<i32, Physical>,
    /// `area.size.w * area.size.h` pixels, four bytes each: red, green, blue,
    /// then alpha.
    pub(crate) pixels: Vec<u8>,
}

/// Draw the shapes, then the text, in order.
pub(crate) fn paint(
    frame: &mut impl Frame,
    solids: &[Solid],
    inked: &[Inked],
) -> Result<(), RenderError> {
    for solid in solids {
        solid_run(frame, solid.area, solid.colour)?;
    }
    for inked in inked {
        let width = inked.area.size.w;
        for (row, pixels) in inked.pixels.chunks(width.max(1) as usize).enumerate() {
            let y = inked.area.loc.y + row as i32;
            let mut start = 0;
            while let Some(first) = pixels.get(start) {
                let run = pixels.get(start..).map_or(1, |rest| {
                    rest.iter().take_while(|pixel| *pixel == first).count()
                });
                solid_run(
                    frame,
                    Rectangle::new(
                        (inked.area.loc.x + start as i32, y).into(),
                        (run as i32, 1).into(),
                    ),
                    *first,
                )?;
                start += run;
            }
        }
    }
    Ok(())
}

/// Draw one uploaded raster over whatever is behind it.
///
/// **The alpha is the texture's own**, per pixel. The `1.0` handed over is the
/// *opacity of the whole thing* — how much of the texture to draw at all — and
/// is not the per-pixel alpha, which is why a fully opaque surface and a shadow
/// can be in the same image.
///
/// `crate::lock_texture` has made this call since long before this function;
/// what is new is only that something other than a lock frame needs it.
pub(crate) fn wash<F: Frame>(
    frame: &mut F,
    texture: &F::TextureId,
    area: Rectangle<i32, Physical>,
) -> Result<(), RenderError> {
    if area.size.w <= 0 || area.size.h <= 0 {
        return Ok(());
    }
    frame
        .render_texture_from_to(
            texture,
            Rectangle::from_size((f64::from(area.size.w), f64::from(area.size.h)).into()),
            area,
            // **Relative to the destination, not to the frame.** Smithay's
            // pixman backend adds the destination's own location to every
            // damage rectangle before clipping — `rect.loc += dst_loc` — so a
            // rectangle given in the frame's coordinates lands at twice the
            // offset, falls outside the clip region, and **nothing is drawn**.
            //
            // `crate::lock_texture` passes the same thing and is correct by
            // accident: a lock frame's destination is the whole output at the
            // origin, where relative and absolute are the same rectangle. The
            // Dock is the first thing to draw a texture anywhere else, and this
            // cost a morning to find — `the_dock_reaches_the_bytes_a_card_is_handed`
            // caught it as *the frame handed to the card is one flat colour*.
            &[Rectangle::from_size(area.size)],
            // **No opaque region, because none of it is.** A shadow is partly
            // transparent at every pixel and the corners are too, so telling the
            // renderer any part is opaque would let it skip blending exactly
            // where the blending is the point.
            &[],
            smithay::utils::Transform::Normal,
            1.0,
        )
        .map_err(|error| RenderError::Submission(format!("wash a raster: {error}")))
}

/// One opaque rectangle.
fn solid_run(
    frame: &mut impl Frame,
    area: Rectangle<i32, Physical>,
    colour: [u8; 3],
) -> Result<(), RenderError> {
    if area.size.w <= 0 || area.size.h <= 0 {
        return Ok(());
    }
    let [red, green, blue] = colour;
    frame
        .draw_solid(
            area,
            &[Rectangle::from_size(area.size)],
            Color32F::new(
                f32::from(red) / 255.0,
                f32::from(green) / 255.0,
                f32::from(blue) / 255.0,
                1.0,
            ),
        )
        .map_err(|error| RenderError::Submission(format!("paint native pixels: {error}")))
}
