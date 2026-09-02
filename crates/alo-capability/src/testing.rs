//! The strings this crate's own tests are written against.
//!
//! Every file here that refuses something has the same two questions to
//! answer — *what does this say on a machine with no translations* and *what
//! does it say when somebody has translated it* — and answering them from one
//! fixture is what stops eight files inventing eight vocabularies that resemble
//! the real one. The real one is [`crate::capability_words`], and all of these
//! are built from it.
//!
//! **Some of them hold more than this crate's own words**, and that is the
//! arrangement a shell has: one vocabulary, one area per crate. Since item 9g a
//! verb is declared from words, so a test that declares a verb declares its
//! strings too, and a fixture holding only this crate's would show the key where
//! the sentence a person approves should be.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

use crate::words::capability_words;

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    speaking(&[])
}

/// The same, with a test's own words beside this crate's.
pub(crate) fn speaking(extra: &[Word]) -> Strings {
    Strings::of(everything(extra))
}

/// The same, with these words translated into German and German preferred.
///
/// German because the refusals here are sentences rather than labels, and
/// German moves the verb — so a translation that came out reading like English
/// with the words swapped would not be exercising anything. It also writes
/// quotation marks its own way, which one of these strings is about.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    translating(&[], words)
}

/// The same again, with a test's own words in the vocabulary as well.
pub(crate) fn translating(extra: &[Word], words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything(extra);
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

/// This crate's words and a test's own, in one vocabulary.
fn everything(extra: &[Word]) -> Vocabulary {
    let mut vocabulary = capability_words().unwrap();
    for word in extra {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    vocabulary
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}
