//! The band above a frame that its name sits in, which is what moves it.
//!
//! ADR 0071: **the edge and the corners resize, and the name above the frame
//! moves it.** So this is the one place a person may take hold of a frame, and
//! `crate::window_move` is what happens next — the gesture, the bounds and the
//! release are already written and are not repeated here.
//!
//! # In screen pixels, and that is not an oversight
//!
//! The band is a fixed height on the glass rather than a fixed height on the
//! plane. A band measured in plane units would shrink with everything else as a
//! person zooms out — to under three pixels at the furthest zoom — and *zoomed
//! out* is exactly when somebody is moving frames around, so the handle would
//! vanish at the moment it is wanted. Its position comes from the plane, because
//! it belongs to a frame; only its thickness is the screen's.
//!
//! # Why the frames are asked in the order they are drawn
//!
//! Two frames can be close enough that one's band lies over another's content, so
//! *which frame did they take hold of* has one right answer: the front-most.
//! `crate::scene::trees` is already front-to-back for the drawing and the hit
//! test, and reading it in the same order here is what keeps the answer the same
//! as what a person sees.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle},
};

/// How thick the band is on the glass, in pixels.
///
/// **Forty-eight, because `docs/design/the-canvas-in-numbers.md` says so** — read
/// off the design file itself, Figma node `70:29`, rather than derived from
/// anything in this repository. `crates/alo-shell/tests/the_frame_in_numbers.rs`
/// holds this constant to that document, so the two cannot part again.
///
/// It was 32 for a day, reasoned out of a native window control's width, with a
/// paragraph here explaining why 32 was the sensible number. The document had
/// carried 48 since before that paragraph was written. A number a design file
/// already gives is not a number to derive, and nothing was checking, so being
/// wrong cost nothing at the time and would have cost a frame's whole feel later.
pub(crate) const THE_NAMES_BAND: f64 = 48.0;

/// How thick the band is, for the test that holds it to the design file.
///
/// The constant itself stays `pub(crate)`: what a frame's name is grabbed by is
/// this crate's business, and widening it so a test can read it would make it
/// somebody's API by accident.
#[must_use]
pub fn the_names_band() -> f64 {
    THE_NAMES_BAND
}

impl crate::Server {
    /// Where this frame's name sits on the glass, or [`None`] if it is not drawn.
    ///
    /// Directly above the frame's own top edge, as wide as the frame is on the
    /// screen. Geometry is the committed window geometry, so a client's own
    /// shadows are outside it and the band sits against the picture rather than
    /// against the buffer.
    pub(crate) fn the_band_of(&self, frame: &WlSurface) -> Option<Rectangle<f64, Logical>> {
        let zoom = crate::scene::drawn_at(self.camera);
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        let (_, buffer) = crate::scene::trees(&roots, &self.popup_surfaces(), self.camera)
            .into_iter()
            .find(|(surface, _)| surface == frame)?;
        let geometry = crate::scene::geometry(frame);
        let at = buffer + geometry.loc.upscale(zoom);
        let width = geometry.size.w * zoom;
        if !width.is_finite() || width <= 0.0 {
            return None;
        }
        Some(Rectangle::new(
            Point::from((at.x, at.y - THE_NAMES_BAND)),
            (width, THE_NAMES_BAND).into(),
        ))
    }

    /// The frame whose name is under this point on the glass, front-most first.
    ///
    /// [`None`] over a frame's content, over the plane, and over anything else: a
    /// name is the only part of a frame this answers about, which is what makes
    /// *inside the frame every click belongs to the application* true rather than
    /// hoped for.
    pub(crate) fn the_name_under(&self, at: Point<f64, Logical>) -> Option<WlSurface> {
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        crate::scene::trees(&roots, &self.popup_surfaces(), self.camera)
            .into_iter()
            .filter(|(surface, _)| roots.contains(surface))
            .find(|(surface, _)| {
                self.the_band_of(surface)
                    .is_some_and(|band| band.contains(at))
            })
            .map(|(surface, _)| surface)
    }
}
