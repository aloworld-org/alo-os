//! Where a window goes when it comes back.
//!
//! Its own file rather than a tuple returned from `putting_aside`, and rather than two
//! `Patch` and `Zoom` arguments threaded through every caller, for a reason that is about
//! what is still coming. **What a person gets back is a view of the plane, not a
//! rectangle**, and it is gaining a third member: the Place. A tuple would have to be
//! rewritten at every call site the day that arrives; a named type gains a field and an
//! accessor.
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

use alo_canvas::Zoom;
use alo_dock::on_the_canvas::Patch;

/// The view a window returns to: the patch it left, at the zoom it was left at.
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
}

impl WhereItGoesBack {
    /// The view saved for a window, from the values that were saved.
    #[must_use]
    pub const fn of(at: Patch, zoom: Zoom) -> Self {
        Self { at, zoom }
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
}
