//! The strings this crate's own tests are written against.
//!
//! The shape is `alo-clipboard`'s `testing.rs`, copied rather than re-decided.
//! The files those tests look at are built in `tests/making/`, beside the
//! integration tests that write them to a real disk: a rule about a zip or a
//! directory is only as good as the file it was shown, and one builder for
//! both is what stops two slightly different fixtures from passing two
//! slightly different rules.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::words::{Word, declare_into, opening_words};

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(opening_words().unwrap())
}

/// The same, with some of them translated into German and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
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
