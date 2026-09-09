//! Paint prepared label scanlines without repeating shaping or input policy.

use crate::{RenderError, WindowControlLabel};
use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::Rectangle,
};

impl WindowControlLabel {
    /// Draw an opaque label after controls and before the pointer into a matching
    /// normal-transform, scale-one frame. Every scanline clips to the viewport.
    /// The host owns submission and must discard a partially painted failed frame.
    /// This paints no accessibility protocol and intercepts no input; hosts must
    /// keep the box away from client input or explicitly own that presentation.
    pub fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        let Some(visible) = self.bounds.intersection(self.viewport) else {
            return Ok(());
        };
        for y in visible.loc.y..visible.loc.y + visible.size.h {
            let mut x = visible.loc.x;
            while x < visible.loc.x + visible.size.w {
                let offset =
                    ((y - self.bounds.loc.y) * self.bounds.size.w + x - self.bounds.loc.x) as usize;
                let pixel = *self.pixels.get(offset).ok_or_else(|| {
                    RenderError::Submission("native label raster is incomplete".into())
                })?;
                let mut width = 1;
                while x + width < visible.loc.x + visible.size.w
                    && self.pixels.get(offset + width as usize) == Some(&pixel)
                {
                    width += 1;
                }
                let rect = Rectangle::new((x, y).into(), (width, 1).into());
                frame
                    .draw_solid(
                        rect,
                        &[Rectangle::from_size(rect.size)],
                        Color32F::new(
                            f32::from(pixel[0]) / 255.0,
                            f32::from(pixel[1]) / 255.0,
                            f32::from(pixel[2]) / 255.0,
                            1.0,
                        ),
                    )
                    .map_err(|error| {
                        RenderError::Submission(format!("paint native control label: {error}"))
                    })?;
                x += width;
            }
        }
        Ok(())
    }
}
