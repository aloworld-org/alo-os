//! A restore travels; it does not relocate.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 4. **Restoring a put-aside
//! window returns it to the Place it was already on, and the view travels
//! there.** A window put aside on Tuesday's Place comes back on Tuesday's Place,
//! and the person is taken to it.
//!
//! # Why this is its own task, and its own file
//!
//! *Cross-Place restoration* and *a frame dragged into another Place* read as one
//! sentence at a glance and are two acts: **one changes where a window lives, the
//! other changes where the person is looking.** The plan says the two were
//! conflated while it was being written, which makes the conflation the easy
//! mistake rather than an unlikely one.
//!
//! A version that fails this does the more destructive of the two things: it
//! moves somebody's window to the Place they happened to be looking at. **So the
//! constraint is stated as a prohibition rather than a goal — restoring never
//! writes a window's Place.** Only task 3 does, and there is no call to
//! `crate::canvas_place::put_on` anywhere in this file.
//!
//! # What travelling is
//!
//! Two things and no more: the Place the person is on becomes the window's, and
//! the camera looks at the window. The window is not moved, not resized, not
//! raised and not activated — `crate::window_minimize`'s own note already says
//! revealing neither activates nor raises, and travelling does not change that.

use alo_canvas::Place;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

impl crate::Server {
    /// Take the person to this window, wherever it is, without moving it.
    ///
    /// Answers whether the view moved: `false` where the window is already on the
    /// Place being looked at, which is the ordinary case and must cost nothing.
    ///
    /// **The Place is read and never written.** The only write in this function is
    /// to what the person is looking at.
    pub fn the_view_travels_to(&mut self, window: &WlSurface) -> bool {
        let Some(its_place) = self.the_place_of_the_window(window) else {
            return false;
        };
        let somewhere_else = its_place != self.the_place_now();
        if somewhere_else {
            self.surfaces.place = its_place;
        }
        // And look at it, on whichever Place it turned out to be on. Done after
        // the Place so that `the_frames_on_the_plane` is already answering about
        // the surface the window is on rather than the one being left.
        let looked = self.look_at_this_window(window);
        somewhere_else || looked
    }

    /// Put the camera where this window is, if the canvas can say where that is.
    fn look_at_this_window(&mut self, window: &WlSurface) -> bool {
        let Some(frame) = self
            .the_frames_on_the_plane()
            .into_iter()
            .find(|frame| Some(frame.id()) == crate::window_number::Numbers::given_to(window))
        else {
            return false;
        };
        self.look_at_the_canvas(frame.at()).is_some()
    }

    /// Which Place this window is on, for a caller that wants to check rather than
    /// travel.
    ///
    /// **Here so that a test can assert the constraint directly** — *its Place is
    /// the same before and after, asserted directly rather than inferred from
    /// where it appears* — which is the plan's own wording and the reason this is
    /// not left to be read off a camera.
    #[must_use]
    pub fn the_place_a_window_lives_on(&self, window: &WlSurface) -> Option<Place> {
        self.the_place_of_the_window(window)
    }
}
