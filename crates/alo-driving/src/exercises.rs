//! The fixed set, and the registry it is scored against.
//!
//! **Ten exercises, one per verb alo OS ships**, which is what makes the set
//! fixed rather than representative: every verb is asked for once, so no
//! argument kind goes unmeasured and nobody has to argue about which verbs were
//! chosen. A path, an application identifier, one name, a number in a range and
//! one of a declared list are all in it, because the ten verbs between them
//! take all five.
//!
//! # Why the set is closed, and why it is checked against the machine
//!
//! Two grades are comparable only if the models that earned them faced the same
//! questions, so the set is `&'static` in the source: an exercise read from a
//! file would be a measurement whose questions somebody could choose after
//! seeing the answers.
//!
//! [`Exercises::over`] then refuses a registry that is **missing** one of the
//! verbs, because a model that was never asked to move a file has not been
//! measured on moving a file, and a grade over nine exercises put beside one
//! over ten is a comparison nobody said was invalid. A registry with **extra**
//! verbs on it is fine and is not an error: an adapter's verbs are that
//! adapter's to measure, and this is alo OS's own bar.
//!
//! # Scoring only happens through here
//!
//! [`Attempt`] has no public constructor. The only road to one is
//! [`Exercises::attempt`], which is a method on the thing that already checked
//! the registry — so there is no way to score an answer against a machine that
//! does not have the verbs the exercise is about.

use alo_capability::Verbs;

use crate::attempt::Attempt;
use crate::exercise::{Exercise, prompt};

/// The ten, in the order a report reads in.
///
/// The requests name Anna, `/home/anna/…` and `org.alo.Writer` throughout,
/// because a set where every exercise invented its own person and its own paths
/// would be measuring how well a model tracks changing names.
pub const THE_SET: [Exercise; 10] = [
    Exercise::asking(
        "list",
        "Anna wants to see what is in the folder /home/anna/Invoices.",
        "list_folder",
    ),
    Exercise::asking(
        "read",
        "Anna wants to know what the file /home/anna/Invoices/march.pdf says.",
        "read_file",
    ),
    Exercise::asking(
        "find",
        "Anna is looking for anything called october inside /home/anna/Invoices, and does not want \
         more than twenty of them back.",
        "find_in_folder",
    ),
    Exercise::asking(
        "rename",
        "Anna wants the file /home/anna/Invoices/scan001.pdf to be called march.pdf instead, where \
         it already is.",
        "rename_file",
    ),
    Exercise::asking(
        "move",
        "Anna wants the file /home/anna/Invoices/march.pdf put into the folder /home/anna/Archive.",
        "move_file",
    ),
    Exercise::asking(
        "archive",
        "Anna wants the folder /home/anna/Invoices packed into one archive called invoices, left \
         in /home/anna/Archive.",
        "archive_folder",
    ),
    Exercise::asking(
        "open",
        "Anna wants the application org.alo.Writer started.",
        "open_application",
    ),
    Exercise::asking(
        "focus",
        "Anna already has org.alo.Writer running behind other windows and wants it in front.",
        "focus_application",
    ),
    Exercise::asking(
        "close",
        "Anna has finished with org.alo.Writer and wants it to close.",
        "close_application",
    ),
    Exercise::asking(
        "arrange",
        "Anna wants the window of org.alo.Writer put on the left half of her screen.",
        "arrange_application",
    ),
];

/// Why this machine cannot be measured against the fixed set.
///
/// English, and it keeps its `Display`, for the reason
/// `alo_models::CatalogueError` does: its reader is whoever is running the
/// measurement, not whoever is using the machine. Nothing in this crate is ever
/// read by a person on a machine — see [`crate`] for the whole of that.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "this machine has no verb called {verb}, so the fixed set cannot be put to a model on it — a \
     score over fewer exercises is not comparable with one over all ten"
)]
pub struct NotComparable {
    /// The verb the set names and the registry does not have.
    pub verb: &'static str,
}

/// The fixed set, bound to the verbs of one machine.
#[derive(Debug, Clone)]
pub struct Exercises {
    /// The registry every answer is scored against, held once so that no two
    /// attempts in one measurement can be scored against different verbs.
    verbs: Verbs,
}

