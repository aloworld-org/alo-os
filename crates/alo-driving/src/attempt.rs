//! What a model produced for one exercise, and what became of it.
//!
//! # It is scored through the daemon's own door
//!
//! An answer is read by [`alo_protocol::FromAnAgent`] and validated by
//! [`alo_capability::Verbs::call`], which is exactly what happens to a real
//! client's bytes inside a real turn. Nothing here parses anything itself.
//!
//! That is the decision this file turns on, and the alternative is what makes
//! it. A lighter shape invented for the measurement — a verb name and a map,
//! say — would be a **second parser for one syntax**, which is the failure item
//! 9g found and removed one level down: two readers of one thing disagree, and
//! the one nobody uses in production is the one the score is about. So the
//! envelope is part of what is measured, and the cost of that is written down
//! rather than hidden: a model wrapped by an agent that builds the envelope for
//! it may do better than its grade says. The error falls toward not offering a
//! model the agent, which is the direction `alo_models::Driving` already
//! chooses.
//!
//! # Six ways to fail, and they are not one failure
//!
//! *It did not produce a message*, *it named a verb that does not exist*, *it
//! named the wrong one*, *it used the wrong door* and *it sent an argument the
//! machine will not take* are five different problems with five different
//! answers — a smaller model, a better prompt, a longer list in the prompt.
//! Collapsing them into a percentage would make a measurement that tells you
//! only that something is wrong.
//!
//! # Nothing here has a `Display`
//!
//! Not [`Outcome`], not [`Attempt`]. **No person ever reads a value from this
//! crate**: what a person reads is `alo_models::Driving`, in the catalogue, in
//! their own language. What a developer running a measurement reads is `Debug`.
//! So there is no English here to externalise and no `words.rs` to hold it —
//! the only two sentences this crate writes are the `thiserror` refusals of our
//! own fixed set and our own run, whose reader is whoever ran the measurement.
//! That is `alo_models::CatalogueError`'s reader, and the rule is that crate's.

use alo_capability::{CallError, Verbs};
use alo_protocol::{FromAnAgent, NotUnderstood};

use crate::exercise::Exercise;

/// What became of one answer.
///
/// Every variant except [`Drove`](Self::Drove) is a way a real turn would have
/// refused it, in the order a turn asks: is it a message, does it call a verb,
/// is that verb on the list, is it the one that was wanted, do the arguments
/// survive, and is it on the right side of ADR 0001 §5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The daemon's door would not read it: not JSON, not the envelope, too
    /// long, or a request only a person may make.
    NotAMessage(NotUnderstood),
    /// A request an agent may make, that calls no verb — a question for a
    /// model rather than an instruction.
    NotACall,
    /// It named a verb this machine does not have. Law 2's closed list met from
    /// the outside.
    NoSuchVerb {
        /// The name it asked for, as it wrote it.
        named: String,
    },
    /// It named a real verb, and not the one the exercise asked for.
    AnotherVerb {
        /// The verb it called instead.
        named: String,
    },
    /// The right verb, with an argument the machine will not take: missing,
    /// unknown, given twice, or not what the verb declared.
    ArgumentRefused(CallError),
    /// The right verb and workable arguments, offered through the wrong door: a
    /// change sent as a read, or a read put up for somebody to approve.
    TheWrongDoor,
    /// A call this machine would act on.
    Drove,
}

impl Outcome {
    /// Whether this is the one outcome that counts toward the bar.
    #[must_use]
    pub fn drove(&self) -> bool {
        matches!(self, Self::Drove)
    }
}

/// One answer, scored.
///
/// No public constructor: the only road to one is
/// [`Exercises::attempt`](crate::Exercises::attempt), so an attempt cannot have
/// been scored against a machine that is missing the verb it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// Which exercise this answers, by its name in the fixed set.
    exercise: &'static str,
    /// What became of it.
    outcome: Outcome,
}

