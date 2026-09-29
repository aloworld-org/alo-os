//! Going somewhere on the canvas, looking without going, and coming back.
//!
//! Three things that all concern the view rather than the windows, which is why
//! they are one file: **a window is never moved by any of them.**
//!
//! # Back returns a view, not an application
//!
//! After travelling, *Back to previous view* restores **the exact pan and
//! zoom** — the whole [`crate::TheView`], not the window that used to be
//! focused. Those come apart the moment somebody travels twice, or travels and
//! then pans by hand, and a Back that restored *the last window* would take
//! people somewhere they had not been.
//!
//! **It goes after the next deliberate action.** A control that lingers is a
//! control whose meaning drifts: press it five minutes later and *previous* is a
//! view you no longer remember. So it is offered for the one move that follows
//! the travel, and then it is gone.
//!
//! **And it is never a second click on the icon.** Clicking again leaves the
//! window focused ([`crate::clicking`]); going back is its own control. A control
//! that is sometimes something else is one nobody can rely on, and *click again
//! to go back* would make the second click's meaning depend on state a person
//! cannot see.
//!
//! # Peek looks without going
//!
//! Holding, or choosing *Peek*, gives a readable look at another window while
//! the canvas **stays exactly where it is**. Releasing, or Escape, ends it.
//! Clicking the preview travels properly, which is the ordinary road.
//!
//! **The keyboard reaches the same answer as the pointer.** Peeking is not a
//! pointer gesture with a keyboard consolation — [`Peeking::begun`] does not ask
//! how it started, for the same reason [`crate::previews`] does not.
//!
//! # What is deliberately not here
//!
//! No easing, no duration, no path. Whether the move is animated, and how, is
//! the compositor's, and it reads the person's motion preference from
//! `alo-appearance` rather than from a number invented here. This file answers
//! *which view*, and how a view is reached is drawing.

use crate::on_the_canvas::TheView;

/// Where the view was before the last travel, while going back is still offered.
///
/// Held rather than a stack: **one step**. *Back* answers *undo that jump*, and a
/// history of jumps is a different feature with a different control — offering
/// one button that sometimes goes back one place and sometimes four is the drift
/// this type exists to refuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GoingBack {
    /// The view to return to, while it is still offered.
    to: Option<TheView>,
}

/// What a person did next, which decides whether *Back* is still offered.
///
/// Named rather than a `bool` so a caller cannot pass *this was deliberate* by
/// accident: the whole rule turns on which acts count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatHappenedNext {
    /// A deliberate act: another travel, a pan, a zoom, opening something.
    /// **Back stops being offered.**
    SomethingDeliberate,
    /// Peeking, which moves nothing and is not a move. Back survives it.
    APeek,
    /// Going back itself, which spends it.
    GoingBack,
}

impl GoingBack {
    /// Nothing to go back to, which is where a session starts.
    #[must_use]
    pub const fn nowhere() -> Self {
        Self { to: None }
    }

    /// The view has travelled away from `from`, so going back is now offered.
    ///
    /// **Replaces whatever was there.** One step, so the place a person can
    /// return to is always the one they just left.
    #[must_use]
    pub const fn travelled_from(from: TheView) -> Self {
        Self { to: Some(from) }
    }

    /// Whether going back is offered at this moment.
    #[must_use]
    pub const fn is_offered(self) -> bool {
        self.to.is_some()
    }

    /// The view it would return to, while it is offered.
    #[must_use]
    pub const fn to(self) -> Option<TheView> {
        self.to
    }

    /// Go back. Answers the view to restore, and leaves nothing to go back to.
    ///
    /// Answering `None` when it is not offered rather than refusing: a control
    /// that is not drawn cannot be pressed, and a shell that asks anyway should
    /// get *nothing to do* rather than an error to handle.
    #[must_use]
    pub const fn go_back(self) -> (Option<TheView>, Self) {
        (self.to, Self::nowhere())
    }

    /// What became of the offer after this happened.
    #[must_use]
    pub const fn after(self, next: WhatHappenedNext) -> Self {
        match next {
            WhatHappenedNext::SomethingDeliberate | WhatHappenedNext::GoingBack => Self::nowhere(),
            WhatHappenedNext::APeek => self,
        }
    }
}

/// Whether a window is being looked at without the canvas moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Peeking {
    /// The window being looked at, if one is.
    at: Option<crate::window::WindowId>,
}

/// How a peek ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeekEnded {
    /// Let go, or Escape. The canvas never moved, so nothing is restored.
    AndNothingMoved,
    /// The preview was chosen, so the ordinary travel happens instead.
    ByTravellingThere(crate::window::WindowId),
}

impl Peeking {
    /// Not peeking at anything.
    #[must_use]
    pub const fn at_nothing() -> Self {
        Self { at: None }
    }

    /// Peeking at this window.
    ///
    /// **Does not ask how it began.** Held with a pointer, chosen from a menu or
    /// reached by keyboard, it is the same state — the keyboard road has to end
    /// somewhere identical or it is a consolation rather than a road.
    #[must_use]
    pub const fn begun(at: crate::window::WindowId) -> Self {
        Self { at: Some(at) }
    }

    /// The window being peeked at, if any.
    #[must_use]
    pub const fn at(self) -> Option<crate::window::WindowId> {
        self.at
    }

