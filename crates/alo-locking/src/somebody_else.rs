//! Somebody else, at a locked screen: the one road off it that is not the way
//! back in.
//!
//! Two people share a machine. The first locks it and goes. Before
//! [ADR 0061](../../../docs/decisions/0061-a-locked-screen-offers-a-road-to-the-greeter.md)
//! the second could not reach the sign-in at all — `alo_leaving::switching`
//! locks an **open** session and hands the screen over, and a screen already
//! locked was refused, because what a lock screen may show is four things and a
//! road to the sign-in is not one of them. So the first person came back and
//! typed their password in front of the second, or the household stopped
//! locking the screen. A lock nobody uses protects nothing.
//!
//! That record widened the rule once and refused to widen it again, and this
//! module is the shape of what it decided:
//!
//! # It carries no session, and cannot
//!
//! [`SomebodyElse`] has **no generic parameter**. A [`crate::Seat`] is generic
//! over what a notification is and holds every one that arrived while the
//! machine was locked; a value with nowhere to put one cannot leak one, whatever
//! a later change does to the shell.
//!
//! ```compile_fail
//! let _: alo_locking::SomebodyElse<String>;
//! ```
//!
//! What it carries instead is `alo_greeting::Standing`, which has two values and
//! both are facts about the machine's account store rather than about anybody's
//! session. There is no list of who has an account here, no count of them and no
//! name: this product's greeter asks for a name typed rather than offering one
//! to pick (ADR 0024), which is why a road to it discloses nothing that a
//! password box on the same screen does not already.
//!
//! # It changes nothing, and the signature is what says so
//!
//! The road takes `&self`. It cannot end the session, sign it out, unlock it or
//! take anything it holds — task 1's clause 8 kept by the type rather than by a
//! paragraph — and because it changes nothing, a surface may ask it twice: once
//! to decide whether to offer the road at all, and again when somebody takes it.
//!
//! # And it is one refusal rather than none
//!
//! A machine whose accounts stand at *make an account* is refused. On a cold
//! machine that state is ADR 0024's first-boot screen and it is right; on a
//! **locked** one it would be an offer, made to whoever is at the desk, to
//! create an account on somebody else's machine and sign in on it. So the road
//! hands over `alo_greeting::Standing::SignIn` and nothing else, and every other
//! answer is [`crate::NotWhileLocked`] — the same sentence the agent's key and a
//! waiting change already get.
//!
//! # Nothing here draws, and nothing here signs anybody in
//!
//! Where the road sits on a locked screen is the shell's, from
//! [`crate::words::SOMEBODY_ELSE`]. Signing in is
//! `alo_greeting::Greeting::signs_in` and stays the only road through;
//! opening the second person's session is `alo-sessiond`'s, which this module
//! neither names nor reaches.

use alo_accounts::Accounts;
use alo_greeting::Standing;

use crate::refusing::NotWhileLocked;
use crate::seat::Seat;

/// What came of asking, at a locked screen, to let somebody else sign in.
///
/// Made only by [`Seat::somebody_else`]. It holds nothing about the session
/// behind the lock, and has no room for anything: no person, no notification,
/// no window, and no type parameter a caller could fill with one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SomebodyElse {
    /// The screen is the sign-in's. This is what the greeter stands at, which
    /// on a machine with an account on it is a name and a password, and says
    /// nothing above them.
    ///
    /// The session behind the lock is still locked, still running and still
    /// the first person's.
    TheGreeter(Standing),
    /// Refused, in the lock screen's own sentence: this machine has no account
    /// for anybody to sign in to, and a lock screen never offers to make one.
    Refused(NotWhileLocked),
    /// The seat is open, so there is no lock screen and no road from one.
    /// *Switch user* from a desktop is `alo_leaving::switching`, which locks
    /// this session first.
    NoLockScreen,
}

impl SomebodyElse {
    /// What the sign-in stands at, when the screen was handed over — and
    /// [`None`] on either answer that did not hand it over.
    #[must_use]
    pub const fn standing(self) -> Option<Standing> {
        match self {
            Self::TheGreeter(standing) => Some(standing),
            Self::Refused(_) | Self::NoLockScreen => None,
        }
    }

    /// Whether the screen was handed to the greeter.
    #[must_use]
    pub const fn is_the_greeter(self) -> bool {
        matches!(self, Self::TheGreeter(_))
    }
}

