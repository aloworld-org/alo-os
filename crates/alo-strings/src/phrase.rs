//! One string the running code can say: its key, its English, and anything a
//! translator needs to know about it.
//!
//! **The English lives here, beside the key, and that is not the bug
//! `CLAUDE.md` calls hardcoded English.** Hardcoded English is English that
//! reaches a screen without anything ever having asked whether there is a
//! translation. A phrase is the opposite: the key is what the code asks for,
//! the English is what is shown when nobody has translated it yet, and
//! [`crate::Said`] always says which of the two happened. Keeping the source
//! beside the key also means a phrase cannot exist without English, so there is
//! no state where the code asks for something no catalogue has ever heard of.
//!
//! It is the same shape `alo-shortcuts` and `alo-appearance` settled on: only
//! the difference is stored. There, the defaults are in the code and the file
//! holds what a person changed. Here, the source is in the code and a file
//! holds what a translator changed — which is what lets a release improve an
//! English sentence for every machine that has no translation of it.
//!
//! **A note is for the strings that need a translator's judgement rather than
//! their typing.** `docs/autonomy/QUEUE.md` names two outright: a key is
//! labelled with what is printed on the person's own keyboard, and several
//! languages have no ordinary word for terracotta — the word a translator
//! reaches for may not be the colour. A note is where that is said, once, to
//! everybody who will ever translate it.

use serde::Serialize;

use crate::key::Key;
use crate::template::{Template, TemplateError};

/// One string the code can say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phrase {
    /// What the code asks for.
    key: Key,
    /// The English, which is what is shown when nothing has been translated.
    source: Template,
    /// What a translator needs to know that the sentence does not say.
    note: Option<String>,
}

impl Phrase {
    /// A phrase: this key says this, in English.
    ///
    /// # Errors
    ///
    /// [`TemplateError`] when the sentence is not one — empty, or with a gap in
    /// it that a translator could not move.
    pub fn says(key: Key, source: &str) -> Result<Self, TemplateError> {
        Ok(Self {
            key,
            source: Template::written(source)?,
            note: None,
        })
    }

    /// The same phrase, with something said to whoever translates it.
    ///
    /// # Errors
    ///
    /// [`PhraseError::EmptyNote`] for a note with nothing in it, because a note
    /// nobody wrote is worse than no note: a translator reads it, learns
    /// nothing, and trusts the next one less.
    pub fn noting(mut self, note: &str) -> Result<Self, PhraseError> {
        if note.trim().is_empty() {
            return Err(PhraseError::EmptyNote {
                key: self.key.clone(),
            });
        }
        self.note = Some(note.trim().to_owned());
        Ok(self)
    }

    /// What the code asks for.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// The English.
    #[must_use]
    pub fn source(&self) -> &Template {
        &self.source
    }

    /// What a translator was told, if anything.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// Why something is not a phrase.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PhraseError {
    /// A note with nothing in it.
    #[error(
        "write the note on {key} or leave it off — an empty one is read by every translator, tells them nothing, and costs the next note its credibility"
    )]
    EmptyNote {
        /// The phrase it was put on.
        key: Key,
    },
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A phrase for one of the awkward strings `docs/autonomy/QUEUE.md` names:
    /// a colour whose name is a translator's judgement.
    fn terracotta() -> Phrase {
        Phrase::says(
            Key::named("appearance.token.terracotta").unwrap(),
            "Terracotta",
        )
        .unwrap()
    }

    #[test]
    fn a_phrase_carries_its_key_and_its_english() {
        let phrase = terracotta();
        assert_eq!(phrase.key().as_str(), "appearance.token.terracotta");
        assert_eq!(phrase.source().as_written(), "Terracotta");
        assert_eq!(phrase.note(), None);
    }

    #[test]
    fn a_phrase_can_have_gaps_and_they_are_the_templates() {
        let phrase = Phrase::says(
            Key::named("files.too-big").unwrap(),
            "{path} holds {bytes} bytes and a verb reads at most {most}",
        )
        .unwrap();
        assert_eq!(phrase.source().gaps(), ["path", "bytes", "most"]);
    }

    /// The note is the answer to *this word needs a decision, not a
    /// dictionary*. Both of the strings the queue singles out are here.
    #[test]
    fn a_note_says_what_the_sentence_cannot() {
        let colour = terracotta()
            .noting(
                "The colour of fired clay, an orange-brown. Several languages have no ordinary \
                 word for it; the nearest one may not be the colour, so describe it rather than \
                 borrowing it.",
            )
            .unwrap();
        assert!(colour.note().unwrap().contains("fired clay"));

        let key = Phrase::says(Key::named("shortcuts.modifier.super").unwrap(), "Super")
            .unwrap()
            .noting("Whatever this key is called on the keyboard most of your readers have.")
            .unwrap();
        assert!(key.note().is_some());
    }

    #[test]
    fn a_note_is_trimmed_and_an_empty_one_is_refused() {
        assert_eq!(
            terracotta().noting("  fired clay  ").unwrap().note(),
            Some("fired clay")
        );
        assert!(matches!(
            terracotta().noting("   "),
            Err(PhraseError::EmptyNote { .. })
        ));
    }

    /// A phrase cannot exist without English, so the lookup has no state where
    /// the code asked for something nothing has ever heard of — except a key
    /// nobody declared, which [`crate::Said`] reports as the bug it is.
    #[test]
    fn a_phrase_cannot_be_made_without_a_sentence() {
        assert!(Phrase::says(Key::named("files.gone").unwrap(), "").is_err());
        assert!(Phrase::says(Key::named("files.gone").unwrap(), "   ").is_err());
    }

    /// A phrase is written out for a translator to work from: the key, the
    /// English and the note. That file is how a translation ever gets made, and
    /// which format it is written in is the shell's decision rather than this
    /// crate's.
    #[test]
    fn a_phrase_is_written_out_for_a_translator() {
        let written = serde_json::to_value(
            terracotta()
                .noting("The colour of fired clay, an orange-brown.")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            written.get("key").and_then(serde_json::Value::as_str),
            Some("appearance.token.terracotta")
        );
        assert_eq!(
            written.get("source").and_then(serde_json::Value::as_str),
            Some("Terracotta")
        );
        assert_eq!(
            written.get("note").and_then(serde_json::Value::as_str),
            Some("The colour of fired clay, an orange-brown.")
        );
    }
}
