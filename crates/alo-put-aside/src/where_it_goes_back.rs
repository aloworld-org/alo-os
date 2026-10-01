//! Where a window goes when it comes back.
//!
//! Its own file rather than a tuple returned from `putting_aside`, and rather than two
//! `Patch` and `Zoom` arguments threaded through every caller, for a reason that is about
//! what is still coming. **What a person gets back is a view of the plane, not a
//! rectangle**, and the third member arrived on 2026-10-01: the Place. A tuple would have
//! had to be rewritten at every call site that day; a named type gained a field and an
//! accessor, which is what this file was written in advance to make possible.
//!
//! It is also the honest shape of the promise. *Restore it where it was* means the same
//! part of the plane **at the same size**, and a return type that carried only the patch
//! would let a caller restore a window to its saved rectangle under whatever zoom the
//! person happened to be at — which looks like a bug in the window and is a bug in what
//! was saved.
//!
//! # Every value here was saved, never recomputed
//!
//! Both members are handed over as they were taken. Nothing in this file calculates a
//! position, and that is what makes the round trip exact rather than close: it returns
//! the same values, so there are no two calculations that have to agree.

use alo_canvas::{Place, Zoom};
use alo_dock::on_the_canvas::Patch;

/// The view a window returns to: the patch it left, at the zoom it was left at, on the Place
/// it was left on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhereItGoesBack {
    /// The part of the plane the window occupied.
    at: Patch,
    /// How far in the person was when they put it aside.
    ///
    /// **Saved rather than read at restore time.** Reading the current zoom would make
    /// *bring it back* mean something different depending on where the person had
    /// wandered since, and the one thing this surface promises is that it does not.
    zoom: Zoom,
    /// Which Place it was on.
    ///
    /// **The member this file was written in advance for**, and the reason it is not
    /// optional: a patch alone is ambiguous, because `(4200, 0)` exists on every surface. A
    /// caller holding only the patch would restore the window to whichever Place the person
    /// happens to be looking at — which is not a near miss, it is a window somewhere else.
    ///
    /// **Handed in, like the zoom.** This crate cannot ask a camera and must not learn to
    /// ask a canvas either: a Place arriving as an argument is not a crate reading the
    /// world, and `tests/the_panel_never_reaches_the_camera.rs` goes on passing while the
    /// panel learns about Places.
    place: Place,
}

impl WhereItGoesBack {
    /// The view saved for a window, from the values that were saved.
    ///
    /// All three at once, which is the whole point of the type: a caller cannot take the
    /// patch and forget the zoom, and now cannot take either and forget the Place.
    #[must_use]
    pub const fn of(at: Patch, zoom: Zoom, place: Place) -> Self {
        Self { at, zoom, place }
    }

    /// The part of the plane it occupied.
    #[must_use]
    pub const fn at(self) -> Patch {
        self.at
    }

    /// How far in the person was.
    #[must_use]
    pub const fn zoom(self) -> Zoom {
        self.zoom
    }

    /// Which Place it was on.
    #[must_use]
    pub const fn place(self) -> Place {
        self.place
    }

    /// Whether this view is on the Place somebody is looking at now.
    ///
    /// The question a restore has to ask before it decides anything about travelling: a
    /// window whose Place is not the one on screen needs the canvas to go to **that Place**,
    /// not to a rectangle on this one. Offered here rather than left to a caller comparing
    /// numbers, because `place() == looking_at` is the comparison somebody writes once
    /// correctly and then writes backwards.
    #[must_use]
    pub fn is_on(self, looking_at: Place) -> bool {
        self.place == looking_at
    }
}
