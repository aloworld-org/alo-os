//! Token-colored control glyphs and clipped GLES-compatible solid drawing.

use crate::WindowControlLayout;
use alo_appearance::{Colour, Scheme, Token};
use alo_shortcuts::Action;
use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle},
};

/// One clipped solid primitive. All are opaque; gaps leave client pixels intact.
type Solid = (Rectangle<i32, Physical>, Colour);

impl WindowControlLayout {
    /// Paint the immutable view into an already active scale-one frame, using
    /// the existing light/dark appearance tokens. Call after client drawing and
    /// before the pointer; the caller owns frame submission and failure recovery.
    ///
    /// Disabled glyphs have a diagonal strike as well as a different ground;
    /// availability is never communicated by color alone. This icon painter does
    /// not render label text: the host must present `action().said(...)` through
    /// its label/accessible surface. No client callbacks or input are consumed.
    /// The frame must have the viewport used at construction and normal transform.
    pub fn paint(&self, frame: &mut impl Frame, scheme: Scheme) -> Result<(), crate::RenderError> {
        for (rect, colour) in self.solids(scheme) {
            let color = Color32F::new(
                f32::from(colour.red()) / 255.0,
                f32::from(colour.green()) / 255.0,
                f32::from(colour.blue()) / 255.0,
                1.0,
            );
            frame
                .draw_solid(rect, &[Rectangle::from_size(rect.size)], color)
                .map_err(|error| {
                    crate::RenderError::Submission(format!("paint window controls: {error}"))
                })?;
        }
        Ok(())
    }

    /// Produce bounded geometry without touching a renderer; shared by paint tests.
    pub(crate) fn solids(&self, scheme: Scheme) -> Vec<Solid> {
        let (ground, ink, disabled) = match scheme {
            Scheme::Light => (Token::Cream, Token::Navy, Token::Porcelain),
            Scheme::Dark => (Token::Charcoal, Token::Cream, Token::Navy),
        };
        let mut solids = Vec::new();
        for control in self.controls() {
            let bounds = control.bounds();
            if let Some(clipped) = bounds.intersection(self.viewport) {
                solids.push((
                    clipped,
                    if control.enabled() { ground } else { disabled }.colour(),
                ));
            }
            for y in 0..12 {
                for x in 0..12 {
                    let glyph = match control.action() {
                        Action::MinimiseWindow => y >= 10,
                        Action::MaximiseWindow if self.restoring => {
                            outline(x, y, 3, 0, 11, 8) || outline(x, y, 0, 3, 8, 11)
                        }
                        Action::MaximiseWindow => outline(x, y, 0, 0, 11, 11),
                        Action::CloseWindow => (x - y).abs() <= 1 || (x + y - 11).abs() <= 1,
                        _ => false,
                    };
                    if glyph || (!control.enabled() && x + y == 11) {
                        let pixel = Rectangle::new(
                            (bounds.loc.x + 10 + x, bounds.loc.y + 10 + y).into(),
                            (1, 1).into(),
                        );
                        if let Some(clipped) = pixel.intersection(self.viewport) {
                            solids.push((clipped, ink.colour()));
                        }
                    }
                }
            }
            // A disabled close already contains a diagonal; a separate bottom
            // strike makes its disabled state distinct too, without hue alone.
            if !control.enabled() {
                let strike =
                    Rectangle::new((bounds.loc.x + 6, bounds.loc.y + 26).into(), (20, 2).into());
                if let Some(clipped) = strike.intersection(self.viewport) {
                    solids.push((clipped, ink.colour()));
                }
            }
        }
        solids
    }
}

/// One-pixel box border in the original twelve-pixel glyph grid.
fn outline(x: i32, y: i32, left: i32, top: i32, right: i32, bottom: i32) -> bool {
    (left..=right).contains(&x)
        && (top..=bottom).contains(&y)
        && (x == left || x == right || y == top || y == bottom)
}
