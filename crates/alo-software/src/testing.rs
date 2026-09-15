//! The strings this crate's own tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//! The vocabulary is this crate's and the two it puts words beside — the
//! egress indicator's lines and the capability model's sentences — so a test
//! reading one of those does not pass by answering with a key.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::words::{Word, declare_into};

/// Everything this crate says, and everything said beside it.
fn its_own_and_its_neighbours() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// Nothing translated: what a machine with no translations shows.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_its_neighbours())
}

/// Some of these words translated into German, and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own_and_its_neighbours();
    let german = Language::written("de").unwrap();
    let mut translation = Translation::into_language(german.clone());
    for (word, says) in words {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german]);
    strings
}
