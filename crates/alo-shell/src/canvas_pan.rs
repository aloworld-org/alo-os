//! Whether a scroll belongs to the frame under the pointer or to the plane.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 5, and it states the
//! rule outright: **scroll over a frame scrolls that frame's content; scroll over
//! empty canvas pans.** One question, asked in one place, because the wrong answer
//! is not a missing feature — it is a person's spreadsheet jumping sideways when
//! they meant to read the next row, or the canvas refusing to move because a frame
//! they cannot see is under the arrow.
//!
//! # The question is already answered, by the hit test
//!
//! *Over a frame* is exactly *the pointer has a client focus*, which
//! `crate::pointer`'s hit test decides for every motion and which already carries
//! the plane's whole transform. So this adds no second opinion about where frames
//! are: a scroll that finds no focus is over the plane, and the same arithmetic
//! that decides what can be clicked decides what can be scrolled.
//!
//! # Nothing is accelerated here
//!
//! A scroll pans by **its own value in screen pixels, one for one**. A wheel notch
//! is ten of those by convention and a trackpad sends something nearer a pixel, so
//! the two feel different — and that is a device-feel question that wants a person
//! looking at a real trackpad, not a multiplier chosen by whoever wrote this file.
//! Named rather than invented; the plan's task 5 is the place it belongs.
//!
//! Screen pixels rather than plane units, which is `alo_canvas::Camera::panned_by`'s
//! own contract: what a person scrolled moves under their finger by that much
//! whatever the zoom, instead of a canvas that crawls when they are far out.

use smithay::input::pointer::AxisFrame;

impl crate::Server {
    /// Pan the plane by this scroll, or refuse.
    ///
    /// Returns whether anything moved. `false` where the scroll rounds to nothing
    /// yet — the remainder is kept, see below — and where the pan would leave the
    /// plane, which `alo-canvas` refuses by name rather than clamping.
    ///
    /// # A trackpad's tenths are kept rather than dropped
    ///
    /// The plane is measured in whole units and a scroll arrives as `f64`, so
    /// truncating each event would make a slow trackpad scroll a canvas that never
    /// moves at all. What is left over is held until the next event and spent then,
    /// which is also why this is the only place a scroll may be turned into a pan:
    /// two callers would be two remainders, and a canvas that lost a third of a
    /// person's gesture depending on which road it came down.
    pub(crate) fn pan_the_plane_by_scroll(&mut self, frame: &AxisFrame) -> bool {
        let Some(pointer) = self.surfaces.pointer.as_mut() else {
            return false;
        };
        let wanted = (
            pointer.unspent_scroll.0 + frame.axis.0,
            pointer.unspent_scroll.1 + frame.axis.1,
        );
        // `trunc` rather than `round`, so the remainder never changes sign: a
        // rounded-up pixel would be a pan the person has not made yet, and it
        // would be taken back on their next event.
        let (across, down) = (wanted.0.trunc(), wanted.1.trunc());
        pointer.unspent_scroll = (wanted.0 - across, wanted.1 - down);
        let Ok(across) = i32::try_from(across as i64) else {
            return false;
        };
        let Ok(down) = i32::try_from(down as i64) else {
            return false;
        };
        if across == 0 && down == 0 {
            return false;
        }
        // A refused pan keeps nothing: the person is at the edge of the plane, and
        // holding their scroll would spend it the moment they turned round.
        if self.pan_the_canvas(across, down).is_none() {
            if let Some(pointer) = self.surfaces.pointer.as_mut() {
                pointer.unspent_scroll = (0.0, 0.0);
            }
            return false;
        }
        true
    }
}