impl Attempt {
    /// Put one answer through the door and the registry.
    pub(crate) fn at(exercise: &Exercise, verbs: &Verbs, produced: &str) -> Self {
        Self {
            exercise: exercise.named(),
            outcome: score(exercise, verbs, produced),
        }
    }

    /// Which exercise this answers.
    #[must_use]
    pub fn exercise(&self) -> &'static str {
        self.exercise
    }

    /// What became of it.
    #[must_use]
    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    /// Whether it counts toward the bar.
    #[must_use]
    pub fn drove(&self) -> bool {
        self.outcome.drove()
    }
}

/// The scoring, in the order a turn asks its questions.
fn score(exercise: &Exercise, verbs: &Verbs, produced: &str) -> Outcome {
    let asked = match FromAnAgent::read(produced.trim()) {
        Ok(asked) => asked,
        Err(why) => return Outcome::NotAMessage(why),
    };
    let Some(named) = asked.verb() else {
        return Outcome::NotACall;
    };
    let Some(verb) = verbs.of(named.trim()) else {
        return Outcome::NoSuchVerb {
            named: named.to_owned(),
        };
    };
    if verb.name() != exercise.verb() {
        return Outcome::AnotherVerb {
            named: verb.name().to_owned(),
        };
    }
    let waits = verb.effect().waits_for_approval();
    match verbs.call(named, &asked.given()) {
        Err(why) => Outcome::ArgumentRefused(why),
        Ok(_) if waits != asked.waits_for_a_person() => Outcome::TheWrongDoor,
        Ok(_) => Outcome::Drove,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Exercises;
    use crate::testing::{answering, the_verbs};

    fn exercises() -> Exercises {
        Exercises::over(&the_verbs()).unwrap()
    }

    fn scoring(named: &str, produced: &str) -> Outcome {
        let exercises = exercises();
        let exercise = exercises.of(named).unwrap();
        exercises.attempt(exercise, produced).outcome().clone()
    }

    /// The answer a model is being measured on producing, end to end: the
    /// daemon's door reads it and the registry validates it, which is the same
    /// gate a real turn puts a real client's bytes through.
    #[test]
    fn an_answer_this_machine_would_act_on_drove() {
        assert_eq!(
            scoring(
                "list",
                &answering(
                    "read",
                    "list_folder",
                    &[("folder", "\"/home/anna/Invoices\"")]
                )
            ),
            Outcome::Drove
        );
        assert_eq!(
            scoring(
                "find",
                &answering(
                    "read",
                    "find_in_folder",
                    &[
                        ("folder", "\"/home/anna/Invoices\""),
                        ("named", "\"october\""),
                        ("most", "20"),
                    ],
                )
            ),
            Outcome::Drove
        );
        assert_eq!(
            scoring(
                "arrange",
                &answering(
                    "propose",
                    "arrange_application",
                    &[
                        ("application", "\"org.alo.Writer\""),
                        ("where", "\"left_half\""),
                    ],
                )
            ),
            Outcome::Drove
        );
    }

    /// **Prose is not a message**, and this is the failure the whole item is
    /// about: a model that manages sentences and loses structure.
    #[test]
    fn a_model_that_answers_in_words_did_not_drive_anything() {
        for produced in [
            "Certainly! I'll list that folder for you.",
            "```json\n{\"format\":1,\"asks\":{\"read\":{\"verb\":\"list_folder\",\"given\":[]}}}\n```",
            "",
        ] {
            assert!(
                matches!(scoring("list", produced), Outcome::NotAMessage(_)),
                "{produced}"
            );
        }
    }

    /// A verb nobody declared is not a verb. Law 2's closed list is what a
    /// model meets first, and it meets it here exactly as it would in a turn.
    #[test]
    fn a_verb_this_machine_does_not_have_is_not_a_call() {
        assert_eq!(
            scoring(
                "list",
                &answering("read", "run_shell", &[("command", "\"ls\"")])
            ),
            Outcome::NoSuchVerb {
                named: "run_shell".to_owned()
            }
        );
    }

    /// **A real verb for the wrong request does not pass**, which is what stops
    /// the whole set being cleared by a model that answers `list_folder` ten
    /// times. The item's own wording — *whether the call names a real verb* —
    /// would have let that through.
    #[test]
    fn always_answering_with_one_valid_call_scores_nothing() {
        let exercises = exercises();
        let same = answering("read", "list_folder", &[("folder", "\"/home/anna\"")]);
        let scored: Vec<bool> = exercises
            .all()
            .map(|exercise| exercises.attempt(exercise, &same).drove())
            .collect();
        assert_eq!(scored.iter().filter(|drove| **drove).count(), 1);
        assert_eq!(
            scoring("read", &same),
            Outcome::AnotherVerb {
                named: "list_folder".to_owned()
            }
        );
    }

    /// An argument that is not what the verb declared is refused here for the
    /// same reason it would be refused in a turn — and a relative path is the
    /// case that matters, because it is what a model that half-understood the
    /// request produces.
    #[test]
    fn an_argument_the_machine_will_not_take_is_a_failure_and_says_which() {
        assert!(matches!(
            scoring(
                "list",
                &answering("read", "list_folder", &[("folder", "\"Invoices\"")])
            ),
            Outcome::ArgumentRefused(CallError::Argument(_))
        ));
        assert!(matches!(
            scoring("list", &answering("read", "list_folder", &[])),
            Outcome::ArgumentRefused(CallError::Missing { .. })
        ));
        assert!(matches!(
            scoring(
                "find",
                &answering(
                    "read",
                    "find_in_folder",
                    &[
                        ("folder", "\"/home/anna/Invoices\""),
                        ("named", "\"october\""),
                        ("most", "5000"),
                    ],
                )
            ),
            Outcome::ArgumentRefused(CallError::Argument(_))
        ));
    }

    /// **ADR 0001 §5 is part of the bar.** A change offered as a read is a
    /// change that would run without anybody approving it, and a turn refuses
    /// it — so a model that cannot tell the two apart has not driven the verbs,
    /// however well-formed its call is.
    #[test]
    fn the_right_verb_through_the_wrong_door_did_not_drive_it() {
        assert_eq!(
            scoring(
                "move",
                &answering(
                    "read",
                    "move_file",
                    &[
                        ("file", "\"/home/anna/Invoices/march.pdf\""),
                        ("into", "\"/home/anna/Archive\""),
                    ],
                )
            ),
            Outcome::TheWrongDoor
        );
        assert_eq!(
            scoring(
                "list",
                &answering(
                    "propose",
                    "list_folder",
                    &[("folder", "\"/home/anna/Invoices\"")]
                )
            ),
            Outcome::TheWrongDoor
        );
    }

    /// A question for a model is a request an agent may make and is not a call,
    /// so it is its own outcome rather than a message that failed to read.
    #[test]
    fn a_question_put_to_a_model_is_not_an_attempt_at_a_verb() {
        assert_eq!(
            scoring(
                "list",
                r#"{"format":1,"asks":{"ask":{"question":"what is in that folder?"}}}"#
            ),
            Outcome::NotACall
        );
    }

    /// A model reaching for the person's own side of the door is refused there,
    /// not here — and it still counts as not having driven anything.
    #[test]
    fn an_answer_only_a_person_could_send_is_not_a_call_an_agent_made() {
        assert_eq!(
            scoring("move", r#"{"format":1,"asks":{"approve":{"number":1}}}"#),
            Outcome::NotAMessage(NotUnderstood::NotForAnAgent)
        );
    }

    /// Whitespace around an answer is not a failure to produce one. A model
    /// that ends its line with a newline has not lost structure.
    #[test]
    fn a_line_with_space_around_it_is_still_the_line() {
        let produced = format!(
            "  \n{}\n ",
            answering(
                "read",
                "list_folder",
                &[("folder", "\"/home/anna/Invoices\"")]
            )
        );
        assert_eq!(scoring("list", &produced), Outcome::Drove);
    }
}
