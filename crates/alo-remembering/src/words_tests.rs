//! What this crate's ten sentences are held to, and the two things about them
//! that are easy to get wrong: a key that is not a key, and a sentence with a
//! hole nothing fills.

use super::*;
use alo_strings::{Filling, Key, Strings, Vocabulary};
use std::collections::BTreeSet;

/// What we ship is held to the rule everybody else is held to: `Word::key`
/// does not check, because a key written in this file cannot arrive from
/// anywhere, and this is the test that makes that true.
#[test]
fn every_key_is_a_key() {
    for word in EVERY_WORD {
        assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
    }
}

/// A key names one string.
#[test]
fn no_two_words_are_named_the_same() {
    let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
    assert_eq!(named.len(), EVERY_WORD.len());
}

/// Every one of them is in this crate's own area, which is what lets one
/// vocabulary hold every crate's strings without a collision.
#[test]
fn everything_this_crate_says_says_it_is_this_crate() {
    for word in EVERY_WORD {
        assert_eq!(word.key().area(), "remembering", "{}", word.named());
    }
}

/// The list declares, and nothing about it is refused by the crate that
/// receives it.
#[test]
fn the_whole_list_declares() {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).expect("the list declares");
    assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
}

/// **Every hole in every sentence is one something fills.**
///
/// The fault this is for: a sentence written with `{mode}` in it, where
/// `said()` fills only `path`, reaches a person with a brace in the middle of
/// it. Nothing else catches that — the word compiles, the key is valid, the
/// vocabulary takes it, and the sentence is only wrong at the moment somebody
/// reads it.
///
/// So this asserts the whole list against the two names `said()` can fill, and
/// names which word broke it rather than that one did.
#[test]
fn every_hole_is_one_that_gets_filled() {
    const FILLED: [&str; 2] = ["path", "said"];
    for word in EVERY_WORD {
        let says = word.says();
        let mut rest = says;
        while let Some(open) = rest.find('{') {
            let after = &rest[open + 1..];
            let close = after
                .find('}')
                .unwrap_or_else(|| panic!("{} has an unclosed {{ in {says:?}", word.named()));
            let name = &after[..close];
            assert!(
                FILLED.contains(&name),
                "{} says {{{name}}}, which nothing in `said()` fills — it would reach a person \
                 with the brace still in it",
                word.named()
            );
            rest = &after[close + 1..];
        }
    }
}

/// **The two endings, and no word with both or neither.**
///
/// Six are about reading and end *does not know how you want code run*. Four
/// are about writing and end *has not been changed*. A word with neither is a
/// sentence that does not say what follows for the person, which is the half
/// they act on.
#[test]
fn every_sentence_says_what_follows_from_it() {
    const READING: &str = "does not know how you want code run";
    const WRITING: &str = "has not been changed";
    let mut reading = 0;
    let mut writing = 0;
    for word in EVERY_WORD {
        let says = word.says();
        let is_reading = says.contains(READING);
        let is_writing = says.contains(WRITING);
        assert!(
            is_reading != is_writing,
            "{} ends neither way, or both: {says:?}",
            word.named()
        );
        if is_reading {
            reading += 1;
        } else {
            writing += 1;
        }
    }
    assert_eq!(
        (reading, writing),
        (8, 2),
        "the split moved; the module header says which sentences are which"
    );
}

/// Every one has a note for whoever translates it.
///
/// A sentence with no note is a sentence a translator has to guess the subject
/// of, and most of these are about a file nobody opened.
#[test]
fn every_word_tells_a_translator_what_it_is_about() {
    for word in EVERY_WORD {
        assert!(
            word.note().is_some_and(|note| note.len() > 40),
            "{} has no usable note for a translator",
            word.named()
        );
    }
}

/// **A `Strings` that was never given these answers with the key, marked** —
/// so a surface that forgot to declare them shows something obviously wrong
/// rather than something plausibly wrong.
#[test]
fn a_vocabulary_without_these_marks_the_key_rather_than_inventing_english() {
    let strings = Strings::of(Vocabulary::empty());
    let said = strings.say(
        &LEVEL_NOT_READ.key(),
        &Filling::of("path", "/var/lib/alo/x.toml"),
    );
    assert!(
        said.text().contains("remembering.level.not-read"),
        "an undeclared key read as {:?}, which does not name itself",
        said.text()
    );
}
