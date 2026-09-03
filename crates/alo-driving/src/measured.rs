//! A whole run, and the grade it earns.
//!
//! # Every exercise, or it is not a measurement
//!
//! [`Measured::of`] refuses a run that skipped one. A model that fails the
//! hardest exercise and is never asked it would grade as reliable, which is the
//! one way a measured property can be worse than an unmeasured one: it carries
//! the authority of having been checked.
//!
//! Everything missing comes back at once rather than one at a time, which is
//! `alo-strings`' rule about telling a translator what is wrong: being told
//! about the next mistake each time you try again is how somebody gives up.
//!
//! # Repeats are welcome and are not required
//!
//! A model is not deterministic, so asking each exercise once is a small
//! sample. More attempts at the same exercise are simply more attempts: the
//! share is over everything that was asked, so a run of thirty tells you more
//! than a run of ten and neither needs a different method. What cannot happen
//! is a run that leaves one exercise out.
//!
//! # The bar is a share, and it is written down here
//!
//! Nine in ten. It is high on purpose: ADR 0007's *since it was accepted*
//! section is about an agent that proposes the wrong thing three times out of
//! five being a product nobody keeps, and the capability model means the cost of
//! a confused model is an experience rather than a file. The middle grade
//! exists because *it can do this and not dependably* is a true and useful
//! thing to say about a model somebody may still want for its answers.
//!
//! The arithmetic is whole numbers, and it floors. Eighty-nine point nine
//! percent grades as [`Driving::Sometimes`], which is the conservative
//! direction and the same one every other decision about this property takes.

use alo_models::Driving;

use crate::attempt::Attempt;
use crate::exercises::Exercises;

/// The share of attempts that must drive the verbs for a model to be given the
/// agent, as a percentage.
pub const RELIABLY: usize = 90;

/// The share below which a model is not producing workable calls at all.
pub const SOMETIMES: usize = 50;

/// Why a run is not a measurement.
///
/// English, and it keeps its `Display`, for the reason
/// [`crate::NotComparable`] does: its reader is whoever ran the measurement.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "these exercises were never put to the model: {} — a grade over the rest would carry the \
     authority of a measurement without being one",
    .never_asked.join(", ")
)]
pub struct NotMeasurable {
    /// Every exercise nothing attempted, in the order the set declares them.
    pub never_asked: Vec<&'static str>,
}

/// One model, measured against the fixed set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measured {
    /// Every answer that was scored, in the order they were made.
    attempts: Vec<Attempt>,
}

impl Measured {
    /// Gather a run, refusing one that left an exercise out.
    ///
    /// # Errors
    /// [`NotMeasurable`], naming every exercise nothing attempted.
    pub fn of(exercises: &Exercises, attempts: Vec<Attempt>) -> Result<Self, NotMeasurable> {
        let never_asked: Vec<&'static str> = exercises
            .all()
            .map(crate::Exercise::named)
            .filter(|named| !attempts.iter().any(|attempt| attempt.exercise() == *named))
            .collect();
        if never_asked.is_empty() {
            Ok(Self { attempts })
        } else {
            Err(NotMeasurable { never_asked })
        }
    }

