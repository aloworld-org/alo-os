//! Evidence that a placement was put in front of the person.
//!
//! Its own file rather than a member of `proposing`, because it is a different reason to
//! change. `Proposal` changes when what is offered changes — a Place beside a patch, a second
//! window named as being in the way. `Shown` changes when what counts as **having been seen**
//! changes, which is a question about people and screens rather than about placement. Law 4,
//! and the split was found by a test rather than by reading: the check that these types have
//! one way in each counts constructors per file, and a file holding two types failed it.
//!
//! # What it is for
//!
//! *Nothing is rearranged without the proposal being visible first.* A rule like that is kept
//! by a **type a caller cannot forge**, not by a rule somebody remembers. Accepting a proposal
//! requires one of these, there is no way to make one without naming the view it was drawn in,
//! and this crate cannot draw — so the only code that can produce one is code that displays.
//!
//! The alternative shapes were a `bool` parameter and an `accept` that took the proposal
//! alone. Both are conventions: they are satisfied by a caller that passes `true`, or by a
//! caller that never showed anything. **A convention is a rule the next person adding a case
//! is not told about.**

use alo_dock::on_the_canvas::TheView;

/// Proof that a placement was put in front of the person.
///
/// **A token, and the point is that it cannot be forged.** It carries the view it was
/// shown in, and there is no other way to make one — so a caller cannot accept a proposal
/// it never displayed. Were this a `bool`, or were [`crate::proposing::Proposal`] acceptable without it, the
/// guarantee *nothing is rearranged without the proposal being visible first* would be a
/// convention that the next person adding a case is not told about.
///
/// It is deliberately **not** `Copy`: accepting consumes it, so one showing authorises one
/// placement. A `Copy` token would let a caller hold a single showing and place a window
/// repeatedly, which is the same fault as an approval that is really a session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The view the person was looking at when it was shown to them.
    ///
    /// Kept because *shown* has to mean **shown somewhere a person could see**, and a
    /// token that recorded only *this happened* would be satisfied by drawing the proposal
    /// off screen.
    in_view: TheView,
}

impl Shown {
    /// Record that a placement was drawn in this view.
    ///
    /// Called by whoever draws. Nothing in this crate can draw, which is exactly why this
    /// is the only way in: the knowledge that a person could see it does not exist here.
    #[must_use]
    pub const fn in_view(in_view: TheView) -> Self {
        Self { in_view }
    }

    /// The view it was shown in.
    #[must_use]
    pub const fn view(&self) -> TheView {
        self.in_view
    }
}
