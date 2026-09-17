//! Unlocking, which is signing in again — through the same composition, and
//! through nothing weaker.
//!
//! A lock screen that accepted less than the sign-in did would make locking a
//! formality: a PIN where the account has a password, a remembered face, a
//! *continue as Anna* button, a timeout that skips the question for somebody
//! who was only away a minute. There is none of that here, because the only
//! thing this file does with a name and a password is hand them to
//! `alo_greeting::Greeting::signs_in` — the composition the greeter uses, with
//! `alo-accounts` deciding and its one refusal carried unchanged, evenly timed
//! for a wrong password and an unknown name exactly as at sign-in.
//!
//! What differs is only the door the greeting knocks at afterwards. At sign-in
//! it is the opener, which starts a session; here it is the locked session
//! itself (`src/the_locked_session.rs`), because the session never ended and
//! starting a second one is the thing an unlock must not do. And the greeting
//! is built for **this session's number and no other**, so another account on
//! the same machine is refused by `alo-accounts`' own rule that a sign-in must
//! be the described person — nobody unlocks somebody else's session.
//!
//! `tests/unlocking_is_signing_in.rs` reads this crate's source and holds it to
//! that: one call that checks a password, and it is the greeting's.

use alo_accounts::{Accounts, NOT_SIGNED_IN};
use alo_greeting::{Greeted, Greeting};

use crate::locked::Locked;
use crate::seat::Seat;
use crate::the_locked_session::TheLockedSession;

/// What an attempt to unlock did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unlocking<N> {
    /// The person proved who they are: the seat is open again, and here is
    /// everything that arrived for them while it was locked, oldest first.
    Unlocked {
        /// The seat, open.
        seat: Seat<N>,
        /// What was held behind the lock.
        held: Vec<N>,
    },
    /// Not unlocked. The seat is still locked and still holding what it held,
    /// and this is what the greeting said — `alo-accounts`' own refusal, read
    /// through [`Greeted::said`].
    StillLocked {
        /// The seat, locked.
        seat: Seat<N>,
        /// Why, as the greeting answered.
        refused: Greeted,
    },
    /// There was nothing to unlock: the seat was open.
    WasNotLocked {
        /// The seat, unchanged.
        seat: Seat<N>,
    },
}

impl<N> Unlocking<N> {
    /// The seat, in whatever state the attempt left it.
    #[must_use]
    pub fn seat(&self) -> &Seat<N> {
        match self {
            Self::Unlocked { seat, .. }
            | Self::StillLocked { seat, .. }
            | Self::WasNotLocked { seat } => seat,
        }
    }
}

impl<N> Seat<N> {
    /// Somebody at the lock screen typed a name and a password.
    ///
    /// `accounts` are the machine's accounts as they are now — read at the
    /// moment of the unlock rather than remembered from the sign-in, so a
    /// password changed or an account removed while the machine was locked is
    /// the password and the account that count.
    #[must_use]
    pub fn unlocks(self, accounts: Accounts, name: &str, password: &str) -> Unlocking<N> {
        let locked = match self {
            Self::Open(session) => {
                return Unlocking::WasNotLocked {
                    seat: Self::Open(session),
                };
            }
            Self::Locked(locked) => locked,
        };
        let uid = locked.session().uid();
        let greeting = Greeting::of(accounts, uid, TheLockedSession::running_as(uid));
        match greeting.signs_in(name, password) {
            Greeted::SignedIn(session) if &session == locked.session() => {
                let (session, held) = locked.into_parts();
                Unlocking::Unlocked {
                    seat: Self::Open(session),
                    held,
                }
            }
            // The greeting was built for this number, so a session for any
            // other account cannot come back; if the name on the account
            // changed underneath, it is not the session that was locked, and
            // it is refused in `alo-accounts`' own words rather than let in.
            Greeted::SignedIn(_) => still_locked(locked, Greeted::Refused(NOT_SIGNED_IN.key())),
            refused @ (Greeted::Refused(_) | Greeted::NotAnswered(_)) => {
                still_locked(locked, refused)
            }
        }
    }
}

