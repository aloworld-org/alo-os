//! Logging out: every application asked to close, in an order that lets each
//! save, and nothing killed.
//!
//! What a log-out is on most systems is a countdown and then a signal. What it
//! is here is a walk:
//!
//! 1. **The machine must not be locked.** Logging out is the signed-in person's
//!    act, like approving a change, and a stranger at a locked desk must not be
//!    able to throw away work that has not been saved. A locked seat is answered
//!    with `alo_locking::NotWhileLocked` — the lock screen's own sentence, and
//!    nothing about what is open.
//! 2. **Every application is asked to close, one at a time, last opened first**
//!    ([`crate::WasOpen::asked_to_close_in_order`]). One at a time because a
//!    save dialogue deserves the screen to itself; last first because a stack
//!    unwinds, and because the application a person was just in is the one they
//!    expect to be asked about first.
//! 3. **Every one of them is asked, even after one has refused.** A log-out that
//!    stopped at the first refusal would leave a person answering one dialogue,
//!    then another, then another; this asks them all and comes back with one
//!    list.
//! 4. **If any stayed, the log-out stops and names them** ([`crate::WouldNotClose`]).
//!    Nothing is killed. The only road on is the person's, having read the names.
//! 5. **If every one closed, the session may end** — which is a
//!    [`crate::MayEnd`] and nothing else, because ending it is not this crate's
//!    to do.
//!
//! # What this crate never touches
//!
//! **An application's own session files.** Whether a text editor reopens the
//! document it had is the editor's business, under its own grants, and nothing
//! here reads or writes a file belonging to one.
//!
//! **A process.** There is no signal, no timeout that becomes a kill, and no
//! path from this module to one. See `crate::refusing`.

use alo_applications::Application;
use alo_locking::{NotWhileLocked, Seat};

use crate::ending::MayEnd;
use crate::open::WasOpen;
use crate::refusing::WouldNotClose;

/// The applications of this session, as something that can be asked to close.
///
/// Implemented by whatever actually reaches a window — the compositor. Named
/// `TheApplications` the way `alo_sessiond::TheAccounts` is: it is *this
/// machine's*, one method wide, and a test stands in for it.
pub trait TheApplications {
    /// Ask this application to close, and wait for its answer.
    ///
    /// The application saves what it has and goes, or it stays — because it has
    /// work the person has not saved, or because it did not answer. Either way
    /// it is one call and one answer: the next application is not asked until
    /// this one has replied, which is what *an order that lets each save* means.
    fn asked_to_close(&mut self, what: &Application) -> Closing;
}

/// What an application did when it was asked to close.
///
/// Two arms, and deliberately no third: there is no *killed*, because nothing
/// here kills anything.
///
/// ```compile_fail
/// let killed = alo_leaving::Closing::Killed;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closing {
    /// It saved what it had and closed.
    Closed,
    /// It is still open.
    StillOpen,
}

/// What came of asking to log out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoggingOut {
    /// Every application closed, so the session may end.
    Ready(MayEnd),
    /// Some would not close. Nothing was killed, and these are their names.
    SomeWouldNotClose(WouldNotClose),
    /// The machine is locked, so nothing was asked and nothing closed.
    NotWhileLocked(NotWhileLocked),
}

