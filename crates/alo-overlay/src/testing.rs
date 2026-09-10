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

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation};

use crate::words::{Word, overlay_words};

/// This crate's own words, with nothing translated: what a machine that has
/// no translations of them shows, which is what most of these tests are
/// about.
pub(crate) fn in_english() -> Strings {
    Strings::of(overlay_words().unwrap())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with, so a translator's file exercised here looks like
/// the one exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = overlay_words().unwrap();
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
