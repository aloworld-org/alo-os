//! The one question with two answers, and how a person types either of them.
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! term 9: when Windows' Fast Startup is on, **the installer asks**, with *Turn
//! off* and *Leave on*. This installer speaks on a console — it has no buttons
//! — so each answer is a word the person types, and the sentence names both
//! words.
//!
//! # The words are the language's, and English is always understood too
//!
//! The two answers are sentences of their own
//! (`crate::words::ANSWER_TURN_OFF`, `ANSWER_LEAVE_ON`), so a translator gives
//! them in their language and the question, which names them, reads as one
//! piece. The source words are accepted as well, in every language: a person
//! whose keyboard cannot type their own language's word, or who is reading a
//! screenshot from somewhere else, still gets through. Nothing else is an
//! answer, and an answer nobody recognises is asked again rather than guessed
//! at — a wrong guess here changes a setting of the person's computer.

use alo_strings::{Filling, Strings, Word};

use crate::words;

/// What the person answered about Fast Startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Turn Fast Startup off.
    TurnOff,
    /// Leave Fast Startup on.
    LeaveOn,
}

/// The answer in what the person typed, or [`None`] when it is neither.
#[must_use]
pub fn answered(typed: &str, strings: &Strings) -> Option<Answer> {
    let typed = normalised(typed);
    if typed.is_empty() {
        return None;
    }
    for (word, answer) in [
        (words::ANSWER_TURN_OFF, Answer::TurnOff),
        (words::ANSWER_LEAVE_ON, Answer::LeaveOn),
    ] {
        if is_the_word(&typed, word, strings) {
            return Some(answer);
        }
    }
    None
}

/// Whether what was typed is this answer, in this language or in the source.
fn is_the_word(typed: &str, word: Word, strings: &Strings) -> bool {
    let in_this_language = strings.say(&word.key(), &Filling::nothing()).into_text();
    typed == normalised(&in_this_language) || typed == normalised(word.says())
}

/// A word, with its case and its white space made not to matter.
fn normalised(written: &str) -> String {
    written
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
    )]

    use super::*;

    /// This installer's own words, in the language they are written in.
    fn source() -> Strings {
        Strings::of(words::installer_words().unwrap())
    }

    /// **Both answers are read, whatever the case and spacing.**
    #[test]
    fn either_answer_is_read_as_the_person_typed_it() {
        for (typed, answer) in [
            ("turn off", Answer::TurnOff),
            ("  Turn   Off ", Answer::TurnOff),
            ("TURN OFF", Answer::TurnOff),
            ("leave on", Answer::LeaveOn),
            ("Leave On", Answer::LeaveOn),
        ] {
            assert_eq!(answered(typed, &source()), Some(answer), "{typed:?}");
        }
    }

    /// **Anything else is not an answer** — including a prefix of one, and the
    /// words a person might expect to work.
    #[test]
    fn nothing_else_is_an_answer() {
        for typed in ["", "   ", "turn", "off", "on", "yes", "no", "y", "leave"] {
            assert_eq!(answered(typed, &source()), None, "{typed:?}");
        }
    }
}
