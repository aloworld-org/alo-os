//! The strings this crate's tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops three files inventing three vocabularies that resemble the real
//! one.
//!
//! # It holds this crate's list and nothing else
//!
//! Which is only right while no sentence here has somebody else's sentence
//! inside it, and today none does: what a rule refused is said in
//! `alo_models::NotAllowed`'s own words by whoever shows it, and this crate
//! adds nothing around it — `crate::bound` has the argument. The day a sentence
//! here fills a gap with another crate's, this fixture gains that crate's list,
//! because a fixture that answered half a sentence with a key would be a test
//! of a vocabulary alo OS does not have.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

use crate::words::choosing_words;

/// This crate's words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything_these_tests_need())
}

/// The same, with these words translated into German and German preferred.
///
/// German for `alo-egress`' reason: what this crate says is sentences rather
/// than labels, and German moves the verb — so a translation that came out
/// reading like English with the words swapped would not be exercising
/// anything.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything_these_tests_need();
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

/// This crate's list, which is everything its own sentences need.
fn everything_these_tests_need() -> Vocabulary {
    choosing_words().unwrap()
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}
