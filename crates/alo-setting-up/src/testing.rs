//! The strings and the fixtures this crate's tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops four files inventing four vocabularies that resemble the real one.
//!
//! # It holds this crate's list and `alo-choosing`'s
//!
//! One of `crate::NotSetUp`'s six reasons is the person's own settings refusing
//! the change, carried in `alo-choosing`'s own words rather than reworded — so
//! a fixture holding only this crate's list would answer a sixth of what this
//! crate can say with a key, and would be a test of a vocabulary alo OS does
//! not have.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use alo_models::{Provider, Region};
use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

use crate::words::setting_up_words;

/// This crate's words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything_these_tests_need())
}

/// The same, with these words translated into German and German preferred.
///
/// German for `alo-choosing`'s reason: what this crate says is sentences rather
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

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// This crate's list and `alo-choosing`'s, which is everything this crate's own
/// sentences need.
fn everything_these_tests_need() -> Vocabulary {
    let mut vocabulary = setting_up_words().unwrap();
    alo_choosing::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// One provider, built the way a setup surface builds one.
pub(crate) fn a_provider(name: &str) -> Provider {
    Provider::checked(
        name,
        "https://api.mistral.ai",
        Region::Declared("the EU".to_owned()),
        Some(alo_models::SecretRef::named(&format!("provider/{name}"))),
    )
    .unwrap()
}

/// A folder on this machine's own disk that only this test uses.
///
/// The settings setup writes are a real file, so the tests that answer setup
/// need somewhere real to put one. Named after what the test is about rather
/// than after a clock, so a failure leaves something a person can go and look
/// at — and made fresh each run, because a test that inherited the last run's
/// file would be measuring that instead.
pub(crate) fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-setting-up-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
