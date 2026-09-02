//! The strings this crate's own tests are written against.
//!
//! Every file here that says something has the same two questions to answer —
//! *what does this say on a machine with no translations* and *what does it say
//! when somebody has translated it* — and answering them from one fixture is
//! what stops ten files inventing ten vocabularies that resemble the real one.
//! The real one is [`crate::appearance_words`], and it is what both of these are
//! built from.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::{Language, Strings, Translation, Word};

use crate::words::appearance_words;

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(appearance_words().unwrap())
}

/// The same, with these words translated into German and German preferred.
///
/// German because the colour names are the hard half of this crate's list and
/// German has an ordinary word for some of them and a borrowed one for others —
/// *Marineblau* beside *Terrakotta* — which is the thing the notes in
/// [`crate::words`] are about, rather than a language chosen for being easy to
/// type.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = appearance_words().unwrap();
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
