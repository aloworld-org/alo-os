//! What would answer, if a question were asked right now.
//!
//! The first of the three readings the overlay shows before anybody has asked
//! it anything. It is a fact this repository has already decided and no screen
//! has ever shown: `alo-choosing` holds what the person picked,
//! `alo_models::InferenceSource` holds where a question put to that choice is
//! answered, and `alo-agentd` walks the same two in `questions.rs` when a turn
//! actually asks something.
//!
//! # It is derived, never assembled
//!
//! [`WouldAnswer::of`] takes the person's own settings and asks them — it
//! never takes a model name and a place from a caller who might have got them
//! from anywhere. Where the answer would come from is asked of
//! `alo_choosing::Picked::source`, which is the one method in that crate
//! deliberately made inconvenient: a provider's place is a fact about the
//! provider and lives in the person's own list, so a version of this that
//! guessed would be a local choice and a remote one becoming the same value.
//! That is the silent switch between local and remote processing that nothing
//! in this system may make, and it would be at its worst here — on the line a
//! person reads *before* they type a question into the machine.
//!
//! # Nothing here probes anything
//!
//! Whether the runtime the person chose is actually up is a different
//! question, it is answered by reaching for it, and the overlay opens on a
//! keystroke. `alo-agentd` tells *nothing chosen* and *nothing running* apart
//! because it is about to put a question somewhere; this reading is about what
//! the machine is **set to**, and it says so without touching a socket. A
//! chosen model whose runtime is down therefore reads as chosen here, which is
//! true, and the refusal when the question is asked is `alo-models`' own.

use alo_choosing::Settings;
use alo_models::InferenceSource;
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What a question asked now would be answered by.
///
/// Two cases and not three: a machine nobody has chosen for, and a machine
/// somebody has. Which of the two lists a local model came from, and which
/// provider a hosted one is, are inside [`InferenceSource`] rather than being
/// cases of their own — because what a person reads here is *where my question
/// goes*, and that is exactly what an `InferenceSource` answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WouldAnswer {
    /// Nobody has chosen a model or added a provider on this machine.
    ///
    /// The state every machine starts in, and not an error: `alo-choosing`
    /// cannot invent a choice, because a default is a choice made by whoever
    /// set it (ADR 0016).
    Nothing,

    /// Something would: this model, in this place.
    Something {
        /// Where a question put to it is answered — this machine, a service
        /// at this machine's address, a machine somebody paired with, or a
        /// provider and the region that provider **stated**.
        source: InferenceSource,
        /// What the thing answering is asked for, exactly as the person wrote
        /// it. Never translated: a model's name is a model's name.
        model: String,
    },
}

impl WouldAnswer {
    /// What this person's settings say would answer.
    ///
    /// `settings` is the person's own file, read by `alo_choosing::Settings`
    /// — the same file and the same type `alo-agentd` reads at the first
    /// question of a turn.
    ///
    /// A choice naming a provider the person's list does not hold answers
    /// [`WouldAnswer::Nothing`], and cannot arrive: `alo_choosing::Settings`
    /// refuses such a file whichever door it came through. It is answered
    /// rather than assumed away, which is the rule `alo-agentd`'s own
    /// `questions.rs` states for the same arm.
    #[must_use]
    pub fn of(settings: &Settings) -> Self {
        let Some(picked) = settings.chosen() else {
            return Self::Nothing;
        };
        let Some(source) = picked.source(settings.providers()) else {
            return Self::Nothing;
        };
        Self::Something {
            source,
            model: picked.model().to_owned(),
        }
    }

