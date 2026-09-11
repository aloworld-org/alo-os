//! The strings and the grants this crate's own tests are written against.
//!
//! Every file in this crate that says something has the same two questions to
//! answer — *what does this say on a machine with no translations* and *what
//! does it say when somebody has translated it* — and every file that reads
//! the list needs the same machine to read it from. Nothing here is compiled
//! into the crate: it exists under `cfg(test)` only.
//!
//! # The vocabulary is this crate's and `alo-capability`'s
//!
//! A row's *what* is worded by `alo-capability`, which is the crate that
//! decides what a grant covers. A fixture holding only this crate's list
//! would answer every row with a key in guillemets and the tests would still
//! pass, which is a fixture proving the tests rather than the code. So both
//! lists are declared, in the order `alo-saying` collects them.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{Grant, Grants, Reach};
use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::words::{Word, declare_into};

/// Everything this crate says, and everything it puts inside what it says.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    declare_into(&mut vocabulary).unwrap();
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
/// tests translation with.
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
fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// A fixed moment, so that everything about time here is arithmetic.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grant every test here reads lasts.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A machine on which one folder has been granted to `@files`, at noon, for
/// an hour — the ordinary machine these tests look at.
pub(crate) fn grants_of_one_folder() -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}
