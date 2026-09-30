//! **A frame is never lost** — task 8 of the canvas plan, and the two ways it
//! could be.
//!
//! *Nothing may be placed, dragged or restored where a person cannot get it back:
//! not off the plane's reachable area, not behind a viewport control, not at a
//! zoom where it cannot be seen.*
//!
//! # What *lost* turns out to mean, once panning exists
//!
//! Most ways a frame can be hard to see are not ways it can be lost. A frame
//! above the viewport, behind another, or off to one side is reached by panning,
//! which moves the plane under a viewport that does not move. So the question is
//! not *can I see it* but **can I get it back**, and only two answers are no.
//!
//! **It is further out than *Show all* can reach.** This is the one the numbers
//! make real. `Camera::showing` fits the frames' own extent, and the widest extent
//! it can fit is bounded by `Zoom::FURTHEST_OUT`: on a 1280×720 output that is
//! 25,600 × 14,400 plane units. The plane reaches ±1,000,000 — **78 times further
//! across than the fit can hold** — so a frame dragged far enough is one *Show
//! all* refuses to bring back, and that refusal is silent to whoever dragged it.
//! Task 7's list still reaches it by keyboard, which is why it is *hard to
//! recover* rather than gone; but this task's acceptance says *found by Show all*,
//! and that is the promise kept here.
//!
//! **It is entirely under the dock.** The dock and its status area are drawn in
//! the viewport layer from the output's own size, so they do not move when the
//! plane does. Panning still slides a *frame* out from under them — but only if
//! there is something to grab, and ADR 0071 makes the name the only handle, since
//! inside the frame every click belongs to the application. So the rule is about
//! the name band, and **entirely** under is the condition: a band half out is a
//! band a pointer can still land on.
//!
//! # It stops a drag rather than undoing one
//!
//! Neither rule moves a frame somebody is holding. A drag that would cross either
//! line keeps the last position that did not, which is what an edge feels like. A
//! compositor that accepted the drag and then took the window back afterwards
//! would be one whose windows fight the person holding them.

use alo_canvas::{At, Frame, Size, Zoom};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Physical, Rectangle},
};

impl crate::Server {
    /// The widest extent *Show all* could fit on this output, in plane units.
    ///
    /// `viewport * 1000 / FURTHEST_OUT`, which is `Camera::showing`'s own
    /// arithmetic read backwards: the fit takes the largest zoom whose span fits
    /// and refuses below the furthest out this canvas goes.
    ///
    /// [`None`] with no output, which is a machine with nothing to fit into.
    #[must_use]
    pub fn as_far_as_show_all_reaches(&self) -> Option<Size> {
        let room = self.surfaces.popups.output_size?;
        let span =
            |side: i32| u32::try_from(i64::from(side) * 1000 / i64::from(Zoom::FURTHEST_OUT)).ok();
        Size::checked(span(room.w)?, span(room.h)?)
    }

    /// Whether *Show all* would still gather every frame with this one at `wanted`.
    ///
    /// Asked of the whole arrangement rather than of one frame, because what the
    /// fit holds is the **span** of all of them: moving one frame out ruins the
    /// fit for every other frame as much as for itself.
    #[must_use]
    pub fn show_all_would_still_reach(&self, frame: &WlSurface, wanted: At) -> bool {
        let Some(reach) = self.as_far_as_show_all_reaches() else {
            // Nothing to fit into, so no promise to keep yet.
            return true;
        };
        let moved = crate::window_number::Numbers::given_to(frame);
        let frames: Vec<Frame> = self
            .the_frames_on_the_plane()
            .into_iter()
            .map(|it| {
                if Some(it.id()) == moved {
                    Frame::of(it.id(), it.place(), wanted, it.size())
                } else {
                    it
                }
            })
            .collect();
        let Some(span) = alo_canvas::plane::reached_by(&frames, self.the_place_now()) else {
            return true;
        };
        let across = i64::from(span.to.x) - i64::from(span.from.x);
        let down = i64::from(span.to.y) - i64::from(span.from.y);
        across <= i64::from(reach.width()) && down <= i64::from(reach.height())
    }

    /// Whether this frame's name would be **entirely** under the dock at `wanted`.
    ///
    /// `band` is the dock's own band in screen pixels — the rectangle the desktop
    /// draws — rather than a second computation of the dock's thickness here.
    #[must_use]
    pub fn the_name_would_be_under(
        &self,
        frame: &WlSurface,
        from: At,
        wanted: At,
        band: Rectangle<i32, Physical>,
    ) -> bool {
        let Some(name) = self.the_band_of(frame) else {
            return false;
        };
        let zoom = crate::scene::drawn_at(self.the_camera());
        let left = name.loc.x + f64::from(wanted.x.saturating_sub(from.x)) * zoom;
        let top = name.loc.y + f64::from(wanted.y.saturating_sub(from.y)) * zoom;
        let right = left + name.size.w;
        let bottom = top + name.size.h;
        if ![left, top, right, bottom].iter().all(|it| it.is_finite()) || name.size.w <= 0.0 {
            return false;
        }
        left >= f64::from(band.loc.x)
            && top >= f64::from(band.loc.y)
            && right <= f64::from(band.loc.x.saturating_add(band.size.w))
            && bottom <= f64::from(band.loc.y.saturating_add(band.size.h))
    }

    /// As far as this frame may be dragged towards `wanted`, under both rules.
    ///
    /// `wanted` where it may go there, and `from` where it may not — so the drag
    /// stops rather than the frame being taken back after it is let go.
    #[must_use]
    pub fn as_far_as_a_frame_may_be_dragged(
        &self,
        frame: &WlSurface,
        from: At,
        wanted: At,
        dock: Rectangle<i32, Physical>,
    ) -> At {
        if wanted == from {
            return from;
        }
        if !self.show_all_would_still_reach(frame, wanted)
            || self.the_name_would_be_under(frame, from, wanted, dock)
        {
            return from;
        }
        wanted
    }
}
