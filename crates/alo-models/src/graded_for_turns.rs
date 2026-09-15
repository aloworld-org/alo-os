//! **Which of an entry's grades says whether it may be given the agent.**
//!
//! Task 15 of `docs/autonomy/v0-5-the-models-measured-plan.md` asked this of the
//! envelope, and task 19 asks it of the words. A grade is a measurement of a way
//! of asking, and the way that decides is the way an agent turn asks — which is
//! now two things rather than one:
//!
//! - **held to the protocol's envelope**, which an agent's next request has been
//!   since `e99be94`
//!   ([ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)),
//!   and
//! - **shown the words this machine shows a model**, which `alo-turn` has built
//!   from `alo-instructing` since 2026-09-15
//!   ([ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)).
//!
//! So the grade that decides is the one earned under
//! [`THE_WORDS_A_TURN_SHOWS`], held to the envelope and nothing further, through
//! the runtime this product pins. An entry that has no such grade **has no grade
//! for the turn**, and is not given the agent — rather than being given it on a
//! grade earned another way, which is the mistake ADR 0034's second cost
//! describes and ADR 0037 decision 4 closes.
//!
//! Every other grade stays exactly where it is, and none is rewritten: an entry
//! carries its free grade, its enveloped grade under the first instructions, and
//! whatever else was measured beside them. They are history a reader can see,
//! and this file simply does not read them.
//!
//! # What is matched, and what is deliberately not
//!
//! **The instructions, by digest**, because that is the one thing that tells two
//! measurements of the same weights apart (ADR 0034 decision 2). **What held the
//! answer**: `held_to` is `None` for a grade the envelope alone held, and
//! `Some(_)` for the two `llama.cpp` grades task 17 earned against a grammar,
//! which ADR 0035 rejected and which no turn asks for. **The runtime's name**,
//! so a grade earned through another program is not read as this one's.
//!
//! **Not the runtime's version.** Matching it would mean that upgrading the
//! pinned runtime silently takes the agent away from every machine at once, and
//! that is a decision for whoever upgrades it — with a re-measurement beside it —
//! rather than a side effect of a version string moving. The version is recorded
//! beside every grade for the reader who needs it.

use alo_strings::{Filling, Said, Strings};

use crate::catalogue::Model;
use crate::driving::Driving;
use crate::measured_on::MeasuredOn;
use crate::words;

/// **The SHA-256 of the words an agent turn shows a model** —
/// `alo_instructing::Instructions::SHOWN_TO_A_TURN`, whose text is where a turn
/// gets them and where a measurement gets them.
///
/// It is written here rather than computed, so that the catalogue can say which
/// grade decides without depending on the verb registry — and
/// `alo-driving`'s `every_grade_names_instructions_this_crate_has.rs` holds the
/// two to each other, so the day the text changes this constant fails a test
/// instead of quietly matching nothing.
pub const THE_WORDS_A_TURN_SHOWS: &str =
    "93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02";

/// The runtime this product pins, as a grade spells its name.
///
/// The version is `crate::pinned::THE_PINNED_RUNTIME` and is deliberately not
/// part of the match; the header says why.
const THE_PINNED_RUNTIMES_NAME: &str = "Ollama ";

/// **How the grade that decides was earned.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskedTheWay {
    /// The way an agent turn asks: shown the words this machine shows a model,
    /// and held to the protocol's envelope by the pinned runtime.
    AsATurnAsks,
    /// Any other way, and no way at all — a grade earned freely, or in the
    /// envelope under other instructions, or through another runtime, or an
    /// entry nobody has measured. None of them says what this machine would
    /// get, so none of them decides.
    NotTheWayATurnAsks,
}

impl AskedTheWay {
    /// The sentence a person is shown beside the grade.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let word = match self {
            Self::AsATurnAsks => words::GRADED_AS_A_TURN_ASKS,
            Self::NotTheWayATurnAsks => words::GRADED_ANOTHER_WAY,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

impl Model {
    /// **The grade that says whether this entry may be given the agent, and how
    /// it was earned**: the grade earned the way a turn asks, or
    /// [`Driving::NotMeasured`] where there is none.
    #[must_use]
    pub fn grade_for_the_turn(&self) -> (Driving, AskedTheWay) {
        self.as_a_turn_asks().map_or(
            (Driving::NotMeasured, AskedTheWay::NotTheWayATurnAsks),
            |grade| (grade, AskedTheWay::AsATurnAsks),
        )
    }

