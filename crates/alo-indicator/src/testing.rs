//! The strings and the departures this crate's own tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops the files inventing vocabularies that resemble the real one. The
//! real one is [`crate::indicator_words`], and both of these are built from it.
//! The shape is `alo-overlay`'s `testing.rs`, copied rather than re-decided.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # It is not only this crate's words
//!
//! A drawn indicator's lines are worded by `alo-egress`, because whoever
//! decided the egress is who knows what it was. A fixture holding only this
//! crate's list would answer every line with its own key in guillemets and the
//! tests about lines would still pass, which is a fixture proving the tests
//! rather than the code. So [`its_own_and_what_it_quotes`] is both lists, in
//! the order `alo-saying` collects them into one vocabulary for the machine.
//!
//! # And the departures are real
//!
//! The two egresses below are made the way a verb makes one, so that a test
//! about a lit lamp is a test about something a policy really permitted. There
//! is deliberately no fixture here for *a line that never was a departure*:
//! there is no way to write one.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::Grantee;
use alo_egress::{Destination, Leaving, Why};
use alo_models::{InferenceSource, Region};
use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::words::{Word, declare_into};

/// Everything this crate says, and everything it puts on a line beside it.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with, so a translator's file exercised here looks like the
/// one exercised everywhere else.
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

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// An agent fetching something from somewhere else.
pub(crate) fn fetching() -> Leaving {
    Leaving::because(
        &Grantee::named("@files"),
        Why::Fetching,
        Destination::at("alo.example").unwrap(),
    )
}

/// An agent putting a question to a provider, which is the departure the plan's
/// acceptance is written about.
pub(crate) fn asking_a_provider() -> Leaving {
    Leaving::asking(
        &Grantee::named("@mail"),
        &InferenceSource::Hosted {
            provider: "alo".to_owned(),
            region: Region::Declared("the EU".to_owned()),
        },
    )
    .unwrap()
}
