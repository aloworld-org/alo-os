//! The question somebody asked, held for the length of one attempt.
//!
//! This is the only type in this repository that holds what a person actually
//! asked, and it is held the way `alo_models::Secret` holds a key: it goes in,
//! it goes to one place, and no rendering of it exists.
//!
//! **That is not caution, it is the record's promise.** ADR 0001 §7 names two
//! things alo OS never keeps, and the first is *the question a person asked*.
//! `alo-record` keeps that promise by having no field for one; `alo-answering`
//! keeps it by holding no question at all and saying so in its own
//! documentation. Neither of those is available here — a question that is going
//! to be sent has to exist somewhere — so the promise is kept the only way left:
//!
//! - **No `Serialize`.** A question cannot be written to a disk, put in a
//!   settings file, or included in a support bundle, because there is no code
//!   that could put it in one.
//! - **No `Display`, and a [`Debug`](fmt::Debug) written by hand.** A derived
//!   `Debug` would put somebody's question in every log line, panic message and
//!   error report that ever formats a structure holding one. The model is shown
//!   there, because that is a name and not a question.
//! - **No accessor outside this crate.** `Question::text` is `pub(crate)`, and
//!   its only caller is the one file that builds the request body.
//!
//! **What this does not claim**, as `alo_models::Secret` says of a key: the
//! bytes are not scrubbed from memory when the value is dropped. Doing that
//! honestly needs `unsafe` or a dependency, and the workspace forbids the first.
//! The promise here is the narrow one that is actually kept.

use std::fmt;

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why what was typed is not yet a question that can be put anywhere.
///
/// **No `Display`, and therefore not a `std::error::Error`**, which is this
/// workspace's rule since item 9f: both of these are read by somebody looking
/// at the box they were about to ask something in, and a `Display` is one
/// `to_string()` away from an English sentence on a Latvian machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAQuestion {
    /// Nothing was asked. Not a failure — nothing has happened yet.
    Nothing,
    /// No model was named to answer it.
    NoModel,
}

impl NotAQuestion {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::Nothing => words::NOTHING_TO_ASK,
            Self::NoModel => words::NO_MODEL_NAMED,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::asking_words`] answers with the key, marked. Neither sentence
    /// has a gap in it, which is deliberate — the only things that could go
    /// into one are the question and the model, and one of those never reaches
    /// a sentence at all.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// One question, and the model it is to be put to.
///
/// The two travel together because they are one request: what was asked, and
/// what is to answer it. Which *place* answers is not here — that is
/// `alo_answering::Answering`'s, decided against the rule this machine is
/// under before anything in this crate is reached.
pub struct Question {
    /// What was asked, as it was typed, with the space around it dropped.
    asked: String,
    /// The model to put it to, as whatever answers names it.
    of: String,
}

impl Question {
    /// A question somebody has just written, to be answered by this model.
    ///
    /// The space around both is dropped, for the reason `Secret::typed` drops
    /// it: a model name copied out of a list arrives with a line break on the
    /// end, and refusing that would be refusing something nobody typed wrong.
    /// Nothing else is removed and nothing is refused for what is *in* it — a
    /// question is somebody's own words, and a system that quietly edited them
    /// would be answering a question they did not ask.
    ///
    /// # Errors
    /// [`NotAQuestion`], saying what to do rather than what is missing.
    pub fn asked(question: &str, of_model: &str) -> Result<Self, NotAQuestion> {
        let asked = question.trim();
        let of = of_model.trim();
        if asked.is_empty() {
            return Err(NotAQuestion::Nothing);
        }
        if of.is_empty() {
            return Err(NotAQuestion::NoModel);
        }
        Ok(Self {
            asked: asked.to_owned(),
            of: of.to_owned(),
        })
    }

    /// The model this question is to be put to, as it is named.
    ///
    /// Shown, unlike the question: it is a name a catalogue or a provider
    /// wrote, and it is what a person types to ask for it again. `alo-files`'
    /// rule about a filename, one crate on — shown as it was written, never
    /// reworded.
    #[must_use]
    pub fn of(&self) -> &str {
        &self.of
    }

    /// What was asked.
    ///
    /// `pub(crate)` on purpose, and it is the only reader of the question that
    /// exists: its one caller is [`crate::hosted`], which puts it in the body
    /// of one request to the one address the rule was asked about. The doctest
    /// on this module asserts that this does not compile from outside.
    pub(crate) fn text(&self) -> &str {
        &self.asked
    }
}

/// Says which model, and nothing about what was asked.
///
/// Written by hand rather than derived, because a derived `Debug` would put
/// somebody's question in every error report, panic message and log line that
/// ever formats a structure holding one.
impl fmt::Debug for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Question")
            .field("of", &self.of)
            .finish_non_exhaustive()
    }
}

/// A question cannot be read back out of this crate.
///
/// ```compile_fail
/// let question = alo_asking::Question::asked("what is in this contract?", "mistral-small-latest")
///     .expect("a question and a model");
/// // `text` is pub(crate): the question goes to one provider and nowhere else.
/// let _ = question.text();
/// ```
///
/// Checked by unmarking it, as every `compile_fail` in this workspace is: it
/// fails with **E0624, method `text` is private**, and not on a typo.
///
/// The twin that passes, so the pair cannot rot into a test of a typo:
///
/// ```
/// let question = alo_asking::Question::asked("what is in this contract?", "mistral-small-latest")
///     .expect("a question and a model");
/// assert_eq!(question.of(), "mistral-small-latest");
/// assert_eq!(
///     format!("{question:?}"),
///     "Question { of: \"mistral-small-latest\", .. }",
/// );
/// ```
#[cfg(doctest)]
pub struct AQuestionCannotBeReadBackOut;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// The load-bearing property, and the reason this type is written by hand:
    /// no rendering of it can show the question, because the only rendering it
    /// has does not have it.
    #[test]
    fn a_question_never_appears_in_anything_that_renders_it() {
        let question =
            Question::asked("what is in this contract?", "mistral-small-latest").unwrap();
        let debugged = format!("{question:?}");
        assert!(!debugged.contains("contract"), "{debugged}");
        assert!(debugged.contains("mistral-small-latest"), "{debugged}");
    }

