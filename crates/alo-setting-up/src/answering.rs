//! What a person hands back when they answer setup.
//!
//! `crate::offered` is the four **as they are listed** — a name and a line, and
//! nothing a machine could act on. This is the four **as they are answered**,
//! and each one carries exactly what that configuration needs in order to
//! exist: a model has to say which of this machine's two lists it is on, a
//! provider has to arrive as a provider, and the other two need nothing at all.
//!
//! # Why the material travels with the answer
//!
//! Because the alternative is a flow that records *a provider* and then goes
//! looking for one. An [`Answer`] is the whole answer: `crate::SettingUp::answer`
//! either writes a configuration the machine can act on or refuses, and there is
//! no state in between where setup has been answered and the machine does not
//! know with what.
//!
//! Everything in here has already been checked by whoever owns it.
//! `alo_choosing::Chosen` refuses a model named nothing; `alo_models::Provider`
//! refuses an address that is not one and a key that would travel in clear.
//! **There is no door here that takes text**, which is `alo-choosing`'s own rule
//! one crate up and is what stops a setup surface from composing somebody's
//! settings by hand.
//!
//! # Two of the four carry nothing, for two different reasons
//!
//! [`Answer::NotAtAll`] carries nothing because there is nothing to carry: no
//! model, no provider, no agent (ADR 0009).
//! [`Answer::OnAMachineOnThisNetwork`] carries nothing because **this machine
//! keeps no list of paired machines** — ADR 0003 says a pairing is made
//! deliberately on both machines, and nothing in this repository makes one yet.
//! So it is answerable and is refused in words, rather than being a variant with
//! a name in it that would have to resolve against a list that does not exist.

use alo_choosing::{Chosen, NoProvider, Picked};
use alo_models::Provider;

use crate::offered::Offered;

/// A person's answer to setup, with whatever that configuration needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// A model on this machine, from one of its two lists.
    OnThisMachine(Chosen),

    /// A machine on this network.
    ///
    /// Carries nothing, because this machine keeps no list of paired machines:
    /// `crate::NotSetUp::NoPairedMachine` is what answering with it gets, and
    /// it is a true sentence about the machine rather than a fault.
    OnAMachineOnThisNetwork,

    /// A provider the person is adding here, and the model they want from it.
    ///
    /// The provider itself rather than its name, because at setup there is no
    /// list to name it in yet: this is the moment it is added. Adding it and
    /// choosing it are two writes to the person's own file and
    /// `alo_choosing::Choosing::setting_up` says why.
    FromAProvider {
        /// The provider, already checked by the crate that owns what one is.
        provider: Provider,
        /// What that provider is asked for, exactly as the person wrote it.
        model: String,
    },

    /// No model, no provider and no agent.
    NotAtAll,
}

impl Answer {
    /// Which of the four this answers.
    ///
    /// The match is exhaustive, so a fifth configuration is a compiler error
    /// here rather than an answer nothing can place against the list.
    #[must_use]
    pub const fn answers(&self) -> Offered {
        match self {
            Self::OnThisMachine(_) => Offered::OnThisMachine,
            Self::OnAMachineOnThisNetwork => Offered::OnAMachineOnThisNetwork,
            Self::FromAProvider { .. } => Offered::FromAProvider,
            Self::NotAtAll => Offered::NotAtAll,
        }
    }

