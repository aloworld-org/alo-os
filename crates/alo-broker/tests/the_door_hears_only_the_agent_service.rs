//! The door as a real socket, with the kernel deciding who is at it.
//!
//! The broker's own tests hand the caller in as a number. This is the half that
//! cannot be: one socket really bound, a connection really made, and the caller
//! read out of `SO_PEERCRED` — what the kernel recorded about the process that
//! connected, which is not a field in a message and cannot be written by one.
//!
//! A test process is one user, so it can only ever connect as itself. What it
//! can do is open a door handed to a user it is **not**, and show that its own
//! perfectly approved request is refused unread — the same refusal from the
//! other side, and the one that matters: a door that believed a caller would
//! pass a test that only ever connected as the agent's service.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_broker::listening::Listening;
use alo_broker::{
    Answer, ApprovingKey, Broker, Carrying, Door, Identity, NotCarried, Request, SystemVerb,
    our_group, our_user,
};
use alo_record::{AtTheBroker, Happened, Record};

/// The bytes of the key the broker in these tests holds.
const THE_KEY: [u8; 32] = [11; 32];

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
    let at = std::env::temp_dir().join(format!("alo-broker-{}-{what}", std::process::id()));
    std::fs::create_dir_all(&at).unwrap();
    at.join("broker.sock")
}

/// A request for a mounted drive, approved as the turn would approve it now.
fn an_approved_request() -> String {
    let verb = SystemVerb::MountDrive(Identity::of_what_was_reported(b"usb-drive"));
    Request::of(
        verb,
        ApprovingKey::of(&THE_KEY).issue(&verb, 1, SystemTime::now()),
    )
    .written()
}

/// Send one line to the door on another thread, and read the answer back.
fn ask(at: &Path, line: String) -> std::thread::JoinHandle<String> {
    let at = at.to_owned();
    std::thread::spawn(move || {
        let mut connection = UnixStream::connect(&at).unwrap();
        connection.write_all(line.as_bytes()).unwrap();
        connection.write_all(b"\n").unwrap();
        let mut answer = String::new();
        BufReader::new(connection).read_line(&mut answer).unwrap();
        answer
    })
}

/// **The agent's service, as the kernel names it, is heard through the door**:
/// its approved request is carried out, and the answer crosses back.
#[test]
fn the_agent_service_is_heard_through_a_real_socket() {
    let at = somewhere("heard");
    let door = Listening::at(&at, our_group()).unwrap();
    let mut broker = Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Counts::default(),
    );

    let asking = ask(&at, an_approved_request());
    let answer = door.answer_one(&mut broker).unwrap();

    assert_eq!(answer, Answer::Carried);
    assert_eq!(asking.join().unwrap(), "carried\n");
    assert_eq!(broker.carrying().0, 1);
    drop(std::fs::remove_file(&at));
}

/// **Anybody the kernel names as someone else is refused, and not read** —
/// even with a request a person really approved, because being able to reach
/// a socket is not being the agent's service.
#[test]
fn a_caller_the_kernel_names_as_anybody_else_is_refused_unread() {
    let at = somewhere("refused");
    let door = Listening::at(&at, our_group()).unwrap();
    let somebody_else = our_user().wrapping_add(1);
    let mut broker = Broker::new(
        Door::handed_to(somebody_else),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Counts::default(),
    );

    let asking = ask(&at, an_approved_request());
    let answer = door.answer_one(&mut broker).unwrap();

    assert_eq!(answer, Answer::Refused(AtTheBroker::NotTheAgentService));
    assert_eq!(asking.join().unwrap(), "refused not-the-agent-service\n");
    assert_eq!(broker.carrying().0, 0);
    let entries: Vec<&Happened> = broker
        .recording()
        .everything()
        .map(|entry| entry.happened())
        .collect();
    assert!(
        matches!(
            entries.as_slice(),
            [Happened::Brokered {
                verb: None,
                from_approval: None,
                refused: Some(AtTheBroker::NotTheAgentService),
            }]
        ),
        "{entries:?}"
    );
    drop(std::fs::remove_file(&at));
}

/// **The one socket is made where only its owner and its group can reach it**,
/// and a line longer than any request is not read as one.
#[test]
fn the_door_is_one_socket_its_group_reaches_and_a_long_line_is_not_a_request() {
    let at = somewhere("mode");
    let door = Listening::at(&at, our_group()).unwrap();
    let mode = std::fs::metadata(door.at_path())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o660);

    let mut broker = Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Counts::default(),
    );
    let asking = ask(
        &at,
        format!("{} {}", an_approved_request(), "0".repeat(4096)),
    );
    let answer = door.answer_one(&mut broker).unwrap();
    assert_eq!(answer, Answer::Refused(AtTheBroker::NotARequest));
    assert_eq!(asking.join().unwrap(), "refused not-a-request\n");
    assert_eq!(broker.carrying().0, 0);
    drop(std::fs::remove_file(&at));
}
