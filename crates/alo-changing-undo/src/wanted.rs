//! The window a person asked for, which is a window.
//!
//! One of the two things a person may do in this pane. [`Wanted`] exists so that
//! what leaves here has already been through the only check there is — a window
//! reaches at least one day and at least one change — and so a caller cannot
//! write a pair of numbers to a person's file that the machine would refuse to
//! read back.
//!
//! # Why this is not simply a `HowFarBack`
//!
//! It is one, inside. What it adds is that it came from a **person**, this
//! minute, in front of this pane — and that is worth a type, because the only
//! thing a caller may do with it is keep it, and the only window a caller may
//! keep is one that came from here. A bare `HowFarBack` would let a surface
//! write the window it had just read, or the window it ships with, or one it
//! worked out, and nothing in the types would notice.
//!
//! # What it does not do
//!
//! It does not write anything. The person's file belongs to the crate that owns it,
//! and it stays there: this crate ships no dependency on that one, for the
//! reason its own header gives. [`Wanted::window`] is what a caller hands to
//! `keep`, and `tests/the_machine_really_obeys_the_changed_window.rs` is how
//! this crate knows that road works without taking it.

use alo_keeping_up::{HowFarBack, NotAWindow};

/// A window a person asked for.
///
/// Consumed by whoever keeps it: it is one person's one decision, and a value
/// that could be kept twice would be a setting written twice from one choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub struct Wanted {
    /// What they asked for, already a window.
    window: HowFarBack,
}

impl Wanted {
    /// A person asked for `days` days and `changes` changing turns.
    ///
    /// # Errors
    ///
    /// [`NotAWindow`] when either is zero, which `alo_keeping_up` says in one
    /// sentence of its own — there is no second wording of it here, because what
    /// a person does about it is the same whichever half they missed.
    pub fn of(days: u32, changes: u32) -> Result<Self, NotAWindow> {
        Ok(Self {
            window: HowFarBack::of(days, changes)?,
        })
    }

    /// The window to keep.
    #[must_use]
    pub const fn window(self) -> HowFarBack {
        self.window
    }

    /// Whether this reaches further back than `than` in either half.
    ///
    /// *Either*, not both: the two numbers are limits and the smaller wins, so a
    /// window of thirty days and fifty changes reaches further than seven days
    /// and fifty changes on the machine whose days run out first, and no further
    /// at all on the machine whose changes do. A pane saying *wider* about the
    /// pair is saying the only thing that is true of both machines.
    #[must_use]
    pub fn reaches_further_than(self, than: HowFarBack) -> bool {
        self.window.days() > than.days() || self.window.turns() > than.turns()
    }

    /// Whether this reaches less far back than `than` in either half.
    #[must_use]
    pub fn reaches_less_far_than(self, than: HowFarBack) -> bool {
        self.window.days() < than.days() || self.window.turns() < than.turns()
    }

    /// Whether this is the window already in force, so that keeping it would
    /// write a file to say nothing had changed.
    #[must_use]
    pub fn is_what_it_already_was(self, already: HowFarBack) -> bool {
        self.window == already
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A window that reaches nothing is refused here**, before anything could
    /// write it — and in the refusal `alo_keeping_up` already has.
    #[test]
    fn a_window_that_reaches_nothing_is_refused() {
        assert_eq!(Wanted::of(0, 50), Err(NotAWindow::NoDays));
        assert_eq!(Wanted::of(7, 0), Err(NotAWindow::NoTurns));
        assert!(Wanted::of(1, 1).is_ok());
    }

    /// **What a person asked for is what comes out**, unrounded and unclamped.
    /// A pane that quietly widened a narrow window would be a machine keeping
    /// more of a person's past than they asked it to.
    #[test]
    fn what_comes_out_is_what_was_asked_for() {
        let wanted = Wanted::of(30, 200).unwrap();
        assert_eq!(wanted.window(), HowFarBack::of(30, 200).unwrap());
        assert_eq!(wanted.window().days(), 30);
        assert_eq!(wanted.window().turns(), 200);
    }

    /// **Wider, narrower, and neither** — each read off the pair rather than off
    /// one half of it.
    #[test]
    fn wider_and_narrower_are_read_off_both_numbers() {
        let shipped = HowFarBack::AS_SHIPPED;
        let wider = Wanted::of(30, 200).unwrap();
        assert!(wider.reaches_further_than(shipped));
        assert!(!wider.reaches_less_far_than(shipped));

        let narrower = Wanted::of(1, 5).unwrap();
        assert!(narrower.reaches_less_far_than(shipped));
        assert!(!narrower.reaches_further_than(shipped));

        let same = Wanted::of(shipped.days(), shipped.turns()).unwrap();
        assert!(!same.reaches_further_than(shipped));
        assert!(!same.reaches_less_far_than(shipped));
        assert!(same.is_what_it_already_was(shipped));
    }

    /// **A window longer in days and shorter in changes is both**, and says so
    /// rather than picking one. On one machine it reaches further and on another
    /// it reaches less far, and which is which depends on how hard the machine
    /// is used — so a pane that answered one of them would be wrong half the
    /// time and never say which half.
    #[test]
    fn a_window_longer_one_way_and_shorter_the_other_is_both() {
        let mixed = Wanted::of(30, 5).unwrap();
        assert!(mixed.reaches_further_than(HowFarBack::AS_SHIPPED));
        assert!(mixed.reaches_less_far_than(HowFarBack::AS_SHIPPED));
    }
}