    /// The entry's grade earned the way a turn asks, wherever it is written:
    /// the entry's own envelope grade, or one of the grades written beside it.
    fn as_a_turn_asks(&self) -> Option<Driving> {
        let its_own = match (
            &self.measured_in_the_envelope,
            self.drives_verbs_in_the_envelope,
        ) {
            (Some(on), Some(grade)) if the_way_a_turn_asks(on) => Some(grade),
            _ => None,
        };
        its_own.or_else(|| {
            self.also_under
                .iter()
                .find(|also| the_way_a_turn_asks(&also.measured_in_the_envelope))
                .map(|also| also.drives_verbs_in_the_envelope)
        })
    }
}

/// Whether this measurement was made the way an agent turn asks.
fn the_way_a_turn_asks(on: &MeasuredOn) -> bool {
    on.instructions.as_deref() == Some(THE_WORDS_A_TURN_SHOWS)
        && on.held_to.is_none()
        && on.runtime.starts_with(THE_PINNED_RUNTIMES_NAME)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Catalogue;
    use crate::testing::in_english;

    /// The digest the first instructions have, which is **not** the turn's.
    const THE_FIRST_INSTRUCTIONS: &str =
        "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce";

    /// **An entry measured the way a turn asks is judged by that grade**, and
    /// every entry the catalogue ships that has such a measurement is judged by
    /// it rather than by the grade beside it.
    #[test]
    fn an_entry_measured_the_way_a_turn_asks_is_judged_by_that_grade() {
        let mut found = 0;
        for model in Catalogue::built_in().unwrap().models {
            let Some(grade) = model.as_a_turn_asks() else {
                continue;
            };
            found += 1;
            assert_eq!(
                model.grade_for_the_turn(),
                (grade, AskedTheWay::AsATurnAsks),
                "{}",
                model.id
            );
            assert_eq!(
                model.can_be_the_agent(),
                grade.clears_the_bar(),
                "{}",
                model.id
            );
        }
        assert!(
            found >= 13,
            "the shipped catalogue has {found} entries measured the way a turn asks, and task 20 \
             measured every entry this machine can hold"
        );
    }

    /// **An entry nobody measured that way has no grade for the turn**, however
    /// well it did when it was asked another way — and it is not given the
    /// agent on the strength of the other measurement.
    #[test]
    fn an_entry_measured_another_way_has_no_grade_for_the_turn() {
        for model in Catalogue::built_in().unwrap().models {
            if model.as_a_turn_asks().is_some() {
                continue;
            }
            assert_eq!(
                model.grade_for_the_turn(),
                (Driving::NotMeasured, AskedTheWay::NotTheWayATurnAsks),
                "{}",
                model.id
            );
            assert!(!model.can_be_the_agent(), "{}", model.id);
        }
    }

    /// **The three ways a measurement is not the turn's**, each on its own: the
    /// wrong instructions, an answer held to more than the envelope, and
    /// another runtime. Every one of them is a real grade in this catalogue.
    #[test]
    fn a_measurement_is_the_turns_only_when_all_three_agree() {
        let on = |instructions: &str, held_to: Option<&str>, runtime: &str| MeasuredOn {
            machine: "a test fixture, 16 GB".to_owned(),
            date: "2026-09-15".to_owned(),
            runtime: runtime.to_owned(),
            drove: Some(20),
            of: Some(20),
            loaded_bytes: Some(1),
            on_the_gpu_bytes: Some(1),
            held_to: held_to.map(str::to_owned),
            instructions: Some(instructions.to_owned()),
        };
        assert!(the_way_a_turn_asks(&on(
            THE_WORDS_A_TURN_SHOWS,
            None,
            "Ollama 0.34.0"
        )));
        assert!(!the_way_a_turn_asks(&on(
            THE_FIRST_INSTRUCTIONS,
            None,
            "Ollama 0.34.0"
        )));
        assert!(!the_way_a_turn_asks(&on(
            THE_WORDS_A_TURN_SHOWS,
            Some("a grammar's digest"),
            "Ollama 0.34.0"
        )));
        assert!(!the_way_a_turn_asks(&on(
            THE_WORDS_A_TURN_SHOWS,
            None,
            "llama.cpp 0.4.0 (build 10809, commit 5266f24da)"
        )));
        // A later version of the pinned runtime still decides: the header says
        // why the version is not part of the match.
        assert!(the_way_a_turn_asks(&on(
            THE_WORDS_A_TURN_SHOWS,
            None,
            "Ollama 0.35.1"
        )));
    }

    /// **No grade is rewritten**: every entry keeps the grades it had, whether
    /// or not any of them is the one that decides.
    #[test]
    fn the_other_grades_are_all_still_there() {
        for model in Catalogue::built_in().unwrap().models {
            if model.drives_verbs.has_been_measured() {
                assert!(model.measured.is_some(), "{}", model.id);
            }
            if model.drives_verbs_in_the_envelope.is_some() {
                assert!(model.measured_in_the_envelope.is_some(), "{}", model.id);
            }
        }
        let qwen = Catalogue::built_in()
            .unwrap()
            .get("qwen2.5-7b-instruct")
            .cloned()
            .unwrap();
        assert_eq!(qwen.drives_verbs, Driving::Rarely, "the free grade stays");
        assert_eq!(
            qwen.drives_verbs_in_the_envelope,
            Some(Driving::Sometimes),
            "the grade under the first instructions stays"
        );
        assert_eq!(
            qwen.grade_for_the_turn(),
            (Driving::Reliably, AskedTheWay::AsATurnAsks),
            "and the one the turn's words earned is what decides"
        );
    }

    /// The two ways are two sentences, and neither is the other.
    #[test]
    fn the_two_ways_are_said_apart() {
        let strings = in_english();
        let asks = AskedTheWay::AsATurnAsks.said(&strings).text().to_owned();
        let another = AskedTheWay::NotTheWayATurnAsks
            .said(&strings)
            .text()
            .to_owned();
        assert_ne!(asks, another);
        assert!(asks.contains("the way an agent turn asks"), "{asks}");
        assert!(
            another.contains("not measured the way an agent turn asks"),
            "{another}"
        );
    }
}
