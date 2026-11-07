//! The prompt on a real pseudo-terminal: the person's keys arrive at one end,
//! and the question is asked at the other.

#![expect(
    clippy::expect_used,
    reason = "in a test, a terminal that will not open is the failure being reported"
)]

use std::io::Write as _;
use std::os::fd::AsFd as _;
use std::time::Duration;

use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};
use rustix::termios::LocalModes;

use super::{TheirAnswer, asked_at, the_answer_in};

/// The prompt, as ADR 0124 draws it.
const PROMPT: &str = "Run this command? [y/N] ";

/// A terminal: the end the person types at, and the end this process asks at.
fn a_terminal() -> (std::fs::File, std::fs::File) {
    let far = openpt(OpenptFlags::RDWR | OpenptFlags::NOCTTY).expect("a terminal opens");
    grantpt(&far).expect("granted");
    unlockpt(&far).expect("unlocked");
    let name = ptsname(&far, Vec::new()).expect("it has a name");
    let near = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(name.to_string_lossy().as_ref())
        .expect("its near end opens");
    (std::fs::File::from(far), near)
}

/// The person presses `typed`, and the question is asked.
fn pressing(typed: &[u8]) -> TheirAnswer {
    let (mut person, near) = a_terminal();
    person.write_all(typed).expect("the key goes in");
    let mut told = Vec::new();
    asked_at(near.as_fd(), PROMPT, &mut told, Duration::from_secs(5))
}

/// **Only `y` is yes.**
#[test]
fn y_is_yes() {
    assert_eq!(pressing(b"y"), TheirAnswer::Yes);
    assert_eq!(pressing(b"Y"), TheirAnswer::Yes);
}

/// **`n`, Enter, Ctrl-C and Ctrl-D are no**, and Ctrl-C arrives as a key
/// rather than a signal, so it is a no that is sent rather than a death.
#[test]
fn n_enter_ctrl_c_and_ctrl_d_are_no() {
    for key in [b"n", b"\r", b"\x03", b"\x04", b"x"] {
        assert_eq!(pressing(key), TheirAnswer::No, "{key:?}");
    }
}

/// **No key while the question stands is no answer**, not a no and never a
/// yes.
#[test]
fn nothing_pressed_in_time_is_no_answer() {
    let (_person, near) = a_terminal();
    let mut told = Vec::new();
    assert_eq!(
        asked_at(near.as_fd(), PROMPT, &mut told, Duration::from_millis(200)),
        TheirAnswer::NoAnswerInTime
    );
}

/// **A terminal that closed has nobody at it.**
#[test]
fn a_terminal_that_closed_is_not_an_answer() {
    let (person, near) = a_terminal();
    drop(person);
    let mut told = Vec::new();
    assert_eq!(
        asked_at(near.as_fd(), PROMPT, &mut told, Duration::from_secs(5)),
        TheirAnswer::TheTerminalWentAway
    );
}

/// **The terminal is put back as it was**, and the question is what was shown.
#[test]
fn the_terminal_is_put_back_and_the_prompt_is_shown() {
    let (mut person, near) = a_terminal();
    let before = rustix::termios::tcgetattr(&near).expect("it has modes");
    assert!(
        before
            .local_modes
            .contains(LocalModes::ICANON | LocalModes::ISIG)
    );
    person.write_all(b"n").expect("the key goes in");
    let mut told = Vec::new();
    let _ = asked_at(near.as_fd(), PROMPT, &mut told, Duration::from_secs(5));
    let after = rustix::termios::tcgetattr(&near).expect("it has modes");
    assert_eq!(
        after.local_modes, before.local_modes,
        "the terminal was left changed"
    );
    assert_eq!(String::from_utf8_lossy(&told), format!("{PROMPT}N\n"));
}

/// Something that is not a terminal is never asked.
#[test]
fn something_that_is_not_a_terminal_is_not_asked() {
    let (reading, _writing) = std::io::pipe().expect("a pipe");
    let mut told = Vec::new();
    assert_eq!(
        asked_at(reading.as_fd(), PROMPT, &mut told, Duration::from_secs(5)),
        TheirAnswer::TheTerminalWentAway
    );
    assert!(told.is_empty(), "a pipe was shown the prompt");
}

/// The meaning of no key at all.
#[test]
fn no_key_is_the_terminal_gone() {
    assert_eq!(the_answer_in(None), TheirAnswer::TheTerminalWentAway);
}
