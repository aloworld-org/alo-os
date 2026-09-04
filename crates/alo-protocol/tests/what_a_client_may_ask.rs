//! A line off a socket, carried through a real turn onto a real disk.
//!
//! The unit tests beside each file say what a message is. These say that what
//! this crate hands back is what the rest of the workspace takes: the name and
//! the values go into `alo_capability::Verbs::call` through `alo-turn`, a file
//! in a real folder is really read and really renamed, and the record says what
//! happened.
//!
//! It is the test that stops this crate being a description of a protocol
//! rather than the protocol. Nothing here mocks anything: the folders are on
//! the disk, the verbs are the six `alo-files` declares, and the turn is the
//! one `alo-agentd` will hold.
//!
//! **The record is in memory**, which is `alo-turn`'s `Kept` for a `Record`.
//! Writing one to a real file is `alo-keeping`'s and is tested there.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grantee, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_protocol::{FromAPerson, FromAnAgent, NotUnderstood};
use alo_record::{Asking, Only, Record};
use alo_strings::Strings;
use alo_turn::{Machine, NotDone, Turning};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn and a grant last in these tests.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Every word a machine reading these messages has loaded.
fn everything_this_machine_says() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own with one file in it, both resolved.
///
/// Resolved because a grant is over a place, and on Windows a resolved path
/// carries a prefix the typed one does not — `docs/quirks.md` records it.
fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-protocol-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// Grants to `@files` over this folder, made at noon and lasting an hour.
fn granting(folder: &Path) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(folder.to_path_buf()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

/// The arrangement every test here needs: a machine that offers the six, a
/// grant over one folder with one file in it, and a turn under way.
fn on_a_machine<T>(
    what: &str,
    record: &mut Record,
    doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Path, &Path) -> T,
) -> T {
    let strings = everything_this_machine_says();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        record,
    )
    .unwrap();
    let mut grants = granting(&folder);
    let mut turning = Turning::beginning(
        Context::at_invocation(noon()),
        "@files",
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    doing(&mut turning, &mut grants, &folder, &invoice)
}

/// One line of JSON, as a client would put it on the socket.
fn a_message(asks: &str) -> String {
    format!(r#"{{"format":1,"asks":{asks}}}"#)
}

/// **A read arrives as a line and answers inside the turn.** The whole journey
/// in one test: bytes off a socket, a name and a value, the closed list, the
/// grants, a real folder, and an entry saying it ran.
#[test]
fn a_read_off_the_wire_is_carried_out_and_written_down() {
    let mut record = Record::default();
    on_a_machine("a-read", &mut record, |turning, grants, folder, _| {
        let line = a_message(&format!(
            r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
            folder.display().to_string().replace('\\', "\\\\")
        ));
        let asked = FromAnAgent::read(&line).unwrap();
        assert!(!asked.waits_for_a_person());

        let answer = turning
            .reading(asked.verb().unwrap(), &asked.given(), grants, noon())
            .unwrap();
        assert_eq!(answer.listed().unwrap().things().len(), 1);
    });

    assert_eq!(record.len(), 1);
    assert!(
        record
            .everything()
            .next()
            .is_some_and(|entry| entry.happened().ran())
    );
}

/// **A change proposed by an agent is answered by a person, over the other
/// door.** Two messages, from two sides, and the file only moves once the
/// second one arrives.
#[test]
fn a_change_is_proposed_on_one_door_and_approved_on_the_other() {
    let mut record = Record::default();
    let renamed = on_a_machine("a-change", &mut record, |turning, grants, _, invoice| {
        let line = a_message(&format!(
            r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
            invoice.display().to_string().replace('\\', "\\\\")
        ));
        let asked = FromAnAgent::read(&line).unwrap();
        assert!(asked.waits_for_a_person());
        let id = turning
            .proposing(
                asked.verb().unwrap(),
                &asked.given(),
                grants,
                hour(),
                noon(),
            )
            .unwrap();
        assert!(invoice.is_file(), "a proposal moved a file");

        // The person's screen sends the number they answered. A number off the
        // wire is not a handle: it is found among what is really waiting.
        let answered = FromAPerson::read(&a_message(&format!(
            r#"{{"approve":{{"number":{}}}}}"#,
            id.as_u64()
        )))
        .unwrap();
        assert!(answered.is_yes());
        let waiting = turning
            .waiting_at(noon())
            .find(|waiting| Some(waiting.id.as_u64()) == answered.number())
            .map(|waiting| waiting.id)
            .unwrap();

        turning
            .approving(waiting, grants, noon())
            .unwrap()
            .now_at()
            .unwrap()
            .to_path_buf()
    });

    assert!(renamed.ends_with("march-final.pdf"));
    assert!(renamed.is_file(), "the file did not move on the disk");
    assert_eq!(record.len(), 1);
}

/// **An agent cannot approve its own change**, walked all the way through: the
/// proposal is made, the approval arrives on the agent's door, it is refused,
/// and the file is still where it was.
#[test]
fn an_agent_that_answers_its_own_proposal_moves_nothing() {
    let mut record = Record::default();
    let still_there = on_a_machine(
        "self-approval",
        &mut record,
        |turning, grants, _, invoice| {
            let propose = a_message(&format!(
                r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"gone.pdf"}}]}}}}"#,
                invoice.display().to_string().replace('\\', "\\\\")
            ));
            let asked = FromAnAgent::read(&propose).unwrap();
            let id = turning
                .proposing(
                    asked.verb().unwrap(),
                    &asked.given(),
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();

            let itself = FromAnAgent::read(&a_message(&format!(
                r#"{{"approve":{{"number":{}}}}}"#,
                id.as_u64()
            )));
            assert_eq!(itself, Err(NotUnderstood::NotForAnAgent));
            assert!(
                itself
                    .unwrap_err()
                    .said(&everything_this_machine_says())
                    .text()
                    .contains("cannot answer a question that was put to a person")
            );

            assert_eq!(
                turning.waiting_at(noon()).count(),
                1,
                "the change stopped waiting for the person"
            );
            invoice.is_file()
        },
    );

    assert!(still_there, "an agent approved its own change");
    assert!(
        record.is_empty(),
        "a question nobody answered became an entry"
    );
}

