//! Stepping out of a Place into the World, and back into one.
//!
//! `docs/autonomy/the-canvas-and-its-places.md` task 2's **gesture**. #364
//! landed the arithmetic — `alo_canvas::World`, Places as tiles, the span they
//! reach, the Place under a point — and said in its own status that **nothing in
//! `alo-shell` answered a step out from `Zoom::FURTHEST_OUT` with it.** This is
//! that half.
//!
//! # No new binding, which is the task's constraint
//!
//! *If the World needs a switcher, this task has failed rather than found a
//! requirement.* So there is no new chord and no new gesture here: the keys and
//! the wheel that already zoom out are the ones that reach the World, at the rung
//! where `one_step_out` used to answer [`None`] and a person's press did nothing.
//! **The step that had nowhere to go now has somewhere.**
//!
//! # Where the Places come from
//!
//! From the windows. A Place exists, as far as the World is concerned, when
//! something is on it — so the World is built from `the_frames_on_the_plane`
//! rather than from a list somebody maintains, and a Place nobody has put
//! anything on is not drawn as an empty tile. That is the same decision
//! `plane::reached_by` makes about the extent: **computed from what is open, not
//! declared.**
//!
//! Laid out in a row in Place order, each tile the size of the viewport, a
//! viewport's width apart. **That is a placeholder and is marked as one**: where
//! a Place sits in the World is something a person will come to rely on, and
//! `World` holds the layout as data precisely so it can later be theirs rather
//! than this function's.

use alo_canvas::{At, Place, Showing, Size, World};

impl crate::Server {
    /// Every Place that has something on it, laid out as the World sees them.
    ///
    /// Built from the frames rather than from a stored list, so a Place with
    /// nothing on it is not a tile and nothing has to be kept in step.
    #[must_use]
    pub fn the_world(&self) -> World {
        let Some(size) = self.the_viewport_size() else {
            return World::new();
        };
        let mut places: Vec<Place> = self
            .the_frames_on_the_plane()
            .into_iter()
            .map(|frame| frame.place())
            .collect();
        places.sort_unstable();
        places.dedup();

        let across = i32::try_from(size.width()).unwrap_or(i32::MAX);
        places
            .into_iter()
            .enumerate()
            .fold(World::new(), |world, (nth, place)| {
                let nth = i32::try_from(nth).unwrap_or(i32::MAX);
                // A viewport's width apart, so tiles do not touch and the gap
                // between two Places is a real place to point at — which
                // `World::the_place_at` answers `None` for, deliberately.
                let at = At::checked(nth.saturating_mul(across.saturating_mul(2)), 0);
                match at {
                    Some(at) => world.with(place, at, size),
                    None => world,
                }
            })
    }

    /// What the person is looking at: one Place, or all of them.
    #[must_use]
    pub const fn showing(&self) -> Showing {
        self.surfaces.showing
    }

    /// Step out of the Place being looked at, into the World.
    ///
    /// Answers whether the step happened. [`false`] where the World has nothing
    /// in it — a person with one Place and nothing on it has nowhere to step out
    /// to, and a view that moved somewhere arbitrary would be worse than a press
    /// that did nothing.
    ///
    /// **Called only when `one_step_out` has already answered [`None`]**, so this
    /// never takes a rung away from zooming within a Place.
    pub fn step_out_into_the_world(&mut self) -> bool {
        if self.showing() == Showing::TheWorld {
            return false;
        }
        let Some(size) = self.the_viewport_size() else {
            return false;
        };
        let world = self.the_world();
        let Some(camera) = world.showing(size) else {
            return false;
        };
        self.surfaces.showing = Showing::TheWorld;
        self.set_the_camera(camera);
        true
    }

    /// Step into the Place under this point on the plane, filling the screen.
    ///
    /// Answers which Place was entered, or [`None`] between tiles — **a gap is a
    /// real answer**, and taking a person to the nearest Place when they aimed at
    /// nothing is the canvas's standing refusal to move somebody without their
    /// saying so.
    pub fn step_into_the_place_at(&mut self, point: At) -> Option<Place> {
        if self.showing() != Showing::TheWorld {
            return None;
        }
        let place = self.the_world().the_place_at(point)?;
        self.surfaces.showing = Showing::OnePlace(place);
        self.surfaces.place = place;
        // Back to life size on the Place that was entered, looking at its origin.
        // Not the zoom the person left it at: that is `alo-arranging`'s to
        // restore and task 5's, and guessing it here would be a second answer to
        // *where was everything*.
        let camera = alo_canvas::Camera::new();
        self.set_the_camera(camera);
        Some(place)
    }

    /// The viewport, as a size the canvas can use.
    fn the_viewport_size(&self) -> Option<Size> {
        let room = self.surfaces.popups.output_size?;
        Size::checked(u32::try_from(room.w).ok()?, u32::try_from(room.h).ok()?)
    }
}