impl Exercises {
    /// Bind the fixed set to a machine's verbs.
    ///
    /// # Errors
    /// [`NotComparable`] if the registry is missing a verb the set names. Extra
    /// verbs are not an error — see this file's header.
    pub fn over(verbs: &Verbs) -> Result<Self, NotComparable> {
        for exercise in &THE_SET {
            if verbs.of(exercise.verb()).is_none() {
                return Err(NotComparable {
                    verb: exercise.verb(),
                });
            }
        }
        Ok(Self {
            verbs: verbs.clone(),
        })
    }

    /// The ten, in order.
    pub fn all(&self) -> impl Iterator<Item = &'static Exercise> {
        THE_SET.iter()
    }

    /// One of them by name.
    #[must_use]
    pub fn of(&self, named: &str) -> Option<&'static Exercise> {
        THE_SET.iter().find(|exercise| exercise.named() == named)
    }

    /// The verbs answers are scored against.
    #[must_use]
    pub fn verbs(&self) -> &Verbs {
        &self.verbs
    }

    /// The whole prompt for one exercise: how to answer, the verbs, the
    /// request.
    #[must_use]
    pub fn prompt(&self, exercise: &Exercise) -> String {
        prompt(exercise, &self.verbs)
    }

    /// Score what a model produced for one exercise.
    ///
    /// The only road to an [`Attempt`], which is what makes *every answer was
    /// scored against verbs this machine really has* a fact rather than a
    /// convention.
    #[must_use]
    pub fn attempt(&self, exercise: &Exercise, produced: &str) -> Attempt {
        Attempt::at(exercise, &self.verbs, produced)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::the_verbs;
    use std::collections::BTreeSet;

    /// **One exercise per verb, and no verb asked twice.** The set is what
    /// makes two grades comparable, so what is in it is a test rather than a
    /// comment.
    #[test]
    fn the_set_asks_for_every_verb_this_system_ships_exactly_once() {
        let verbs = the_verbs();
        let asked: BTreeSet<&str> = THE_SET.iter().map(Exercise::verb).collect();
        assert_eq!(asked.len(), THE_SET.len(), "a verb is asked for twice");
        let declared: BTreeSet<&str> = verbs.all().map(alo_capability::Verb::name).collect();
        assert_eq!(asked, declared, "the set and the verbs have drifted apart");
    }

    /// A name is how a report says which exercise failed, so two exercises
    /// cannot share one.
    #[test]
    fn no_two_exercises_are_named_the_same() {
        let named: BTreeSet<&str> = THE_SET.iter().map(Exercise::named).collect();
        assert_eq!(named.len(), THE_SET.len());
    }

    /// **A machine missing a verb is refused rather than scored over nine.**
    /// The refusal names the verb, because whoever is running the measurement
    /// is the person who can put it back on the list.
    #[test]
    fn a_machine_that_cannot_do_one_of_them_is_not_measured_at_all() {
        let mut incomplete = alo_capability::Verbs::default();
        alo_files::declare_into(&mut incomplete).unwrap();
        let refused = Exercises::over(&incomplete).unwrap_err();
        assert_eq!(refused.verb, "open_application");
        assert!(refused.to_string().contains("not comparable"), "{refused}");
    }

    /// A registry with somebody else's verbs on it as well is still this
    /// machine: an adapter's verbs are that adapter's to measure.
    #[test]
    fn a_machine_with_more_verbs_than_ours_is_still_measurable() {
        let mut more = the_verbs();
        more.declare(crate::testing::a_verb_of_somebody_elses())
            .unwrap();
        let exercises = Exercises::over(&more).unwrap();
        assert_eq!(exercises.all().count(), 10);
        assert!(exercises.verbs().of("water_the_plants").is_some());
    }

    /// The set is reachable by name, which is what a report that says *find
    /// failed* is looked up against.
    #[test]
    fn an_exercise_can_be_found_by_name() {
        let exercises = Exercises::over(&the_verbs()).unwrap();
        assert_eq!(exercises.of("find").unwrap().verb(), "find_in_folder");
        assert!(exercises.of("water").is_none());
    }
}
