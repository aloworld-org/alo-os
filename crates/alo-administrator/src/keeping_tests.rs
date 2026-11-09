//! What the sign-in screen sends, what the keeper answers, and the screen's
//! side of the socket against a keeper played by the test.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;

use super::*;

/// **A number and a password on two lines is what is kept**, and the password
/// is the very bytes that were sent.
#[test]
fn a_number_and_a_password_are_read() {
    assert_eq!(what_was_sent("1000\n1234\n"), Ok((1000, "1234")));
    assert_eq!(
        what_was_sent("1000\ncorrect horse battery staple\n"),
        Ok((1000, "correct horse battery staple"))
    );
}

/// **Anything else is refused before `chpasswd` is asked**: no line end, no
/// number, a password the list cannot hold.
#[test]
fn anything_else_is_refused_before_anything_is_written() {
    for sent in ["1000\n1234", "1234\n", "alo\n1234\n", "\n1234\n", ""] {
        assert_eq!(what_was_sent(sent), Err(NotKept::NotTwoLines), "{sent:?}");
    }
    for sent in [
        "1000\n\n",
        "1000\na:b\n",
        "1000\na\rb\n",
        "1000\na\0b\n",
        "1000\na\nb\n",
    ] {
        assert_eq!(what_was_sent(sent), Err(NotKept::CannotHoldIt), "{sent:?}");
    }
    let too_long = format!("1000\n{}\n", "a".repeat(THE_LONGEST_PASSWORD + 1));
    assert_eq!(what_was_sent(&too_long), Err(NotKept::CannotHoldIt));
}

/// **Every answer reads back as itself**, and a line that is not an answer is
/// no answer rather than a success.
#[test]
fn every_answer_reads_back_as_itself() {
    assert_eq!(answered(&answer_of(Ok(()))), Ok(()));
    for why in NotKept::EVERY_ONE {
        assert_eq!(answered(&answer_of(Err(why))), Err(why));
        assert!(!why.word().contains(' '), "{why:?}");
    }
    for said in ["", "yes", "kept it", "not-kept", "not-kept maybe"] {
        assert_eq!(answered(said), Err(NotKept::NoAnswer), "{said:?}");
    }
}

/// **The screen sends exactly the two lines and believes only the answer.**
#[test]
fn the_screen_sends_two_lines_and_reads_the_answer() {
    let place = tempfile::tempdir().unwrap();
    let socket = place.path().join("keeper.socket");
    let listener = UnixListener::bind(&socket).unwrap();
    let keeper = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reading = BufReader::new(stream.try_clone().unwrap());
        let mut sent = String::new();
        reading.read_line(&mut sent).unwrap();
        reading.read_line(&mut sent).unwrap();
        (&stream)
            .write_all(answer_of(Err(NotKept::Refused)).as_bytes())
            .unwrap();
        sent
    });
    assert_eq!(
        handed_to_the_keeper(&socket, 1000, "1234"),
        Err(NotKept::Refused)
    );
    assert_eq!(keeper.join().unwrap(), "1000\n1234\n");
}

/// **No keeper is no answer**, said rather than taken for a kept password.
#[test]
fn no_keeper_is_no_answer() {
    let place = tempfile::tempdir().unwrap();
    assert_eq!(
        handed_to_the_keeper(&place.path().join("nobody.socket"), 1000, "1234"),
        Err(NotKept::NoAnswer)
    );
    assert_eq!(
        handed_to_the_keeper(&place.path().join("nobody.socket"), 1000, "a:b"),
        Err(NotKept::CannotHoldIt),
        "a password the list cannot hold is refused before anything is sent"
    );
}
