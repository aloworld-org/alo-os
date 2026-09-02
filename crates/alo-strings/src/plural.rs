//! One countable string the code can say — a sentence with a number in it, and
//! the shapes it takes.
//!
//! A [`crate::Phrase`] is one sentence. A `Plural` is one sentence that counts,
//! and it is a different thing rather than a phrase with a flag on it: the code
//! asks for it differently ([`crate::Strings::count`]), a translator answers it
//! with a line per form, and how many lines that is depends on their language
//! rather than on ours.
//!
//! **The source is English and English counts in two.** So a plural is declared
//! with two sentences — one, and everything else — and a language that needs
//! five gets five keys, `files.too-big.one` through `files.too-big.other`, from
//! [`crate::Key::for_form`]. Which forms a language needs is [`crate::cldr`]'s
//! and nobody else's.
//!
//! **A plural names the gap that holds the number.** That is not decoration: it
//! is what lets [`crate::Strings::count`] fill the number in from the same
//! value that picked the form, so the two cannot disagree, and it is what lets
//! a translator write *ein Ordner* where their language can — which is only
//! where the form is exactly one number, and [`crate::cldr::names_one_number`]
//! is where that is decided. Everywhere else the number has to be shown, because
//! a sentence that counts and does not say how many has counted at somebody
//! without telling them the count.

use serde::Serialize;

use crate::form::Form;
use crate::key::Key;
use crate::template::{Template, TemplateError};

/// One countable string the code can say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plural {
    /// What the code asks for. The keys a translator sees are this with a form
    /// on the end.
    key: Key,
    /// The gap that holds the number.
    number: String,
    /// The English for exactly one.
    one: Template,
    /// The English for every other whole number.
    other: Template,
    /// What a translator needs to know that the sentences do not say.
    note: Option<String>,
}

impl Plural {
    /// A countable string: this key says this for one of the thing and this for
    /// any other number, in English, with the number itself in `number`.
    ///
    /// # Errors
    ///
    /// [`PluralError`], which says what to write instead.
    pub fn counting(key: Key, number: &str, one: &str, other: &str) -> Result<Self, PluralError> {
        let one = Template::written(one).map_err(|why| PluralError::NotASentence {
            key: key.clone(),
            form: Form::One,
            why,
        })?;
        let other = Template::written(other).map_err(|why| PluralError::NotASentence {
            key: key.clone(),
            form: Form::Other,
            why,
        })?;
        if !other.has(number) {
            return Err(PluralError::NumberNotSaid {
                key,
                number: number.to_owned(),
            });
        }
        Ok(Self {
            key,
            number: number.to_owned(),
            one,
            other,
            note: None,
        })
    }

