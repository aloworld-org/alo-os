//! *Switch user*: this session locked, and the screen handed to the sign-in.
//!
//! It is the shortest thing in this crate and the easiest to get wrong, because
//! the tempting implementation is to show the sign-in and lock afterwards — and
//! between the two there is a machine showing a greeter over an unlocked
//! desktop. So the order is carried in the code rather than in a habit:
//! [`asked`] locks through `alo_locking::Seat::locked` and only then makes the
//! `alo_greeting::Standing` a greeter draws from, and what comes back is the
//! locked seat beside it.
//!
//! **Nothing ends, and nothing closes.** The first person's applications keep
//! running, their downloads keep going and their approved turns carry on
//! (`alo_locking::Running`), because switching user is a lock and not a
//! log-out. A test holds that no application is asked to close here.
//!
//! **Nothing is written down either.** What was open is kept at a sign-out
//! ([`crate::keeping::at_sign_out`]) and a switch is not one: the session is
//! still there, and the list would be a list of what somebody currently has
//! open, written to a disk while they are still using it.
//!
//! # A locked machine is refused, and this is a finding rather than a silence
//!
//! Every other desktop offers *sign in as somebody else* on its lock screen.
//! Here it is refused, because task 1 of this plan decided what a lock screen
//! may show — the time, the lock image, the battery, and that the machine is
//! locked — and a road to the sign-in would be a fifth thing on it. That
//! decision is `alo-locking`'s and not this crate's to reopen, so what this
//! module does is refuse in the lock screen's own sentence and say so in the
//! report: a household sharing one machine has to have the first person unlock
//! before the second can sign in, and the change that would fix it is a change
//! to what a lock screen may show.

use alo_accounts::Accounts;
use alo_greeting::Standing;
use alo_locking::{NotWhileLocked, Seat};
use alo_overlay::Summoning;

/// What came of asking to switch user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Switching<N> {
    /// This session is locked, and this is what the sign-in stands at.
    HandedOver {
        /// The seat, locked and still the first person's.
        seat: Seat<N>,
        /// What the greeter shows, which on a machine somebody is signed in to
        /// is a name and a password.
        standing: Standing,
    },
    /// The machine was already locked, so nothing happened.
    NotWhileLocked {
        /// The seat, exactly as it was.
        seat: Seat<N>,
        /// The lock screen's own sentence, and nothing more.
        refused: NotWhileLocked,
    },
}

impl<N> Switching<N> {
    /// The seat, in whatever state the attempt left it — locked either way.
    pub fn seat(&self) -> &Seat<N> {
        match self {
            Self::HandedOver { seat, .. } | Self::NotWhileLocked { seat, .. } => seat,
        }
    }
}

/// A person chose *Switch user*.
///
/// `accounts` are the machine's accounts as they are now, because what the
/// greeter stands at is worked out from them and not remembered from a sign-in.
/// Nothing here authenticates anybody: the person who arrives next signs in
/// through `alo_greeting::Greeting`, the way they would at a cold machine.
pub fn asked<N>(seat: Seat<N>, summoning: &mut Summoning, accounts: &Accounts) -> Switching<N> {
    if seat.is_locked() {
        return Switching::NotWhileLocked {
            seat,
            refused: NotWhileLocked,
        };
    }
    // Locked first, and the screen only afterwards. The order is the property.
    let seat = seat.locked(summoning);
    Switching::HandedOver {
        seat,
        standing: Standing::of(accounts),
    }
}

#[cfg(test)]
mod tests {
    use alo_overlay::{Compositor, Pressed, SurfaceRefused, SurfaceRequest};

    use super::*;
    use crate::testing::{anna, the_machine};

    /// A compositor with room for the agent's overlay.
    struct Room;

    impl Compositor for Room {
        fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
            Ok(())
        }
    }

    /// **The session is locked, and the screen is the sign-in's** — and it is
    /// still the same session, not a signed-out one.
    #[test]
    fn switching_user_locks_this_session_and_hands_over_the_screen() {
        let seat = Seat::<String>::opened(anna());
        let Switching::HandedOver {
            seat: after,
            standing,
        } = asked(seat.clone(), &mut Summoning::closed(), &the_machine())
        else {
            unreachable!("an open seat was not handed over")
        };
        assert!(after.is_locked());
        assert_eq!(after.session(), seat.session());
        assert_eq!(standing, Standing::SignIn);
    }

    /// **The agent's overlay, open when the screen is handed over, is closed.**
    /// The next person at the desk does not meet the last person's agent.
    #[test]
    fn the_agents_overlay_is_closed_on_the_way_out() {
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut Room)), Pressed::Summoned);
        let switched = asked(
            Seat::<String>::opened(anna()),
            &mut summoning,
            &the_machine(),
        );
        assert!(!summoning.is_open());
        assert!(switched.seat().is_locked());
    }

    /// **A machine that is already locked is refused**, in the lock screen's own
    /// sentence, and the seat is untouched.
    #[test]
    fn an_already_locked_machine_is_refused() {
        let locked = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let switched = asked(locked.clone(), &mut Summoning::closed(), &the_machine());
        assert_eq!(
            switched,
            Switching::NotWhileLocked {
                seat: locked,
                refused: NotWhileLocked
            }
        );
    }
}
