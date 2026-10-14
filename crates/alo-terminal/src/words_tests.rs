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
    assert_eq!(EVERY_WORD.len(), 9);
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

/// **The two groups together are the whole list**, so a word added to the file
/// and to neither group is caught here rather than going unchecked.
#[test]
fn the_two_groups_are_the_whole_list() {
    assert_eq!(
        IN_THE_GUTTER.len() + ABOUT_A_COMMAND_THAT_WAITS.len(),
        EVERY_WORD.len(),
        "a word is declared and in neither group, so no rule is applied to it"
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
