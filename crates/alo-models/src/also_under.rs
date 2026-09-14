//! The same weights, asked in the envelope another way, graded on their own.
//!
//! Task 16 of `docs/autonomy/v0-5-the-models-measured-plan.md` and
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md).
//! An entry's envelope grade names the instructions it was earned under, the
//! runtime that served the weights, and what held the answer. A grade earned
//! **any** of those three ways differently is a second measurement, so it is
//! written here — beside the entry's, never over it — with everything a grade
//! per quantisation carries.
//!
//! Three ways so far: ADR 0034's second set of instructions (task 16), and, for
//! the trial [ADR 0035](../../../docs/decisions/0035-the-wrapper-or-the-engine.md)
//! was rejected on, another runtime and an answer held to a grammar for the
//! whole call rather than to the envelope (task 17).

use serde::Deserialize;

use crate::driving::Driving;
use crate::measured_on::MeasuredOn;

/// **The entry's own weights, asked another way, and what they earned.**
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlsoUnder {
    /// The grade earned asked in the envelope (ADR 0032) under the instructions
    /// [`measured_in_the_envelope`](Self::measured_in_the_envelope) names.
    pub drives_verbs_in_the_envelope: Driving,
    /// Where, when, under which runtime and which instructions, with what counts
    /// and what residency.
    pub measured_in_the_envelope: MeasuredOn,
}

impl AlsoUnder {
    /// **The way this grade was earned**: the instructions, the runtime and what
    /// held the answer — the three things that tell it apart from the entry's
    /// own envelope grade and from every other one written beside it.
    #[must_use]
    pub fn the_way(&self) -> (Option<&str>, &str, Option<&str>) {
        let on = &self.measured_in_the_envelope;
        (
            on.instructions.as_deref(),
            on.runtime.as_str(),
            on.held_to.as_deref(),
        )
    }

