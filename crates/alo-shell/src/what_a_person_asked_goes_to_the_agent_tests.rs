//! The road a person's words take, against a stand-in agent on a real socket.
//!
//! **What is substituted and what is not.** The socket is real, the thread is
//! real, the lines are real, and the stand-in is a listener in this process
//! answering the way `alo-agent` answers. What it does not prove is that
//! systemd starts the real agent on the other end — that is
//! `alo-asking.socket`'s and is held by `alo-image`'s tests against the units
//! themselves.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::WhoAsked;
use super::{TheAgentsDoor, WHERE_WORDS_GO, WhatCameBack};
use crate::the_tasks_a_person_opened::WhichTask;

/// A socket of this test's own.
///
/// Named from `what` and not from a count, because two tests that happened to
/// want the same number of anything would otherwise share a path — which is a
/// mistake this repository has now made in three crates.
fn a_socket(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-asking-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at.join("socket")
}

/// A stand-in agent that answers `said` to one request and stops.
///
/// Reports the line it was handed, so a test can say what really crossed rather
/// than describe it.
fn an_agent_answering(at: &PathBuf, said: &'static str) -> std::sync::mpsc::Receiver<String> {
    let listening = UnixListener::bind(at).unwrap();
    let (telling, heard) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let Ok((talking, _)) = listening.accept() else {
            return;
        };
        let mut line = String::new();
        if BufReader::new(&talking).read_line(&mut line).is_err() {
            return;
        }
        drop(telling.send(line));
        let mut writing = &talking;
        let _ = writing.write_all(said.as_bytes());
        let _ = writing.flush();
        drop(talking);
    });
    heard
}

/// Wait for one answer without blocking a test for ever.
fn one_answer(door: &TheAgentsDoor) -> WhatCameBack {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(came) = door.whatever_came_back().into_iter().next() {
            return came;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("nothing came back within ten seconds");
}

/// **The whole road: one line out, and what the agent said comes back.**
#[test]
fn what_a_person_typed_goes_out_and_the_answer_comes_back() {
    let at = a_socket("round-trip");
    let heard = an_agent_answering(&at, "Three are unpaid.\n");
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(0)),
        "how many invoices are unpaid?",
        &[],
    );

    assert_eq!(heard.recv().unwrap(), "how many invoices are unpaid?\n");
    let came = one_answer(&door);
    assert_eq!(came.said, "Three are unpaid.");
    assert!(!came.the_machines_fault);
    assert_eq!(came.who, WhoAsked::ATaskWindow(WhichTask::numbered(0)));

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **Every line the agent wrote, not the first one.**
///
/// `alo-agent` prints a waiting change as two lines and **the sentence is on
/// the second**. Reading one line would hand a person *a change is waiting for
/// you to answer, as number 7* and silently drop the sentence they are being
/// asked to approve — which is the one thing on this wire that must never be
/// lost, because what a person approves is that sentence.
#[test]
fn a_waiting_change_keeps_the_sentence_a_person_approves() {
    let at = a_socket("two-lines");
    let heard = an_agent_answering(
        &at,
        "a change is waiting for you to answer, as number 7:\n  rename march.pdf to \
         march-final.pdf\n",
    );
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(3)),
        "rename the march invoice",
        &[],
    );
    assert_eq!(heard.recv().unwrap(), "rename the march invoice\n");

    let came = one_answer(&door);
    assert!(
        came.said.contains("rename march.pdf to march-final.pdf"),
        "the sentence a person approves was dropped: {:?}",
        came.said
    );
    assert!(came.said.contains("number 7"), "{:?}", came.said);
    assert!(!came.the_machines_fault);

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **Nothing listening is an answer, not a fault to stop for.**
///
/// A desktop with no daemon running is in exactly this state, and a compositor
/// that treated it as an error would refuse to draw because nobody had signed
/// in yet.
#[test]
fn a_machine_with_nothing_listening_says_so_and_keeps_going() {
    let at = a_socket("nobody-there");
    let _ = std::fs::remove_file(&at);
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(1)),
        "anything",
        &[],
    );

    let came = one_answer(&door);
    assert!(came.the_machines_fault, "{:?}", came.said);
    assert!(
        came.said.contains("nothing is listening"),
        "{:?}",
        came.said
    );

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **An agent that closes with nothing written is the machine's fault**, not an
/// empty answer put in front of a person.
#[test]
fn an_agent_that_says_nothing_is_the_machines_fault() {
    let at = a_socket("silent");
    let _heard = an_agent_answering(&at, "");
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(2)),
        "anything",
        &[],
    );

    let came = one_answer(&door);
    assert!(came.the_machines_fault, "{:?}", came.said);
    assert!(came.said.contains("without answering"), "{:?}", came.said);

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **Asking does not block**, which is what lets this be read from a frame
/// loop.
///
/// The stand-in accepts and then says nothing for a second. `ask` must come
/// back immediately regardless — a compositor that waited here would stop every
/// window on the machine from drawing until a provider answered.
#[test]
fn asking_comes_back_at_once_however_slow_the_agent_is() {
    let at = a_socket("slow");
    let listening = UnixListener::bind(&at).unwrap();
    std::thread::spawn(move || {
        let Ok((talking, _)) = listening.accept() else {
            return;
        };
        std::thread::sleep(Duration::from_secs(1));
        let mut writing = &talking;
        let _ = writing.write_all(b"late\n");
    });
    let door = TheAgentsDoor::at(&at);

    let began = Instant::now();
    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(0)),
        "anything",
        &[],
    );
    let took = began.elapsed();
    assert!(
        took < Duration::from_millis(200),
        "asking took {took:?}, so it waited for the agent"
    );
    // And nothing has come back yet, which is the other half of not waiting.
    assert!(door.whatever_came_back().is_empty());

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **One answer reaches one window once.** Drained rather than read, the same
/// rule the other end of this road keeps.
#[test]
fn an_answer_is_handed_over_once() {
    let at = a_socket("once");
    let _heard = an_agent_answering(&at, "done\n");
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(0)),
        "anything",
        &[],
    );
    let came = one_answer(&door);
    assert_eq!(came.said, "done");
    assert!(
        door.whatever_came_back().is_empty(),
        "the same answer came back twice"
    );

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// The default door is the one ADR 0109 published.
#[test]
fn the_default_door_is_the_one_the_decision_published() {
    assert_eq!(
        TheAgentsDoor::default().socket(),
        std::path::Path::new(WHERE_WORDS_GO)
    );
    assert_eq!(WHERE_WORDS_GO, "/run/alo-asking/socket");
}

