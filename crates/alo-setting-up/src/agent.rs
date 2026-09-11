//! Whether this machine has an agent at all.
//!
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md):
//! *the agent's surfaces disappear rather than nag. The hotkey does nothing, the
//! overlay does not exist, and Grants, Models and providers are absent from
//! Settings rather than present-but-disabled. A greyed-out feature is an
//! advertisement.*
//!
//! That is a rule about what a shell draws, and a shell needs somewhere to ask
//! it. This is that answer, read off the person's own settings and nothing else.
//!
//! # It is one question and not two
//!
//! [`TheAgent::Absent`] is the answer for a person who declined **and** for a
//! machine nobody has configured, because in both cases nothing answers
//! questions and there is nothing for an agent surface to do. The two are still
//! told apart — `alo_choosing::Settings::setup` is what does it — and what they
//! decide is different things: this decides whether the agent's surfaces exist,
//! and setup's own bit decides whether the person is asked.
//!
//! # It is not a feature switch
//!
//! Nothing here is stored. There is no *agent: off* anywhere in a person's
//! settings and there must not be, because it would be a second answer to *what
//! answers my questions* that could disagree with the first. What a person
//! chose is the setting; whether an agent exists follows from it.

use alo_choosing::Settings;

/// Whether this machine's agent surfaces exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAgent {
    /// Something answers this person's questions, so the agent's surfaces are
    /// there.
    Present,

    /// Nothing answers, so they are **absent** — not disabled, not greyed out,
    /// not present with a message in them. ADR 0009 calls a greyed-out feature
    /// an advertisement, and this is the value that keeps one from being drawn.
    Absent,
}

impl TheAgent {
    /// What these settings say about whether this machine has an agent.
    #[must_use]
    pub fn of(settings: &Settings) -> Self {
        if settings.chosen().is_some() {
            Self::Present
        } else {
            Self::Absent
        }
    }

    /// Whether the agent's surfaces are drawn at all.
    #[must_use]
    pub const fn is_present(self) -> bool {
        matches!(self, Self::Present)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_choosing::{Chosen, Picked, Setup, Which};
    use alo_models::{Brought, Providers};

    /// Settings holding a choice and an answered setup, or neither.
    fn settings(chosen: Option<Picked>, setup: Setup) -> Settings {
        Settings::of(
            chosen,
            Brought::default(),
            Providers::default(),
            Vec::new(),
            setup,
        )
        .unwrap()
    }

    /// **A machine whose owner declined has no agent surfaces at all** — absent
    /// rather than greyed out, which is the whole of ADR 0009's answer to
    /// somebody who said no.
    #[test]
    fn a_machine_whose_owner_declined_has_no_agent_surfaces() {
        let declined = settings(None, Setup::Answered);
        assert_eq!(TheAgent::of(&declined), TheAgent::Absent);
        assert!(!TheAgent::of(&declined).is_present());
    }

    /// **And neither has a machine nobody has configured**, for the plainer
    /// reason that nothing has been chosen to answer questions (ADR 0025).
    #[test]
    fn a_machine_nobody_has_configured_has_none_either() {
        assert_eq!(TheAgent::of(&Settings::untouched()), TheAgent::Absent);
    }

    /// **A machine with something chosen has them**, whichever of the sources
    /// was chosen and whether or not it was chosen at setup.
    #[test]
    fn a_machine_with_something_chosen_has_its_agent() {
        for setup in [Setup::NotAnswered, Setup::Answered] {
            let chosen = settings(
                Some(Picked::OnThisMachine(
                    Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
                )),
                setup,
            );
            assert_eq!(TheAgent::of(&chosen), TheAgent::Present, "{setup:?}");
        }
    }

    /// **Turning it off again removes the agent at once**, which is ADR 0009's
    /// other half: a person who un-chooses in Settings has a machine whose
    /// agent surfaces go, and does not have to be asked setup again to get
    /// there.
    #[test]
    fn un_choosing_afterwards_removes_the_agent_without_reopening_setup() {
        let chosen = settings(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )),
            Setup::Answered,
        );
        assert!(TheAgent::of(&chosen).is_present());

        let off = settings(None, Setup::Answered);
        assert_eq!(TheAgent::of(&off), TheAgent::Absent);
        assert!(
            off.setup().is_answered(),
            "turning the agent off asked setup again"
        );
    }
}
