//! Paint a laid-out sign-in screen into a frame, and nothing else with it.
//!
//! The picture is the whole output: there is no client beneath it and no
//! control strip beside it, because nobody is signed in to own a window.

use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle, Size},
};

use crate::RenderError;
use crate::sign_in_raster::SignInPicture;

impl SignInPicture {
    /// Refuse a frame the picture was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::SignInScene)
        }
    }

    /// Draw the shapes, then the text, one run of equal pixels at a time.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        for solid in &self.solids {
            solid_run(frame, solid.area, solid.colour)?;
        }
        for inked in &self.inked {
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
        .map_err(|error| RenderError::Submission(format!("paint sign-in screen: {error}")))
}
