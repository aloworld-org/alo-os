//! One window edge, laid out in owned pixels and ready for a frame.
//!
//! `crate::window_edge` says where everything is, `crate::window_edge_paint`
//! says what colour, and `crate::window_edge_title` says what text. This is
//! the three of them turned into the pixels a frame takes — the same shape
//! every other native surface in this crate has, so the edge paints the way
//! the sign-in screen and the egress indicator already do rather than
//! inventing a second road into a frame.
//!
//! # Logical units become output pixels exactly once
//!
//! `window_edge` works in logical units, which is what the design is drawn in
//! and what a person's scale setting multiplies. `crate::painted` works in
//! output pixels, which is what a frame takes. **The conversion happens here
//! and nowhere else**, which is the contract's *apply display scaling once*.
//!
//! # What it refuses
//!
//! A picture whose region falls outside the output is not drawn at all. The
//! edge sits above its window, so a window at the very top of a display has an
//! edge partly or wholly off-screen — and a half-drawn edge is worse than
//! none, because a person would reach for a control that is not where it
//! looks.

use smithay::{
    backend::renderer::Frame,
    utils::{Logical, Physical, Rectangle},
};

use crate::painted::{self, Inked, Solid};
use crate::window_edge::Edge;
use crate::window_edge_paint::Pointing;
use crate::window_edge_title::Fitted;
use crate::{RenderError, painted_text};

/// A window's edge, in output pixels.
pub struct EdgePicture {
    /// The flat shapes: the strip, the grip, any highlight and focus outline.
    solids: Vec<Solid>,
    /// The title, where there is one and it fits.
    inked: Vec<Inked>,
    /// The whole title, for whoever says it aloud, whatever was drawn.
    whole: String,
}

impl EdgePicture {
    /// Lay this edge out for an output of `output` pixels, at `scale`
    /// hundredths.
    ///
    /// `title` is the application's own string and is never translated here.
    /// [`None`] for an edge that carries no title — an application that draws
    /// its own header, or an edge at rest.
    ///
    /// Returns [`None`] when the edge would not be wholly on the output.
    #[must_use]
    pub fn of(
        edge: &Edge,
        pointing: Pointing,
        title: Option<&str>,
        fonts: &mut cosmic_text::FontSystem,
        output: (i32, i32),
        scale: u32,
    ) -> Option<Self> {
        let whole_output = Rectangle::from_size((output.0, output.1).into());
        let region = into_pixels(edge.region, scale);
        if whole_output.intersection(region) != Some(region) {
            return None;
        }

        let solids = crate::window_edge_paint::solids(edge, pointing)
            .into_iter()
            .map(|(area, colour)| Solid {
                area: into_pixels(area, scale),
                colour: [colour.red(), colour.green(), colour.blue()],
            })
            .collect();

        // The title, fitted to what the drag region leaves after the grip and
        // then inked. `fitted` is what decides how much of it is drawn; this
        // only places what came back.
        let mut inked = Vec::new();
        let mut whole = String::new();
        if let (Some(title), Some(at)) = (title, edge.title) {
            let room = edge.drag.size.w - crate::window_edge::THE_GRIP_IS.0;
            let metrics = cosmic_text::Metrics::new(THE_LABEL_IS, THE_LINE_IS);
            let Fitted {
                shown, whole: all, ..
            } = crate::window_edge_title::fitted(fonts, title, room, metrics);
            whole = all;
            if !shown.is_empty() {
                let ground = alo_appearance::Role::BgSurface.colour();
                let ink = alo_appearance::Role::TextPrimary.colour();
                let shaped = painted_text::sentence(
                    fonts,
                    &shown,
                    scaled(room, scale),
                    metrics,
                    [ground.red(), ground.green(), ground.blue()],
                    [ink.red(), ink.green(), ink.blue()],
                );
                let placed =
                    into_pixels(Rectangle::new(at, (room, THE_LINE_IS as i32).into()), scale);
                inked.extend(shaped.placed(placed.loc.x, placed.loc.y, output.1));
            }
        }
        Some(Self {
            solids,
            inked,
            whole,
        })
    }

    /// The whole title, however little of it was drawn.
    ///
    /// **What a reader is given.** The contract: *preserve the full title for
    /// accessibility.* Anything announcing the drawn string would make a
    /// long-titled window unfindable by name.
    #[must_use]
    pub fn the_whole_title(&self) -> &str {
        &self.whole
    }

    /// Draw it: the shapes, then the text, in that order.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        painted::paint(frame, &self.solids, &self.inked)
    }
}

/// The design's `Label/Medium` size.
const THE_LABEL_IS: f32 = 13.0;

/// Its line height.
const THE_LINE_IS: f32 = 18.0;

/// One logical length in output pixels, at `scale` hundredths.
fn scaled(length: i32, scale: u32) -> i32 {
    let scale = i64::from(scale.max(1));
    i32::try_from(i64::from(length) * scale / 100).unwrap_or(i32::MAX)
}

/// One logical rectangle in output pixels.
///
/// **The only place the conversion happens**, so drawing and hit testing
/// cannot round differently — the contract asks for one authoritative
/// calculation and this is where its units change.
fn into_pixels(of: Rectangle<i32, Logical>, scale: u32) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (scaled(of.loc.x, scale), scaled(of.loc.y, scale)).into(),
        (scaled(of.size.w, scale), scaled(of.size.h, scale)).into(),
    )
}

#[cfg(test)]
#[path = "window_edge_picture_tests.rs"]
mod tests;
