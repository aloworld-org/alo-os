//! **The same weights, measured on each road, held to each other.**
//!
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md): *a GPU
//! changes speed, not capability*. That sentence is a claim about what a
//! machine with a graphics card gets, and until something puts one question to
//! both roads and compares what comes back it stays a claim. This file is the
//! comparing.
//!
//! # What *the same answers* has to mean, and what it cannot
//!
//! It cannot mean the same text. A model is sampled, so two runs of one
//! exercise on **one** road differ — `measured.rs` says as much where it
//! explains why repeats are a larger sample rather than a different method. A
//! comparison that held two roads to identical tokens would be a test of the
//! sampler, failing on a machine where everything was right.
//!
//! So the roads are held to each other at the resolution the product actually
//! uses: **the grade**. [`alo_models::Driving`] is what the catalogue records,
//! what decides whether a machine is given an agent, and the whole of what
//! *capability* means here — so *a GPU changes speed, not capability* is,
//! exactly, *the grade is the same on both roads*. [`BothRoads::agreed`] is
//! that sentence.
//!
//! [`BothRoads::where_they_differed`] is the detail beside it: the exercises
//! one road drove and the other did not. It is **reported and not asserted**,
//! because a single round of a sampled model will differ somewhere and that is
//! the sampler rather than the card. Whoever runs the comparison reads it, and
//! a run of several rounds is what makes it worth reading.
//!
//! # Two runs on one road are not a comparison
//!
//! [`BothRoads::of`] refuses a pair that does not have a card road and a
//! processor road in it. Two processor runs compared on a machine with no card
//! would produce a green result saying nothing at all — the failure `LOOP.md`
//! describes for a suite that passes because nothing ran, and the one this
//! file's subject is most exposed to, since the machine this was written on has
//! no card.

use alo_models::{Driving, Road};

use crate::measured::Measured;

/// **One run, and the road the machine took to make it.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnARoad {
    /// Which road it was, as the machine reported it rather than as the caller
    /// intended it: asking for the processor is not the same as having run
    /// there, and only [`alo_models::Road::of`] can say which happened.
    road: Road,
    /// What the run earned.
    measured: Measured,
}

impl OnARoad {
    /// A run, with the road it was made on.
    #[must_use]
    pub fn of(road: Road, measured: Measured) -> Self {
        Self { road, measured }
    }

    /// Which road it was.
    #[must_use]
    pub fn road(&self) -> &Road {
        &self.road
    }

    /// What the run earned.
    #[must_use]
    pub fn measured(&self) -> &Measured {
        &self.measured
    }
}

/// Why a pair of runs is not a comparison of the two roads.
///
/// English, and it keeps its `Display`, for [`crate::NotMeasurable`]'s reason:
/// its reader is whoever ran the comparison.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotAComparison {
    /// Neither run was on a card.
    #[error(
        "neither of these runs was on a graphics card, so comparing them says nothing about one \
         — this needs a machine with a card the runtime can use"
    )]
    NeitherWasOnACard,
    /// Both were.
    #[error(
        "both of these runs were on a graphics card, so there is no processor road here to hold \
         the card's answers to"
    )]
    BothWereOnACard,
}

/// **One set of weights, measured on each road.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BothRoads {
    /// The run the card made.
    on_the_card: OnARoad,
    /// The run the processor made.
    on_the_processor: OnARoad,
}

impl BothRoads {
    /// Hold two runs to each other, in either order.
    ///
    /// # Errors
    /// [`NotAComparison`] where the two are not one of each. A machine with no
    /// card cannot make this value, which is the point: what it can say about
    /// the card road is *there is no card here*, and that is a different
    /// sentence from *the two roads agreed*.
    pub fn of(one: OnARoad, other: OnARoad) -> Result<Self, NotAComparison> {
        match (one.road.is_the_card(), other.road.is_the_card()) {
            (true, false) => Ok(Self {
                on_the_card: one,
                on_the_processor: other,
            }),
            (false, true) => Ok(Self {
                on_the_card: other,
                on_the_processor: one,
            }),
            (true, true) => Err(NotAComparison::BothWereOnACard),
            (false, false) => Err(NotAComparison::NeitherWasOnACard),
        }
    }

    /// The run the card made.
    #[must_use]
    pub fn on_the_card(&self) -> &OnARoad {
        &self.on_the_card
    }

    /// The run the processor made.
    #[must_use]
    pub fn on_the_processor(&self) -> &OnARoad {
        &self.on_the_processor
    }

    /// The grade each road earned, the card's first.
    #[must_use]
    pub fn grades(&self) -> (Driving, Driving) {
        (
            self.on_the_card.measured.grade(),
            self.on_the_processor.measured.grade(),
        )
    }

    /// **Whether the card changed what the model could do**, which ADR 0007
    /// says it must not.
    ///
    /// True when both roads earned the same grade. That is the claim this file
    /// exists to hold: the card is allowed to change how long an answer took
    /// and nothing else.
    #[must_use]
    pub fn agreed(&self) -> bool {
        let (card, processor) = self.grades();
        card == processor
    }

