//! The list is held to the things a list like this gets wrong: a key that
//! means two strings, a word declared and left out of the array, a note
//! missing from the one word a translator cannot work out alone.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::*;

/// The whole list declares, and nothing about it is refused.
#[test]
fn the_whole_list_declares() {
    let vocabulary = terminal_words().unwrap();
    assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
}

/// A vocabulary that already holds one of these keeps its own.
#[test]
fn a_key_already_taken_is_not_replaced() {
    let mut vocabulary = terminal_words().unwrap();
    let again = declare_into(&mut vocabulary).unwrap_err();
    assert!(matches!(again, WordsError::List(_)), "{again}");
}

/// **Every word in the file is in the array.**
///
/// The array is what [`declare_into`] walks, so a word declared above it and
/// left out is a string nothing can look up — which looks exactly like a
/// missing translation at the far end.
#[test]
fn every_word_declared_is_in_the_array() {
    for word in IN_THE_GUTTER.into_iter().chain(ABOUT_A_COMMAND_THAT_WAITS) {
        assert!(
            EVERY_WORD
                .iter()
                .any(|listed| listed.named() == word.named()),
            "{} is declared and not in EVERY_WORD",
            word.named()
        );
    }
    assert_eq!(EVERY_WORD.len(), 13);
}

/// **No key appears twice.**
#[test]
fn no_key_is_used_twice() {
    for (one, word) in EVERY_WORD.iter().enumerate() {
        for (other, against) in EVERY_WORD.iter().enumerate() {
            assert!(
                one == other || word.named() != against.named(),
                "{} is declared twice",
                word.named()
            );
        }
    }
}

/// Everything this crate says, by what constrains it.
///
/// **The gutter's labels are one word and the rest are sentences**, and the
/// difference is not a style preference: the design gives a gutter label 80 of
/// the gutter's 96, and a phrase has nowhere to go. A sentence answering *why
/// is it waiting* is read in a panel and is under no such rule.
const IN_THE_GUTTER: [Word; 4] = [A_PROGRAM, ALO, THE_SYSTEM, THE_PERSON];

/// The sentences said about a command that looks stuck.
const ABOUT_A_COMMAND_THAT_WAITS: [Word; 5] = [
    WAITING_FOR_THE_KEYBOARD,
    WAITING_FOR_THE_NETWORK,
    WAITING_FOR_ANOTHER_PROCESS,
    STILL_COMPUTING,
    NOBODY_CAN_TELL,
];

/// The sentences said about a command **nobody has run yet**.
///
/// Three marks and the disclaimer that goes with every one of them. They are
/// under a rule the group above is not: each describes *what an answer claimed*
/// rather than what this machine observed, so none of them may read as a
/// verdict — which is what their notes are mostly about.
const ABOUT_A_COMMAND_NOBODY_HAS_RUN: [Word; 4] = [
    IT_DESTROYS_SOMETHING,
    IT_NEEDS_MORE_THAN_YOUR_RIGHTS,
    SOMETHING_LEAVES_THIS_MACHINE,
    AN_EXPLANATION_AND_NOT_A_GUARANTEE,
];

/// **Each gutter label is one word**, because the design gives it 80 of the
/// gutter's 96 and a phrase has nowhere to go.
///
/// Held in English, where it is checkable. A translation is checked by the
/// person writing it, which is what the note on each word is for.
#[test]
fn each_gutter_label_is_a_single_word() {
    for word in IN_THE_GUTTER {
        assert!(
            !word.says().contains(' '),
            "{} is {:?}, which is more than one word",
            word.named(),
            word.says()
        );
    }
}

/// **Nothing this crate says is empty**, label or sentence.
#[test]
fn nothing_this_crate_says_is_empty() {
    for word in EVERY_WORD {
        assert!(!word.says().is_empty(), "{} says nothing", word.named());
    }
}

/// **The groups together are the whole list**, so a word added to the file and
/// to none of them is caught here rather than going unchecked.
///
/// *It was two groups until the explanation words landed, and this test is the
/// reason the third group exists rather than the four words going unruled.*
#[test]
fn the_groups_are_the_whole_list() {
    assert_eq!(
        IN_THE_GUTTER.len()
            + ABOUT_A_COMMAND_THAT_WAITS.len()
            + ABOUT_A_COMMAND_NOBODY_HAS_RUN.len(),
        EVERY_WORD.len(),
        "a word is declared and in no group, so no rule is applied to it"
    );
}

/// **No mark reads as a verdict this machine reached**, which is the one rule
/// the explanation words have and the waiting words do not.
///
/// A mark says what an answer claimed. Held in English, where it is checkable:
/// a translation is checked by the person writing it, which is what the note on
/// each word is for. The words to catch are the ones that would turn a claim
/// into a finding — `warning`, `danger`, `unsafe`, `blocked` — and the test
/// names them rather than describing them, so it can fail.
#[test]
fn no_mark_reads_as_a_verdict() {
    for word in ABOUT_A_COMMAND_NOBODY_HAS_RUN {
        let said = word.says().to_lowercase();
        for verdict in ["warning", "danger", "unsafe", "forbidden", "blocked"] {
            assert!(
                !said.contains(verdict),
                "{} says {:?}, and {:?} makes a model's claim read as this machine's finding",
                word.named(),
                word.says(),
                verdict
            );
        }
    }
}

/// **The disclaimer is short**, because it is shown under every explanation and
/// a long one becomes noise a person stops reading.
///
/// Its own note says so, and a note is not a check. Sixty characters is the
/// bound in English; a translation is the translator's to keep short, which is
/// what the note asks them for.
#[test]
fn the_disclaimer_is_short_enough_to_be_read_every_time() {
    let said = AN_EXPLANATION_AND_NOT_A_GUARANTEE.says();
    assert!(
        said.len() <= 60,
        "the disclaimer is {} characters: {said:?}",
        said.len()
    );
}

/// **Every one carries a note.**
///
/// None of these four can be translated from the word alone: `program` must
/// not become a word that could also describe alo, `alo` must not be
/// translated at all, `system` must not become *automatic*, and `you` is the
/// second person addressed to the reader. A translator with no product in
/// front of them cannot know any of that.
#[test]
fn every_one_carries_a_note_a_translator_needs() {
    for word in EVERY_WORD {
        assert!(word.note().is_some(), "{} has no note", word.named());
    }
}

/// **They are the terminal's, and say so in the key.**
///
/// One area at the front of every key, which is what lets the shell hold one
/// vocabulary that every crate adds to without two crates colliding.
#[test]
fn every_key_is_the_terminals_own() {
    for word in EVERY_WORD {
        assert!(
            word.named().starts_with("terminal."),
            "{} is not under this crate's own area",
            word.named()
        );
    }
}
