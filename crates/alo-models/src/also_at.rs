//! The same weights at another quantisation, graded on their own.
//!
//! Task 13 of `docs/autonomy/v0-5-the-models-measured-plan.md`. An entry names
//! one artefact and grades it; which quantisation the image pins is chosen from
//! this catalogue, so a grade for the model in general would be a claim about
//! files nobody measured. Each quantisation measured beside the entry's own is
//! written here with its own artefact, size and grade — never folded into the
//! entry's, and never a way to change which file the entry names.

use serde::Deserialize;

use crate::also_under::AlsoUnder;
use crate::driving::Driving;
use crate::measured_on::MeasuredOn;

/// **One more quantisation of an entry's weights, and what it earned.**
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlsoAt {
    /// The quantisation, as the entry's own `quantisation` is spelled.
    pub quantisation: String,
    /// The artefact that quantisation names, as the pinned runtime spells it.
    pub artefact: String,
    /// What the disk loses for that artefact.
    pub download_bytes: u64,
    /// The grade it earned asked in the envelope (ADR 0032).
    pub drives_verbs_in_the_envelope: Driving,
    /// Where, when, under which runtime, with what counts and what residency.
    pub measured_in_the_envelope: MeasuredOn,
    /// **This quantisation under other instructions**, each graded on its own
    /// beside the grade above ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)).
    #[serde(default)]
    pub also_under: Vec<AlsoUnder>,
}

impl AlsoAt {
    /// **The quantisation and the artefact it names** — how a reader tells this
    /// grade apart from the entry's own, and from every other quantisation's.
    #[must_use]
    pub fn quantised_at(&self) -> (&str, &str) {
        (&self.quantisation, &self.artefact)
    }

    /// What is wrong with this statement, or [`None`] when it holds together.
    ///
    /// `the_entrys` is the entry's own quantisation and artefact: another
    /// quantisation that names either is the entry's grade written twice.
    pub(crate) fn what_is_wrong_with_it(
        &self,
        the_entrys: Option<(&str, &str)>,
    ) -> Option<&'static str> {
        if self.quantisation.trim().is_empty() || self.artefact.trim().is_empty() {
            return Some("another quantisation that does not say which quantisation or which file");
        }
        if let Some((quantisation, artefact)) = the_entrys
            && (quantisation == self.quantisation || artefact == self.artefact)
        {
            return Some(
                "another quantisation that is the entry's own: the entry's grade is written once, \
                 beside the entry",
            );
        }
        if self.download_bytes == 0 {
            return Some("another quantisation with no size: say what the disk loses for its file");
        }
        if !self.drives_verbs_in_the_envelope.has_been_measured() {
            return Some(
                "another quantisation graded not-measured: leave it out rather than state a \
                 measurement nobody ran",
            );
        }
        if let Some(what) = self.measured_in_the_envelope.what_is_wrong_with_it() {
            return Some(what);
        }
        let on = &self.measured_in_the_envelope;
        if on.instructions.is_none() {
            return Some(
                "another quantisation's grade that does not name its instructions: two grades \
                 under different instructions are two measurements (ADR 0034)",
            );
        }
        if on.drove.is_none() || on.loaded_bytes.is_none() {
            return Some(
                "another quantisation's grade without its counts and its residency: a grade per \
                 quantisation is compared across quantisations, and that needs both",
            );
        }
        for (at, also) in self.also_under.iter().enumerate() {
            if let Some(what) = also.what_is_wrong_with_it(Some(on)) {
                return Some(what);
            }
            if self
                .also_under
                .iter()
                .skip(at + 1)
                .any(|later| later.the_way() == also.the_way())
            {
                return Some("two grades measured the same way: write the larger sample once");
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sound() -> AlsoAt {
        AlsoAt {
            quantisation: "Q5_K_M".to_owned(),
            artefact: "runtime:model-q5_K_M".to_owned(),
            download_bytes: 5_444_831_648,
            drives_verbs_in_the_envelope: Driving::Rarely,
            measured_in_the_envelope: MeasuredOn {
                machine: "Apple M3, 8 GB unified memory".to_owned(),
                date: "2026-09-14".to_owned(),
                runtime: "Ollama 0.34.0".to_owned(),
                drove: Some(18),
                of: Some(40),
                loaded_bytes: Some(5_959_592_178),
                on_the_gpu_bytes: Some(4_563_287_407),
                held_to: None,
                instructions: Some(
                    "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce".to_owned(),
                ),
            },
            also_under: Vec::new(),
        }
    }

    #[test]
    fn another_quantisation_that_says_what_it_is_and_what_it_earned_holds() {
        let also = sound();
        assert_eq!(
            also.what_is_wrong_with_it(Some(("Q4_K_M", "runtime:model-q4_K_M"))),
            None
        );
        assert_eq!(also.quantised_at(), ("Q5_K_M", "runtime:model-q5_K_M"));
    }

    /// **Another quantisation carries its grades under other instructions**,
    /// held to its own grade's instructions rather than the entry's.
    #[test]
    fn another_quantisation_under_other_instructions_is_held_to_its_own_grade() {
        let theirs = Some(("Q4_K_M", "runtime:model-q4_K_M"));
        let under = |digest: &str| AlsoUnder {
            drives_verbs_in_the_envelope: Driving::Reliably,
            measured_in_the_envelope: MeasuredOn {
                drove: Some(40),
                held_to: None,
                instructions: Some(digest.to_owned()),
                ..sound().measured_in_the_envelope
            },
        };
        let other = "93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02";
        let own = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce";

        let mut beside = sound();
        beside.also_under.push(under(other));
        assert_eq!(beside.what_is_wrong_with_it(theirs), None);

        let mut the_same = sound();
        the_same.also_under.push(under(own));
        assert!(
            the_same
                .what_is_wrong_with_it(theirs)
                .is_some_and(|why| why.contains("the entry's own"))
        );

        let mut twice = sound();
        twice.also_under.push(under(other));
        twice.also_under.push(under(other));
        assert!(
            twice
                .what_is_wrong_with_it(theirs)
                .is_some_and(|why| why.contains("larger sample"))
        );
    }

    #[test]
    fn every_way_another_quantisation_can_be_stated_wrongly_is_refused() {
        let theirs = Some(("Q4_K_M", "runtime:model-q4_K_M"));
        let mut own = sound();
        own.quantisation = "Q4_K_M".to_owned();
        let mut sizeless = sound();
        sizeless.download_bytes = 0;
        let mut ungraded = sound();
        ungraded.drives_verbs_in_the_envelope = Driving::NotMeasured;
        let mut no_residency = sound();
        no_residency.measured_in_the_envelope.loaded_bytes = None;
        no_residency.measured_in_the_envelope.on_the_gpu_bytes = None;
        let mut impossible = sound();
        impossible.measured_in_the_envelope.on_the_gpu_bytes = Some(6_000_000_000);
        for (wrong, saying) in [
            (own, "the entry's own"),
            (sizeless, "no size"),
            (ungraded, "nobody ran"),
            (no_residency, "residency"),
            (impossible, "cannot be"),
        ] {
            assert!(
                wrong
                    .what_is_wrong_with_it(theirs)
                    .is_some_and(|why| why.contains(saying)),
                "{wrong:?} was not refused for `{saying}`"
            );
        }
    }
}