    /// The same countable string, with something said to whoever translates it.
    ///
    /// # Errors
    ///
    /// [`PluralError::EmptyNote`], for the reason [`crate::Phrase::noting`]
    /// gives.
    pub fn noting(mut self, note: &str) -> Result<Self, PluralError> {
        if note.trim().is_empty() {
            return Err(PluralError::EmptyNote {
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

    /// The gap that holds the number.
    #[must_use]
    pub fn number(&self) -> &str {
        &self.number
    }

    /// The English for this form.
    ///
    /// English has two, so everything that is not [`Form::One`] is answered
    /// with the general sentence — which is also what a translation falls back
    /// to when it is checked, because that is the form that carries every gap.
    #[must_use]
    pub fn source(&self, form: Form) -> &Template {
        if form == Form::One {
            &self.one
        } else {
            &self.other
        }
    }

    /// What a translator was told, if anything.
    #[must_use]
    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// Why something is not a countable string.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum PluralError {
    /// One of the two English sentences is not a sentence.
    #[error("{key} ({form}): {why}")]
    NotASentence {
        /// The countable string it was declared under.
        key: Key,
        /// Which of the two.
        form: Form,
        /// What is wrong with it.
        why: TemplateError,
    },

    /// The general sentence does not say how many.
    #[error(
        "put {{{number}}} into the sentence {key} says for any other number — it is the sentence every number without a word of its own comes out as, and one that does not say how many leaves a person counting nothing"
    )]
    NumberNotSaid {
        /// The countable string.
        key: Key,
        /// The gap that was supposed to hold the number.
        number: String,
    },

    /// A note with nothing in it.
    #[error(
        "write the note on {key} or leave it off — an empty one is read by every translator, tells them nothing, and costs the next note its credibility"
    )]
    EmptyNote {
        /// The countable string it was put on.
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

    fn key() -> Key {
        Key::named("files.too-big").unwrap()
    }

    /// The string `alo-files` needs today, and the reason item 9a exists: it
    /// says "{bytes} bytes" and is wrong in English for a one-byte file.
    fn too_big() -> Plural {
        Plural::counting(
            key(),
            "bytes",
            "{path} holds 1 byte and a verb reads at most {most}",
            "{path} holds {bytes} bytes and a verb reads at most {most}",
        )
        .unwrap()
    }

    #[test]
    fn a_countable_string_carries_both_english_sentences() {
        let plural = too_big();
        assert_eq!(plural.key().as_str(), "files.too-big");
        assert_eq!(plural.number(), "bytes");
        assert!(plural.source(Form::One).as_written().contains("1 byte"));
        assert!(plural.source(Form::Other).as_written().contains("{bytes}"));
        assert_eq!(plural.note(), None);
    }

    /// English counts in two, so every form that is not *one* is answered with
    /// the general sentence — including the ones English has no word for.
    #[test]
    fn every_form_but_one_is_the_general_sentence() {
        let plural = too_big();
        for form in crate::form::EVERY_FORM {
            let expected = if form == Form::One {
                plural.source(Form::One)
            } else {
                plural.source(Form::Other)
            };
            assert_eq!(plural.source(form), expected, "{form}");
        }
    }

    /// **The one form may leave the number out**, because *one file* is how the
    /// sentence is written in most languages and spelling it `1 file` to satisfy
    /// a check would be this crate deciding a translator's sentence for them.
    #[test]
    fn the_one_form_need_not_say_the_number() {
        let plural = Plural::counting(
            Key::named("files.found").unwrap(),
            "how_many",
            "One file",
            "{how_many} files",
        )
        .unwrap();
        assert!(!plural.source(Form::One).has("how_many"));
    }

    /// **The general sentence must say how many**, because it is what every
    /// number without a word of its own comes out as: a person told *some files
    /// are too big* has been counted at and not told the count.
    #[test]
    fn a_general_sentence_that_does_not_say_how_many_is_refused() {
        let refused = Plural::counting(
            Key::named("files.found").unwrap(),
            "how_many",
            "One file",
            "Several files",
        )
        .unwrap_err();
        assert!(matches!(refused, PluralError::NumberNotSaid { .. }));
        assert!(refused.to_string().contains("{how_many}"));
    }

    /// A gap name that is not one cannot appear in a template either, so it
    /// arrives here as the same refusal — the sentence does not say how many,
    /// and it says which gap it was looking for.
    #[test]
    fn a_gap_name_that_is_not_one_is_refused() {
        let refused = Plural::counting(
            Key::named("files.found").unwrap(),
            "How Many",
            "One file",
            "{how_many} files",
        )
        .unwrap_err();
        assert!(matches!(refused, PluralError::NumberNotSaid { .. }));
    }

    #[test]
    fn a_sentence_that_is_not_one_is_refused_and_says_which() {
        let refused = Plural::counting(key(), "bytes", "   ", "{bytes} bytes").unwrap_err();
        assert!(matches!(
            refused,
            PluralError::NotASentence {
                form: Form::One,
                ..
            }
        ));
        assert!(refused.to_string().contains("files.too-big (one)"));

        let refused = Plural::counting(key(), "bytes", "1 byte", "").unwrap_err();
        assert!(matches!(
            refused,
            PluralError::NotASentence {
                form: Form::Other,
                ..
            }
        ));
    }

    #[test]
    fn a_note_is_trimmed_and_an_empty_one_is_refused() {
        let noted = too_big()
            .noting("  A byte is the unit, not a translated word.  ")
            .unwrap();
        assert_eq!(
            noted.note(),
            Some("A byte is the unit, not a translated word.")
        );
        assert!(matches!(
            too_big().noting("  "),
            Err(PluralError::EmptyNote { .. })
        ));
    }

    /// A countable string is written out for a translator the way a phrase is,
    /// and it carries both sentences: somebody translating *one* needs to see
    /// what the general one says.
    #[test]
    fn a_countable_string_is_written_out_for_a_translator() {
        let written = serde_json::to_value(too_big()).unwrap();
        assert_eq!(
            written.get("key").and_then(serde_json::Value::as_str),
            Some("files.too-big")
        );
        assert_eq!(
            written.get("number").and_then(serde_json::Value::as_str),
            Some("bytes")
        );
        assert!(
            written
                .get("one")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|one| one.contains("1 byte"))
        );
    }
}