/// A person chose *Log out*.
///
/// `was_open` is what this session has open, which is also what would be kept
/// for the next sign-in ([`crate::keeping::at_sign_out`]) — one list, so that
/// what is asked to close and what is written down cannot disagree.
///
/// Nothing is written and nothing is ended here: this answers what happened to
/// the applications, and the two things that follow from it are the caller's.
pub fn asked<N>(
    seat: &Seat<N>,
    was_open: &WasOpen,
    applications: &mut impl TheApplications,
) -> LoggingOut {
    if seat.is_locked() {
        return LoggingOut::NotWhileLocked(NotWhileLocked);
    }
    let mut closed = Vec::new();
    let mut would_not = Vec::new();
    for what in was_open.asked_to_close_in_order() {
        match applications.asked_to_close(what) {
            Closing::Closed => closed.push(what.clone()),
            Closing::StillOpen => would_not.push(what.clone()),
        }
    }
    if would_not.is_empty() {
        LoggingOut::Ready(MayEnd::everything_closed())
    } else {
        LoggingOut::SomeWouldNotClose(WouldNotClose::of(would_not, closed))
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
    use crate::ending::AfterWhat;
    use crate::split::Split;
    use crate::testing::{a_desk, anna, the_editor};

    /// A compositor that says yes to everything, and remembers the order it was
    /// asked in.
    #[derive(Default)]
    struct Agreeable {
        /// What it was asked to close, in order.
        asked: Vec<String>,
    }

    impl TheApplications for Agreeable {
        fn asked_to_close(&mut self, what: &Application) -> Closing {
            self.asked.push(what.identifier().to_owned());
            Closing::Closed
        }
    }

    /// One that says no for one named application, and yes for the rest.
    struct AllBut {
        /// The one that stays.
        stays: &'static str,
        /// What it was asked to close, in order.
        asked: Vec<String>,
    }

    impl TheApplications for AllBut {
        fn asked_to_close(&mut self, what: &Application) -> Closing {
            self.asked.push(what.identifier().to_owned());
            if what.identifier() == self.stays {
                Closing::StillOpen
            } else {
                Closing::Closed
            }
        }
    }

    /// **Every application is asked, last opened first, and then the session may
    /// end.**
    #[test]
    fn everything_closes_and_then_the_session_may_end() {
        let seat = Seat::<String>::opened(anna());
        let mut applications = Agreeable::default();
        let LoggingOut::Ready(may_end) = asked(&seat, &a_desk(), &mut applications) else {
            unreachable!("a session where everything closed did not become ready")
        };
        assert_eq!(may_end.after(), AfterWhat::EverythingClosed);
        assert_eq!(
            applications.asked,
            vec![the_editor().to_owned(), "org.example.Mail".to_owned()],
            "the editor was opened last and is asked first"
        );
    }

    /// **One that will not close stops the log-out and is named** — and the
    /// others were still asked, so the person gets one list rather than one
    /// dialogue after another.
    #[test]
    fn one_that_will_not_close_is_named_and_the_rest_are_still_asked() {
        let seat = Seat::<String>::opened(anna());
        let mut applications = AllBut {
            stays: the_editor(),
            asked: Vec::new(),
        };
        let LoggingOut::SomeWouldNotClose(refused) = asked(&seat, &a_desk(), &mut applications)
        else {
            unreachable!("an application that stayed did not stop the log-out")
        };
        assert_eq!(applications.asked.len(), 2, "both were asked");
        assert_eq!(
            refused
                .would_not_close()
                .iter()
                .map(|it| it.identifier())
                .collect::<Vec<&str>>(),
            vec![the_editor()]
        );
        assert_eq!(
            refused
                .already_closed()
                .iter()
                .map(|it| it.identifier())
                .collect::<Vec<&str>>(),
            vec!["org.example.Mail"]
        );
    }

    /// **A locked machine asks nothing at all.** Not the first application, not
    /// any of them: a stranger at the desk cannot throw away work by choosing
    /// *Log out*.
    #[test]
    fn a_locked_machine_asks_nothing_to_close() {
        let seat = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
        let mut applications = Agreeable::default();
        assert_eq!(
            asked(&seat, &a_desk(), &mut applications),
            LoggingOut::NotWhileLocked(NotWhileLocked)
        );
        assert!(applications.asked.is_empty(), "something was asked anyway");
    }

    /// **A session with nothing open may end at once**, without a list and
    /// without a question.
    #[test]
    fn a_session_with_nothing_open_may_end_at_once() {
        let seat = Seat::<String>::opened(anna());
        let mut applications = Agreeable::default();
        assert_eq!(
            asked(&seat, &WasOpen::nothing(), &mut applications),
            LoggingOut::Ready(MayEnd::everything_closed())
        );
        assert!(applications.asked.is_empty());
    }

    /// **Two windows of one application are one question.** Nobody is asked
    /// twice to close the same thing.
    #[test]
    fn two_windows_of_one_application_are_one_question() {
        let seat = Seat::<String>::opened(anna());
        let mut applications = Agreeable::default();
        let twice = WasOpen::nothing()
            .and(
                crate::open::Open::of(
                    the_editor(),
                    crate::testing::the_laptop(),
                    Split::TheWholeScreen,
                )
                .unwrap(),
            )
            .and(
                crate::open::Open::of(
                    the_editor(),
                    crate::testing::the_screen(),
                    Split::TheWholeScreen,
                )
                .unwrap(),
            );
        assert!(matches!(
            asked(&seat, &twice, &mut applications),
            LoggingOut::Ready(_)
        ));
        assert_eq!(applications.asked, vec![the_editor().to_owned()]);
    }
}