/// **A verb nobody declared is refused by the registry and not by this crate.**
/// The name comes off the wire exactly as it was written — `/bin/sh` and all —
/// and the closed list is what turns it away, with the refusal recorded.
#[test]
fn a_verb_nobody_declared_comes_off_the_wire_and_is_turned_away_by_the_list() {
    let mut record = Record::default();
    on_a_machine("no-such-verb", &mut record, |turning, grants, _, _| {
        let line = a_message(r#"{"read":{"verb":"/bin/sh","given":[]}}"#);
        let asked = FromAnAgent::read(&line).unwrap();
        assert_eq!(asked.verb(), Some("/bin/sh"));

        let turned_away = turning
            .reading(asked.verb().unwrap(), &asked.given(), grants, noon())
            .unwrap_err();
        assert!(
            matches!(turned_away, NotDone::TurnedAway(_)),
            "{turned_away:?}"
        );
    });

    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );
}

/// **An argument named twice survives the wire and is refused by the registry.**
/// This is what a JSON object would have thrown away before anybody could
/// refuse it, in the one place a person's approval sentence is built from.
#[test]
fn an_argument_named_twice_reaches_the_registry_and_is_refused_there() {
    let mut record = Record::default();
    on_a_machine("twice", &mut record, |turning, grants, _, invoice| {
        let path = invoice.display().to_string().replace('\\', "\\\\");
        let line = a_message(&format!(
            r#"{{"read":{{"verb":"read_file","given":[{{"named":"file","is":"{path}"}},{{"named":"file","is":"{path}"}}]}}}}"#
        ));
        let asked = FromAnAgent::read(&line).unwrap();
        assert_eq!(asked.given().len(), 2);

        let refused = turning
            .reading(asked.verb().unwrap(), &asked.given(), grants, noon())
            .unwrap_err();
        assert!(matches!(
            refused,
            NotDone::TurnedAway(alo_capability::CallError::SameArgumentTwice { .. })
        ));
    });

    assert_eq!(record.len(), 1);
}

/// **A path nobody granted is refused before the disk is touched**, whatever
/// arrived on the wire. This crate hands over what it was given and changes
/// nothing about it, so the grants see exactly what the agent asked for.
#[test]
fn a_path_nobody_granted_is_refused_however_it_was_written() {
    let mut record = Record::default();
    let elsewhere = std::env::temp_dir().join("alo-protocol-not-granted");
    let _ = fs::create_dir_all(&elsewhere);
    on_a_machine("not-granted", &mut record, |turning, grants, _, _| {
        let line = a_message(&format!(
            r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
            elsewhere.display().to_string().replace('\\', "\\\\")
        ));
        let asked = FromAnAgent::read(&line).unwrap();
        let refused = turning
            .reading(asked.verb().unwrap(), &asked.given(), grants, noon())
            .unwrap_err();
        assert!(refused.was_refused(), "{refused:?}");
    });

    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );
}

/// **A message that is not a request never reaches a turn at all**, so nothing
/// is written down about it: the record keeps what an agent did, and a client
/// that could not spell has not done anything to this machine.
#[test]
fn a_message_that_is_not_a_request_reaches_no_turn_and_no_record() {
    let mut record = Record::default();
    on_a_machine("gibberish", &mut record, |turning, _, _, _| {
        for line in [
            "not json at all",
            r#"{"format":9,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
            r#"{"format":1,"asks":{"run":{"command":"rm -rf /"}}}"#,
        ] {
            assert!(FromAnAgent::read(line).is_err(), "{line}");
        }
        assert!(!turning.is_closed());
    });

    assert!(
        record.is_empty(),
        "a message that was never a request became an entry"
    );
}

/// The agent a turn belongs to is the machine's, not the message's: nothing on
/// the wire names one, so a request cannot arrive claiming to be somebody else.
#[test]
fn nothing_on_the_wire_says_which_agent_asked() {
    let mut record = Record::default();
    on_a_machine("whose-turn", &mut record, |turning, _, _, _| {
        assert_eq!(turning.grantee(), &Grantee::named("@files"));
        for line in [
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"agent":"@mail"}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[],"as":"@mail"}}}"#,
        ] {
            assert_eq!(FromAnAgent::read(line), Err(NotUnderstood::NotReadable));
        }
    });
    assert!(record.is_empty());
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships.
///
/// `alo_turn::bounding` says why there is no such implementation in any library
/// here — it would be ADR 0015's guarantee turned off by default on every host —
/// and why what a test needs is these four lines, written where whoever reads
/// the test can see them.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }
}
