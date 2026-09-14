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
