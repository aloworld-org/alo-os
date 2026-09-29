//! **The edges and corners a frame is resized by**, and which one a pointer is on.
//!
//! Task 4 of `docs/autonomy/the-smallest-canvas-worth-showing.md`, and the shape
//! is `crate::frame_handle`'s: position from the plane, thickness on the glass. A
//! band that scaled with the zoom would be six plane units, which at 40 % is two
//! pixels and at 250 % is fifteen — a grab that gets harder the further out a
//! person looks, for no reason they could name.
//!
//! # The numbers are the design file's, not this file's
//!
//! `docs/design/the-canvas-in-numbers.md`: *a **6 px** band down the right side
//! and along the bottom*, and *a **24 × 24** corner*. They are held to that
//! document by `crates/alo-shell/tests/the_frame_in_numbers.rs`, which reads the
//! row rather than a line number — the same guard the name band has, and for the
//! same reason: the band was invented as 32 once when the document already said
//! 48.
//!
//! # Eight, where the design file's reading shows two
//!
//! That reading says *down the right side and along the bottom*, because that is
//! what page 07 draws. This task's acceptance says **each edge and corner
//! resizes**, and ADR 0071 says *four shapes cover eight directions*. So all eight
//! exist here and the discrepancy is named rather than settled quietly: the
//! document describes a picture, the plan states a promise, and a frame a person
//! can only resize rightwards is not what either of them wants.
//!
//! # A corner wins over an edge
//!
//! They overlap by construction — a 24-pixel corner covers the last 24 of two
//! 6-pixel edges — and the corner is the more specific gesture, so it is answered
//! first. A person aiming at a corner and getting one edge of it would be a
//! compositor that made them aim more carefully than they should have to.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point, Rectangle},
};

/// How thick a resize edge is, on the glass.
///
/// `docs/design/the-canvas-in-numbers.md`'s *6 px band*.
pub(crate) const THE_EDGE: f64 = 6.0;

/// How big a resize corner is, on the glass.
///
/// That document's *24 × 24*.
pub(crate) const THE_CORNER: f64 = 24.0;

/// The thickness of a resize edge, for whoever draws or tests against it.
#[must_use]
pub fn the_resize_edge() -> f64 {
    THE_EDGE
}

/// The size of a resize corner.
#[must_use]
pub fn the_resize_corner() -> f64 {
    THE_CORNER
}

/// Which part of a frame's border a pointer is on.
///
/// Named for the direction the pointer drags, which is what the cursor shows and
/// what the resize means — not for which coordinates change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameEdge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
    /// The top-left corner.
    TopLeft,
    /// The top-right corner.
    TopRight,
    /// The bottom-left corner.
    BottomLeft,
    /// The bottom-right corner.
    BottomRight,
}

impl FrameEdge {
    /// Every edge and corner, corners first.
    ///
    /// **Corners first is the order this is searched in**, and it is why a corner
    /// wins where the two overlap.
    pub const ALL: [Self; 8] = [
        Self::TopLeft,
        Self::TopRight,
        Self::BottomLeft,
        Self::BottomRight,
        Self::Top,
        Self::Bottom,
        Self::Left,
        Self::Right,
    ];

    /// Whether this is a corner rather than an edge.
    #[must_use]
    pub const fn is_a_corner(self) -> bool {
        matches!(
            self,
            Self::TopLeft | Self::TopRight | Self::BottomLeft | Self::BottomRight
        )
    }
}

impl crate::Server {
    /// Where this edge or corner of a frame is, on the glass.
    ///
    /// [`None`] for a surface that is not a mapped frame, or one drawn at no
    /// width — a frame with nothing to grab is not one to offer a grab on.
    pub(crate) fn the_edge_band_of(
        &self,
        frame: &WlSurface,
        edge: FrameEdge,
    ) -> Option<Rectangle<f64, Logical>> {
        let zoom = crate::scene::drawn_at(self.the_camera());
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        let (_, buffer) = crate::scene::trees(&roots, &self.popup_surfaces(), self.the_camera())
            .into_iter()
            .find(|(surface, _)| surface == frame)?;
        let geometry = crate::scene::geometry(frame);
        let at = buffer + geometry.loc.upscale(zoom);
        let (width, height) = (geometry.size.w * zoom, geometry.size.h * zoom);
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        // Bands sit **outside** the frame, as the name band does: inside the frame
        // every click belongs to the application (ADR 0065), so a resize band that
        // overlapped the content would be taking presses that are not the shell's.
        let (left, top, right, bottom) = (at.x, at.y, at.x + width, at.y + height);
        let rect = |x: f64, y: f64, w: f64, h: f64| Rectangle::new((x, y).into(), (w, h).into());
        Some(match edge {
            FrameEdge::Top => rect(left, top - THE_EDGE, width, THE_EDGE),
            FrameEdge::Bottom => rect(left, bottom, width, THE_EDGE),
            FrameEdge::Left => rect(left - THE_EDGE, top, THE_EDGE, height),
            FrameEdge::Right => rect(right, top, THE_EDGE, height),
            FrameEdge::TopLeft => rect(left - THE_CORNER, top - THE_CORNER, THE_CORNER, THE_CORNER),
            FrameEdge::TopRight => rect(right, top - THE_CORNER, THE_CORNER, THE_CORNER),
            FrameEdge::BottomLeft => rect(left - THE_CORNER, bottom, THE_CORNER, THE_CORNER),
            FrameEdge::BottomRight => rect(right, bottom, THE_CORNER, THE_CORNER),
        })
    }

    /// Which frame's edge or corner is under this point, front-most first.
    ///
    /// [`None`] over a frame's content, over its name, and over the plane — this
    /// answers about the border and nothing else, which is what keeps *inside the
    /// frame every click belongs to the application* true rather than hoped for.
    #[must_use]
    pub fn the_edge_under(&self, at: Point<f64, Logical>) -> Option<(WlSurface, FrameEdge)> {
        let roots: Vec<_> = self.mapped_surfaces().cloned().collect();
        for frame in roots.iter().rev() {
            for edge in FrameEdge::ALL {
                if self
                    .the_edge_band_of(frame, edge)
                    .is_some_and(|band| band.contains(at))
                {
                    return Some((frame.clone(), edge));
                }
            }
        }
        None
    }
}
