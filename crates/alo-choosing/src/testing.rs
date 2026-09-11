//! The strings this crate's tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops three files inventing three vocabularies that resemble the real
//! one.
//!
//! # It holds this crate's list and `alo-models`'
//!
//! **It used to hold this crate's alone**, on the argument that no sentence
//! here has somebody else's sentence inside it. That stopped being true with
//! `crate::NotWritten`: two of its six reasons are about the two **lists** a
//! settings file holds, those lists are `alo-models`', and their refusals are
//! carried in `alo-models`' own words rather than reworded — for the reason
//! `crate::NotSet::NotAProvider` already carries one, which is that two
//! accounts of one moment is one account too many.
//!
//! So the fixture gains that crate's list, because a fixture that answered half
//! of what this crate can say with a key would be a test of a vocabulary alo OS
//! does not have. It does not gain anything else: what a rule refused is said
//! in `alo_models::NotAllowed`'s own words by whoever shows it and this crate
//! adds nothing around it — `crate::bound` has the argument.
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

/// This crate's list and `alo-models`', which is everything this crate's own
/// sentences need.
fn everything_these_tests_need() -> Vocabulary {
    let mut vocabulary = choosing_words().unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// A folder on this machine's own disk that only this test uses.
///
/// The settings this crate writes are a real file, so the tests that write one
/// need somewhere real to put it. Named after what the test is about rather
/// than after a clock, so a failure leaves something a person can go and look
/// at — and made fresh each run, because a test that inherited the last run's
/// file would be measuring that instead.
pub(crate) fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-choosing-{what}"));
    // Whatever a previous run left is gone, so what a test measures is what it
    // wrote.
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