    /// What is wrong with this statement, or [`None`] when it holds together.
    ///
    /// `the_entrys` is the entry's own envelope measurement: a grade earned the
    /// same three ways is the entry's grade written twice, and one beside an
    /// entry with no envelope grade of its own has nothing to sit beside.
    pub(crate) fn what_is_wrong_with_it(
        &self,
        the_entrys: Option<&MeasuredOn>,
    ) -> Option<&'static str> {
        let Some(the_entrys) = the_entrys else {
            return Some(
                "a grade measured another way beside an entry with no envelope grade of its own: \
                 it sits beside that grade, never in its place",
            );
        };
        if !self.drives_verbs_in_the_envelope.has_been_measured() {
            return Some(
                "a grade measured another way graded not-measured: leave it out rather than state \
                 a measurement nobody ran",
            );
        }
        if let Some(what) = self.measured_in_the_envelope.what_is_wrong_with_it() {
            return Some(what);
        }
        let on = &self.measured_in_the_envelope;
        if on.instructions.is_none() {
            return Some(
                "a grade measured another way that does not name its instructions: give their \
                 SHA-256 as `instructions`",
            );
        }
        if (
            on.instructions.as_deref(),
            on.runtime.as_str(),
            on.held_to.as_deref(),
        ) == (
            the_entrys.instructions.as_deref(),
            the_entrys.runtime.as_str(),
            the_entrys.held_to.as_deref(),
        ) {
            return Some(
                "a grade measured the way the entry's own was: the entry's grade is written once, \
                 beside the entry, and what is written here differs in its instructions, its \
                 runtime or what held the answer",
            );
        }
        if on.drove.is_none() || on.loaded_bytes.is_none() {
            return Some(
                "a grade measured another way without its counts and its residency: it is \
                 compared with the entry's own, and that needs both",
            );
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THE_ENTRYS: &str = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce";
    const ANOTHER: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const A_GRAMMAR: &str = "efa8b951cbb44e6fdd41dfb7b1bcd5dc3196e14a5ab725be67dc26b0975bbae8";

    /// The entry's own envelope measurement: the pinned runtime, the first
    /// instructions, nothing but the envelope holding the answer.
    fn the_entrys() -> MeasuredOn {
        MeasuredOn {
            machine: "Apple M3, 8 GB unified memory".to_owned(),
            date: "2026-09-14".to_owned(),
            runtime: "Ollama 0.34.0".to_owned(),
            drove: Some(71),
            of: Some(80),
            loaded_bytes: Some(5_197_833_172),
            on_the_gpu_bytes: Some(4_583_210_351),
            held_to: None,
            instructions: Some(THE_ENTRYS.to_owned()),
        }
    }

    fn sound() -> AlsoUnder {
        AlsoUnder {
            drives_verbs_in_the_envelope: Driving::Sometimes,
            measured_in_the_envelope: MeasuredOn {
                drove: Some(70),
                instructions: Some(ANOTHER.to_owned()),
                ..the_entrys()
            },
        }
    }

    /// **A grade earned another way holds**, and says which way.
    #[test]
    fn a_grade_measured_another_way_that_says_which_way_holds() {
        let also = sound();
        assert_eq!(also.what_is_wrong_with_it(Some(&the_entrys())), None);
        assert_eq!(also.the_way(), (Some(ANOTHER), "Ollama 0.34.0", None));
    }

    /// **Any of the three is another way**: other instructions, another
    /// runtime, or another shape holding the answer.
    #[test]
    fn another_runtime_or_another_shape_is_another_way_even_under_the_entrys_instructions() {
        let mut another_runtime = sound();
        another_runtime.measured_in_the_envelope.instructions = Some(THE_ENTRYS.to_owned());
        another_runtime.measured_in_the_envelope.runtime = "llama.cpp 0.4.0".to_owned();
        assert_eq!(
            another_runtime.what_is_wrong_with_it(Some(&the_entrys())),
            None
        );

        let mut another_shape = sound();
        another_shape.measured_in_the_envelope.instructions = Some(THE_ENTRYS.to_owned());
        another_shape.measured_in_the_envelope.held_to = Some(A_GRAMMAR.to_owned());
        assert_eq!(
            another_shape.what_is_wrong_with_it(Some(&the_entrys())),
            None
        );
        assert_eq!(
            another_shape.the_way(),
            (Some(THE_ENTRYS), "Ollama 0.34.0", Some(A_GRAMMAR))
        );
    }

    #[test]
    fn every_way_a_grade_measured_another_way_can_be_stated_wrongly_is_refused() {
        let mut the_same = sound();
        the_same.measured_in_the_envelope.instructions = Some(THE_ENTRYS.to_owned());
        let mut unnamed = sound();
        unnamed.measured_in_the_envelope.instructions = None;
        let mut ungraded = sound();
        ungraded.drives_verbs_in_the_envelope = Driving::NotMeasured;
        let mut no_residency = sound();
        no_residency.measured_in_the_envelope.loaded_bytes = None;
        no_residency.measured_in_the_envelope.on_the_gpu_bytes = None;
        let mut no_counts = sound();
        no_counts.measured_in_the_envelope.drove = None;
        no_counts.measured_in_the_envelope.of = None;
        let mut not_a_digest = sound();
        not_a_digest.measured_in_the_envelope.instructions =
            Some("one-example-per-door".to_owned());
        let mut no_shape_digest = sound();
        no_shape_digest.measured_in_the_envelope.held_to = Some("a grammar".to_owned());
        for (wrong, saying) in [
            (the_same, "the way the entry's own was"),
            (unnamed, "does not name its instructions"),
            (ungraded, "nobody ran"),
            (no_residency, "residency"),
            (no_counts, "counts"),
            (not_a_digest, "not a digest"),
            (no_shape_digest, "not a digest"),
        ] {
            assert!(
                wrong
                    .what_is_wrong_with_it(Some(&the_entrys()))
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?} was not refused for `{saying}`"
            );
        }
        assert!(
            sound()
                .what_is_wrong_with_it(None)
                .is_some_and(|why| why.contains("no envelope grade of its own"))
        );
    }
}
