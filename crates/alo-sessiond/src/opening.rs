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
//!    file the surface authenticated against, and before systemd is asked
//!    anything at all.
//! 3. **Is somebody signed in already?** A second session nobody asked for is a
//!    machine with two of everything. **Asked of systemd, not remembered**:
//!    signing out is the desktop ending, so a desktop this component started
//!    that has since ended is nobody signed in, and the next person may.
//! 4. **Their place for what an undo would put back, then their own name**,
//!    each of which only a privileged
//!    process can make ([ADR 0121](../../../docs/decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)).
//!    After 3, so a knock nobody asked for causes no work; before 5, because
//!    the session is what writes into the folder. **The only step whose failure
//!    is not a refusal** — `crate::their_folder` says why, at length, and
//!    `CLAUDE.md` law 5 is the reason that decides it.
//!    The name ([ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md))
//!    is placed the same way for the same reasons: after 3, and before 5
//!    because the session is what starts the desktop that reads it. Its
//!    failure refuses nobody either (`crate::their_name`).
//! 5. **Then, and only then, start their desktop**, whose own unit opens the
//!    session with the desktop inside it ([ADR 0141](../../../docs/decisions/0141-the-desktop-is-put-inside-the-persons-session-by-what-already-has-the-privilege.md)).
//!
//! Reversing any two of 1, 2, 3 and 5 would give something away. Asking systemd first
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
use crate::their_desktop::TheirDesktop;
use crate::their_folder::TheirFolder;
use crate::their_name::TheirName;
use crate::words::{ALREADY_SIGNED_IN, NOBODY_HERE, NOT_OPENED, NOT_YOURS_TO_ASK};

/// The decision this privileged component makes, and the only one it makes.
#[derive(Debug)]
pub struct Opening<A: TheAccounts, L: TheirDesktop, F: TheirFolder, N: TheirName> {
    /// Who may knock.
    door: Door,
    /// The accounts this machine has.
    accounts: A,
    /// What starts a person's desktop, and says whether it is still running.
    desktop: L,
    /// What makes a person their place for what an undo would put back.
    folders: F,
    /// What gives a person their own name, where their desktop can read it.
    names: N,
    /// Whose desktop this component last started, which is the whole of its
    /// memory. Whether that desktop is still running is asked of the machine at
    /// the next knock, never assumed.
    started: Option<u32>,
}

impl<A: TheAccounts, L: TheirDesktop, F: TheirFolder, N: TheirName> Opening<A, L, F, N> {
    /// This door, these accounts, this way of starting a desktop, this way of
    /// making a person their folder, and this way of giving them their name.
    pub const fn at(door: Door, accounts: A, desktop: L, folders: F, names: N) -> Self {
        Self {
            door,
            accounts,
            desktop,
            folders,
            names,
            started: None,
        }
    }

