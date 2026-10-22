//! The door, against a real socket this test binds itself.
//!
//! **A real `UnixStream` and not a seam.** What this file is about is a
//! conversation on a socket — one line in, a shutdown, lines out — and a double
//! standing in for the socket would be testing the double's idea of that.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::thread;

use super::{NothingCameBack, what_the_agent_says};

/// A socket of this test's own, in a directory it removes.
///
/// **`/tmp` and never `$HOME`**: a Unix socket's path is bounded by `SUN_LEN`,
/// and `docs/quirks.md` records a `TMPDIR` under a home directory failing about
/// ten of this repository's socket tests for that reason alone.
fn a_socket_called(what: &str) -> PathBuf {
    let at = PathBuf::from("/tmp").join(format!("alo-ask-door-{}-{what}", std::process::id()));
    let _ = std::fs::remove_file(&at);
    at
}

/// **A sentence goes in and what the agent said comes back.**
#[test]
fn what_the_agent_says_comes_back() {
    let at = a_socket_called("answers");
    let listening = UnixListener::bind(&at).expect("binding a socket in /tmp");
    let heard = thread::spawn(move || {
        let (stream, _) = listening.accept().expect("a caller");
        let mut line = String::new();
        BufReader::new(&stream)
            .read_line(&mut line)
            .expect("the caller's line");
        (&stream)
            .write_all(b"it would open the text editor\n")
            .expect("answering");
        line
    });

    let said = what_the_agent_says(&at, "open the text editor").expect("an answer");
    assert_eq!(said, "it would open the text editor");
    assert_eq!(
        heard.join().expect("the listening thread"),
        "open the text editor\n",
        "the agent is handed exactly what the person typed, with one newline"
    );
    let _ = std::fs::remove_file(&at);
}

/// **A sentence spanning lines is handed over as one**, so the agent cannot
/// read its second half as a second request.
#[test]
fn what_was_typed_crosses_as_one_line() {
    let at = a_socket_called("one-line");
    let listening = UnixListener::bind(&at).expect("binding");
    let heard = thread::spawn(move || {
        let (stream, _) = listening.accept().expect("a caller");
        let mut all = String::new();
        BufReader::new(&stream)
            .read_to_string(&mut all)
            .expect("everything the caller wrote");
        (&stream).write_all(b"ok\n").expect("answering");
        all
    });

    what_the_agent_says(&at, "open the\neditor\r\nnow").expect("an answer");
    let all = heard.join().expect("the thread");
    assert_eq!(
        all.lines().count(),
        1,
        "three lines were typed and {all:?} crossed the door"
    );
    let _ = std::fs::remove_file(&at);
}

/// **Nobody there is its own sentence**, and names where it knocked.
#[test]
fn a_door_with_nobody_behind_it_says_so() {
    let at = a_socket_called("absent");
    match what_the_agent_says(&at, "open the text editor") {
        Err(NothingCameBack::NobodyThere { at: said }) => {
            assert!(said.contains("absent"), "{said}");
        }
        other => panic!("a socket nothing bound answered {other:?}"),
    }
}

/// **Closing without a word is told apart from answering nothing.**
///
/// A person reads *the agent closed* as something wrong with the machine, and
/// an empty answer as the agent having nothing to say. Collapsing them would
/// report the first as the second.
#[test]
fn closing_without_a_word_is_not_an_answer() {
    let at = a_socket_called("closes");
    let listening = UnixListener::bind(&at).expect("binding");
    thread::spawn(move || {
        let (stream, _) = listening.accept().expect("a caller");
        // **Read the line first, as a real agent does.** A server that closes
        // with the caller's bytes still unread makes Linux reset the
        // connection, and the caller's *write* then fails — which is a
        // different fault from the one this test is about, and it is not one
        // `alo-asking@.service` can produce, because it reads its input.
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        drop(stream);
    });

    match what_the_agent_says(&at, "open the text editor") {
        Err(NothingCameBack::ClosedWithoutAnswering) => {}
        other => panic!("a door that closed answered {other:?}"),
    }
    let _ = std::fs::remove_file(&at);
}

/// **Several lines back are all of the answer**, not the first of them.
#[test]
fn every_line_the_agent_wrote_comes_back() {
    let at = a_socket_called("many");
    let listening = UnixListener::bind(&at).expect("binding");
    thread::spawn(move || {
        let (stream, _) = listening.accept().expect("a caller");
        // Read before answering, for the reason the test above gives.
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        (&stream)
            .write_all(b"it would open the text editor\nit needs a grant for applications\n")
            .expect("answering");
    });

    let said = what_the_agent_says(&at, "open it").expect("an answer");
    assert_eq!(said.lines().count(), 2, "{said:?}");
    assert!(said.contains("grant"), "{said:?}");
    let _ = std::fs::remove_file(&at);
}
