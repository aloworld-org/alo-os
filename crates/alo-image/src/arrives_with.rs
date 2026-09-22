//! Which model a machine of the certified class arrives with — or nothing, and
//! why.
//!
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! promises *the local model is what the machine arrives ready to run*, and
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) says the
//! model is sized for the machine it lands on. Between them they leave exactly
//! one question for the image: **which entry**. This file answers it, and it
//! answers it by asking the catalogue rather than by holding a name.
//!
//! # Why this is not a constant in `crate::checking`
//!
//! A checker that held the model's id would be a second opinion about which
//! model this product ships, kept in a file nobody looks at, and the first
//! thing it would do is disagree with
//! [`alo_models::Catalogue::agent_for_cpu`] — which is the method the rest of
//! alo OS uses to answer *which model gets the agent here*. Two answers to one
//! question is the drift `crate::checking` exists to catch everywhere else, so
//! the image's answer is the catalogue's answer, read at check time.
//!
//! # And the answer may be *none*, which is not a failure
//!
//! `agent_for_cpu` refuses when nothing a machine of this class can run has
//! been measured driving the verbs. On a machine that is the honest state and
//! [`alo_models::NoAgentHere`] is the sentence a person reads for it — three
//! lines, why and what is still open to them, which `alo-telling` already
//! walks. The image's part of that sentence is to **carry no weights**: a
//! machine that arrived with a model graded `rarely` would be a machine whose
//! first promise is untrue and whose disk is several gigabytes smaller for
//! nothing.
//!
//! So this type has two states and the second carries the refusal rather than
//! discarding it, because the refusal is what the image has to say in the
//! person's own words rather than in a new sentence of its own.

use alo_models::{Catalogue, Model, NoAgentHere};

/// The memory of the machine the weights on this image are sized for.
///
/// `docs/hardware.md` certifies two machines and says which of them matters
/// more: *an ordinary business laptop — no discrete graphics, 16 GB of memory*,
/// because the Windows 10 fleet this product exists to catch has almost no
/// discrete GPUs in it. One image is built, so the model it carries is sized for
/// that machine rather than for the workstation; a GPU workstation runs the same
/// weights and can fetch something larger, which is a choice its owner makes.
pub const THE_CERTIFIED_LAPTOP_GB: f32 = 16.0;

/// What a machine of a given class arrives with.
///
/// Not comparable, on purpose: `alo_models::Model` is not, and two answers to
/// *which model* are compared by the id a person would read rather than by
/// every field of a catalogue entry.
#[derive(Debug, Clone)]
pub enum ArrivesWith<'a> {
    /// The catalogue's own recommendation for that machine.
    TheModel(&'a Model),
    /// Nothing, because no entry that machine can run has been measured
    /// driving the verbs — with the refusal a person is shown for it.
    NoModel(NoAgentHere),
}

impl<'a> ArrivesWith<'a> {
    /// What a machine with this much memory and no graphics card arrives with.
    ///
    /// One line, and it is one line on purpose: the whole value of this file is
    /// that the image asks the same method the product asks.
    #[must_use]
    pub fn of(catalogue: &'a Catalogue, ram_gb: f32) -> Self {
        match catalogue.agent_for_cpu(ram_gb) {
            Ok(model) => Self::TheModel(model),
            Err(why) => Self::NoModel(why),
        }
    }

