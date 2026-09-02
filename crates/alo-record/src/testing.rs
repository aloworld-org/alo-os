//! The strings the record's own tests are written against.
//!
//! A refusal reaches the record as the value it was, and
//! [`crate::Entry::refused`] asks it for words. What it asks with, here, is the
//! vocabulary the refusals came from — `alo-capability`'s list and
//! `alo-egress`' — because that is the arrangement on a real machine: one
//! vocabulary, every crate's strings in it, one rendering shown to the person
//! and written down.
//!
//! Two crates rather than one since item 9h, and the second is the reason this
//! fixture is worth having: [`crate::Entry::held_back`] renders a refusal the
//! egress policy made, and a record that rendered it against a vocabulary
//! missing those words would keep a key where the person read a sentence.
//!
//! Since item 9g it also holds the fixture verbs' own words, because a record
//! now renders the sentence a person approved rather than keeping a copy of it.
//! On a real machine that is `alo-files`' list; here it is
//! [`crate::test_calls`]'.
//!
//! A file of its own rather than more of [`crate::test_calls`], which is the
//! afternoon those tests are about: calls, grants and departures. What the
//! machine can say changes for a different reason.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Word};

/// Every string a record renders from, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything())
}

/// The same, with these said in German and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything();
    let mut into_german = Translation::into_language(german());
    for (word, says) in words {
        into_german = into_german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(into_german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german() -> Language {
    Language::written("de").unwrap()
}

/// The vocabulary a record renders against here: two crates' refusals, and the
/// fixture verbs' own words.
fn everything() -> alo_strings::Vocabulary {
    let mut vocabulary = alo_capability::capability_words().unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    for word in crate::test_calls::THE_WORDS {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    vocabulary
}