    /// **Which exercises the two roads disagreed about**, in the order they
    /// were attempted, each named once.
    ///
    /// Reported rather than asserted: a sampled model differs from itself
    /// between two runs on one road, so a name here is a thing to look at
    /// rather than a fault in a card. An exercise attempted on one road and not
    /// on the other cannot appear, because [`Measured::of`] refuses a run that
    /// skipped one.
    #[must_use]
    pub fn where_they_differed(&self) -> Vec<&'static str> {
        let mut differed: Vec<&'static str> = Vec::new();
        for exercise in self
            .on_the_card
            .measured
            .attempts()
            .iter()
            .map(crate::Attempt::exercise)
        {
            if differed.contains(&exercise) {
                continue;
            }
            if drove_every_time(&self.on_the_card, exercise)
                != drove_every_time(&self.on_the_processor, exercise)
            {
                differed.push(exercise);
            }
        }
        differed
    }
}

/// Whether every attempt at one exercise on one road drove the verbs.
fn drove_every_time(run: &OnARoad, exercise: &'static str) -> bool {
    run.measured
        .attempts()
        .iter()
        .filter(|attempt| attempt.exercise() == exercise)
        .all(crate::Attempt::drove)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_whole_run, the_verbs};
    use alo_models::WhyTheProcessor;

    /// A card road, as a runtime that loaded onto one would report it.
    fn the_card() -> Road {
        Road::OnTheCard {
            loaded_bytes: 2_600_000_000,
            on_the_gpu_bytes: 2_600_000_000,
        }
    }

    /// The processor road of the machine this was written on.
    fn the_processor() -> Road {
        Road::OnTheProcessor(WhyTheProcessor::NoCard)
    }

    /// A complete run in which every exercise is answered the same way.
    fn a_run(well: bool) -> Measured {
        a_whole_run(well)
    }

    /// **Two roads that earned the same grade agreed**, which is ADR 0007's
    /// sentence as a measurement.
    #[test]
    fn a_card_that_changed_nothing_about_what_the_model_could_do_agrees() {
        let both = BothRoads::of(
            OnARoad::of(the_card(), a_run(true)),
            OnARoad::of(the_processor(), a_run(true)),
        )
        .expect("one run on each road");
        assert!(both.agreed(), "{:?}", both.grades());
        assert_eq!(both.where_they_differed(), Vec::<&str>::new());
        assert!(both.on_the_card().road().is_the_card());
        assert!(!both.on_the_processor().road().is_the_card());
    }

    /// **The order the two runs are handed over in does not decide which is
    /// which** — the road each was made on does.
    #[test]
    fn the_card_run_is_found_by_its_road_and_not_by_its_place() {
        let one_way = BothRoads::of(
            OnARoad::of(the_processor(), a_run(true)),
            OnARoad::of(the_card(), a_run(false)),
        )
        .expect("one run on each road");
        assert_eq!(one_way.on_the_card().road(), &the_card());
        assert_eq!(one_way.grades().0, Driving::Rarely);
        assert_eq!(one_way.grades().1, Driving::Reliably);
    }

    /// **A card that changed what the model could do is a disagreement**, said
    /// rather than swallowed: this is the result that would put ADR 0007's
    /// claim in question, and a comparison that could not report it would be
    /// worth nothing.
    #[test]
    fn a_card_that_changed_the_grade_is_reported_as_a_disagreement() {
        let both = BothRoads::of(
            OnARoad::of(the_card(), a_run(true)),
            OnARoad::of(the_processor(), a_run(false)),
        )
        .expect("one run on each road");
        assert!(!both.agreed(), "{:?}", both.grades());
        assert_eq!(both.grades(), (Driving::Reliably, Driving::Rarely));
        assert_eq!(
            both.where_they_differed().len(),
            crate::THE_SET.len(),
            "every exercise differed and each should be named once",
        );
    }

    /// **Two runs on the processor are not a comparison**, and on a machine
    /// with no card that is the only pair available — so it is refused rather
    /// than passing green.
    #[test]
    fn two_runs_on_the_processor_are_refused_rather_than_agreeing_with_themselves() {
        let refused = BothRoads::of(
            OnARoad::of(the_processor(), a_run(true)),
            OnARoad::of(
                Road::OnTheProcessor(WhyTheProcessor::TheRuntimeLeftItThere),
                a_run(true),
            ),
        )
        .expect_err("two processor runs are not two roads");
        assert_eq!(refused, NotAComparison::NeitherWasOnACard);
        assert!(refused.to_string().contains("graphics card"), "{refused}");
    }

    /// **And neither are two runs on the card.**
    #[test]
    fn two_runs_on_the_card_have_no_processor_road_to_be_held_to() {
        let refused = BothRoads::of(
            OnARoad::of(the_card(), a_run(true)),
            OnARoad::of(the_card(), a_run(true)),
        )
        .expect_err("two card runs are not two roads");
        assert_eq!(refused, NotAComparison::BothWereOnACard);
    }

    /// **An exercise the two roads disagree about is named once**, however
    /// many rounds were put to it.
    #[test]
    fn an_exercise_that_differs_over_several_rounds_is_named_once() {
        let exercises = crate::Exercises::over(&the_verbs()).expect("the fixed set");
        let twice = |well: bool| {
            let one = a_whole_run(well);
            let mut attempts = one.attempts().to_vec();
            attempts.extend_from_slice(one.attempts());
            Measured::of(&exercises, attempts).expect("every exercise, twice")
        };
        let both = BothRoads::of(
            OnARoad::of(the_card(), twice(true)),
            OnARoad::of(the_processor(), twice(false)),
        )
        .expect("one run on each road");
        let differed = both.where_they_differed();
        let mut once = differed.clone();
        once.sort_unstable();
        once.dedup();
        assert_eq!(once.len(), differed.len(), "{differed:?}");
        assert_eq!(differed.len(), crate::THE_SET.len());
    }
}
