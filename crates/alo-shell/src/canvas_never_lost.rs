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

/// How much of a frame's name must stay reachable, in logical pixels.
///
/// **44 × 24, and the owner's reason is the whole of it: *one exposed pixel is
/// technically reachable and practically lost*.** The rule this replaces asked
/// whether a name was *entirely* under the dock, which is satisfied by a single
/// visible pixel — a frame nobody could actually pick up, passing a check written
/// to stop exactly that.
///
/// Forty-four is the width a finger or a hurried pointer can be relied on to hit;
/// twenty-four is a frame's name band at the height it is drawn. Settled by the
/// owner on 2026-09-30 with the status area's placement, and recorded in task 8 of
/// `docs/autonomy/the-smallest-canvas-worth-showing.md`.
pub const A_USABLE_HANDLE: (f64, f64) = (44.0, 24.0);

impl crate::Server {
    /// [`A_USABLE_HANDLE`], at the size this person's settings draw text.
    ///
    /// **A person who has made everything larger has made this larger too.** A
    /// fixed 44 × 24 would be a target that shrinks against everything around it
    /// exactly for the person who most needs it not to — which is the same fault as
    /// a resize band measured in plane units, one setting over.
    ///
    /// `alo_access::TurnedOn::larger_text` answers the percentage and
    /// `TextScale::ordinary` is a hundred, so the ordinary case multiplies by one
    /// and this costs nothing when nothing is turned on.
    #[must_use]
    pub fn a_usable_handle_at(scale: alo_appearance::TextScale) -> (f64, f64) {
        let factor = f64::from(scale.as_percent())
            / f64::from(alo_appearance::TextScale::ordinary().as_percent());
        (A_USABLE_HANDLE.0 * factor, A_USABLE_HANDLE.1 * factor)
    }

    /// Whether enough of this frame's name is reachable at `wanted`.
    ///
    /// **Outside every fixed control, not outside the dock.** `controls` is the set
    /// — the Dock, the status area, and the expanded minimized-window panel — taken
    /// as a slice rather than three arguments **because the promise is *outside
    /// every fixed control* and a fourth control added later must join it by
    /// construction.** Three named parameters would be a list somebody has to
    /// remember to extend, which is how *outside the three we thought of* happens.
    ///
    /// The arithmetic is [`the_longest_reachable_run`], which needs no compositor
    /// and is tested on its own: what a band and some rectangles come to is not a
    /// fact about Wayland, and the first version of this could only be exercised
    /// through a fixture whose frames are sixteen pixels wide.
    #[must_use]
    pub fn enough_of_the_name_is_reachable(
        &self,
        frame: &WlSurface,
        from: At,
        wanted: At,
        controls: &[Rectangle<i32, Physical>],
        handle: (f64, f64),
    ) -> bool {
        let Some(name) = self.the_band_of(frame) else {
            // No band drawn is not a frame this rule can protect, and refusing a
            // drag on that basis would stop a frame moving for a reason nobody
            // could see.
            return true;
        };
        let zoom = crate::scene::drawn_at(self.the_camera());
        let band = (
            name.loc.x + f64::from(wanted.x.saturating_sub(from.x)) * zoom,
            name.loc.y + f64::from(wanted.y.saturating_sub(from.y)) * zoom,
            name.size.w,
            name.size.h,
        );
        enough_of_it_is_reachable(band, controls, handle)
    }
}

/// Whether a band this wide keeps a usable run clear of every control.
///
/// **The handle a frame owes is capped by the band it has.** The first version
/// required a flat 44 logical pixels, and every frame in this repository's
/// fixtures is sixteen wide — so no frame could ever satisfy it and three tests
/// failed at once saying so. The rule exists to stop **controls covering a
/// handle**, not to refuse small windows: a frame drawn narrower than a handle is
/// small, not hidden, and its whole name is its handle.
///
/// So what must stay clear is `min(handle, band)`. A 300-pixel name owes 44; a
/// 16-pixel name owes all 16, which is the same sentence as *one exposed pixel is
/// technically reachable and practically lost* read at the small end.
#[must_use]
pub fn enough_of_it_is_reachable(
    band: (f64, f64, f64, f64),
    controls: &[Rectangle<i32, Physical>],
    handle: (f64, f64),
) -> bool {
    let (_, _, width, _) = band;
    if !band.0.is_finite() || !band.1.is_finite() || !width.is_finite() || width <= 0.0 {
        return true;
    }
    the_longest_reachable_run(band, controls) >= handle.0.min(width)
}

/// The longest single unobstructed run of this band, in logical pixels.
///
/// **The longest run, never the total left over.** Two controls leaving twenty
/// pixels each at opposite ends of a name leave forty pixels and no handle, and a
/// check that summed them would pass a frame nobody can grab.
///
/// A control counts only where it crosses the band's **full height**. One that
/// clips the top few pixels leaves a shorter but still grabbable strip, and calling
/// that hidden would refuse drags a person can plainly make.
#[must_use]
pub fn the_longest_reachable_run(
    band: (f64, f64, f64, f64),
    controls: &[Rectangle<i32, Physical>],
) -> f64 {
    let (left, top, width, height) = band;
    let mut taken: Vec<(f64, f64)> = controls
        .iter()
        .filter_map(|control| {
            let (cl, ct) = (f64::from(control.loc.x), f64::from(control.loc.y));
            let cr = cl + f64::from(control.size.w);
            let cb = ct + f64::from(control.size.h);
            (cb >= top + height && ct <= top).then(|| (cl.max(left), cr.min(left + width)))
        })
        .filter(|(start, end)| end > start)
        .collect();
    taken.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut longest: f64 = 0.0;
    let mut open = left;
    for (start, end) in taken {
        if start > open {
            longest = longest.max(start - open);
        }
        open = open.max(end);
    }
    longest.max(left + width - open)
}

#[cfg(test)]
#[path = "canvas_never_lost_handle_tests.rs"]
mod handle_tests;