/// The seat stays locked, holding what it held.
fn still_locked<N>(locked: Locked<N>, refused: Greeted) -> Unlocking<N> {
    Unlocking::StillLocked {
        seat: Seat::Locked(locked),
        refused,
    }
}

/// The session a seat is locked over — for the tests below, which compare it.
#[cfg(test)]
fn locked_over<N>(seat: &Seat<N>) -> Option<&alo_accounts::Session> {
    match seat {
        Seat::Locked(locked) => Some(locked.session()),
        Seat::Open(_) => None,
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
    use crate::testing::{
        ANNAS, BENS, a_notification, anna, in_english, the_machine, the_machine_with_ben,
    };

    /// **The right name and password unlock, and hand back what was held** —
    /// in the order it arrived, to the person who proved who they are.
    #[test]
    fn the_right_password_unlocks_and_hands_back_what_was_held() {
        let mut seat = Seat::opened(anna()).locked(&mut Summoning::closed());
        let _ = seat.arrives(a_notification("first"));
        let _ = seat.arrives(a_notification("second"));

        let Unlocking::Unlocked { seat, held } = seat.unlocks(the_machine(), "anna", ANNAS) else {
            unreachable!("the right password did not unlock")
        };
        assert_eq!(seat, Seat::opened(anna()));
        assert_eq!(
            held,
            vec![a_notification("first"), a_notification("second")]
        );
    }

    /// **A wrong password and an unknown name are refused with the sign-in's
    /// own sentence**, and the seat stays locked holding everything it held.
    #[test]
    fn a_wrong_password_or_an_unknown_name_leaves_the_seat_locked() {
        for (name, password) in [("anna", "wrong"), ("nobody", ANNAS), ("", "")] {
            let mut seat = Seat::opened(anna()).locked(&mut Summoning::closed());
            let _ = seat.arrives(a_notification("kept"));
            let before = seat.clone();

            let Unlocking::StillLocked { seat, refused } =
                seat.unlocks(the_machine(), name, password)
            else {
                unreachable!("{name} unlocked with {password}")
            };
            assert_eq!(seat, before, "{name}");
            assert_eq!(refused, Greeted::Refused(NOT_SIGNED_IN.key()), "{name}");
            let said = refused.said(&in_english()).unwrap();
            assert_eq!(said.text(), NOT_SIGNED_IN.says());
        }
    }

    /// **Another account on the same machine does not unlock this session**,
    /// with its own correct password. Unlocking is the locked person proving
    /// who they are, not anybody who can sign in.
    #[test]
    fn somebody_elses_correct_password_does_not_unlock_this_session() {
        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let Unlocking::StillLocked { seat, refused } =
            seat.unlocks(the_machine_with_ben(), "ben", BENS)
        else {
            unreachable!("ben unlocked anna's session")
        };
        assert!(seat.is_locked());
        assert_eq!(locked_over(&seat), Some(&anna()));
        assert!(matches!(refused, Greeted::Refused(_)));
    }

    /// **Unlocking reads the accounts as they are now.** A password changed
    /// while the machine was locked is the one that counts, and the old one
    /// no longer opens it.
    #[test]
    fn a_password_changed_while_locked_is_the_one_that_counts() {
        /// The machine's accounts after Anna changed her password.
        fn changed() -> alo_accounts::Accounts {
            let mut accounts = alo_accounts::Accounts::none().unwrap();
            accounts
                .created("anna", anna().uid(), "a new password nobody else knows")
                .unwrap();
            accounts
        }

        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let Unlocking::StillLocked { seat, .. } = seat.unlocks(changed(), "anna", ANNAS) else {
            unreachable!("the old password unlocked")
        };
        assert!(matches!(
            seat.unlocks(changed(), "anna", "a new password nobody else knows"),
            Unlocking::Unlocked { .. }
        ));
    }

    /// **An open seat has nothing to unlock**, and a password typed at it
    /// changes nothing.
    #[test]
    fn an_open_seat_is_not_unlocked_again() {
        let seat = Seat::<String>::opened(anna());
        let unlocking = seat.clone().unlocks(the_machine(), "anna", ANNAS);
        assert_eq!(unlocking, Unlocking::WasNotLocked { seat: seat.clone() });
        assert_eq!(unlocking.seat(), &seat);
    }
}
