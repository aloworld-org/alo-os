//! The socket and the protocol, met on a real Unix socket.
//!
//! `crates/alo-protocol` has two readers and neither can produce the other's
//! requests; `crates/alo-agentd` decides which of them a connection gets, and
//! decides it from what the kernel says rather than from anything on the wire.
//! Each of those is tested in its own crate. This is the join: a real socket in
//! a real directory, a real client connecting to it, and a request read with
//! the reader the door chose.
//!
//! **Only one side can be tested against a real connection here**, and the
//! reason is honest: telling the two apart takes two logins, and a test process
//! has one. Whoever runs the tests is the person, so a real connection lands on
//! the person's door and the request an agent would have sent is refused by the
//! person's reader — which is the property, from the side that can be reached.
//! A real agent connecting as its own user is owed with the rest of the
//! verification that needs a machine set up as a machine.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use alo_agentd::side::Side;
use alo_agentd::unix::{our_group, us};
use alo_agentd::{Listening, Place, Sides};
use alo_protocol::{FromAPerson, FromAnAgent, NotUnderstood};

/// The login this test gives the agent, which is not the one it runs as.
const AN_AGENT: u32 = 989;

/// The one it gives the agent if this test happens to run as [`AN_AGENT`].
const ANOTHER_AGENT: u32 = 990;

/// A folder of this test's own, on the disk the tests are running on.
fn a_directory_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-agentd-tests-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// This machine, with whoever is running the tests as the person.
fn this_machine() -> Sides {
    let person = us().unwrap();
    let agent = if person.raw() == AN_AGENT {
        ANOTHER_AGENT
    } else {
        AN_AGENT
    };
    Sides::of(
        person,
        alo_agentd::Uid::of(agent).unwrap(),
        our_group().unwrap(),
    )
    .unwrap()
}

/// A read, as an agent puts one on the wire.
fn a_read() -> String {
    r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}"#
        .to_owned()
}

/// An approval, as a person's shell puts one on the wire.
fn an_approval() -> String {
    r#"{"format":1,"asks":{"approve":{"number":7}}}"#.to_owned()
}

/// **A connection is served by the reader its door names.** The client here is
/// this process, which is the person, so what comes off the socket is read with
/// the person's reader — and an approval is a request that reader understands.
#[test]
fn what_arrives_is_read_with_the_reader_the_door_chose() {
    let folder = a_directory_of_our_own("read-by-the-door");
    let place = Place::under(&folder);
    let listening = Listening::at(place.clone(), this_machine()).unwrap();

    let mut client = UnixStream::connect(place.socket()).unwrap();
    let accepted = listening.next().unwrap();
    assert_eq!(accepted.side(), Side::Person);

    writeln!(client, "{}", an_approval()).unwrap();
    let mut line = String::new();
    BufReader::new(accepted.connection())
        .read_line(&mut line)
        .unwrap();

    let asked = FromAPerson::read(line.trim_end()).unwrap();
    assert_eq!(asked.number(), Some(7));
    assert!(asked.is_yes());
}

/// **A request for the other door is refused on this one**, in the reader
/// rather than in the daemon: the connection was accepted, the kernel said who
/// was there, and what makes a read impossible on a person's connection is that
/// the person's reader has no shape for one.
#[test]
fn a_request_for_the_other_door_is_refused_on_this_one() {
    let folder = a_directory_of_our_own("wrong-door");
    let place = Place::under(&folder);
    let listening = Listening::at(place.clone(), this_machine()).unwrap();

    let mut client = UnixStream::connect(place.socket()).unwrap();
    let accepted = listening.next().unwrap();

    writeln!(client, "{}", a_read()).unwrap();
    let mut line = String::new();
    BufReader::new(accepted.connection())
        .read_line(&mut line)
        .unwrap();

    let refused = FromAPerson::read(line.trim_end()).unwrap_err();
    assert!(matches!(refused, NotUnderstood::NotForAPerson));

    // And it is a request — the same line an agent's door would have
    // understood. What refused it is which door it arrived on.
    assert!(FromAnAgent::read(line.trim_end()).is_ok());
}

/// **The two doors of this machine are two users**, and a machine that named
/// one login twice has no socket at all rather than one door pretending to be
/// two.
#[test]
fn a_machine_with_one_login_gets_no_socket() {
    let person = us().unwrap();
    assert!(Sides::of(person, person, our_group().unwrap()).is_err());
}

/// Two connections at once are two callers, each answered with its own side.
/// The daemon that will hold a turn per connection is item 21d's; what is true
/// here is that the door does not confuse them.
#[test]
fn two_connections_are_two_callers() {
    let folder = a_directory_of_our_own("two-at-once");
    let place = Place::under(&folder);
    let listening = Listening::at(place.clone(), this_machine()).unwrap();

    let first = UnixStream::connect(place.socket()).unwrap();
    let second = UnixStream::connect(place.socket()).unwrap();

    let one = listening.next().unwrap();
    let two = listening.next().unwrap();

    assert_eq!(one.side(), Side::Person);
    assert_eq!(two.side(), Side::Person);
    assert_eq!(one.caller(), two.caller());

    drop(first);
    drop(second);
}
