//! Logging out, walked end to end: every application asked in an order that
//! lets each save, whatever stayed named, and the session ending only when
//! somebody has said so.
//!
//! The crate's own unit tests hold each clause where it is decided. This walks
//! the whole thing the way a shell will: a real sign-in under the seat, a
//! compositor that answers for each application, and the two roads out of a
//! refusal — back to the desktop, or the person insisting having read the names.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_accounts::{Accounts, Session};
use alo_appearance::DisplayId;
use alo_applications::Application;
use alo_leaving::{AfterWhat, Closing, LoggingOut, Open, Split, TheApplications, WasOpen};
use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_strings::{Strings, Vocabulary};

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "a password nobody else knows";

/// Anna's session, as a sign-in really opens one.
fn anna() -> Session {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    let signed_in = accounts.signs_in("anna", ANNAS).unwrap();
    Session::opened(signed_in, PERSON).unwrap()
}

/// This crate's words, in English.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_leaving::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The laptop's own screen.
fn the_laptop() -> DisplayId {
    DisplayId::named("eDP-1 Built-in display").unwrap()
}

/// What Anna has open: a file manager, then mail, then the editor she opened a
/// document in from the file manager.
fn a_morning() -> WasOpen {
    WasOpen::nothing()
        .and(Open::of("org.example.Files", the_laptop(), Split::TheWholeScreen).unwrap())
        .and(
            Open::of(
                "org.example.Mail",
                the_laptop(),
                Split::Of(alo_dividing::Place::LeftHalf),
            )
            .unwrap(),
        )
        .and(
            Open::of(
                "org.example.Editor",
                the_laptop(),
                Split::Of(alo_dividing::Place::RightHalf),
            )
            .unwrap(),
        )
}

/// A compositor that answers for each application, and remembers the order.
struct TheDesk {
    /// The identifiers that will not close.
    staying: Vec<&'static str>,
    /// What it was asked to close, in order.
    asked: Vec<String>,
}

impl TheDesk {
    /// A desk where everything closes.
    fn agreeable() -> Self {
        Self {
            staying: Vec::new(),
            asked: Vec::new(),
        }
    }

    /// A desk where these will not.
    fn where_these_stay(staying: &[&'static str]) -> Self {
        Self {
            staying: staying.to_vec(),
            asked: Vec::new(),
        }
    }
}

impl TheApplications for TheDesk {
    fn asked_to_close(&mut self, what: &Application) -> Closing {
        self.asked.push(what.identifier().to_owned());
        if self.staying.contains(&what.identifier()) {
            Closing::StillOpen
        } else {
            Closing::Closed
        }
    }
}

/// **A log-out asks every application, last opened first, and then the session
/// may end** — and the order is the one that lets each save: the editor a
/// document was opened in is asked while the file manager it came from is still
/// running.
#[test]
fn every_application_is_asked_last_first_and_then_the_session_may_end() {
    let seat = Seat::<String>::opened(anna());
    let mut desk = TheDesk::agreeable();
    let LoggingOut::Ready(may_end) =
        alo_leaving::logging_out::asked(&seat, &a_morning(), &mut desk)
    else {
        panic!("a desk where everything closed did not become ready")
    };
    assert_eq!(may_end.after(), AfterWhat::EverythingClosed);
    assert_eq!(
        desk.asked,
        vec![
            "org.example.Editor".to_owned(),
            "org.example.Mail".to_owned(),
            "org.example.Files".to_owned(),
        ]
    );
}

/// **Whatever would not close is named, one sentence each, and nothing is
/// killed** — and the person is told that the others have already gone, because
/// they have.
#[test]
fn whatever_would_not_close_is_named_and_nothing_is_killed() {
    let seat = Seat::<String>::opened(anna());
    let mut desk = TheDesk::where_these_stay(&["org.example.Editor", "org.example.Mail"]);
    let LoggingOut::SomeWouldNotClose(refused) =
        alo_leaving::logging_out::asked(&seat, &a_morning(), &mut desk)
    else {
        panic!("a desk with two applications still open became ready to end")
    };
    assert_eq!(desk.asked.len(), 3, "every one was still asked");

    let named: Vec<&str> = refused
        .would_not_close()
        .iter()
        .map(Application::identifier)
        .collect();
    assert_eq!(named, vec!["org.example.Editor", "org.example.Mail"]);
    assert_eq!(
        refused
            .already_closed()
            .iter()
            .map(Application::identifier)
            .collect::<Vec<&str>>(),
        vec!["org.example.Files"]
    );

    let strings = in_english();
    let said = refused.said(&strings);
    assert_eq!(said.len(), 2, "one sentence per application");
    for (sentence, what) in said.iter().zip(named.iter()) {
        assert!(!sentence.is_a_bug(), "{sentence}");
        assert!(sentence.unfilled().is_empty(), "{sentence}");
        assert!(sentence.text().contains(what), "{sentence}");
    }
    assert!(
        refused.said_about_what_closed(&strings).is_some(),
        "the file manager went and nobody was told"
    );
}

/// **A person who does not insist ends nothing.** The refusal is a value: drop
/// it, and there is no way left to say the session may end.
#[test]
fn a_person_who_does_not_insist_ends_nothing() {
    let seat = Seat::<String>::opened(anna());
    let mut desk = TheDesk::where_these_stay(&["org.example.Editor"]);
    let answered = alo_leaving::logging_out::asked(&seat, &a_morning(), &mut desk);
    assert!(matches!(answered, LoggingOut::SomeWouldNotClose(_)));
    // The one road on is `even_so`, and the value carrying it is dropped here.
    drop(answered);
    assert_eq!(desk.asked.len(), 3);
}

/// **A person who insists, having read the names, may end the session** — and
/// what happened is written into the value, so *the person insisted* can be told
/// apart from an ordinary log-out afterwards.
#[test]
fn a_person_who_insists_having_read_the_names_may_end_the_session() {
    let seat = Seat::<String>::opened(anna());
    let mut desk = TheDesk::where_these_stay(&["org.example.Editor"]);
    let LoggingOut::SomeWouldNotClose(refused) =
        alo_leaving::logging_out::asked(&seat, &a_morning(), &mut desk)
    else {
        panic!("an application that stayed did not stop the log-out")
    };
    let strings = in_english();
    let insisting = alo_leaving::WouldNotClose::what_insisting_is_called(&strings);
    assert!(!insisting.is_a_bug(), "{insisting}");

    let may_end = refused.even_so();
    assert_eq!(may_end.after(), AfterWhat::ThePersonInsisted);
}

/// **A locked machine logs nobody out.** Nothing is asked, nothing closes, and
/// what comes back says only that the machine is locked — the lock screen's own
/// sentence, with nothing in it about what is open behind it.
#[test]
fn a_locked_machine_logs_nobody_out() {
    let locked = Seat::<String>::opened(anna()).locked(&mut Summoning::closed());
    let mut desk = TheDesk::agreeable();
    let LoggingOut::NotWhileLocked(refused) =
        alo_leaving::logging_out::asked(&locked, &a_morning(), &mut desk)
    else {
        panic!("a locked machine walked its applications")
    };
    assert!(
        desk.asked.is_empty(),
        "something was asked at a locked desk"
    );

    let mut vocabulary = Vocabulary::empty();
    alo_locking::declare_into(&mut vocabulary).unwrap();
    let said = refused.said(&Strings::of(vocabulary));
    assert_eq!(said.text(), "This machine is locked");
    assert!(
        !said.text().contains("org.example"),
        "a refusal named what is open: {said}"
    );
}