    /// A question is somebody's own words. The space around it goes, because a
    /// question pasted out of a document arrives with a line on the end;
    /// nothing else does, because editing what somebody asked would be
    /// answering a different question.
    #[test]
    fn a_question_is_kept_as_it_was_written() {
        let question = Question::asked(
            "  summarise this, in two lines:\n\n  the tenant may…  ",
            " mistral-small-latest\n",
        )
        .unwrap();
        assert_eq!(
            question.text(),
            "summarise this, in two lines:\n\n  the tenant may…"
        );
        assert_eq!(question.of(), "mistral-small-latest");
    }

    /// Nothing typed is not a question, and it is not a failure either — the
    /// sentence has to read like an empty box rather than like something that
    /// went wrong.
    #[test]
    fn nothing_asked_says_what_to_do_and_does_not_sound_like_a_failure() {
        assert_eq!(
            Question::asked("   \n ", "mistral-small-latest").unwrap_err(),
            NotAQuestion::Nothing
        );
        let said = NotAQuestion::Nothing.said(&in_english());
        assert!(said.text().contains("write the question first"), "{said}");
        for alarming in ["failed", "error", "wrong"] {
            assert!(!said.text().contains(alarming), "{said}");
        }
    }

    /// A question with nothing to answer it goes nowhere, and the person is
    /// sent to the choice rather than told a name is missing.
    #[test]
    fn a_question_with_no_model_says_which_choice_to_make() {
        assert_eq!(
            Question::asked("what is in this contract?", "  ").unwrap_err(),
            NotAQuestion::NoModel
        );
        assert!(
            NotAQuestion::NoModel
                .said(&in_english())
                .text()
                .contains("choose a model")
        );
    }

    /// **And both are read in the language the person reads.**
    #[test]
    fn what_is_missing_is_said_in_the_readers_own_language() {
        let strings = translated(&[
            (words::NOTHING_TO_ASK, "Es gibt noch nichts zu fragen"),
            (
                words::NO_MODEL_NAMED,
                "Wählen Sie ein Modell für diese Frage",
            ),
        ]);
        let said = NotAQuestion::Nothing.said(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "Es gibt noch nichts zu fragen");
        assert!(NotAQuestion::NoModel.said(&strings).is_translated());
    }
}
