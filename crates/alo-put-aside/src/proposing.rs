//! When the saved place is taken: what is offered, and the fact that it was shown.
//!
//! Task 4 of `docs/autonomy/putting-a-window-aside.md`. A window's saved place may be
//! occupied by the time a person brings it back, and **it must not stack invisibly
//! behind** what is there. The intended placement is shown, a nearby position is offered,
//! and the person accepts it or drags the window where they want.
//!
//! # Nothing moves before the proposal is visible
//!
//! That one sentence decides the whole shape of this file. A collision **cannot be
//! resolved while restoring** — the window is not placed, and its state does not change,
//! until the person has accepted something. So this is a two-step, and the first step
//! changes nothing at all.
//!
//! The failure being guarded is **a proposal accepted without having been shown**. That is
//! invisible stacking wearing a confirmation dialog's paperwork: the behaviour both the
//! older design note and the owner's specification refuse, dressed in the words of the
//! thing that was supposed to prevent it. A caller cannot make that mistake here, because
//! [`crate::shown::Shown`] cannot be constructed without saying where it was shown, and [`Proposal`] has
//! no other way in.
//!
//! # The two design notes agree, and the newer decides more
//!
//! `docs/design/the-alo-dock.md` says the restored window *comes forward* and *nothing is
//! rearranged*. The owner's specification says it must not stack invisibly: show the
//! placement, offer a nearby position, let the person accept or drag.
//!
//! *Comes forward* and *must not stack invisibly* are the same refusal, and *nothing is
//! rearranged* survives — **offering a position that the person takes is not the system
//! rearranging anything.** What is new is that the person is shown the collision rather
//! than having it quietly settled for them.
//!
//! # What this file does not do, and will not pretend to
//!
//! **It does not find the collision.** It is told which windows occupy the saved place, by
//! the caller that holds them, for the same reason `restoring` is handed a view rather than
//! a camera. There is also no public overlap predicate to find one with: `alo-dock`'s
//! `Patch::wholly_inside` is private and *overlaps* is a different question. Asked of the
//! lane that owns that crate rather than answered again here, because geometry about
//! somebody else's type living in this crate is a second opinion that agrees today.
//!
//! **It does not write History.** *The original position is kept in History* is part of
//! this task's acceptance and History is a `[v1]` feature (`docs/features.md`), while the
//! panel is `[v0.01]`. `CLAUDE.md` gates building to what that file says, so the clause
//! stays owed and the task does not close. What happens instead is that the original
//! position is **carried** — [`Proposal::saved`] survives both roads, accepted and dragged
//! — so History can read it when History exists. Keeping a value is not building a surface.

use alo_dock::on_the_canvas::{Patch, TheView};
use alo_dock::window::WindowId;

/// A placement offered because the saved one is taken.
///
/// Holds **both** positions. The saved one is not replaced by the offer: it is the thing
/// the person is being asked about, it is what *comes forward* refers to, and it is the
/// value History is owed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proposal {
    /// Which window is coming back.
    window: WindowId,
    /// Where it was put aside from — **the place being asked about, not abandoned**.
    saved: Patch,
    /// Where it is offered instead.
    offered: Patch,
    /// What is in the way. **Named, not counted**: a person shown *something is here*
    /// learns less than a person shown which window, and a count cannot be pointed at.
    taken_by: WindowId,
}

impl Proposal {
    /// The proposal for a window whose saved place is occupied.
    ///
    /// # Panics
    ///
    /// Never. `offered` is not validated against `saved` here, and that is deliberate
    /// rather than an omission — see [`Self::is_worth_showing`], which is a question a
    /// caller asks rather than a refusal this constructor makes. A constructor that
    /// refused would have to decide what *nearby* means, and this crate cannot measure a
    /// screen.
    #[must_use]
    pub const fn of(window: WindowId, saved: Patch, offered: Patch, taken_by: WindowId) -> Self {
        Self {
            window,
            saved,
            offered,
            taken_by,
        }
    }

    /// Which window is coming back.
    #[must_use]
    pub const fn window(self) -> WindowId {
        self.window
    }

    /// Where it was put aside from.
    ///
    /// **This is the value History is owed**, and it survives whichever road the person
    /// takes: accepting the offer does not overwrite it, and neither does dragging.
    #[must_use]
    pub const fn saved(self) -> Patch {
        self.saved
    }

    /// Where it is offered instead.
    #[must_use]
    pub const fn offered(self) -> Patch {
        self.offered
    }

    /// Which window is in the way.
    #[must_use]
    pub const fn taken_by(self) -> WindowId {
        self.taken_by
    }

    /// Whether this proposal says anything a person could act on.
    ///
    /// An offer identical to the saved place is a proposal that asks nothing — it shows a
    /// person the position they already had and calls it a choice. This reports rather than
    /// refuses, because what to do about it belongs to whoever chose the offer.
    #[must_use]
    pub fn is_worth_showing(self) -> bool {
        self.offered != self.saved
    }

    /// Whether this offer is somewhere the person can see it.
    ///
    /// **An offer drawn off screen is not shown**, however carefully it was drawn. Uses the
    /// same strict *wholly inside* reading as restoring's travel decision, and for the same
    /// reason: a rectangle with a corner on screen is one a person cannot read.
    #[must_use]
    pub fn can_be_seen_in(self, view: TheView) -> bool {
        view.already_shows(self.offered)
    }
}
