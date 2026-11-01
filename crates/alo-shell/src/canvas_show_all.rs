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
use smithay::utils::{Logical, Physical, Point, Rectangle};

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
                let place = crate::canvas_place::the_place_of(surface)?;
                a_frame(
                    crate::window_number::Numbers::of(surface),
                    place,
                    origin,
                    geometry,
                )
            })
            .collect()
    }

    /// Fit every frame on the screen at once, and look at it.
    ///
    /// **Into the room the fixed controls leave clear, a margin inside it** —
    /// `crate::canvas_the_room_show_all_fits_into`. Until 2026-11-01 the fit was
    /// given the whole output, so one window open was magnified to the screen's
    /// full size and ran under the Dock.
    ///
    /// Where the frames are spread too far for that room even at the furthest
    /// zoom out, they are fitted across the whole output instead, as they always
    /// were. That is the reach `crate::canvas_never_lost` holds a drag to, so a
    /// window a person was allowed to put somewhere is never one this refuses.
    ///
    /// # Errors
    /// [`None`] with no frames open, with no output to show them on, or where the
    /// frames are spread further apart than this canvas can zoom out to hold —
    /// each refused rather than shown as a canvas that moved somewhere arbitrary.
    pub fn show_all_on_the_canvas(&mut self) -> Option<Camera> {
        // **And alo's terminal**, which is a window on the canvas since ADR
        // 0145 and is not a client, so it is not among the surfaces above.
        let mut frames = self.the_frames_on_the_plane();
        frames.extend(self.the_terminal_as_a_frame());
        self.look_at_these_frames(&frames)
    }

    /// Fit these frames on the screen, into the room *Show all* fits into, and
    /// look at them — the arithmetic *Show all* and *fit the selected window*
    /// share.
    ///
    /// # Errors
    /// [`None`] with no frames on the Place being looked at, with no output, or
    /// where they are spread further than this canvas can zoom out to hold.
    pub(crate) fn look_at_these_frames(&mut self, frames: &[Frame]) -> Option<Camera> {
        let room = self.surfaces.popups.output_size?;
        let viewport = Size::checked(u32::try_from(room.w).ok()?, u32::try_from(room.h).ok()?)?;
        let span = alo_canvas::plane::reached_by(frames, self.the_place_now())?;
        let showing = self
            .the_room_show_all_fits_into()
            .and_then(|clear| {
                let inside = Size::checked(
                    u32::try_from(clear.size.w).ok()?,
                    u32::try_from(clear.size.h).ok()?,
                )?;
                // Fitted to the clear room as though it began at the screen's
                // corner, then moved by where it really begins.
                Camera::showing(span, inside)?.panned_by(-clear.loc.x, -clear.loc.y)
            })
            .or_else(|| Camera::showing(span, viewport))?;
        self.set_the_camera(showing);
        Some(showing)
    }

    /// The part of the output *Show all* fits into, in its pixels: clear of the
    /// fixed controls the last draw laid out, and a margin inside that.
    ///
    /// [`None`] with no output, or where the margin would leave nothing — a very
    /// small screen — in which case the caller fits the whole output.
    pub(crate) fn the_room_show_all_fits_into(&self) -> Option<Rectangle<i32, Physical>> {
        use crate::canvas_the_room_show_all_fits_into::{
            THE_MARGIN_SHOW_ALL_LEAVES, inside_the_margin, the_room_left_clear,
        };
        let size = self.surfaces.popups.output_size?;
        let output = Rectangle::new((0, 0).into(), size);
        let controls = self
            .the_controls_of_the_display_for(None)
            .map_or(&[][..], crate::canvas_fixed_controls::FixedControls::bounds);
        let clear = the_room_left_clear(output, controls);
        // The output and the scale it is measured at are one display's: both are
        // written together, by the draw of the display that drew last.
        let margin =
            crate::display_scale::DisplayScale::of_per_cent(self.the_scale_edges_are_measured_at)
                .px(THE_MARGIN_SHOW_ALL_LEAVES);
        inside_the_margin(clear, margin)
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
    place: alo_canvas::Place,
    origin: Point<f64, Logical>,
    geometry: Rectangle<f64, Logical>,
) -> Option<Frame> {
    let (at, size) = a_rectangle(origin, geometry)?;
    Some(Frame::of(id, place, at, size))
}

/// Where one surface's frame sits on the plane, from its origin and geometry.
///
/// **The one place this sum is done.** `the_frames_on_the_plane`'s own note says
/// the canvas and `crate::frame_handle` agree about a frame's rectangle *by
/// reading the same two functions rather than by each doing the sum* — and this
/// is that sentence made true of a third caller.
/// `crate::window_mode_plan` captures a frame's ordinary rectangle as it leaves
/// `Normal`, and a second copy of this arithmetic there would be a second answer
/// to where a window was.
///
/// **No camera, and that is the point of taking a buffer origin.** An origin is
/// already in plane units; `crate::scene::trees` applies the camera when it draws.
/// So a rectangle captured here is comparable with one captured at any other zoom,
/// which is what lets an ordinary geometry outlive the camera it was taken under.
pub(crate) fn a_rectangle(
    origin: Point<f64, Logical>,
    geometry: Rectangle<f64, Logical>,
) -> Option<(At, Size)> {
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
    Some((at, size))
}
