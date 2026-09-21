//! *Switch user*: this session locked, and the screen handed to the sign-in.
//!
//! It is the shortest thing in this crate and the easiest to get wrong, because
//! the tempting implementation is to show the sign-in and lock afterwards — and
//! between the two there is a machine showing a greeter over an unlocked
//! desktop. So the order is carried in the code rather than in a habit:
//! [`asked`] locks through `alo_locking::Seat::locked` and only then asks the
//! locked seat for the `alo_greeting::Standing` a greeter draws from, and what
//! comes back is the locked seat beside it.
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
//! # A machine that is already locked is handed over as it is
//!
//! It was refused until 2026-09-21, and the refusal is worth recording because
//! it was not an oversight: task 1 of this plan decided what a lock screen may
//! show — the time, the lock image, the battery, and that the machine is
//! locked — and a road to the sign-in looked like a fifth thing on it. The cost
//! was that two people sharing a machine had the first come back and type their
//! password in front of the second before the second could sign in at all.
//!
//! [ADR 0061](../../../docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md)
//! settled it, in `alo-locking`, which is where the lock screen's rules live
//! and where this crate does not make any of its own: what a lock screen may
//! **show** is unchanged, and what it may **offer** is a road that carries no
//! session at all. So [`asked`] locks — which leaves a locked seat exactly as it
//! is — and then asks `alo_locking::Seat::somebody_else`, whose answer is the
//! same on both roads into this function.
//!
//! What still refuses is a machine with no account for anybody to sign in to,
//! in the lock screen's own sentence, because a greeter standing at *make an
//! account* over somebody's locked session would be an offer to create one on
//! their machine. That is `alo-locking`'s rule too, and this crate carries it
//! rather than restating it.

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
    /// The machine has no account for anybody to sign in to, so the screen was
    /// not handed over. The session is locked either way.
    NotWhileLocked {
        /// The seat, locked and still the first person's.
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

/// A person chose *Switch user*, at their desk or at a locked screen.
///
/// `accounts` are the machine's accounts as they are now, because what the
/// greeter stands at is worked out from them and not remembered from a sign-in.
/// Nothing here authenticates anybody: the person who arrives next signs in
/// through `alo_greeting::Greeting`, the way they would at a cold machine.
pub fn asked<N>(seat: Seat<N>, summoning: &mut Summoning, accounts: &Accounts) -> Switching<N> {
    // Locked first, and the screen only afterwards. The order is the property,
    // and a seat that is already locked is left exactly as it is, holding
    // everything it held.
    let seat = seat.locked(summoning);
    // Asked of the locked seat, so that the answer is `alo-locking`'s on both
    // roads in. `standing` is `None` only where that crate refuses, which after
    // the line above is the machine with no account on it.
    match seat.somebody_else(accounts).standing() {
        Some(standing) => Switching::HandedOver { seat, standing },
        None => Switching::NotWhileLocked {
            seat,
            refused: NotWhileLocked,
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
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

    /// **A machine that is already locked is handed over as it is** (ADR 0061),
    /// which is what a household sharing one machine needs: the person who has
    /// to prove who they are is the one who wants in. The seat that comes back
    /// is the same locked seat, untouched.
    #[test]
    fn an_already_locked_machine_is_handed_over_as_it_is() {
        let locked = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let switched = asked(locked.clone(), &mut Summoning::closed(), &the_machine());
        assert_eq!(
            switched,
            Switching::HandedOver {
                seat: locked,
                standing: Standing::SignIn
            }
        );
    }

    /// **A machine with no account for anybody to sign in to is refused**, in
    /// the lock screen's own sentence — and the session is locked either way,
    /// because the lock is not what was refused.
    #[test]
    fn a_machine_with_no_account_on_it_is_refused() {
        let nobody = Accounts::none().unwrap();
        let switched = asked(
            Seat::<String>::opened(anna()),
            &mut Summoning::closed(),
            &nobody,
        );
        let Switching::NotWhileLocked { seat, refused } = switched else {
            unreachable!("a machine with no account on it handed the screen over")
        };
        assert!(seat.is_locked());
        assert_eq!(seat.session(), &anna());
        assert_eq!(refused, NotWhileLocked);
    }
}
