//! The door as a real socket, with the kernel deciding who is at it.
//!
//! Everything in `crate::opening`'s own tests hands the caller's group in as a
//! number, which is what makes those refusals testable on any host. This is the
//! half that cannot be: a socket really bound, a connection really made, and
//! the group read out of `SO_PEERCRED` — which is not a field in a message and
//! cannot be written by a caller.
//!
//! # What it can and cannot show on one machine
//!
//! A test process is one uid in one group, so it can knock as itself and it
//! cannot knock as somebody else. What it *can* do is open a door handed to a
//! group this process is **not** in and show that its own knock is refused —
//! which is the same refusal from the other side, and it is the one that
//! matters: a door that let a caller in because the caller said so would pass a
//! test that only ever knocked as the greeter.

#![cfg(unix)]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use alo_sessiond::listening::Listening;
use alo_sessiond::{
    ALREADY_SIGNED_IN, Answered, Door, Knock, Logind, NOBODY_HERE, NOT_YOURS_TO_ASK, NotAsked,
    NotOpened, Opening, TheAccounts, our_group,
};

/// The number this machine's one account has, in these tests.
const THE_PERSON: u32 = 1000;

/// Accounts a test decides the answer for.
struct Numbered(Vec<u32>);

impl TheAccounts for Numbered {
    fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked> {
        Ok(self.0.contains(&person))
    }
}

/// A `logind` that opens whatever it is asked for, and remembers.
#[derive(Default)]
struct Willing(Vec<u32>);

impl Logind for Willing {
    fn open_a_session_for(&mut self, person: u32) -> Result<(), NotOpened> {
        self.0.push(person);
        Ok(())
    }
}

/// Somewhere this test may bind a socket.
fn somewhere(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-sessiond-door-{}-{what}", std::process::id()));
    drop(std::fs::create_dir_all(&at));
    at.join("sign-in.sock")
}

/// Knock at this door and read what comes back, as the sign-in surface would.
fn knock_at(socket: &std::path::Path, knock: Knock) -> Answered {
    let Ok(connection) = UnixStream::connect(socket) else {
        panic!("nothing is listening at {}", socket.display());
    };
    let line = format!("{}\n", knock.written());
    if (&connection).write_all(line.as_bytes()).is_err() {
        panic!("the door would not take a knock");
    }
    let mut back = String::new();
    if BufReader::new(&connection).read_line(&mut back).is_err() {
        panic!("the door said nothing");
    }
    match Answered::read(back.trim_end()) {
        Ok(answered) => answered,
        Err(why) => panic!("the door said something that is not an answer: {why}"),
    }
}

/// **A knock from this process, at a door handed to this process's group,
/// opens a session** — and the number that reaches `logind` is the number on
/// the wire and nothing else.
#[test]
fn the_greeter_knocking_over_a_real_socket_opens_a_session() {
    let at = somewhere("heard");
    let ours = our_group();
    let door = Door::handed_to(ours);
    let Ok(listening) = Listening::at(&at, ours) else {
        panic!("the door would not open at {}", at.display());
    };
    let mut opening = Opening::at(door, Numbered(vec![THE_PERSON]), Willing::default());

    let knocking = std::thread::spawn({
        let at = at.clone();
        move || knock_at(&at, Knock::on_behalf_of(THE_PERSON))
    });
    let Ok(answered) = listening.answer_one(&mut opening) else {
        panic!("the door did not answer");
    };

    assert_eq!(answered, Answered::Opened);
    assert_eq!(knocking.join().ok(), Some(Answered::Opened));
    assert!(opening.is_signed_in());
    drop(std::fs::remove_file(&at));
}