    /// Whether somebody has chosen anything at all.
    #[must_use]
    pub const fn is_chosen(&self) -> bool {
        matches!(self, Self::Something { .. })
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::Nothing => words::ANSWERS_NOTHING,
            Self::Something { .. } => words::ANSWERS_SOMETHING,
        }
    }

    /// The line a person reads, in the language they read it in.
    ///
    /// The place goes in through `alo_strings::Filling::and_said` rather than
    /// as text, so that whether *it* was translated is part of whether the
    /// whole line was — a German line with `on this machine` in the middle of
    /// it must not answer `Said::is_translated` with `true`. The model's name
    /// goes in as data, because it is one.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Nothing => Filling::nothing(),
            Self::Something { source, model } => {
                Filling::of("model", model.clone()).and_said("where", &source.said(strings))
            }
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use alo_choosing::{Chosen, Picked, Setup, Which};
    use alo_models::{Brought, Provider, Providers, Region};

    /// A settings file holding this local choice and nothing else.
    fn chose_locally(model: &str) -> Settings {
        Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, model).unwrap(),
            )),
            Brought::default(),
            Providers::default(),
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap()
    }

    /// A settings file holding a provider the person added, and a choice of
    /// one of its models.
    fn chose_a_provider(region: Region) -> Settings {
        let mut providers = Providers::default();
        providers
            .add(Provider::checked("Mistral", "https://api.mistral.ai", region, None).unwrap())
            .unwrap();
        Settings::of(
            Some(Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap()),
            Brought::default(),
            providers,
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap()
    }

    /// **A machine nobody has set up has nothing answering**, and says so in a
    /// clause rather than showing an empty line.
    #[test]
    fn a_machine_nobody_has_chosen_for_has_nothing_answering() {
        let would = WouldAnswer::of(&Settings::untouched());
        assert_eq!(would, WouldAnswer::Nothing);
        assert!(!would.is_chosen());

        let said = would.said(&in_english());
        assert!(!said.is_a_bug(), "the reading is not declared");
        assert_eq!(said.text(), "no model or provider has been chosen");
    }

    /// **A model on this machine reads as answering on this machine**, with
    /// the name the person wrote and the place `alo-models` words.
    #[test]
    fn a_model_on_this_machine_says_so_and_names_itself() {
        let would = WouldAnswer::of(&chose_locally("mistral-small"));
        assert_eq!(
            would,
            WouldAnswer::Something {
                source: InferenceSource::ThisMachine,
                model: "mistral-small".to_owned(),
            }
        );
        assert_eq!(
            would.said(&in_english()).text(),
            "mistral-small answers, on this machine"
        );
    }

    /// **A provider the person chose never reads as answering here.** This is
    /// the no-fallback rule at the layer a person reads: a line saying *on
    /// this machine* above a question that is about to be sent to a company
    /// would be the reassuring answer and the false one.
    #[test]
    fn a_provider_choice_never_reads_as_answering_on_this_machine() {
        let would = WouldAnswer::of(&chose_a_provider(Region::Declared("the EU".to_owned())));
        assert!(would.is_chosen());
        assert!(
            !matches!(
                &would,
                WouldAnswer::Something { source, .. } if *source == InferenceSource::ThisMachine
            ),
            "a question addressed to a provider read as answering here: {would:?}"
        );
        assert_eq!(
            would.said(&in_english()).text(),
            "mistral-small-latest answers, by Mistral, in the EU"
        );
    }

    /// **A provider that has not said where it runs is not read as safe.**
    /// `alo-models` has a word for exactly that and this line uses it, so the
    /// person is told the region is unstated rather than shown a blank.
    #[test]
    fn a_provider_that_has_not_said_where_it_runs_says_that_much() {
        let would = WouldAnswer::of(&chose_a_provider(Region::Unknown));
        let said = would.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("Mistral"), "{said}");
        assert!(
            !said.text().contains("in the EU"),
            "a region nobody stated was shown: {said}"
        );
    }

    /// **A line is only as translated as the place inside it.** The clause
    /// arrives through `and_said`, so a translated sentence with an English
    /// place in the middle of it cannot report itself as translated.
    #[test]
    fn a_line_with_an_untranslated_place_in_it_does_not_claim_to_be_translated() {
        let strings = translated(&[(words::ANSWERS_SOMETHING, "{model} antwortet, {where}")]);
        let said = WouldAnswer::of(&chose_locally("mistral-small")).said(&strings);
        assert_eq!(said.text(), "mistral-small antwortet, on this machine");
        assert!(
            !said.is_translated(),
            "an English place inside a German line reported the line as German"
        );
    }
}