    /// The catalogue id of the model, where there is one.
    #[must_use]
    pub fn id(&self) -> Option<&'a str> {
        match self {
            Self::TheModel(model) => Some(model.id.as_str()),
            Self::NoModel(_) => None,
        }
    }

    /// Why there is none, where there is none — as the key of the string a
    /// person reads, so that the image's refusal and the machine's are one
    /// sentence rather than two.
    #[must_use]
    pub fn why(&self) -> Option<&'static str> {
        match self {
            Self::TheModel(_) => None,
            Self::NoModel(why) => Some(why.word().named()),
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

    /// A catalogue written for one question, in the shape `data/catalogue.toml`
    /// insists on: the sizes are derived from the parameter count because the
    /// loader refuses a size that belongs to no artefact of that many
    /// parameters, and a fixture that would not load would be testing the
    /// loader.
    ///
    /// `drives` is the grade in the words a turn shows, which is the one
    /// [`Catalogue::agent_for_cpu`] reads (ADR 0037).
    fn catalogue(entries: &[(&str, f32, &str, &str, &str)]) -> Catalogue {
        let mut text = String::new();
        for (id, parameters_b, on_cpu, commercial, drives) in entries {
            let measured = if *drives == "not-measured" {
                String::new()
            } else {
                format!(
                    "measured = {{ machine = \"a test fixture, 16 GB\", date = \"2026-09-13\", \
                     runtime = \"Ollama 0.34.0\", drove = 10, of = 20, instructions = \
                     \"d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce\" }}\n\
                     drives_verbs_in_the_envelope = \"{drives}\"\n\
                     measured_in_the_envelope = {{ machine = \"a test fixture, 16 GB\", \
                     date = \"2026-09-15\", runtime = \"Ollama 0.34.0\", drove = 10, of = 20, \
                     instructions = \"{}\" }}\n",
                    alo_models::THE_WORDS_A_TURN_SHOWS
                )
            };
            let gigabytes = f64::from(*parameters_b) * 0.62;
            let (bytes, vram, ram) = (
                format!("{:.0}", gigabytes * 1e9),
                gigabytes + 2.0,
                gigabytes + 4.0,
            );
            text.push_str(&format!(
                "[[model]]\nid = \"{id}\"\nname = \"{id}\"\npublisher = \"p\"\n\
                 parameters_b = {parameters_b}\nquantisation = \"Q4_K_M\"\n\
                 artefact = \"runtime:{id}-q4_K_M\"\n\
                 download_bytes = {bytes}\nmin_vram_gb = {vram:.2}\nmin_ram_gb = {ram:.2}\n\
                 on_cpu = \"{on_cpu}\"\ndrives_verbs = \"{drives}\"\n{measured}\
                 upstream = \"https://example.test/{id}\"\n\
                 licence = {{ name = \"L\", commercial_use = \"{commercial}\", \
                 note = \"conditions, stated\" }}\n\n"
            ));
        }
        Catalogue::parse(&text).unwrap()
    }

    /// **The recommendation is the catalogue's, and the largest of the ones
    /// that clear the bar wins** — which is `agent_for_cpu`'s own ordering,
    /// asked here so that the image cannot start disagreeing with it.
    #[test]
    fn the_model_a_machine_arrives_with_is_the_one_the_catalogue_recommends() {
        let shipped = catalogue(&[
            ("small", 3.0, "comfortable", "permitted", "reliably"),
            ("large", 7.0, "workable", "permitted", "reliably"),
            ("largest", 9.0, "workable", "permitted", "rarely"),
        ]);
        let arrives = ArrivesWith::of(&shipped, THE_CERTIFIED_LAPTOP_GB);

        assert_eq!(arrives.id(), Some("small"));
        assert_eq!(arrives.why(), None);
    }

    /// **A class with nothing that clears the bar arrives with nothing, and
    /// says which of the three reasons it was.** The reason is the key of the
    /// string a person is shown, so the image's answer and the machine's are
    /// one sentence.
    #[test]
    fn a_class_with_nothing_that_clears_the_bar_arrives_with_nothing_and_says_why() {
        let measured_and_short = catalogue(&[
            ("one", 3.0, "comfortable", "permitted", "rarely"),
            ("two", 7.0, "workable", "permitted", "sometimes"),
        ]);
        let arrives = ArrivesWith::of(&measured_and_short, THE_CERTIFIED_LAPTOP_GB);
        assert_eq!(arrives.id(), None);
        assert_eq!(
            arrives.why(),
            Some(alo_models::words::NONE_CLEARS_THE_BAR.named())
        );

        let unmeasured = catalogue(&[("one", 3.0, "comfortable", "permitted", "not-measured")]);
        assert_eq!(
            ArrivesWith::of(&unmeasured, THE_CERTIFIED_LAPTOP_GB).why(),
            Some(alo_models::words::NONE_MEASURED.named())
        );

        let conditioned = catalogue(&[("one", 3.0, "comfortable", "with-conditions", "reliably")]);
        assert_eq!(
            ArrivesWith::of(&conditioned, THE_CERTIFIED_LAPTOP_GB).why(),
            Some(alo_models::words::NOTHING_TO_CHOOSE_FROM.named())
        );
    }

    /// **A machine too small for the entry that clears the bar arrives with
    /// nothing**, rather than with something smaller that does not — which is
    /// the whole difference between sizing a model for a machine and picking
    /// the largest one that fits.
    #[test]
    fn a_machine_too_small_for_the_one_that_clears_the_bar_arrives_with_nothing() {
        let shipped = catalogue(&[
            ("small", 3.0, "comfortable", "permitted", "rarely"),
            ("large", 9.0, "workable", "permitted", "reliably"),
        ]);

        assert_eq!(ArrivesWith::of(&shipped, 16.0).id(), Some("large"));
        assert_eq!(ArrivesWith::of(&shipped, 8.0).id(), None);
    }

    /// **And the machine this image is built for is the laptop, not the
    /// workstation.** One image is built; the constant is read from here by the
    /// check and by the tests, so the two cannot drift apart.
    #[test]
    fn the_class_this_image_is_sized_for_is_the_certified_laptop() {
        assert!((THE_CERTIFIED_LAPTOP_GB - 16.0).abs() < f32::EPSILON);
    }
}
