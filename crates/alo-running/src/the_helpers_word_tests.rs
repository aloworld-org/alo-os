//! Hearing the helper's line among a command's error stream, and nothing
//! else.

use std::io::Write as _;

use super::{Listening, StartRefused, a_word_of_its_own, the_refusal_line};

/// `said`, written in pieces of `at_a_time` bytes, through a listener for
/// `word`: what reached the person, and what was heard.
fn heard(word: &str, said: &[u8], at_a_time: usize) -> (Vec<u8>, Option<StartRefused>) {
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
/// pieces, and what the command said around it is not touched.
#[test]
fn the_helpers_line_is_heard_and_not_shown() {
    let word = a_word_of_its_own();
    let line = the_refusal_line(&word, std::io::ErrorKind::PermissionDenied);
    for at_a_time in [1, 3, 64] {
        let (person, heard) = heard(&word, line.as_bytes(), at_a_time);
        assert_eq!(heard, Some(StartRefused::PermissionDenied), "{at_a_time}");
        assert!(person.is_empty(), "{:?}", String::from_utf8_lossy(&person));
    }
    let other = the_refusal_line(&word, std::io::ErrorKind::NotFound);
    assert_eq!(
        heard(&word, other.as_bytes(), 5).1,
        Some(StartRefused::Otherwise)
    );
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