    /// How many answers were scored.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.attempts.len()
    }

    /// How many of them this machine would have acted on.
    #[must_use]
    pub fn drove(&self) -> usize {
        self.attempts
            .iter()
            .filter(|attempt| attempt.drove())
            .count()
    }

    /// Every answer, in the order they were made.
    #[must_use]
    pub fn attempts(&self) -> &[Attempt] {
        &self.attempts
    }

    /// **The grade this run earns**, which is what goes into
    /// `data/catalogue.toml`.
    ///
    /// [`Driving::NotMeasured`] is answered only for a run with nothing in it,
    /// which [`Measured::of`] cannot build: every exercise must have been
    /// attempted and the set is not empty. So on any `Measured` that exists this
    /// is one of the three measured grades — a measurement that happened is not
    /// a measurement that did not.
    #[must_use]
    pub fn grade(&self) -> Driving {
        let how_many = self.how_many();
        if how_many == 0 {
            return Driving::NotMeasured;
        }
        let share = self.drove().saturating_mul(100) / how_many;
        if share >= RELIABLY {
            Driving::Reliably
        } else if share >= SOMETIMES {
            Driving::Sometimes
        } else {
            Driving::Rarely
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{answering, the_verbs};

    /// A run where the first `driving` exercises are answered correctly and the
    /// rest are answered with prose — the failure this whole item is about.
    fn run(driving: usize) -> Measured {
        let exercises = Exercises::over(&the_verbs()).unwrap();
        let attempts = exercises
            .all()
            .enumerate()
            .map(|(at, exercise)| {
                let produced = if at < driving {
                    right_answer(exercise.named())
                } else {
                    "Of course — I'll take care of that for you.".to_owned()
                };
                exercises.attempt(exercise, &produced)
            })
            .collect();
        Measured::of(&exercises, attempts).unwrap()
    }

    /// One correct answer per exercise, written out here rather than generated,
    /// because a generated one would be this crate marking its own homework.
    fn right_answer(named: &str) -> String {
        match named {
            "list" => answering(
                "read",
                "list_folder",
                &[("folder", "\"/home/anna/Invoices\"")],
            ),
            "read" => answering(
                "read",
                "read_file",
                &[("file", "\"/home/anna/Invoices/march.pdf\"")],
            ),
            "find" => answering(
                "read",
                "find_in_folder",
                &[
                    ("folder", "\"/home/anna/Invoices\""),
                    ("named", "\"october\""),
                    ("most", "20"),
                ],
            ),
            "rename" => answering(
                "propose",
                "rename_file",
                &[
                    ("file", "\"/home/anna/Invoices/scan001.pdf\""),
                    ("name", "\"march.pdf\""),
                ],
            ),
            "move" => answering(
                "propose",
                "move_file",
                &[
                    ("file", "\"/home/anna/Invoices/march.pdf\""),
                    ("into", "\"/home/anna/Archive\""),
                ],
            ),
            "archive" => answering(
                "propose",
                "archive_folder",
                &[
                    ("folder", "\"/home/anna/Invoices\""),
                    ("into", "\"/home/anna/Archive\""),
                    ("name", "\"invoices\""),
                ],
            ),
            "open" => answering(
                "propose",
                "open_application",
                &[("application", "\"org.alo.Writer\"")],
            ),
            "focus" => answering(
                "propose",
                "focus_application",
                &[("application", "\"org.alo.Writer\"")],
            ),
            "close" => answering(
                "propose",
                "close_application",
                &[("application", "\"org.alo.Writer\"")],
            ),
            _ => answering(
                "propose",
                "arrange_application",
                &[
                    ("application", "\"org.alo.Writer\""),
                    ("where", "\"left_half\""),
                ],
            ),
        }
    }

    /// **A model that drives every verb is the only one given the agent**, and
    /// the ten answers it needs are ten real messages this machine would act
    /// on.
    #[test]
    fn a_model_that_answers_all_ten_clears_the_bar() {
        let measured = run(10);
        assert_eq!(measured.drove(), 10);
        assert_eq!(measured.how_many(), 10);
        assert_eq!(measured.grade(), Driving::Reliably);
        assert!(measured.grade().clears_the_bar());
    }

    /// Nine in ten is the bar and it is reached exactly, not approached: a
    /// model that misses one of the ten still drives the verbs.
    #[test]
    fn the_bar_is_nine_in_ten_and_it_is_reachable() {
        assert_eq!(run(9).grade(), Driving::Reliably);
        assert_eq!(run(8).grade(), Driving::Sometimes);
        assert_eq!(run(5).grade(), Driving::Sometimes);
        assert_eq!(run(4).grade(), Driving::Rarely);
        assert_eq!(run(0).grade(), Driving::Rarely);
    }

    /// **A model that produces beautiful prose and no calls grades as what it
    /// is.** ADR 0007's whole point, as a test: sentences it manages, structure
    /// it loses.
    #[test]
    fn a_model_that_only_writes_sentences_is_never_given_the_agent() {
        let measured = run(0);
        assert_eq!(measured.drove(), 0);
        assert!(!measured.grade().clears_the_bar());
        assert!(measured.grade().has_been_measured());
    }

    /// **A run that skipped an exercise is not a measurement**, and the refusal
    /// names every one that was skipped rather than the first.
    #[test]
    fn a_run_that_left_an_exercise_out_is_refused_and_names_all_of_them() {
        let exercises = Exercises::over(&the_verbs()).unwrap();
        let attempts = exercises
            .all()
            .filter(|exercise| !matches!(exercise.named(), "find" | "archive"))
            .map(|exercise| exercises.attempt(exercise, &right_answer(exercise.named())))
            .collect();
        let refused = Measured::of(&exercises, attempts).unwrap_err();
        assert_eq!(refused.never_asked, vec!["find", "archive"]);
        assert!(refused.to_string().contains("find, archive"), "{refused}");
    }

    /// Asking an exercise more than once is a bigger sample and not a different
    /// method: the share is over everything that was asked.
    #[test]
    fn asking_the_same_exercise_twice_is_two_attempts() {
        let exercises = Exercises::over(&the_verbs()).unwrap();
        let mut attempts: Vec<_> = exercises
            .all()
            .map(|exercise| exercises.attempt(exercise, &right_answer(exercise.named())))
            .collect();
        let list = exercises.of("list").unwrap();
        attempts.push(exercises.attempt(list, "not a message at all"));
        attempts.push(exercises.attempt(list, "nor is this"));
        let measured = Measured::of(&exercises, attempts).unwrap();
        assert_eq!(measured.how_many(), 12);
        assert_eq!(measured.drove(), 10);
        // Ten in twelve is eighty-three percent, which floors below the bar.
        assert_eq!(measured.grade(), Driving::Sometimes);
    }

    /// **A measurement that happened never grades as one that did not.** The
    /// unmeasured state belongs to a catalogue entry nobody has run this
    /// against, and there is no road from here to it.
    #[test]
    fn no_run_ever_grades_as_not_measured() {
        for driving in 0..=10 {
            assert!(run(driving).grade().has_been_measured(), "{driving}");
        }
    }

    /// Every answer is kept, so a report can say which exercise failed and how
    /// — the reason [`crate::Outcome`] is six things rather than a boolean.
    #[test]
    fn a_run_keeps_what_became_of_every_answer() {
        let measured = run(3);
        assert_eq!(measured.attempts().len(), 10);
        let failed: Vec<&str> = measured
            .attempts()
            .iter()
            .filter(|attempt| !attempt.drove())
            .map(crate::Attempt::exercise)
            .collect();
        assert_eq!(
            failed,
            vec![
                "rename", "move", "archive", "open", "focus", "close", "arrange"
            ]
        );
    }
}