    /// What this answer becomes in the person's own settings.
    ///
    /// [`None`] for ADR 0009's fourth choice, which is a person choosing that
    /// nothing answers their questions — and is deliberately the same [`None`]
    /// `alo_choosing::Choosing::answered_by` takes, because it is the same
    /// fact about the machine. What tells it apart from a person nobody has
    /// asked is `alo_choosing::Settings::setup`, which the door that writes it
    /// sets.
    ///
    /// [`None`] for a machine on this network too, and that one never reaches a
    /// file: `crate::SettingUp::answer` refuses it before this is asked.
    ///
    /// # Errors
    ///
    /// [`NoProvider`] when a provider was answered with no model to ask it for.
    /// It is an error rather than a [`None`], and that distinction is the whole
    /// reason this returns a `Result`: a provider with an empty model read as
    /// *nothing chosen* would write ADR 0009's fourth choice into the settings
    /// of somebody who had just picked a provider.
    pub fn chosen(&self) -> Result<Option<Picked>, NoProvider> {
        match self {
            Self::OnThisMachine(chosen) => Ok(Some(Picked::OnThisMachine(chosen.clone()))),
            Self::FromAProvider { provider, model } => {
                Picked::from_a_provider(&provider.name, model).map(Some)
            }
            Self::OnAMachineOnThisNetwork | Self::NotAtAll => Ok(None),
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
    use crate::offered::THE_FOUR;
    use crate::testing::a_provider;
    use alo_choosing::Which;

    /// **Every one of the four can be answered**, and each answer places itself
    /// against exactly the choice it is. A configuration that was listed and
    /// could not be answered would be a screen with a button that does nothing.
    #[test]
    fn every_one_of_the_four_has_an_answer_that_places_itself_against_it() {
        let answers = [
            Answer::OnThisMachine(Chosen::of(Which::Catalogue, "mistral-small").unwrap()),
            Answer::OnAMachineOnThisNetwork,
            Answer::FromAProvider {
                provider: a_provider("Mistral"),
                model: "mistral-small-latest".to_owned(),
            },
            Answer::NotAtAll,
        ];
        assert_eq!(
            answers.iter().map(Answer::answers).collect::<Vec<_>>(),
            THE_FOUR
        );
    }

    /// **A local answer becomes a choice naming its list.** The same name in
    /// the catalogue and on the person's own list are two different answers to
    /// *what runs my turn*, and `alo-choosing` is where that is resolved.
    #[test]
    fn a_local_answer_becomes_a_choice_that_names_which_list_it_is_on() {
        for which in [Which::Catalogue, Which::Brought] {
            let answer = Answer::OnThisMachine(Chosen::of(which, "mistral-small").unwrap());
            let chosen = answer.chosen().unwrap().unwrap();
            assert_eq!(chosen.on_this_machine().unwrap().which(), which);
            assert_eq!(chosen.model(), "mistral-small");
        }
    }

    /// **A provider answer becomes a choice naming the provider by the name it
    /// was added under**, which is what makes the choice a reference into the
    /// person's own list rather than a copy of it.
    #[test]
    fn a_provider_answer_becomes_a_choice_naming_that_provider() {
        let answer = Answer::FromAProvider {
            provider: a_provider("Mistral"),
            model: "mistral-small-latest".to_owned(),
        };
        let chosen = answer.chosen().unwrap().unwrap();
        assert_eq!(chosen.provider(), Some("Mistral"));
        assert_eq!(chosen.model(), "mistral-small-latest");
    }

    /// **Declining chooses nothing**, which is the same fact about the machine
    /// as a person who cleared the setting later — and is told apart from a
    /// person nobody asked by the setup bit rather than by this value.
    #[test]
    fn declining_chooses_nothing() {
        assert_eq!(Answer::NotAtAll.chosen(), Ok(None));
    }

    /// **A provider with no model to ask it for is an error rather than a
    /// choice of nothing.** Read as `None` it would write ADR 0009's fourth
    /// choice into the settings of somebody who had just picked a provider —
    /// the one confusion in this file that would cost a person their answer.
    #[test]
    fn a_provider_with_no_model_to_ask_for_is_not_a_person_declining() {
        let answer = Answer::FromAProvider {
            provider: a_provider("Mistral"),
            model: "   ".to_owned(),
        };
        assert_eq!(answer.chosen(), Err(NoProvider));
        assert_eq!(answer.answers(), Offered::FromAProvider);
    }
}
