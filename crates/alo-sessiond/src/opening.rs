//! The four answers a knock can get, and the order they are decided in.
//!
//! This is the whole of what the privileged component decides. Everything else
//! in this crate is a socket, a line, or somebody else's library; this is the
//! part that would be worth attacking, and it is forty lines of arithmetic on
//! numbers that runs on any host — which is the point. The refusals are tested
//! on the machine this repository is written on, in the gate, every time.
//!
//! # The order is the design
//!
//! 1. **Is this the sign-in surface?** Decided on what the kernel recorded
//!    about the process that connected, before its line is looked at. A caller
//!    that may not ask is told nothing about what it asked for.
//! 2. **Is there anybody here with that number?** Asked of the same accounts
//!    file the surface authenticated against, and before `logind` is asked
//!    anything at all.
//! 3. **Is somebody signed in already?** A second session nobody asked for is a
//!    machine with two of everything.
//! 4. **Then, and only then, open one.**
//!
//! Reversing any two of those would give something away. Asking `logind` first
//! would let an unauthorised caller start a session for a number and be refused
//! afterwards; asking about the accounts before asking who is knocking would
//! answer *is there an account numbered 1000* to whatever managed to connect.
//!
//! # A refusal is never silence
//!
//! Every one of the three refusals answers with a key, and the key names a
//! sentence in `crate::words`. That is `CLAUDE.md`'s *every execution and every
//! refusal leaves a record* in the shape this component can keep it: a caller
//! that is turned away is told it was turned away, and a person watching a
//! sign-in that did not finish is told which of the four things happened.

use crate::accounts::TheAccounts;
use crate::asking::{Answered, Knock};
use crate::door::Door;
use crate::logind::Logind;
use crate::words::{ALREADY_SIGNED_IN, NOBODY_HERE, NOT_OPENED, NOT_YOURS_TO_ASK};

/// The decision this privileged component makes, and the only one it makes.
#[derive(Debug)]
pub struct Opening<A: TheAccounts, L: Logind> {
    /// Who may knock.
    door: Door,
    /// The accounts this machine has.
    accounts: A,
    /// What opens a session, and what holds it open once it is.
    logind: L,
    /// Whether one is open, which is the whole of this component's memory.
    signed_in: bool,
}

impl<A: TheAccounts, L: Logind> Opening<A, L> {
    /// This door, these accounts, and this way of opening a session.
    pub const fn at(door: Door, accounts: A, logind: L) -> Self {
        Self {
            door,
            accounts,
            logind,
            signed_in: false,
        }
    }

    /// Whether a session is open.
    #[must_use]
    pub const fn is_signed_in(&self) -> bool {
        self.signed_in
    }

