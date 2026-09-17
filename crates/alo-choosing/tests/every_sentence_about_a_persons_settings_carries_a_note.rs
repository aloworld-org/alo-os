//! Every sentence the five crates that keep a person's settings can say is in
//! the machine's one vocabulary, with a note for whoever translates it.
//!
//! Each crate's own tests hold its list. This holds the promise the plan for
//! where a person's settings are kept makes across all five
//! (`docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`, task 4): a
//! person may open one of these files in an editor, so what they are told when
//! it did not read must reach them in their own language — and a translator
//! handed *your appearance settings at {path} say {key}* with no word about
//! what `{path}` and `{key}` are is a translator who may render a file name.
//!
//! It asks the vocabulary `alo-saying` collects rather than each crate's list,
//! because that is the one a translation is checked against and the one every
//! sentence a person reads comes out of. A sentence a crate declares and the
//! machine does not collect is a sentence nobody is ever told.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Phrase, Vocabulary};

/// The area at the front of each of the five crates' keys.
const THE_FIVE: [&str; 5] = ["appearance", "dock", "shortcuts", "choosing", "changing"];

/// The three crates that keep a file in the person's folder by `alo-kept`'s
/// rule, whose refusals share one shape of key.
const KEPT_BY_THE_RULE: [&str; 3] = ["appearance", "dock", "shortcuts"];

/// The machine's one vocabulary.
fn the_machines() -> Vocabulary {
    alo_saying::everything_this_machine_can_say().unwrap()
}

/// Every phrase under this area.
fn phrases_of<'a>(vocabulary: &'a Vocabulary, area: &'a str) -> impl Iterator<Item = &'a Phrase> {
    vocabulary
        .phrases()
        .filter(move |phrase| phrase.key().area() == area)
}

/// One phrase, by its whole key.
fn phrase<'a>(vocabulary: &'a Vocabulary, key: &str) -> &'a Phrase {
    vocabulary
        .phrase(&alo_strings::Key::named(key).unwrap())
        .unwrap_or_else(|| panic!("{key} is not collected by the machine"))
}

/// **Every sentence the five crates can say carries a translator's note**, and
/// none of them is a note in name only.
#[test]
fn every_sentence_the_five_crates_say_carries_a_note() {
    let vocabulary = the_machines();
    for area in THE_FIVE {
        let phrases: Vec<&Phrase> = phrases_of(&vocabulary, area).collect();
        assert!(
            !phrases.is_empty(),
            "{area} says nothing the machine collects"
        );
        for phrase in phrases {
            assert!(
                phrase.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                phrase.key()
            );
        }
        for plural in vocabulary
            .counted()
            .filter(|plural| plural.key().area() == area)
        {
            assert!(
                plural.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                plural.key()
            );
        }
    }
}

/// **The sentence for a file with a key nobody declared names the file and the
/// key**, in each of the three files kept by the rule, and its note tells a
/// translator that neither is a word to render.
#[test]
fn a_key_nobody_declared_is_said_with_the_file_and_the_key_named() {
    let vocabulary = the_machines();
    for area in KEPT_BY_THE_RULE {
        let unknown = phrase(&vocabulary, &format!("{area}.kept.unknown-key"));
        assert!(unknown.source().has("path"), "{}", unknown.key());
        assert!(unknown.source().has("key"), "{}", unknown.key());
        let note = unknown.note().unwrap_or_default();
        assert!(note.contains("{path}"), "{}: {note}", unknown.key());
        assert!(note.contains("{key}"), "{}: {note}", unknown.key());
        assert!(note.contains("translated"), "{}: {note}", unknown.key());
    }
}

/// **Every sentence about one of these files names the file.** *Your settings
/// could not be read* is a sentence a person cannot act on on a machine with
/// four files in their folder and several people signing in.
#[test]
fn every_sentence_about_a_file_names_which_file() {
    let vocabulary = the_machines();
    for area in KEPT_BY_THE_RULE {
        let kept: Vec<&Phrase> = phrases_of(&vocabulary, area)
            .filter(|phrase| {
                phrase
                    .key()
                    .to_string()
                    .starts_with(&format!("{area}.kept."))
            })
            .collect();
        assert_eq!(
            kept.len(),
            8,
            "{area}: read, not understood, at a line, another format, a key, not written, not expressible, not replaced"
        );
        for phrase in kept {
            assert!(phrase.source().has("path"), "{}", phrase.key());
        }
        let at_a_line = phrase(&vocabulary, &format!("{area}.kept.not-understood-at"));
        assert!(at_a_line.source().has("line"), "{}", at_a_line.key());
    }
    // Every `choosing` sentence but one is about a person's settings file and
    // names it. The exception is a session that has nowhere to keep a change
    // at all, said before any change is made: there is no file to name, and
    // the crate's own `is_about_the_file` holds the same exemption to the same
    // one key, so a second pathless sentence fails both.
    const SAID_BY_A_SESSION_WITH_NOWHERE_TO_KEEP_ANYTHING: &str = "choosing.session.no-folder";
    let mut exempted = 0_u8;
    for phrase in phrases_of(&vocabulary, "choosing") {
        if phrase.key().to_string() == SAID_BY_A_SESSION_WITH_NOWHERE_TO_KEEP_ANYTHING {
            exempted += 1;
            continue;
        }
        assert!(phrase.source().has("path"), "{}", phrase.key());
    }
    assert_eq!(
        exempted, 1,
        "the sentence a session with no folder says is no longer in the vocabulary"
    );
}