/// **A knock from a process the kernel says is in another group is refused**,
/// and the refusal gives nothing away — not whether the number exists, not
/// whether anybody is signed in. The door here is handed to a group this
/// process is not in, which is the same fact from the other side.
#[test]
fn a_caller_the_kernel_puts_in_another_group_is_refused() {
    let at = somewhere("turned-away");
    let ours = our_group();
    let door = Door::handed_to(ours.wrapping_add(1));
    let Ok(listening) = Listening::at(&at, ours) else {
        panic!("the door would not open at {}", at.display());
    };
    // Accounts that hold this person and a logind that would open anything: if
    // either were reached, the answer below would be a session rather than a
    // refusal, so the assertion is also that neither was.
    let mut opening = Opening::at(door, Numbered(vec![THE_PERSON]), Willing::default());

    let knocking = std::thread::spawn({
        let at = at.clone();
        move || knock_at(&at, Knock::on_behalf_of(THE_PERSON))
    });
    let Ok(answered) = listening.answer_one(&mut opening) else {
        panic!("the door did not answer");
    };

    assert_eq!(answered, Answered::Refused(NOT_YOURS_TO_ASK.key()));
    assert_eq!(
        knocking.join().ok(),
        Some(Answered::Refused(NOT_YOURS_TO_ASK.key()))
    );
    assert!(!opening.is_signed_in());
    drop(std::fs::remove_file(&at));
}

/// **A line that is not a knock is answered with the refusal that gives
/// nothing away**, and the connection is answered rather than dropped. A door
/// that hung up would leave whatever wrote the other end with no way to tell a
/// refusal from a service that is not running.
#[test]
fn a_line_that_is_not_a_knock_is_answered_and_told_nothing() {
    let at = somewhere("not-a-knock");
    let ours = our_group();
    let door = Door::handed_to(ours);
    let Ok(listening) = Listening::at(&at, ours) else {
        panic!("the door would not open at {}", at.display());
    };
    let mut opening = Opening::at(door, Numbered(vec![THE_PERSON]), Willing::default());

    let knocking = std::thread::spawn({
        let at = at.clone();
        move || {
            let Ok(connection) = UnixStream::connect(&at) else {
                panic!("nothing is listening");
            };
            drop((&connection).write_all(b"open 1000 hunter2\n"));
            let mut back = String::new();
            drop(BufReader::new(&connection).read_line(&mut back));
            back
        }
    });
    let Ok(answered) = listening.answer_one(&mut opening) else {
        panic!("the door did not answer");
    };

    assert_eq!(answered, Answered::Refused(NOT_YOURS_TO_ASK.key()));
    assert_eq!(
        knocking.join().ok().map(|line| line.trim_end().to_owned()),
        Some(answered.written())
    );
    assert!(!opening.is_signed_in());
    drop(std::fs::remove_file(&at));
}

/// **Over a real socket, a number this machine has no account for opens
/// nothing, and a second knock while somebody is signed in opens nothing
/// either.** The whole sequence a machine really sees, in the order it sees it.
#[test]
fn a_real_door_refuses_a_stranger_and_a_second_sign_in() {
    let at = somewhere("sequence");
    let ours = our_group();
    let door = Door::handed_to(ours);
    let Ok(listening) = Listening::at(&at, ours) else {
        panic!("the door would not open at {}", at.display());
    };
    let mut opening = Opening::at(door, Numbered(vec![THE_PERSON]), Willing::default());

    for (knock, expected) in [
        (
            Knock::on_behalf_of(4242),
            Answered::Refused(NOBODY_HERE.key()),
        ),
        (Knock::on_behalf_of(THE_PERSON), Answered::Opened),
        (
            Knock::on_behalf_of(THE_PERSON),
            Answered::Refused(ALREADY_SIGNED_IN.key()),
        ),
    ] {
        let knocking = std::thread::spawn({
            let at = at.clone();
            move || knock_at(&at, knock)
        });
        let Ok(answered) = listening.answer_one(&mut opening) else {
            panic!("the door did not answer");
        };
        assert_eq!(answered, expected);
        assert_eq!(knocking.join().ok(), Some(expected));
    }

    drop(std::fs::remove_file(&at));
}
