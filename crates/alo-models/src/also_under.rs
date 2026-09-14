//! The same weights, asked in the envelope under other instructions, graded on
//! their own.
//!
//! Task 16 of `docs/autonomy/v0-5-the-models-measured-plan.md` and
//! [ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md).
//! An entry's envelope grade names the instructions it was earned under. A grade
//! earned under a second set is a second measurement, so it is written here —
//! beside the entry's, never over it — with the digest of the instructions it
//! was earned under and everything a grade per quantisation carries.

use serde::Deserialize;

use crate::driving::Driving;
use crate::measured_on::MeasuredOn;

/// **The entry's own weights, under other instructions, and what they earned.**
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
    /// **The digest of the instructions this grade was earned under** — how a
    /// reader tells it apart from the entry's own envelope grade.
    #[must_use]
    pub fn instructions(&self) -> Option<&str> {
        self.measured_in_the_envelope.instructions.as_deref()
    }

    /// What is wrong with this statement, or [`None`] when it holds together.
    ///
    /// `the_entrys` is the digest the entry's own envelope grade names: a grade
    /// under the same instructions is the entry's grade written twice, and one
    /// beside an entry with no envelope grade of its own has nothing to sit
    /// beside.
    pub(crate) fn what_is_wrong_with_it(&self, the_entrys: Option<&str>) -> Option<&'static str> {
        let Some(the_entrys) = the_entrys else {
            return Some(
                "a grade under other instructions beside an entry with no envelope grade of its \
                 own: it sits beside that grade, never in its place",
            );
        };
        if !self.drives_verbs_in_the_envelope.has_been_measured() {
            return Some(
                "a grade under other instructions graded not-measured: leave it out rather than \
                 state a measurement nobody ran",
            );
        }
        if let Some(what) = self.measured_in_the_envelope.what_is_wrong_with_it() {
            return Some(what);
        }
        let on = &self.measured_in_the_envelope;
        match on.instructions.as_deref() {
            None => {
                return Some(
                    "a grade under other instructions that does not name them: give their \
                     SHA-256 as `instructions`",
                );
            }
            Some(digest) if digest == the_entrys => {
                return Some(
                    "a grade under other instructions that names the entry's own: the entry's \
                     grade is written once, beside the entry",
                );
            }
            Some(_) => {}
        }
        if on.drove.is_none() || on.loaded_bytes.is_none() {
            return Some(
                "a grade under other instructions without its counts and its residency: it is \
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

    fn sound() -> AlsoUnder {
        AlsoUnder {
            drives_verbs_in_the_envelope: Driving::Sometimes,
            measured_in_the_envelope: MeasuredOn {
                machine: "Apple M3, 8 GB unified memory".to_owned(),
                date: "2026-09-14".to_owned(),
                runtime: "Ollama 0.34.0".to_owned(),
                drove: Some(70),
                of: Some(80),
                loaded_bytes: Some(5_197_833_172),
                on_the_gpu_bytes: Some(4_583_210_351),
                instructions: Some(ANOTHER.to_owned()),
            },
        }
    }

    #[test]
    fn a_grade_under_other_instructions_that_says_what_they_were_holds() {
        let also = sound();
        assert_eq!(also.what_is_wrong_with_it(Some(THE_ENTRYS)), None);
        assert_eq!(also.instructions(), Some(ANOTHER));
    }

    #[test]
    fn every_way_a_grade_under_other_instructions_can_be_stated_wrongly_is_refused() {
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
        for (wrong, saying) in [
            (the_same, "the entry's own"),
            (unnamed, "does not name them"),
            (ungraded, "nobody ran"),
            (no_residency, "residency"),
            (no_counts, "counts"),
            (not_a_digest, "not a digest"),
        ] {
            assert!(
                wrong
                    .what_is_wrong_with_it(Some(THE_ENTRYS))
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
