//! Text-preserving reader gutters and complete frame composition.
use super::{WindowControlReaderInteraction, command};
use crate::{ReaderKeyCommand, ReaderPointerFeedback, RenderError};
use alo_appearance::{Colour, Scheme, Token};
use smithay::{
    backend::renderer::{Color32F, Frame},
    utils::{Physical, Rectangle},
};

/// One bounded opaque feedback primitive, always outside the wording raster.
type Solid = (Rectangle<i32, Physical>, Colour);

impl WindowControlReaderInteraction<'_> {
    /// Paint the complete page, chrome and opaque gutters in one matching frame.
    /// Enabled rows have an underline, hover a one-pixel outline, armed press a
    /// two-pixel outline; disabled rows have no command affordance. All marks lie
    /// outside text, use existing scheme tokens, and never obscure source marks.
    /// Inconsistent/disabled feedback refuses before drawing. Fresh semantic
    /// feedback must come from live pointer validation; this cannot verify age.
    /// The caller must discard the entire frame if any renderer operation fails.
    pub fn paint(
        &self,
        frame: &mut impl Frame,
        feedback: ReaderPointerFeedback,
    ) -> Result<(), RenderError> {
        let solids = self.solids(feedback)?;
        self.page.paint(frame)?;
        self.chrome.paint(frame)?;
        for (rect, colour) in solids {
            frame
                .draw_solid(
                    rect,
                    &[Rectangle::from_size(rect.size)],
                    Color32F::new(
                        f32::from(colour.red()) / 255.0,
                        f32::from(colour.green()) / 255.0,
                        f32::from(colour.blue()) / 255.0,
                        1.0,
                    ),
                )
                .map_err(|error| {
                    RenderError::Submission(format!("paint reader feedback: {error}"))
                })?;
        }
        Ok(())
    }

    /// Prevalidate semantic state and generate only bounded gutter rectangles.
    pub(super) fn solids(
        &self,
        feedback: ReaderPointerFeedback,
    ) -> Result<Vec<Solid>, RenderError> {
        let [previous, next] = self.chrome.available();
        let enabled = |command| match command {
            ReaderKeyCommand::Previous => previous,
            ReaderKeyCommand::Next => next,
            ReaderKeyCommand::Dismiss => true,
        };
        if feedback.hovered.is_some_and(|command| !enabled(command))
            || feedback.pressed.is_some_and(|command| !enabled(command))
            || (feedback.pressed.is_some() && feedback.pressed != feedback.hovered)
        {
            return Err(RenderError::ControlScene);
        }
        let (ground, ink) = match self.chrome.scheme {
            Scheme::Light => (Token::Cream.colour(), Token::Navy.colour()),
            Scheme::Dark => (Token::Charcoal.colour(), Token::Cream.colour()),
        };
        let mut solids = Vec::with_capacity(24);
        for (index, bounds) in self.rows.iter().enumerate().skip(1) {
            let Some(command) = command(index) else {
                continue;
            };
            // Clear the complete gutter even for idle/disabled rows.
            solids.extend(outline(*bounds, 2).map(|rect| (rect, ground)));
            let thickness = if feedback.pressed == Some(command) {
                2
            } else {
                1
            };
            if feedback.hovered == Some(command) {
                solids.extend(outline(*bounds, thickness).map(|rect| (rect, ink)));
            } else if enabled(command) {
                solids.push((
                    Rectangle::new(
                        (bounds.loc.x, bounds.loc.y + bounds.size.h - 1).into(),
                        (bounds.size.w, 1).into(),
                    ),
                    ink,
                ));
            }
        }
        Ok(solids)
    }
}

/// Four disjoint opaque strips strictly outside the original two-pixel inset row.
fn outline(bounds: Rectangle<i32, Physical>, width: i32) -> [Rectangle<i32, Physical>; 4] {
    let (x, y, w, h) = (bounds.loc.x, bounds.loc.y, bounds.size.w, bounds.size.h);
    [
        Rectangle::new((x, y).into(), (w, width).into()),
        Rectangle::new((x, y + h - width).into(), (w, width).into()),
        Rectangle::new((x, y + width).into(), (width, h - 2 * width).into()),
        Rectangle::new(
            (x + w - width, y + width).into(),
            (width, h - 2 * width).into(),
        ),
    ]
}