/// A stand-in agent that reads `lines` lines, answers `said`, and stops —
/// reporting everything it read, so a test can hold what crossed whole.
fn an_agent_reading(
    at: &PathBuf,
    lines: usize,
    said: &'static str,
) -> std::sync::mpsc::Receiver<String> {
    let listening = UnixListener::bind(at).unwrap();
    let (telling, heard) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let Ok((talking, _)) = listening.accept() else {
            return;
        };
        let mut reading = BufReader::new(&talking);
        let mut all = String::new();
        for _ in 0..lines {
            if reading.read_line(&mut all).is_err() {
                return;
            }
        }
        drop(telling.send(all));
        let mut writing = &talking;
        let _ = writing.write_all(said.as_bytes());
        let _ = writing.flush();
        drop(talking);
    });
    heard
}

/// **A follow-up carries exactly the conversation so far, after the request**
/// — and a reply typed over two lines still crosses as one request (ADR 0136).
#[test]
fn a_follow_up_carries_the_conversation_after_the_request() {
    use alo_instructing::{AnEarlierTurn, TheConversationSoFar};

    let at = a_socket("follow-up");
    let heard = an_agent_reading(&at, 3, "april.pdf, at 40 KB.\n");
    let door = TheAgentsDoor::at(&at);
    let earlier = TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("list the files in /home/anna".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
    ]);

    door.ask_following(
        WhoAsked::ATaskWindow(WhichTask::numbered(0)),
        "and how big is\nthe biggest one?",
        &earlier,
        &[PathBuf::from("/home/anna")],
    );

    let crossed = heard.recv().unwrap();
    let lines: Vec<&str> = crossed.lines().collect();
    assert_eq!(
        lines.as_slice(),
        [
            "alo-may-reach:/home/anna",
            "and how big is the biggest one?",
            earlier.written().as_str(),
        ],
        "{crossed}"
    );
    assert_eq!(
        TheConversationSoFar::read(lines.get(2).copied().unwrap_or_default()),
        Ok(earlier)
    );
    assert_eq!(one_answer(&door).said, "april.pdf, at 40 KB.");

    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}

/// **A first request carries no conversation**: the marker alone, so a current
/// agent is not left waiting for a line that will not come.
#[test]
fn a_first_request_carries_the_marker_and_no_conversation() {
    let at = a_socket("first-request");
    let heard = an_agent_reading(&at, 2, "Three are unpaid.\n");
    let door = TheAgentsDoor::at(&at);

    door.ask(
        WhoAsked::ATaskWindow(WhichTask::numbered(0)),
        "how many invoices are unpaid?",
        &[],
    );

    assert_eq!(
        heard.recv().unwrap(),
        "how many invoices are unpaid?\nalo-earlier:\n"
    );
    let _ = one_answer(&door);
    let _ = std::fs::remove_dir_all(at.parent().unwrap());
}
