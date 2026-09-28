//! *Show all*: the frames' own extent, and the camera that holds it.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 6, and its
//! constraint is the whole of this file's design: **the extent is computed from
//! the frames**. There is no stored bound here to drift from what is open, which
//! is what `alo_canvas::plane::reached_by` exists for and why `alo-canvas` holds
//! no extent of its own.
//!
//! # What this file does and does not decide
//!
//! It turns the compositor's idea of a window — a surface, a buffer origin and a
//! committed geometry, all in `f64` logical units — into `alo_canvas::Frame`s,
//! and hands them to the canvas. **The fit itself is not here**:
//! `alo_canvas::Camera::showing` does that arithmetic, beside the transform it has
//! to agree with. A fit computed in the compositor would be a second answer to
//! where a frame lands, and the first landing of this seam is exactly the lesson
//! that two answers means one of them is wrong in a way no test notices.
//!
//! # The viewport is the output's, and an output is not guaranteed
//!
//! The room to fit into is `Surfaces::popups.output_size`, which is the same
//! field popup placement reads and is `None` before an output arrives or after one
//! is retired. *Show all* with nowhere to show refuses rather than inventing a
//! viewport, in the crate's usual way.

use alo_canvas::{At, Camera, Frame, Size};
use smithay::utils::{Logical, Point, Rectangle};

impl crate::Server {
    /// Every mapped frame, as the canvas sees it.
    ///
    /// A frame's rectangle on the plane is its buffer origin plus its committed
    /// geometry, which is the same pair `crate::frame_handle` measures a name band
    /// from — the one place the two must agree, and they agree by reading the same
    /// two functions rather than by each doing the sum.
    ///
    /// A surface whose rectangle will not fit the plane's integer units is left
    /// out rather than clamped: it is not a frame anybody can be shown, and
    /// including a wrong one would move the extent for every other.
    ///
    /// **The identity is `crate::window_number`'s, not the surface's protocol
    /// id.** That module's own header is about why: a protocol id is the number a
    /// surface's *client* knows it by and starts again at 1 for every client, so
    /// two applications with one window each are both `wl_surface@3` and would
    /// arrive here as one frame with two rectangles. It cost a real walk to find
    /// once already.
    #[must_use]
    pub fn the_frames_on_the_plane(&self) -> Vec<Frame> {
        self.mapped_surfaces()
            .filter_map(|surface| {
                let origin = crate::window_buffer_origin(surface);
                let geometry = crate::scene::geometry(surface);
                a_frame(crate::window_number::Numbers::of(surface), origin, geometry)
            })
            .collect()
    }

    /// Fit every frame on the screen at once, and look at it.
    ///
    /// # Errors
    /// [`None`] with no frames open, with no output to show them on, or where the
    /// frames are spread further apart than this canvas can zoom out to hold —
    /// each refused rather than shown as a canvas that moved somewhere arbitrary.
    pub fn show_all_on_the_canvas(&mut self) -> Option<Camera> {
        let room = self.surfaces.popups.output_size?;
        let viewport = Size::checked(u32::try_from(room.w).ok()?, u32::try_from(room.h).ok()?)?;
        let frames = self.the_frames_on_the_plane();
        let span = alo_canvas::plane::reached_by(&frames)?;
        let showing = Camera::showing(span, viewport)?;
        self.camera = showing;
        self.surfaces.popups.camera = showing;
        Some(showing)
    }
}

/// One surface's rectangle as a frame on the plane, or [`None`] where it is not
/// one the plane can hold.
///
/// Rounding is outwards — the origin down, the far corner up — so a frame is
/// never made smaller than it is drawn. *Show all* promises nothing is clipped,
/// and a frame rounded inwards by half a unit is a frame whose last column that
/// promise was never made about.
fn a_frame(
    id: u64,
    origin: Point<f64, Logical>,
    geometry: Rectangle<f64, Logical>,
) -> Option<Frame> {
    let left = origin.x + geometry.loc.x;
    let top = origin.y + geometry.loc.y;
    let (width, height) = (geometry.size.w, geometry.size.h);
    if ![left, top, width, height].iter().all(|it| it.is_finite()) {
        return None;
    }
    // A float past an integer's range saturates at its bound rather than
    // wrapping, and `At::checked` and `Size::checked` refuse anything off the
    // plane after that, so a surface with an absurd geometry is left out here
    // rather than moving the extent every other frame is fitted by.
    let at = At::checked(left.floor() as i32, top.floor() as i32)?;
    let size = Size::checked(width.ceil().max(0.0) as u32, height.ceil().max(0.0) as u32)?;
    Some(Frame::of(id, at, size))
}