impl<N> Seat<N> {
    /// Somebody at a locked screen asked to let another person sign in.
    ///
    /// `accounts` are the machine's accounts as they are now, read at the
    /// moment of the asking rather than remembered from the sign-in — the same
    /// rule [`Seat::unlocks`] follows, and for the same reason: an account
    /// added or taken away while the machine was locked is the machine as it
    /// is.
    ///
    /// Nothing about this seat leaves in the answer, and nothing about it
    /// changes. Whoever arrives signs in through `alo_greeting::Greeting`, the
    /// way they would at a cold machine.
    #[must_use]
    pub fn somebody_else(&self, accounts: &Accounts) -> SomebodyElse {
        if !self.is_locked() {
            return SomebodyElse::NoLockScreen;
        }
        let standing = Standing::of(accounts);
        match standing {
            Standing::SignIn => SomebodyElse::TheGreeter(standing),
            Standing::MakeAnAccount => SomebodyElse::Refused(NotWhileLocked),
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
    use crate::testing::{
        ANNAS, a_notification, anna, in_english, the_machine, the_machine_with_ben,
    };
    use crate::{Arrived, Unlocking};

    /// **A locked screen hands the screen to the greeter**, which stands at a
    /// name and a password and says nothing above them.
    #[test]
    fn a_locked_screen_hands_the_screen_to_the_greeter() {
        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let asked = seat.somebody_else(&the_machine());
        assert_eq!(asked, SomebodyElse::TheGreeter(Standing::SignIn));
        assert_eq!(asked.standing(), Some(Standing::SignIn));
        assert!(asked.is_the_greeter());
        assert!(
            Standing::SignIn.said(&in_english()).is_none(),
            "the greeter says something above a name and a password"
        );
    }

    /// **The machine's accounts are read as they are now.** A second account
    /// made while the machine was locked changes nothing about the road, which
    /// is the point: the road never says how many there are.
    #[test]
    fn a_second_account_made_while_locked_changes_nothing_about_the_road() {
        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        assert_eq!(
            seat.somebody_else(&the_machine_with_ben()),
            seat.somebody_else(&the_machine())
        );
    }

    /// **A machine with no account on it is refused**, in the lock screen's own
    /// sentence. *Make an account* is the first screen a cold machine shows and
    /// it must never be an offer made over somebody's locked session.
    #[test]
    fn a_machine_with_no_account_on_it_is_refused() {
        let nobody = alo_accounts::Accounts::none().unwrap();
        assert_eq!(Standing::of(&nobody), Standing::MakeAnAccount);

        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let asked = seat.somebody_else(&nobody);
        assert_eq!(asked, SomebodyElse::Refused(NotWhileLocked));
        assert_eq!(asked.standing(), None);
        assert!(!asked.is_the_greeter());

        let said = NotWhileLocked.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(said.text(), "This machine is locked");
    }

    /// **An open seat has no lock screen, and so no road from one.** Asking it
    /// is not a refusal and not a handover: it is a caller looking at the
    /// wrong screen, and *switch user* from a desktop is `alo-leaving`'s.
    #[test]
    fn an_open_seat_has_no_lock_screen_to_ask_it_of() {
        let seat = Seat::<String>::opened(anna());
        assert_eq!(
            seat.somebody_else(&the_machine()),
            SomebodyElse::NoLockScreen
        );
        assert_eq!(seat.somebody_else(&the_machine()).standing(), None);
    }

    /// **The session is not ended, not signed out and not unlocked by any of
    /// it** — and it still holds everything it held, which the person gets when
    /// they come back and prove who they are.
    #[test]
    fn the_locked_session_is_untouched_and_still_holds_what_it_held() {
        let mut seat = Seat::opened(anna()).locked(&mut Summoning::closed());
        assert_eq!(
            seat.arrives(a_notification("Re: the Lisbon offer")),
            Arrived::Held
        );
        let before = seat.clone();

        for _ in 0..3 {
            assert!(seat.somebody_else(&the_machine_with_ben()).is_the_greeter());
        }
        assert_eq!(seat, before);
        assert!(seat.is_locked());
        assert_eq!(seat.session(), before.session());

        let Unlocking::Unlocked { seat, held } = seat.unlocks(the_machine(), "anna", ANNAS) else {
            unreachable!("the right password did not unlock after the road was taken")
        };
        assert_eq!(held, vec![a_notification("Re: the Lisbon offer")]);
        assert_eq!(seat, Seat::opened(anna()));
    }

    /// **Nothing a person could read on the way names who was signed in or
    /// what was waiting.** Everything the road hands back, printed whole, says
    /// nothing of the session it came from.
    #[test]
    fn nothing_the_road_hands_back_names_the_session_it_came_from() {
        let mut seat = Seat::opened(anna()).locked(&mut Summoning::closed());
        for sent in ["Re: the Lisbon offer", "Your test results", "Anna, call me"] {
            assert_eq!(seat.arrives(a_notification(sent)), Arrived::Held);
        }
        let everything = format!("{:?}", seat.somebody_else(&the_machine_with_ben()));
        for private in ["anna", "Anna", "ben", "Lisbon", "test results", "1000"] {
            assert!(!everything.contains(private), "{everything}");
        }
    }
}
