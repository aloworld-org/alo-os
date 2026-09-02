//! One string a crate declares it can say, written as a literal in that crate.
//!
//! A [`Phrase`] is what the lookup holds. A [`Word`] is what a crate *writes*:
//! a key, the English beside it, and the note a translator needs — three
//! `&'static str`s in a `const`, so that the whole of what a crate can say is a
//! list somebody can read down in one file rather than a set of calls spread
//! through the code that makes them.
//!
//! **It is here because it was written three times.** `alo-files` declared it
//! in item 9b and `alo-shortcuts` in item 9c, with the same four fields and the
//! same [`Key::unchecked`], and two copies were deliberately not treated as a
//! pattern. `alo-appearance` is the third, and a third copy is one — so the type
//! moved here rather than being written again. Nobody's constants moved with it;
//! only where the type they are is written down.
//!
//! # What a `Word` is not
//!
//! **It is not checked when it is made.** [`Word::key`] goes through
//! [`Key::unchecked`], because a key written as a literal in a crate's own
//! source cannot arrive from a file and the alternative is every constant being
//! a `Result` that the calling code has to invent a fallback for — and there is
//! no honest fallback, since a sentence that could not be looked up is a
//! sentence nobody can read. What holds it to the rule instead is a test in each
//! crate that puts every one of its own words back through [`Key::named`], the
//! same shape as `alo-shortcuts` putting its shipped bindings back through
//! `Chord::checked`.
//!
//! **It is not a vocabulary.** [`Word::phrase`] turns one into the thing the
//! lookup accepts; collecting them and handing them to a [`crate::Vocabulary`]
//! is the declaring crate's, because only that crate knows which of its strings
//! count something and what its own list is called.

use crate::key::Key;
use crate::phrase::{Phrase, PhraseError};
use crate::template::TemplateError;

/// One string a crate can say.
///
/// The key and the English live together, because a key with its sentence
/// somewhere else is two files to change and one of them will be forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word {
    /// What names it.
    named: &'static str,
    /// What it says in the language the code is written in.
    says: &'static str,
    /// What a translator needs to know that the sentence does not tell them.
    note: Option<&'static str>,
}

impl Word {
    /// A string a crate can say: this key says this, in English.
    #[must_use]
    pub const fn saying(named: &'static str, says: &'static str) -> Self {
        Self {
            named,
            says,
            note: None,
        }
    }

    /// The same string, with something a translator has to be told.
    ///
    /// A note is for the strings that need a translator's judgement rather than
    /// their typing — a colour with no ordinary name in their language, a gap
    /// that arrives already translated, a word that must not be translated at
    /// all. A note nobody wrote is a sentence somebody guesses at.
    #[must_use]
    pub const fn noting(self, note: &'static str) -> Self {
        Self {
            note: Some(note),
            ..self
        }
    }

    /// What names it.
    ///
    /// [`Key::unchecked`], for the reason written at the top of this module: the
    /// test that every one of a crate's own keys is a key lives in that crate,
    /// beside the keys.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }

    /// The key as it was written, for a test that has to name it in a failure.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// What it says in the language the code is written in.
    ///
    /// This is also what a declaration elsewhere is built from — `alo-files`
    /// declares its six verbs out of these, so that the sentence a person
    /// approves and the sentence a translator is handed cannot be two different
    /// sentences.
    #[must_use]
    pub const fn says(&self) -> &'static str {
        self.says
    }

    /// What a translator needs to know that the sentence does not tell them.
    #[must_use]
    pub const fn note(&self) -> Option<&'static str> {
        self.note
    }

    /// This word as the thing a [`crate::Vocabulary`] accepts.
    ///
    /// # Errors
    ///
    /// [`WordError`], which a list a crate wrote itself cannot cause — the tests
    /// beside that list are what say so.
    pub fn phrase(&self) -> Result<Phrase, WordError> {
        let phrase = Phrase::says(self.key(), self.says)?;
        match self.note {
            Some(note) => Ok(phrase.noting(note)?),
            None => Ok(phrase),
        }
    }
}

/// Why a word a crate wrote is not a phrase.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum WordError {
    /// A sentence that is not one.
    #[error(transparent)]
    Sentence(#[from] TemplateError),
    /// A note that could not be attached.
    #[error(transparent)]
    Note(#[from] PhraseError),
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// One of the words this repository actually has: a colour name, which is
    /// the shape a note exists for.
    const TERRACOTTA: Word = Word::saying("appearance.token.terracotta", "Terracotta")
        .noting("The colour of fired clay: an orange-brown.");

    /// One with a gap in it, which is the other shape.
    const TOO_SMALL: Word = Word::saying(
        "appearance.text.too-small",
        "{percent}% is smaller than this screen can be read at",
    );

    #[test]
    fn a_word_carries_its_key_its_english_and_its_note() {
        assert_eq!(
            TERRACOTTA.key(),
            Key::named("appearance.token.terracotta").unwrap()
        );
        assert_eq!(TERRACOTTA.named(), "appearance.token.terracotta");
        assert_eq!(TERRACOTTA.says(), "Terracotta");
        assert!(
            TERRACOTTA
                .note()
                .is_some_and(|note| note.contains("fired clay"))
        );
        assert_eq!(TOO_SMALL.note(), None);
    }

    /// The whole point of the type: it becomes the thing the lookup holds, with
    /// its note and its gaps intact.
    #[test]
    fn a_word_becomes_the_phrase_the_lookup_holds() {
        let phrase = TERRACOTTA.phrase().unwrap();
        assert_eq!(phrase.key(), &TERRACOTTA.key());
        assert_eq!(phrase.source().as_written(), "Terracotta");
        assert_eq!(
            phrase.note(),
            Some("The colour of fired clay: an orange-brown.")
        );

        let with_a_gap = TOO_SMALL.phrase().unwrap();
        assert_eq!(with_a_gap.source().gaps(), ["percent"]);
        assert_eq!(with_a_gap.note(), None);
    }

    /// A word that is not a sentence is refused where it is turned into one,
    /// which is what the test beside each crate's list exercises.
    #[test]
    fn a_word_with_nothing_in_it_is_refused() {
        let empty = Word::saying("appearance.token.navy", "");
        assert!(matches!(empty.phrase(), Err(WordError::Sentence(_))));

        let blank_note = Word::saying("appearance.token.navy", "Navy").noting("   ");
        assert!(matches!(blank_note.phrase(), Err(WordError::Note(_))));
    }
}
