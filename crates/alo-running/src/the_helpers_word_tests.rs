//! Hearing the helper's line among a command's error stream, and nothing
//! else — and that a refusal the runner can name puts nothing else in front of
//! the person.

use std::io::Write as _;

use super::{
    Listening, StartRefused, TheHelperSaid, a_word_of_its_own, the_refusal_line,
    what_the_helper_writes,
};

/// The account the helper gave on a booted machine on 2026-11-08.
const ACCOUNT: &str = "/usr/bin/ls did not start inside /sys/fs/cgroup/user.slice/alo-try: the box \
                       was joined, and from inside it /usr/bin/ls could not be started: \
                       Permission denied (os error 13)";

/// `said`, written in pieces of `at_a_time` bytes, through a listener for
/// `word`: what reached the person, and what was heard.
fn heard(word: &str, said: &[u8], at_a_time: usize) -> (Vec<u8>, Option<TheHelperSaid>) {
    let mut person: Vec<u8> = Vec::new();
    let heard = {
        let mut listening = Listening::for_the_word(word, &mut person);
        for piece in said.chunks(at_a_time) {
            listening.write_all(piece).ok();
        }
        listening.heard()
    };
    (person, heard)
}

/// **The helper's line is heard and kept from the person**, whole or in
/// pieces, with its account; and what the command said around it is not
/// touched.
#[test]
fn the_helpers_line_is_heard_and_not_shown() {
    let word = a_word_of_its_own();
    let line = the_refusal_line(&word, std::io::ErrorKind::PermissionDenied, ACCOUNT);
    for at_a_time in [1, 3, 64] {
        let (person, heard) = heard(&word, line.as_bytes(), at_a_time);
        assert_eq!(
            heard,
            Some(TheHelperSaid {
                refused: StartRefused::PermissionDenied,
                account: ACCOUNT.to_owned(),
            }),
            "{at_a_time}"
        );
        assert!(person.is_empty(), "{:?}", String::from_utf8_lossy(&person));
    }
    let other = the_refusal_line(&word, std::io::ErrorKind::NotFound, "gone");
    assert_eq!(
        heard(&word, other.as_bytes(), 5).1.map(|said| said.refused),
        Some(StartRefused::Otherwise)
    );
}

/// **A line with no account is still heard**, as the kind alone.
#[test]
fn a_line_with_no_account_is_still_heard() {
    let word = a_word_of_its_own();
    let (person, heard) = heard(&word, format!("{word} denied\n").as_bytes(), 4);
    assert!(person.is_empty());
    assert_eq!(
        heard,
        Some(TheHelperSaid {
            refused: StartRefused::PermissionDenied,
            account: String::new(),
        })
    );
}

/// **An account cannot break out of its line**: a line break in it — a
/// program's name may hold one — becomes a space, so nothing after it reaches
/// the person.
#[test]
fn an_account_cannot_break_out_of_its_line() {
    let word = a_word_of_its_own();
    let line = the_refusal_line(
        &word,
        std::io::ErrorKind::PermissionDenied,
        "/tmp/a\nshown to the person\r\n",
    );
    assert_eq!(line.matches('\n').count(), 1, "{line:?}");
    let (person, heard) = heard(&word, line.as_bytes(), 2);
    assert!(person.is_empty(), "{:?}", String::from_utf8_lossy(&person));
    assert_eq!(
        heard.map(|said| said.account),
        Some("/tmp/a shown to the person  ".to_owned())
    );
}

/// **Given a word and a joined box, the helper writes its line and nothing
/// else**, so everything it writes is kept from the person. This is what the
/// walk of 2026-11-08 found missing: a second, English line after the word's.
#[test]
fn a_joined_refusal_writes_only_the_word_line() {
    let word = a_word_of_its_own();
    let written = what_the_helper_writes(
        Some(&word),
        Some(std::io::ErrorKind::PermissionDenied),
        ACCOUNT,
    );
    let (person, heard) = heard(&word, written.as_bytes(), 7);
    assert!(
        person.is_empty(),
        "reached the person: {:?}",
        String::from_utf8_lossy(&person)
    );
    assert_eq!(heard.map(|said| said.account), Some(ACCOUNT.to_owned()));
}

/// **The refusal path: a refusal nobody listens for is still said**, as an
/// ordinary line — with no word, or before the box was joined — and so reaches
/// whoever is reading.
#[test]
fn a_refusal_nobody_listens_for_is_still_said() {
    let word = a_word_of_its_own();
    for written in [
        what_the_helper_writes(None, Some(std::io::ErrorKind::PermissionDenied), ACCOUNT),
        what_the_helper_writes(Some(&word), None, ACCOUNT),
    ] {
        assert_eq!(written, format!("alo-enter-the-box: {ACCOUNT}\n"));
        let (person, heard) = heard(&word, written.as_bytes(), 9);
        assert_eq!(person, written.as_bytes());
        assert_eq!(heard, None);
    }
}

/// **A command's own error stream reaches the person byte for byte**, and a
/// command that writes something like the line — without the word it was
/// never given — is not heard as the helper.
#[test]
fn a_commands_own_words_pass_and_cannot_pose_as_the_helper() {
    let word = a_word_of_its_own();
    let theirs = b"warning: careful\nalo-enter-the-box-0000 denied\npartial";
    for at_a_time in [1, 7, 100] {
        let (person, heard) = heard(&word, theirs, at_a_time);
        assert_eq!(person, theirs.to_vec(), "{at_a_time}");
        assert_eq!(heard, None);
    }
}

/// **Two runs do not share a word.**
#[test]
fn every_run_has_a_word_of_its_own() {
    assert_ne!(a_word_of_its_own(), a_word_of_its_own());
}
