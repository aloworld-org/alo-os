//! What issues a token asks at the real door and reads the real answer back:
//! carried once, refused the second time, and a door that is not there or says
//! something else is not an answer.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::Write as _;
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::SystemTime;

use alo_broker::asking::{NotAsked, ask};
use alo_broker::listening::Listening;
use alo_broker::{
    Answer, ApprovingKey, Broker, Carrying, Door, Identity, NotCarried, Request, SystemVerb,
    our_group, our_user,
};
use alo_record::{AtTheBroker, Record};

/// The key both sides hold in this test.
const THE_KEY: [u8; 32] = [23; 32];

/// Counts what was carried out.
#[derive(Debug, Default)]
struct Counts(usize);

impl Carrying for Counts {
    fn carry(&mut self, _: SystemVerb, _: u64) -> Result<(), NotCarried> {
        self.0 += 1;
        Ok(())
    }
}

/// Somewhere this test may bind a socket of its own.
fn somewhere(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-broker-ask-{}-{what}", std::process::id()));
    std::fs::create_dir_all(&at).unwrap();
    at.join("door.sock")
}

/// **Asked once, carried once; asked again with the same approval, refused** —
/// and both answers are the ones the door gave.
#[test]
fn a_request_is_carried_once_and_its_approval_refused_after() {
    let at = somewhere("once");
    let door = Listening::at(&at, our_group()).unwrap();
    let answering = std::thread::spawn(move || {
        let mut broker = Broker::new(
            Door::handed_to(our_user()),
            ApprovingKey::of(&THE_KEY),
            Record::default(),
            Counts::default(),
        );
        let first = door.answer_one(&mut broker).unwrap();
        let second = door.answer_one(&mut broker).unwrap();
        (first, second, broker.carrying().0)
    });

    let verb = SystemVerb::RemovePrinter(Identity::of_what_was_reported(b"alo-canon"));
    let request = Request::of(
        verb,
        ApprovingKey::of(&THE_KEY).issue(&verb, 4, SystemTime::now()),
    );
    assert_eq!(ask(&at, &request).unwrap(), Answer::Carried);
    assert_eq!(
        ask(&at, &request).unwrap(),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    let (first, second, carried) = answering.join().unwrap();
    assert_eq!(first, Answer::Carried);
    assert_eq!(second, Answer::Refused(AtTheBroker::ApprovalSpent));
    assert_eq!(carried, 1);
    drop(std::fs::remove_file(&at));
}

/// **A door handed to somebody else answers without reading**, and the asking
/// side still reads that refusal rather than failing on the write.
#[test]
fn a_refusal_unread_still_reaches_the_side_that_asked() {
    let at = somewhere("unread");
    let door = Listening::at(&at, our_group()).unwrap();
    let answering = std::thread::spawn(move || {
        let mut broker = Broker::new(
            Door::handed_to(our_user().wrapping_add(1)),
            ApprovingKey::of(&THE_KEY),
            Record::default(),
            Counts::default(),
        );
        door.answer_one(&mut broker).unwrap()
    });
    let verb = SystemVerb::SetDefaultPrinter(Identity::of_what_was_reported(b"alo-canon"));
    let request = Request::of(
        verb,
        ApprovingKey::of(&THE_KEY).issue(&verb, 5, SystemTime::now()),
    );
    assert_eq!(
        ask(&at, &request).unwrap(),
        Answer::Refused(AtTheBroker::NotTheAgentService)
    );
    assert_eq!(
        answering.join().unwrap(),
        Answer::Refused(AtTheBroker::NotTheAgentService)
    );
    drop(std::fs::remove_file(&at));
}

/// **No door, or a door that says anything but an answer, is not an answer.**
#[test]
fn no_door_or_something_else_at_it_is_not_an_answer() {
    let verb = SystemVerb::AddPrinter(Identity::of_what_was_reported(b"ipp://x"));
    let request = Request::of(
        verb,
        ApprovingKey::of(&THE_KEY).issue(&verb, 6, SystemTime::now()),
    );

    let nowhere = somewhere("nowhere");
    drop(std::fs::remove_file(&nowhere));
    assert!(matches!(ask(&nowhere, &request), Err(NotAsked::NoDoor(_))));

    for said in [
        &b"yes\n"[..],
        b"carried",
        b"refused because\n",
        &[b'x'; 200],
    ] {
        let at = somewhere("something-else");
        drop(std::fs::remove_file(&at));
        let listener = UnixListener::bind(&at).unwrap();
        let saying = said.to_vec();
        let impostor = std::thread::spawn(move || {
            let (mut connection, _) = listener.accept().unwrap();
            drop(connection.write_all(&saying));
        });
        assert!(
            matches!(ask(&at, &request), Err(NotAsked::NoAnswer)),
            "{:?} was read as an answer",
            String::from_utf8_lossy(said)
        );
        impostor.join().unwrap();
        drop(std::fs::remove_file(&at));
    }
}