    /// Whether anything is being peeked at.
    #[must_use]
    pub const fn is_peeking(self) -> bool {
        self.at.is_some()
    }

    /// Let go, or Escape.
    #[must_use]
    pub const fn let_go(self) -> (PeekEnded, Self) {
        (PeekEnded::AndNothingMoved, Self::at_nothing())
    }

    /// Choose it, which ends the peek and travels there the ordinary way.
    #[must_use]
    pub const fn choose_it(self) -> (Option<PeekEnded>, Self) {
        match self.at {
            Some(id) => (Some(PeekEnded::ByTravellingThere(id)), Self::at_nothing()),
            None => (None, self),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::on_the_canvas::{Patch, Spot};
    use crate::window::WindowId;

    fn view(x: i64, wide: u32) -> TheView {
        TheView::showing(Patch::of(Spot::at(x, 0), wide, 1_080).unwrap())
    }

    /// **Back restores the exact pan and zoom**, which is the whole view rather
    /// than the window that used to be focused — a different rectangle *and* a
    /// different size, so a Back that only restored position would fail this.
    #[test]
    fn back_restores_the_exact_pan_and_zoom() {
        let was = view(0, 1_920);
        let back = GoingBack::travelled_from(was);
        assert!(back.is_offered());

        let (to, after) = back.go_back();
        assert_eq!(to, Some(was));
        assert_eq!(to.unwrap().shown().corner().x(), 0, "the pan");
        assert_eq!(to.unwrap().shown().wide(), 1_920, "and the zoom");
        assert!(!after.is_offered(), "and it is spent");
    }

    /// **It goes after the next deliberate action**, because *previous* stops
    /// meaning anything a person remembers.
    #[test]
    fn a_deliberate_action_takes_the_offer_away() {
        let back = GoingBack::travelled_from(view(0, 1_920));
        assert!(
            !back
                .after(WhatHappenedNext::SomethingDeliberate)
                .is_offered()
        );
        assert!(!back.after(WhatHappenedNext::GoingBack).is_offered());
    }

    /// **A peek is not a deliberate move and does not spend it.** Looking at
    /// something without going there leaves the way back exactly as it was,
    /// which is what makes peeking safe to use while travelling.
    #[test]
    fn peeking_does_not_spend_the_way_back() {
        let back = GoingBack::travelled_from(view(4_000, 1_920));
        let after = back.after(WhatHappenedNext::APeek);
        assert!(after.is_offered());
        assert_eq!(after.to(), back.to(), "and it is the same view");
    }

    /// **One step, not a history.** Travelling again replaces where Back goes,
    /// so the place it returns to is always the one just left — never four jumps
    /// ago, which is what a stack behind one button would give.
    #[test]
    fn travelling_again_replaces_where_back_goes() {
        let first = GoingBack::travelled_from(view(0, 1_920));
        let second = GoingBack::travelled_from(view(9_000, 1_920));
        assert_ne!(first.to(), second.to());
        assert_eq!(second.go_back().0, Some(view(9_000, 1_920)));
    }

    /// Asking to go back when it is not offered answers *nothing to do* rather
    /// than refusing: a control that is not drawn cannot be pressed.
    #[test]
    fn going_back_when_there_is_nowhere_is_not_an_error() {
        let (to, after) = GoingBack::nowhere().go_back();
        assert_eq!(to, None);
        assert!(!after.is_offered());
    }

    /// **Peek moves nothing.** It ends with the canvas where it was, and there
    /// is nothing to restore because nothing was disturbed.
    #[test]
    fn a_peek_ends_with_the_canvas_where_it_was() {
        let peek = Peeking::begun(WindowId::numbered(7));
        assert!(peek.is_peeking());
        assert_eq!(peek.at(), Some(WindowId::numbered(7)));

        let (ended, after) = peek.let_go();
        assert_eq!(ended, PeekEnded::AndNothingMoved);
        assert!(!after.is_peeking());
    }

    /// **Choosing what is being peeked at travels there the ordinary way**, so
    /// there is one road to a window rather than a second one hiding in the
    /// peek.
    #[test]
    fn choosing_the_peeked_window_becomes_an_ordinary_travel() {
        let (ended, after) = Peeking::begun(WindowId::numbered(3)).choose_it();
        assert_eq!(
            ended,
            Some(PeekEnded::ByTravellingThere(WindowId::numbered(3)))
        );
        assert!(!after.is_peeking());
    }

    /// Choosing when nothing is being peeked at answers nothing, and leaves the
    /// state alone.
    #[test]
    fn choosing_nothing_answers_nothing() {
        let (ended, after) = Peeking::at_nothing().choose_it();
        assert_eq!(ended, None);
        assert!(!after.is_peeking());
    }

    /// **A whole trip, in the order a person makes it:** travel, peek at
    /// something else, come back — and Back still goes to where they started,
    /// because the peek never moved anything.
    #[test]
    fn travel_then_peek_then_back_lands_where_the_person_started() {
        let started = view(0, 1_920);
        let back = GoingBack::travelled_from(started);

        let peek = Peeking::begun(WindowId::numbered(2));
        let (ended, peek) = peek.let_go();
        assert_eq!(ended, PeekEnded::AndNothingMoved);
        assert!(!peek.is_peeking());

        let back = back.after(WhatHappenedNext::APeek);
        assert_eq!(back.go_back().0, Some(started));
    }
}