    /// What this knock, from a caller in this group, is answered with.
    ///
    /// `knocked_by` is the group the kernel recorded for the process that
    /// connected — `SO_PEERCRED`, which is not a field in a message and cannot
    /// be written by a caller. It is a parameter rather than something read
    /// here so that every refusal below can be shown happening on any host;
    /// `crate::listening` is what puts the kernel behind it.
    ///
    /// It answers rather than refusing, in the `Result` sense, because from
    /// here every outcome is something to say to somebody: this is the file
    /// where a refusal is a sentence rather than an error.
    pub fn heard(&mut self, knock: Knock, knocked_by: u32) -> Answered {
        if !self.door.may_knock(knocked_by) {
            return Answered::Refused(NOT_YOURS_TO_ASK.key());
        }
        match self.accounts.an_account_numbered(knock.person()) {
            Ok(true) => {}
            Ok(false) => return Answered::Refused(NOBODY_HERE.key()),
            Err(why) => {
                eprintln!("alo-sessiond: {why}");
                return Answered::Refused(NOT_OPENED.key());
            }
        }
        if self.signed_in {
            return Answered::Refused(ALREADY_SIGNED_IN.key());
        }
        match self.logind.open_a_session_for(knock.person()) {
            Ok(()) => {
                self.signed_in = true;
                Answered::Opened
            }
            Err(why) => {
                eprintln!("alo-sessiond: {why}");
                Answered::Refused(NOT_OPENED.key())
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    use alo_strings::Key;

    use crate::refusing::{NotAsked, NotOpened};
    use crate::words::Word;

    /// The greeter's group, on the image this repository ships.
    const THE_GREETER: u32 = 60990;

    /// The person's number, on the image this repository ships.
    const THE_PERSON: u32 = 1000;

    /// Accounts a test decides the answer for.
    struct Numbered(Vec<u32>);

    impl TheAccounts for Numbered {
        fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked> {
            Ok(self.0.contains(&person))
        }
    }

    /// An accounts file that will not read.
    struct Unreadable;

    impl TheAccounts for Unreadable {
        fn an_account_numbered(&self, _: u32) -> Result<bool, NotAsked> {
            Err(NotAsked::about("/etc/alo/accounts.toml", &"no such file"))
        }
    }

    /// A `logind` that opens whatever it is asked for, and remembers.
    #[derive(Default)]
    struct Willing(RefCell<Vec<u32>>);

    impl Logind for Willing {
        fn open_a_session_for(&mut self, person: u32) -> Result<(), NotOpened> {
            self.0.borrow_mut().push(person);
            Ok(())
        }
    }

    /// A `logind` that refuses, the way an unprivileged caller is refused.
    #[derive(Default)]
    struct Unwilling(RefCell<Vec<u32>>);

    impl Logind for Unwilling {
        fn open_a_session_for(&mut self, person: u32) -> Result<(), NotOpened> {
            self.0.borrow_mut().push(person);
            Err(NotOpened::Refused {
                person,
                named: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
                why: "Access denied".to_owned(),
            })
        }
    }

    /// The door this machine's opener has.
    fn the_door() -> Door {
        Door::handed_to(THE_GREETER)
    }

    /// The ordinary morning: the surface knocks for somebody it has just
    /// authenticated, and a session is open.
    #[test]
    fn the_surface_knocking_for_a_person_here_opens_a_session() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Willing::default());

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(answered, Answered::Opened);
        assert!(opening.is_signed_in());
    }

    /// **A caller that is not the surface is refused, and nothing is asked of
    /// anything.** The accounts file is not read and `logind` is not spoken to
    /// — a caller that may not ask learns nothing about what it asked for,
    /// including whether that number exists.
    #[test]
    fn a_caller_that_is_not_the_surface_is_refused_before_anything_is_asked() {
        let logind = Willing::default();
        let mut opening = Opening::at(the_door(), Unreadable, logind);

        for knocker in [0, THE_PERSON, 60989, THE_GREETER + 1] {
            let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), knocker);
            assert_eq!(
                answered,
                Answered::Refused(NOT_YOURS_TO_ASK.key()),
                "{knocker} was answered with something else"
            );
        }
        assert!(!opening.is_signed_in());
        assert!(
            opening.logind.0.borrow().is_empty(),
            "logind was asked about a caller that may not ask"
        );
    }

    /// **A number this machine has no account for opens nothing**, and
    /// `logind` is never asked — which is what makes *it cannot be asked to
    /// open a session for a number the caller has not authenticated* true
    /// across a process boundary rather than only inside one.
    #[test]
    fn a_number_this_machine_has_no_account_for_is_refused() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Willing::default());

        for nobody in [0, 1, 999, 1001, 60989, u32::MAX] {
            let answered = opening.heard(Knock::on_behalf_of(nobody), THE_GREETER);
            assert_eq!(
                answered,
                Answered::Refused(NOBODY_HERE.key()),
                "a session was opened for {nobody}"
            );
        }
        assert!(!opening.is_signed_in());
        assert!(
            opening.logind.0.borrow().is_empty(),
            "logind was asked about somebody this machine has no account for"
        );
    }

    /// **Root is not a special case.** A machine whose accounts file happened
    /// to have no root account refuses root exactly as it refuses anybody else
    /// — there is no branch here that treats a small number differently.
    #[test]
    fn root_is_refused_like_anybody_else_this_machine_has_no_account_for() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Willing::default());
        assert_eq!(
            opening.heard(Knock::on_behalf_of(0), THE_GREETER),
            Answered::Refused(NOBODY_HERE.key())
        );
    }

    /// **A second knock while a session is open opens nothing**, and says so
    /// in the one sentence here a person can act on.
    #[test]
    fn a_second_knock_while_somebody_is_signed_in_is_refused() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Willing::default());
        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );

        let again = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(again, Answered::Refused(ALREADY_SIGNED_IN.key()));
        assert_eq!(
            opening.logind.0.borrow().len(),
            1,
            "a second session was opened"
        );
    }

    /// **A machine that would not open a session is told as a machine, not as a
    /// password.** The person typed the right thing and the sentence they get
    /// says so.
    #[test]
    fn a_machine_that_refuses_is_answered_with_the_machines_sentence() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Unwilling::default());

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(answered, Answered::Refused(NOT_OPENED.key()));
        assert!(!opening.is_signed_in(), "a refused session was remembered");
    }

    /// **And a refused session leaves the machine able to try again.** The one
    /// thing that must not happen after `logind` says no is a component that
    /// believes somebody is signed in, because then the next correct password
    /// is answered with *already signed in* for ever.
    #[test]
    fn a_machine_that_refused_once_can_be_asked_again() {
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Unwilling::default());
        drop(opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER));

        let again = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(again, Answered::Refused(NOT_OPENED.key()));
        assert_eq!(opening.logind.0.borrow().len(), 2, "it stopped asking");
    }

    /// **An accounts file that will not read is not a machine with no
    /// accounts.** It is answered as the machine's own failure, `logind` is not
    /// asked, and nobody is told they do not exist.
    #[test]
    fn accounts_that_will_not_read_are_the_machines_failure_and_not_a_persons() {
        let mut opening = Opening::at(the_door(), Unreadable, Willing::default());

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(answered, Answered::Refused(NOT_OPENED.key()));
        assert!(opening.logind.0.borrow().is_empty());
    }

    /// Every answer this file can give is one of `crate::words`, so there is no
    /// road out of here that ends in a key nothing declares.
    #[test]
    fn every_answer_is_a_sentence_this_crate_declares() {
        let ours: Vec<Key> = crate::EVERY_WORD.iter().map(Word::key).collect();
        let mut opening = Opening::at(the_door(), Numbered(vec![THE_PERSON]), Unwilling::default());

        for (knock, knocker) in [
            (Knock::on_behalf_of(THE_PERSON), 0),
            (Knock::on_behalf_of(7), THE_GREETER),
            (Knock::on_behalf_of(THE_PERSON), THE_GREETER),
        ] {
            match opening.heard(knock, knocker) {
                Answered::Opened => panic!("this fixture opens nothing"),
                Answered::Refused(key) => assert!(ours.contains(&key), "{key} is nobody's"),
            }
        }
    }
}
