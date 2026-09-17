//! A change waiting for approval, at a locked machine.
//!
//! **A proposal waiting for approval cannot be put or answered on the lock
//! screen**, because approval is the signed-in person's act (ADR 0001 §5:
//! *what a person approves is that sentence*). A lock screen that showed the
//! sentence would tell a stranger what the agent is about to do with the
//! person's files; one that let it be answered would let the stranger do it.
//!
//! So all three of `alo_approving::Approving`'s calls are answered here first,
//! and on a locked seat none of them reaches the turn or the compositor. The
//! change itself is not touched: it keeps waiting, under its own clock, and
//! when the person is back it is theirs to answer — or it has lapsed the way
//! any unanswered change lapses, which is `alo-capability`'s rule and not
//! this crate's.
//!
//! A turn already under way is another matter and carries on (`crate::running`):
//! the agent may keep working behind the lock screen, and what it proposes
//! waits for the person like everything else.
//!
//! # A question up when the screen locked
//!
//! If a change was in front of the person when the machine locked, the lock
//! screen covers it. It cannot be answered through the seat until the seat is
//! open, and whatever redraws the desktop on unlock puts it up again with
//! `Seat::ask` — so what a person approves after an unlock is a sentence they
//! are looking at, not one they last saw before they walked away.

use std::time::SystemTime;

use alo_approving::{Answered, Approving, Asks, Compositor};
use alo_capability::{Grants, ProposalId};
use alo_turn::Turning;

use crate::refusing::NotWhileLocked;
use crate::seat::Seat;

impl<N> Seat<N> {
    /// Put a waiting change in front of the person.
    ///
    /// On an open seat this is `alo_approving::Approving::ask`, unchanged.
    ///
    /// # Errors
    /// [`NotWhileLocked`] on a locked seat: the compositor is not asked, and
    /// the change keeps waiting.
    pub fn ask(
        &self,
        approving: &mut Approving,
        compositor: Option<&mut dyn Compositor>,
        turning: &Turning<'_, '_>,
        id: ProposalId,
        now: SystemTime,
    ) -> Result<Asks, NotWhileLocked> {
        match self {
            Self::Locked(_) => Err(NotWhileLocked),
            Self::Open(_) => Ok(approving.ask(compositor, turning, id, now)),
        }
    }

    /// The person approved the change in front of them.
    ///
    /// On an open seat this is `alo_approving::Approving::approve`, unchanged.
    ///
    /// # Errors
    /// [`NotWhileLocked`] on a locked seat: nothing is carried out, nothing is
    /// spent, and the change keeps waiting.
    pub fn approve(
        &self,
        approving: &mut Approving,
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Answered, NotWhileLocked> {
        match self {
            Self::Locked(_) => Err(NotWhileLocked),
            Self::Open(_) => Ok(approving.approve(turning, grants, now)),
        }
    }

    /// The person said no to the change in front of them.
    ///
    /// On an open seat this is `alo_approving::Approving::decline`, unchanged.
    ///
    /// # Errors
    /// [`NotWhileLocked`] on a locked seat: a *no* is an answer too, and only
    /// the signed-in person gives one.
    pub fn decline(
        &self,
        approving: &mut Approving,
        turning: &mut Turning<'_, '_>,
        now: SystemTime,
    ) -> Result<Answered, NotWhileLocked> {
        match self {
            Self::Locked(_) => Err(NotWhileLocked),
            Self::Open(_) => Ok(approving.decline(turning, now)),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_overlay::Summoning;

    use super::*;
    use crate::testing::{ANNAS, Screen, an_invoice, anna, noon, on_a_machine, the_machine};
    use crate::unlocking::Unlocking;

    /// **A proposal waiting for approval cannot be answered from the lock
    /// screen.** It was in front of the person when the machine locked; while
    /// locked, neither *approve* nor *no* reaches the turn, the file stays
    /// where it is and the change is still waiting. Back at their desk, the
    /// person is asked again and one approval carries it out once.
    #[test]
    fn a_waiting_change_cannot_be_answered_from_the_lock_screen() {
        let (_record, places) = on_a_machine(|turning, grants, places| {
            let id = an_invoice(turning, grants, places);
            let mut approving = Approving::nothing_to_answer();
            let mut screen = Screen::default();

            let seat = Seat::<String>::opened(anna());
            assert!(matches!(
                seat.ask(&mut approving, Some(&mut screen), turning, id, noon()),
                Ok(Asks::Asked(_))
            ));

            let seat = seat.locked(&mut Summoning::closed());
            assert_eq!(
                seat.approve(&mut approving, turning, grants, noon()),
                Err(NotWhileLocked)
            );
            assert_eq!(
                seat.decline(&mut approving, turning, noon()),
                Err(NotWhileLocked)
            );
            assert!(places.march.exists(), "the change ran from the lock screen");
            assert!(turning.proposed(id).is_some(), "the change stopped waiting");

            let Unlocking::Unlocked { seat, .. } = seat.unlocks(the_machine(), "anna", ANNAS)
            else {
                unreachable!("the right password did not unlock")
            };
            assert!(matches!(
                seat.ask(&mut approving, Some(&mut screen), turning, id, noon()),
                Ok(Asks::Asked(_))
            ));
            assert!(matches!(
                seat.approve(&mut approving, turning, grants, noon()),
                Ok(Answered::Carried(_))
            ));
            assert!(!places.march.exists());
            assert_eq!(screen.asked, 2);
        });
        assert!(places.archive.join("march.pdf").exists());
    }

    /// **A change proposed while locked is not put on the lock screen.** The
    /// compositor is asked nothing and the change waits for the person.
    #[test]
    fn a_change_proposed_while_locked_is_not_shown() {
        let _ = on_a_machine(|turning, grants, places| {
            let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
            let id = an_invoice(turning, grants, places);
            let mut approving = Approving::nothing_to_answer();
            let mut screen = Screen::default();

            assert_eq!(
                seat.ask(&mut approving, Some(&mut screen), turning, id, noon()),
                Err(NotWhileLocked)
            );
            assert_eq!(screen.asked, 0);
            assert!(!approving.is_asking());
            assert!(turning.proposed(id).is_some());
        });
    }

    /// **A turn under way when the machine locked carries on.** The lock takes
    /// nothing from it: the agent can still propose, and the proposal is
    /// really made — it waits for the person rather than being refused.
    #[test]
    fn a_turn_under_way_carries_on_while_locked() {
        let _ = on_a_machine(|turning, grants, places| {
            let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
            assert!(seat.is_locked());
            let id = an_invoice(turning, grants, places);
            let waiting = turning.proposed(id).unwrap();
            assert_eq!(waiting.proposal.call().verb(), "move_file");
        });
    }
}
