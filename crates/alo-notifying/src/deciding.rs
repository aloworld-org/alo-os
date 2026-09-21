//! What becomes of a notification that arrives: shown, or held — and why.
//!
//! One function, and the order of its questions is the whole of it.
//!
//! 1. **Is the machine locked?** Asked first, and answered by task 1's own
//!    code: [`arrives`] hands the notification to `alo_locking::Seat::arrives`,
//!    which holds it and answers `alo_locking::Arrived::Held` — a value that
//!    carries **nothing back to draw**. Nothing in this crate can put a
//!    notification on a lock screen because nothing in this crate is ever
//!    handed one back to put there.
//! 2. **Is anything holding notifications?** [`crate::Quiet::now`] has already
//!    answered that, and its first question was whether the screen is being
//!    read. What is held goes on [`crate::Missed`] until the person dismisses
//!    it.
//! 3. Otherwise it is shown.
//!
//! # What comes back says why, and never what
//!
//! [`Became::Held`] carries a [`Why`] and no notification. That is the same
//! shape `alo_locking::Arrived::Held` has and it is the same argument: a value
//! that carried the notification would be a value a shell could draw, and the
//! sentence beside a held notification is read on the screen it was held off.
//! [`Why::said`] therefore names the reason and nothing else — no sender, no
//! title, no count.

use alo_locking::{Arrived, Seat};
use alo_strings::{Filling, Said, Strings, Word};

use crate::missed::Missed;
use crate::notification::Notification;
use crate::quiet::{Because, Quiet};
use crate::shown::Shown;
use crate::words;

/// Why a notification was held rather than shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// The machine is locked, and task 1 is holding it until the person is
    /// back.
    TheMachineIsLocked,
    /// Notifications are being held, for one of [`Because`]'s reasons.
    Quiet(Because),
}

impl Why {
    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::TheMachineIsLocked => words::HELD_LOCKED,
            Self::Quiet(because) => because.word(),
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// The reason, and **nothing about what was held**.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// What one arriving notification became.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Became {
    /// Shown, and here is what a shell draws.
    Shown(Shown),
    /// Held, and this is why — with nothing to draw.
    Held(Why),
}

impl Became {
    /// Whether it was shown.
    #[must_use]
    pub const fn is_shown(&self) -> bool {
        matches!(self, Self::Shown(_))
    }

    /// Why it was held, or [`None`] when it was shown.
    #[must_use]
    pub const fn why(&self) -> Option<Why> {
        match self {
            Self::Held(why) => Some(*why),
            Self::Shown(_) => None,
        }
    }

    /// What a shell draws, or [`None`] when nothing is drawn.
    #[must_use]
    pub const fn shown(&self) -> Option<&Shown> {
        match self {
            Self::Shown(shown) => Some(shown),
            Self::Held(_) => None,
        }
    }
}

/// A notification has arrived for this person.
///
/// `seat` is task 1's, and it is asked first: a locked machine holds the
/// notification behind the lock screen and hands it back at the unlock, and
/// this crate never sees it again until then. `quiet` is
/// [`crate::Quiet::now`]'s answer at this moment, and `missed` is the list
/// waiting for the person.
pub fn arrives(
    notification: Notification,
    seat: &mut Seat<Notification>,
    quiet: Quiet,
    missed: &mut Missed,
) -> Became {
    match seat.arrives(notification) {
        Arrived::Held => Became::Held(Why::TheMachineIsLocked),
        Arrived::ToShow(notification) => match quiet {
            Quiet::Yes(because) => {
                missed.keeps(notification);
                Became::Held(Why::Quiet(because))
            }
            Quiet::No => Became::Shown(Shown::of(notification)),
        },
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_notification_titled, an_open_seat, in_english, locked};

    /// **An open machine with nothing holding notifications shows them.**
    #[test]
    fn an_open_machine_with_nothing_holding_notifications_shows_them() {
        let mut seat = an_open_seat();
        let mut missed = Missed::nothing();
        let became = arrives(
            a_notification_titled("Anna Pärt"),
            &mut seat,
            Quiet::No,
            &mut missed,
        );
        assert!(became.is_shown());
        assert_eq!(
            became.shown().map(|shown| shown.notification().title()),
            Some("Anna Pärt")
        );
        assert!(missed.is_empty(), "a notification shown was not missed");
    }

    /// **A locked machine holds it, and hands nothing back to draw.** Task 1's
    /// rule, asked before anything else here.
    #[test]
    fn a_locked_machine_holds_it_and_hands_nothing_back_to_draw() {
        let mut seat = locked(an_open_seat());
        let mut missed = Missed::nothing();
        let became = arrives(
            a_notification_titled("Anna Pärt"),
            &mut seat,
            Quiet::No,
            &mut missed,
        );
        assert_eq!(became, Became::Held(Why::TheMachineIsLocked));
        assert_eq!(became.shown(), None);
        assert!(
            missed.is_empty(),
            "what the lock screen holds is task 1's until the unlock"
        );
    }

    /// **The lock is asked before do-not-disturb**, so an unlocked machine
    /// that is quiet and a locked machine that is not are told apart — and a
    /// locked machine says the one sentence the lock screen already says.
    #[test]
    fn the_lock_is_asked_before_anything_else() {
        let mut missed = Missed::nothing();
        for quiet in [
            Quiet::No,
            Quiet::Yes(Because::YouAskedForQuiet),
            Quiet::Yes(Because::TheScreenIsShared),
        ] {
            let mut seat = locked(an_open_seat());
            assert_eq!(
                arrives(
                    a_notification_titled("Anna Pärt"),
                    &mut seat,
                    quiet,
                    &mut missed
                ),
                Became::Held(Why::TheMachineIsLocked),
                "{quiet:?}"
            );
        }
    }

    /// **What do-not-disturb holds goes on the list**, with the reason said
    /// and nothing drawn.
    #[test]
    fn what_do_not_disturb_holds_goes_on_the_list() {
        let strings = in_english();
        for because in Because::EVERY {
            let mut seat = an_open_seat();
            let mut missed = Missed::nothing();
            let became = arrives(
                a_notification_titled("Anna Pärt"),
                &mut seat,
                Quiet::Yes(because),
                &mut missed,
            );
            assert_eq!(became, Became::Held(Why::Quiet(because)));
            assert_eq!(became.shown(), None);
            assert_eq!(missed.how_many(), 1, "{because:?}");

            let said = became.why().unwrap().said(&strings);
            assert!(!said.is_a_bug(), "{because:?}");
            assert!(!said.text().contains("Anna"), "{said} names what was held");
        }
    }

    /// **No sentence about a held notification names it.** The one thing a
    /// held notification may never do is appear, and a reason that quoted its
    /// sender or its title would be it appearing in another shape.
    #[test]
    fn no_sentence_about_a_held_notification_names_it() {
        let strings = in_english();
        let mut why = vec![Why::TheMachineIsLocked];
        why.extend(Because::EVERY.map(Why::Quiet));
        for one in why {
            let said = one.said(&strings);
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(
                one.word().phrase().unwrap().source().gaps().is_empty(),
                "{said} has a gap anything could be put into"
            );
        }
    }
}
