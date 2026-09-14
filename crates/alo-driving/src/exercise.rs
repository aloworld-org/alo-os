//! One thing a model is asked to do, and the prompt that asks it.
//!
//! An exercise is a request in words and the verb a correct answer calls. It is
//! deliberately not a whole expected call: what the bar measures is whether a
//! model reaches the right verb **with arguments this machine would act on**,
//! and scoring the argument *values* would be measuring how closely it copied a
//! sentence rather than whether it can produce a call at all.
//!
//! That is weaker than it sounds, because [`alo_capability::Takes`] is not
//! weak. A path must be a full path with no `..` and no control characters in
//! it, a name is one name and never a journey, a count is inside the range the
//! verb declared, and a choice is one of the options the verb wrote down. A
//! model that answers `folder: "the invoices folder"` fails, because that is
//! not a path — so the structural gate carries real weight without anybody
//! having to write down a right answer.
//!
//! # The prompt is the words the product shows
//!
//! What a model is shown — how to answer, every verb in the verb's own words,
//! the request last — is `alo-instructing`'s, not this crate's. It moved there
//! on 2026-09-14
//! ([ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md))
//! so that an agent turn can be shown the same text a grade was earned under;
//! the functions here are the same text for one [`Exercise`], and a prompt this
//! crate built out of its own words would be a measurement of words nothing
//! ships. `alo_instructing::verb_as_told` has why a verb is described in its own
//! sentence, and the crate header has why the words are in English.

use alo_capability::Verbs;
use alo_instructing::{Instructions, shown_to_a_model};

/// One request put to a model, and the verb a correct answer calls.
///
/// Both halves are `&'static str`, so the set is written in the source and
/// cannot arrive from anywhere: an exercise read from a file would be a
/// measurement whose questions somebody could choose after seeing the answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exercise {
    /// A short stable name, so a report can say which one failed.
    named: &'static str,
    /// What the model is asked to do, in words.
    asked: &'static str,
    /// The verb a correct answer calls.
    verb: &'static str,
}

impl Exercise {
    /// Declare one.
    #[must_use]
    pub const fn asking(named: &'static str, asked: &'static str, verb: &'static str) -> Self {
        Self { named, asked, verb }
    }

    /// The short stable name of this exercise.
    #[must_use]
    pub fn named(&self) -> &'static str {
        self.named
    }

    /// What the model is asked to do.
    #[must_use]
    pub fn asked(&self) -> &'static str {
        self.asked
    }

    /// The verb a correct answer calls.
    #[must_use]
    pub fn verb(&self) -> &'static str {
        self.verb
    }
}

/// The whole prompt for one exercise: how to answer, the verbs, and the
/// request.
///
/// The verbs come from the registry the measurement is being run against, so a
/// model is asked about the verbs the machine really has. Every model faces the
/// same text for the same registry, which is what makes two grades comparable
/// at all.
#[must_use]
pub fn prompt(exercise: &Exercise, verbs: &Verbs) -> String {
    prompt_under(Instructions::AsFirstWritten, exercise, verbs)
}

/// The whole prompt for one exercise under the given [`Instructions`] — the
/// same verbs and the same request, only how to answer differs
/// ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)).
///
/// It is `alo_instructing::shown_to_a_model` with this exercise's request, so a
/// model being measured and a model being asked for a turn's next request are
/// shown one text built by one function (ADR 0037).
#[must_use]
pub fn prompt_under(instructions: Instructions, exercise: &Exercise, verbs: &Verbs) -> String {
    shown_to_a_model(instructions, verbs, exercise.asked())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;

    /// **An exercise is put to a model in the words the product shows**, and
    /// the exercise's request is what it carries. What those words are — every
    /// verb in the verb's own sentence, the bounds it declared, the door it
    /// takes, the request last — is `alo-instructing`'s and is held to there
    /// (ADR 0037). What is held here is that a measurement adds nothing to them
    /// and drops nothing from them, because a prompt this crate shaped would be
    /// a measurement of words nothing ships.
    #[test]
    fn an_exercise_is_asked_in_the_words_a_turn_would_be_shown() {
        let verbs = the_verbs();
        let exercise = Exercise::asking("a", "list what is in /home/anna", "list_folder");
        for instructions in Instructions::ALL {
            assert_eq!(
                prompt_under(instructions, &exercise, &verbs),
                shown_to_a_model(instructions, &verbs, exercise.asked())
            );
        }
        assert_eq!(
            prompt(&exercise, &verbs),
            shown_to_a_model(Instructions::AsFirstWritten, &verbs, exercise.asked()),
            "the unnamed prompt is the instructions every existing grade was earned under"
        );
    }

    /// The request is the last thing in the prompt, so nothing after it can be
    /// mistaken for part of it — and it is the exercise's own words, not a
    /// summary of them.
    #[test]
    fn the_request_comes_last() {
        let text = prompt(
            &Exercise::asking("a", "list what is in /home/anna", "list_folder"),
            &the_verbs(),
        );
        assert!(
            text.ends_with("The request: list what is in /home/anna"),
            "{text}"
        );
    }
}
