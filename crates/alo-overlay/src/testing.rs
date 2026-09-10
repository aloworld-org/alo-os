//! The strings this crate's own tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it
//! say when somebody has translated it* — and answering them from one fixture
//! is what stops the files inventing vocabularies that resemble the real one.
//! The real one is [`crate::overlay_words`], and both of these are built from
//! it. The shape is `alo-shortcuts`' `testing.rs`, copied rather than
//! re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # It is not only this crate's words any more
//!
//! What the overlay shows at rest puts *other crates'* clauses inside its own
//! lines — `alo-models` words where a question would be answered, and this
//! crate's line has a gap for it. A fixture holding only this crate's list
//! would answer that clause with its own key in guillemets and every test
//! below would still pass, which is a fixture proving the tests rather than
//! the code. So [`its_own_and_what_it_quotes`] is the two lists, in the order
//! `alo-saying` collects them into one vocabulary for the machine.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::words::{Word, declare_into};

/// Everything this crate says, and everything it puts inside what it says.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has
/// no translations of them shows, which is what most of these tests are
/// about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with, so a translator's file exercised here looks like
/// the one exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own_and_what_it_quotes();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}