    /// Whether this component has started a desktop it has not since seen end.
    #[must_use]
    pub const fn is_signed_in(&self) -> bool {
        self.started.is_some()
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
        // **Somebody signed in is a desktop still running**, asked of the
        // machine. A desktop that ended is somebody who signed out, and the
        // next correct password is let in rather than told *already signed in*
        // for ever. A machine that cannot say is not assumed either way: it is
        // refused as the machine's failure, because starting a second desktop
        // beside one that may still hold the seat is what `logind` refuses.
        if let Some(who) = self.started {
            match self.desktop.is_running_for(who) {
                Ok(true) => return Answered::Refused(ALREADY_SIGNED_IN.key()),
                Ok(false) => self.started = None,
                Err(why) => {
                    eprintln!("alo-sessiond: {why}");
                    return Answered::Refused(NOT_OPENED.key());
                }
            }
        }

        // **Their folder, between the two, and a failure here refuses nobody.**
        //
        // Here rather than after, because the session is what writes into it and
        // a session that opens first would have a window in which it could not.
        // Here rather than before the check above, because a second knock
        // nobody asked for should not reach the machine at all.
        //
        // `crate::their_folder` carries the three reasons the person signs in
        // anyway, and `CLAUDE.md` law 5 is the one that decides: refusing the
        // sign-in would take the machine away from its owner to protect a
        // feature nothing yet writes to. The journal is where this goes, and
        // `a_folder_that_was_not_made_refuses_nobody` is what holds it.
        if let Err(why) = self.folders.made_for(knock.person()) {
            eprintln!("alo-sessiond: {why}");
        }

        // **Their name, after their folder and for the same reasons.** Before
        // the desktop, because the desktop is what reads it; and a failure
        // refuses nobody, because what it costs is a word in a strip
        // (`crate::their_name`). `a_name_that_was_not_given_refuses_nobody` is
        // what holds it.
        if let Err(why) = self.names.given_to(knock.person()) {
            eprintln!("alo-sessiond: {why}");
        }

        match self.desktop.started_for(knock.person()) {
            Ok(()) => {
                self.started = Some(knock.person());
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
    use std::cell::{Cell, RefCell};

    use alo_strings::Key;

    use crate::refusing::{NotAsked, NotOpened, NotTheirs};
    use crate::words::Word;

    /// A way of making a folder that always manages it.
    ///
    /// The default in every test but the one about a failure, because *the
    /// folder was made* is the uninteresting case and a test that had to say so
    /// each time would bury the one that matters.
    #[derive(Debug, Clone, Copy, Default)]
    struct Giving;

    impl TheirFolder for Giving {
        fn made_for(&self, _person: u32) -> Result<(), NotTheirs> {
            Ok(())
        }
    }

    /// A way of making a folder that counts how often it was asked.
    ///
    /// `Rc<RefCell<_>>` rather than a field, because the trait takes `&self` —
    /// which is itself the honest signature, since making a directory leaves
    /// this value holding nothing. A test that needed `&mut self` to count
    /// would be a test asking for a worse signature.
    #[derive(Debug, Clone, Default)]
    struct Counting(std::rc::Rc<RefCell<usize>>);

    impl Counting {
        fn how_many(&self) -> usize {
            *self.0.borrow()
        }
    }

    impl TheirFolder for Counting {
        fn made_for(&self, _person: u32) -> Result<(), NotTheirs> {
            *self.0.borrow_mut() += 1;
            Ok(())
        }
    }

    /// A way of making a folder that never manages it.
    #[derive(Debug, Clone, Copy, Default)]
    struct NotGiving;

    impl TheirFolder for NotGiving {
        fn made_for(&self, person: u32) -> Result<(), NotTheirs> {
            Err(NotTheirs::NotFinished {
                person,
                unit: "alo-giving-a-folder@test.service".to_owned(),
                said: "a test said so".to_owned(),
            })
        }
    }

    /// A way of giving a name that always manages it — the uninteresting case,
    /// for [`Giving`]'s reason.
    #[derive(Debug, Clone, Copy, Default)]
    struct Naming;

    impl TheirName for Naming {
        fn given_to(&self, _person: u32) -> Result<(), NotTheirs> {
            Ok(())
        }
    }

    /// A way of giving a name that never manages it.
    #[derive(Debug, Clone, Copy, Default)]
    struct NotNaming;

    impl TheirName for NotNaming {
        fn given_to(&self, person: u32) -> Result<(), NotTheirs> {
            Err(NotTheirs::NotFinished {
                person,
                unit: "alo-giving-a-name@test.service".to_owned(),
                said: "a test said so".to_owned(),
            })
        }
    }

    /// [`Counting`] counts names as well, so one value can be handed as either.
    impl TheirName for Counting {
        fn given_to(&self, _person: u32) -> Result<(), NotTheirs> {
            *self.0.borrow_mut() += 1;
            Ok(())
        }
    }

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

    /// A machine that starts whatever desktop it is asked for, and remembers.
    ///
    /// The second field is whether that desktop has since ended — the person
    /// signing out — which a test sets to say so.
    #[derive(Default)]
    struct Willing(RefCell<Vec<u32>>, Cell<bool>);

    impl TheirDesktop for Willing {
        fn started_for(&mut self, person: u32) -> Result<(), NotOpened> {
            self.0.borrow_mut().push(person);
            self.1.set(false);
            Ok(())
        }

        fn is_running_for(&self, person: u32) -> Result<bool, NotOpened> {
            Ok(self.0.borrow().contains(&person) && !self.1.get())
        }
    }

    /// A machine that refuses, the way an unprivileged caller is refused.
    #[derive(Default)]
    struct Unwilling(RefCell<Vec<u32>>);

    impl TheirDesktop for Unwilling {
        fn started_for(&mut self, person: u32) -> Result<(), NotOpened> {
            self.0.borrow_mut().push(person);
            Err(NotOpened::Refused {
                person,
                named: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
                why: "Access denied".to_owned(),
            })
        }

        fn is_running_for(&self, _person: u32) -> Result<bool, NotOpened> {
            Ok(false)
        }
    }

    /// A machine that starts a desktop and then cannot say whether it runs.
    #[derive(Default)]
    struct Unanswering(RefCell<Vec<u32>>);

    impl TheirDesktop for Unanswering {
        fn started_for(&mut self, person: u32) -> Result<(), NotOpened> {
            self.0.borrow_mut().push(person);
            Ok(())
        }

        fn is_running_for(&self, _person: u32) -> Result<bool, NotOpened> {
            Err(NotOpened::NoBus {
                why: "the bus went away".to_owned(),
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
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            Naming,
        );

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
        let desktop = Willing::default();
        let mut opening = Opening::at(the_door(), Unreadable, desktop, Giving, Naming);

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
            opening.desktop.0.borrow().is_empty(),
            "logind was asked about a caller that may not ask"
        );
    }

    /// **A number this machine has no account for opens nothing**, and
    /// `logind` is never asked — which is what makes *it cannot be asked to
    /// open a session for a number the caller has not authenticated* true
    /// across a process boundary rather than only inside one.
    #[test]
    fn a_number_this_machine_has_no_account_for_is_refused() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            Naming,
        );

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
            opening.desktop.0.borrow().is_empty(),
            "logind was asked about somebody this machine has no account for"
        );
    }

    /// **Root is not a special case.** A machine whose accounts file happened
    /// to have no root account refuses root exactly as it refuses anybody else
    /// — there is no branch here that treats a small number differently.
    #[test]
    fn root_is_refused_like_anybody_else_this_machine_has_no_account_for() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            Naming,
        );
        assert_eq!(
            opening.heard(Knock::on_behalf_of(0), THE_GREETER),
            Answered::Refused(NOBODY_HERE.key())
        );
    }

    /// **A second knock while a session is open opens nothing**, and says so
    /// in the one sentence here a person can act on.
    #[test]
    fn a_second_knock_while_somebody_is_signed_in_is_refused() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            Naming,
        );
        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );

        let again = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(again, Answered::Refused(ALREADY_SIGNED_IN.key()));
        assert_eq!(
            opening.desktop.0.borrow().len(),
            1,
            "a second session was opened"
        );
    }

    /// **A person who signed out can sign in again.**
    ///
    /// Signing out is the desktop ending, and this component is not told. It
    /// used to keep a flag that was set at the first sign-in and never cleared,
    /// so the second correct password of a day would have been answered
    /// *already signed in*. It asks the machine instead.
    #[test]
    fn a_person_who_signed_out_can_sign_in_again() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            Naming,
        );
        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );

        // The desktop ends: the person signed out.
        opening.desktop.1.set(true);

        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened,
            "a person who signed out was told they are still signed in"
        );
        assert_eq!(*opening.desktop.0.borrow(), vec![THE_PERSON, THE_PERSON]);
        assert!(opening.is_signed_in());
    }

    /// **A machine that cannot say whether the last desktop still runs starts
    /// no second one.** Refused as the machine's failure, never guessed: a
    /// second desktop beside one that may still hold the seat is one `logind`
    /// would refuse, after the person had been told they were in.
    #[test]
    fn a_machine_that_cannot_say_whether_a_desktop_runs_starts_no_second_one() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Unanswering::default(),
            Giving,
            Naming,
        );
        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );

        let again = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(again, Answered::Refused(NOT_OPENED.key()));
        assert_eq!(
            opening.desktop.0.borrow().len(),
            1,
            "a second desktop was started"
        );
    }

    /// **A machine that would not open a session is told as a machine, not as a
    /// password.** The person typed the right thing and the sentence they get
    /// says so.
    #[test]
    fn a_machine_that_refuses_is_answered_with_the_machines_sentence() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Unwilling::default(),
            Giving,
            Naming,
        );

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
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Unwilling::default(),
            Giving,
            Naming,
        );
        drop(opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER));

        let again = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(again, Answered::Refused(NOT_OPENED.key()));
        assert_eq!(opening.desktop.0.borrow().len(), 2, "it stopped asking");
    }

    /// **An accounts file that will not read is not a machine with no
    /// accounts.** It is answered as the machine's own failure, `logind` is not
    /// asked, and nobody is told they do not exist.
    #[test]
    fn accounts_that_will_not_read_are_the_machines_failure_and_not_a_persons() {
        let mut opening = Opening::at(the_door(), Unreadable, Willing::default(), Giving, Naming);

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(answered, Answered::Refused(NOT_OPENED.key()));
        assert!(opening.desktop.0.borrow().is_empty());
    }

    /// Every answer this file can give is one of `crate::words`, so there is no
    /// road out of here that ends in a key nothing declares.
    #[test]
    fn every_answer_is_a_sentence_this_crate_declares() {
        let ours: Vec<Key> = crate::EVERY_WORD.iter().map(Word::key).collect();
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Unwilling::default(),
            Giving,
            Naming,
        );

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

    /// **A folder that could not be made refuses nobody**, which is the one
    /// decision in this file that is not a refusal.
    ///
    /// `crate::their_folder` carries the three reasons, and `CLAUDE.md` law 5 is
    /// the one that decides: a sign-in turned away over a directory for a
    /// feature nothing yet writes to would take the machine away from its owner
    /// to protect a feature. **This is the test that keeps that true**, because
    /// the obvious way to write the call site — `?`, or a match that returns a
    /// refusal — is one character from being the opposite.
    #[test]
    fn a_folder_that_was_not_made_refuses_nobody() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            NotGiving,
            Naming,
        );

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(
            answered,
            Answered::Opened,
            "a person was turned away because their undo folder could not be made"
        );
        assert!(
            opening.is_signed_in(),
            "the session was not opened, so the failure did more than it was allowed to"
        );
    }

    /// **And the folder is asked for exactly once per knock that gets that far.**
    ///
    /// One approval, one execution, in the shape this component has it: a retry
    /// loop or a second call site would mean a person signing in once causing
    /// two privileged acts, and the unit is `Type=oneshot` so a second start is
    /// a second job on the machine.
    #[test]
    fn the_folder_is_asked_for_once_for_one_knock() {
        let counted = Counting::default();
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            counted.clone(),
            Naming,
        );

        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );
        assert_eq!(
            counted.how_many(),
            1,
            "one knock asked for {} folders",
            counted.how_many()
        );
    }

    /// **A knock that is refused before the folder does not ask for one.**
    ///
    /// The order is the design, and this is the half of it the new step added:
    /// a caller that may not knock, and a number nobody here has, must cause no
    /// privileged act at all. A component that made a directory for every number
    /// anybody asked about would be answering *is there an account numbered
    /// this* on the disk.
    #[test]
    fn a_refused_knock_asks_for_no_folder() {
        for (knocked_by, person, what) in [
            (THE_PERSON, THE_PERSON, "a caller who may not knock"),
            (THE_GREETER, THE_PERSON + 7, "a number nobody here has"),
        ] {
            let counted = Counting::default();
            let mut opening = Opening::at(
                the_door(),
                Numbered(vec![THE_PERSON]),
                Willing::default(),
                counted.clone(),
                Naming,
            );

            let answered = opening.heard(Knock::on_behalf_of(person), knocked_by);

            assert!(
                matches!(answered, Answered::Refused(_)),
                "{what} was not refused"
            );
            assert_eq!(
                counted.how_many(),
                0,
                "{what} caused a folder to be asked for"
            );
        }
    }

    /// **And a second knock, once somebody is signed in, asks for no folder
    /// either.**
    ///
    /// The reason the step sits after the `signed_in` check rather than before
    /// it. Written as its own test because the two orderings are both plausible
    /// and only one of them is what this file does.
    #[test]
    fn a_second_knock_asks_for_no_second_folder() {
        let counted = Counting::default();
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            counted.clone(),
            Naming,
        );

        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );
        assert!(matches!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Refused(_)
        ));
        assert_eq!(
            counted.how_many(),
            1,
            "two knocks asked for {} folders",
            counted.how_many()
        );
    }
    /// **A name that could not be given refuses nobody.**
    ///
    /// The cost of failing is a word: the person's terminal says `Local` alone
    /// rather than `Local · anna`. Turning them away over it would take their
    /// machine away to protect a label — `CLAUDE.md` law 5, and
    /// `crate::their_name` at length.
    #[test]
    fn a_name_that_was_not_given_refuses_nobody() {
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            NotNaming,
        );

        let answered = opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER);

        assert_eq!(
            answered,
            Answered::Opened,
            "a person was turned away because their name could not be given"
        );
        assert!(opening.is_signed_in());
    }

    /// **The name is asked for once per sign-in, and never for a refused
    /// knock** — the folder's rule, held for the second unit separately, so a
    /// later change that moved one call and not the other fails here.
    #[test]
    fn the_name_is_asked_for_once_and_never_for_a_refused_knock() {
        let counted = Counting::default();
        let mut opening = Opening::at(
            the_door(),
            Numbered(vec![THE_PERSON]),
            Willing::default(),
            Giving,
            counted.clone(),
        );

        // A caller who may not knock, and a number nobody here has.
        drop(opening.heard(Knock::on_behalf_of(THE_PERSON), THE_PERSON));
        drop(opening.heard(Knock::on_behalf_of(THE_PERSON + 7), THE_GREETER));
        assert_eq!(counted.how_many(), 0, "a refused knock asked for a name");

        assert_eq!(
            opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER),
            Answered::Opened
        );
        // And a second knock once somebody is signed in.
        drop(opening.heard(Knock::on_behalf_of(THE_PERSON), THE_GREETER));
        assert_eq!(
            counted.how_many(),
            1,
            "one sign-in asked for {} names",
            counted.how_many()
        );
    }
}
