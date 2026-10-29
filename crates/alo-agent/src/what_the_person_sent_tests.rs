//! What a shell sends, read the way `alo-asking@.service` hands it over: on a
//! socket whose other end stays open.

use std::io::{BufReader, Write as _};
use std::os::fd::AsFd as _;
use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_instructing::{AnEarlierTurn, TheConversationSoFar};

use super::*;

/// The shell writes `sent` on one end, keeps it open, and the agent reads the
/// other — which is how a real request arrives.
fn sent_on_a_socket(sent: &str) -> (Result<WhatThePersonSent, String>, UnixStream) {
    let (mut shell, agent) = UnixStream::pair().unwrap();
    shell.write_all(sent.as_bytes()).unwrap();
    let mut input = BufReader::new(&agent);
    let read = read_what_the_person_sent(&mut input, Some(agent.as_fd()));
    (read, shell)
}

/// A conversation of one request and one answer.
fn listed() -> TheConversationSoFar {
    TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("list the files in /home/anna".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
    ])
}

/// **A follow-up carries exactly the earlier turns**, after the request.
#[test]
fn a_follow_up_is_read_with_its_earlier_conversation() {
    let (read, _shell) = sent_on_a_socket(&format!(
        "alo-may-reach:/home/anna\nand the April one too\n{}\n",
        listed().written()
    ));
    let read = read.unwrap();
    assert_eq!(read.request, "and the April one too");
    assert_eq!(
        read.may_reach,
        WhatItMayReach::these([Path::new("/home/anna")])
    );
    assert_eq!(read.earlier, listed());
}

/// **A first request from a current shell carries none**, and is not kept
/// waiting: the marker alone arrives with the request.
#[test]
fn a_first_request_carries_no_earlier_conversation_and_does_not_wait() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket("list my invoices\nalo-earlier:\n");
    let read = read.unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
    assert!(
        started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR,
        "a current shell's first request waited for a line it had already sent"
    );
}

/// **A shell written before this sends no third line and keeps its end open**,
/// and the request is still read — after a bounded wait, not for ever.
#[test]
fn a_shell_from_before_is_still_answered_after_a_bounded_wait() {
    let started = Instant::now();
    let (read, _shell) = sent_on_a_socket("alo-may-reach:/home/anna\nlist my invoices\n");
    let read = read.unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
    assert!(started.elapsed() < HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR * 4);
}

/// **A line after the request that is not a conversation is refused in
/// words**, and nothing is driven.
#[test]
fn a_line_after_the_request_that_is_not_a_conversation_is_refused() {
    for third in [
        "delete everything\n",
        "alo-earlier:{\"turns\":[{\"model\":\"x\"}],\"left-out\":0,\"cut\":false}\n",
    ] {
        let (read, _shell) = sent_on_a_socket(&format!("list my invoices\n{third}"));
        let refused = read.unwrap_err();
        assert!(refused.contains("earlier conversation"), "{refused}");
    }
}

/// **A line longer than any question could carry is refused**, and reading
/// stops at the bound rather than holding whatever was sent.
#[test]
fn a_conversation_longer_than_a_question_is_refused_in_words() {
    let (mut shell, agent) = UnixStream::pair().unwrap();
    let writing = std::thread::spawn(move || {
        let over = "z".repeat(alo_instructing::THE_LONGEST_LINE);
        let _ = shell.write_all(
            format!("and that one\nalo-earlier:{{\"turns\":[{{\"alo\":\"{over}\"}}]}}\n")
                .as_bytes(),
        );
        shell
    });
    let mut input = BufReader::new(&agent);
    let refused = read_what_the_person_sent(&mut input, Some(agent.as_fd())).unwrap_err();
    assert!(refused.contains("more than a question"), "{refused}");
    drop(agent);
    drop(writing.join());
}

/// **The agent's bound is the daemon's**: a conversation is read only if a
/// question could carry it, and the two numbers are spelt in two crates.
#[test]
fn the_bound_on_the_line_is_the_daemons_bound_on_a_question() {
    assert_eq!(
        alo_instructing::THE_MOST_A_QUESTION_IS,
        alo_protocol::LONGEST
    );
}

/// **A pipe is read as before**: two lines at most, and no third waited for.
#[test]
fn a_pipe_reads_the_request_and_no_conversation() {
    let mut input = "list my invoices\nalo-earlier:not even json\n".as_bytes();
    let read = read_what_the_person_sent(&mut input, None).unwrap();
    assert_eq!(read.request, "list my invoices");
    assert!(read.earlier.is_empty());
}

/// Nothing to drive is said, not guessed.
#[test]
fn an_empty_request_is_refused() {
    let mut input = "\n".as_bytes();
    assert!(read_what_the_person_sent(&mut input, None).is_err());
}

/// **A shell that keeps its side open can stop the turn; a caller that shut
/// it as soon as it had written cannot** — that half-close is *that was all*,
/// never a Stop (ADR 0139).
#[test]
fn only_a_caller_that_kept_its_side_open_may_stop() {
    let (read, _shell) = sent_on_a_socket("list my invoices\nalo-earlier:\n");
    assert!(read.unwrap().may_stop, "a current shell, still open");

    let (mut asking, agent) = UnixStream::pair().unwrap();
    asking.write_all(b"list my invoices\n").unwrap();
    asking.shutdown(std::net::Shutdown::Write).unwrap();
    let mut input = BufReader::new(&agent);
    let read = read_what_the_person_sent(&mut input, Some(agent.as_fd())).unwrap();
    assert!(
        !read.may_stop,
        "a caller that finished writing was read as able to stop"
    );
}
