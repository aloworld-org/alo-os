//! Every string this crate can say, and the English beside each one.
//!
//! Three groups: what a person is told when an approved adapter verb could not
//! be carried out; everything the accessibility fallback says
//! ([`crate::fallback_words`]); and — declared by each adapter beside its own
//! verbs, and put into the vocabulary here — the words every loaded adapter's
//! verbs are made of ([`crate::text_editor::WORDS`]).
//!
//! # Nothing a person reads names the machinery
//!
//! Not the bus, not the interface, not the method and not the packaging. A
//! person approved *open a document in the text editor*; how it was asked is
//! not what they need, and a test at the bottom of this file reads every
//! sentence and every note for the words that would undo it.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

use crate::{fallback_words, text_editor};

/// The gap holding an application's identifier.
pub const APPLICATION: &str = "application";

/// The gap holding a verb's name.
pub const VERB: &str = "verb";

/// The application is not on the bus and could not be started.
pub const NOT_THERE: Word = Word::saying(
    "adapters.not-carried-out.not-there",
    "{application} could not be reached, so nothing was done in it",
)
.noting(
    "Said after a person approved something the agent asked an application to do, when the \
     application could not be found or started. {application} is its identifier, like \
     org.gnome.TextEditor, and is not translated. Nothing happened.",
);

/// The application does not offer what the adapter asked of it.
pub const DOES_NOT_OFFER: Word = Word::saying(
    "adapters.not-carried-out.does-not-offer",
    "{application} does not offer what was approved, so nothing was done in it. This release of \
     it may not be one this machine knows how to ask",
)
.noting(
    "Said when an application is running but has no way to do what was approved — usually a \
     newer or older release than the one this was written for. {application} is its identifier \
     and is not translated. Nothing happened.",
);

/// The application answered with a refusal.
pub const REFUSED_IT: Word = Word::saying(
    "adapters.not-carried-out.refused",
    "{application} refused what was approved, and nothing was done in it",
)
.noting(
    "Said when the application itself said no. {application} is its identifier and is not \
     translated.",
);

/// The application did not answer, so what happened is not known.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "adapters.not-carried-out.did-not-answer",
    "{application} did not answer in time, so whether it did what was approved is not known",
)
.noting(
    "Said when the request reached the application and no answer came back. It may have been \
     done; say so plainly rather than as a failure. {application} is its identifier and is not \
     translated.",
);

/// What was approved could not be put into the application's terms.
pub const NOT_BUILT: Word = Word::saying(
    "adapters.not-carried-out.not-built",
    "What was approved could not be put to {application}, so nothing was done in it",
)
.noting(
    "A fault in this machine's own description of the application, not in anything the person \
     did. {application} is its identifier and is not translated. Nothing happened.",
);

/// An authority for a verb no loaded adapter declares.
pub const NOT_AN_ADAPTERS_VERB: Word = Word::saying(
    "adapters.not-carried-out.not-an-adapters-verb",
    "{verb} is not something an application on this machine was asked to do, so nothing was sent",
)
.noting(
    "A fault in this machine rather than in anything the person did. {verb} is the name of a \
     thing an agent can do, like text_editor.open_document, and is not translated.",
);

/// Every word of this crate's own.
pub const EVERY_WORD: [Word; 6] = [
    NOT_THERE,
    DOES_NOT_OFFER,
    REFUSED_IT,
    DID_NOT_ANSWER,
    NOT_BUILT,
    NOT_AN_ADAPTERS_VERB,
];

/// Why this crate's own list would not declare.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the lists above cannot cause.
pub fn adapter_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say — its own words and every reference
/// adapter's — into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD
        .iter()
        .chain(text_editor::WORDS.iter())
        .chain(fallback_words::EVERY_WORD.iter())
    {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Everything this crate declares, as one list.
    fn everything() -> Vec<Word> {
        EVERY_WORD
            .iter()
            .chain(text_editor::WORDS.iter())
            .chain(fallback_words::EVERY_WORD.iter())
            .copied()
            .collect()
    }

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in everything() {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "adapters", "{}", word.named());
        }
        let named: BTreeSet<&str> = everything().iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), everything().len());
    }

    /// **Nothing a person reads names the machinery**: not the bus, the
    /// interface, the packaging or the accessibility layer.
    #[test]
    fn nothing_a_person_reads_names_the_machinery() {
        for word in everything() {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_lowercase();
            for machinery in [
                "d-bus",
                "dbus",
                "bus",
                "flatpak",
                "flathub",
                "at-spi",
                "interface",
                "method",
                "gapplication",
            ] {
                assert!(
                    !read.contains(machinery),
                    "{} names {machinery}: {read}",
                    word.named()
                );
            }
        }
    }

    /// Every string a translator is handed has a note.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in everything() {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// The list declares, and nothing is replaced if it is declared twice.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = adapter_words().unwrap();
        assert_eq!(vocabulary.how_many(), everything().len());
        assert!(matches!(
            declare_into(&mut vocabulary).unwrap_err(),
            WordsError::List(_)
        ));
        assert_eq!(vocabulary.how_many(), everything().len());
    }
}
