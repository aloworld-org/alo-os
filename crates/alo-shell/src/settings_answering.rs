//! What answers this person's questions: the four ways alo OS runs, as setup
//! offered them, and under each the choices this person's own settings hold.
//!
//! Two crates own this section and neither is re-decided here.
//! `alo-setting-up` owns **the four** — `alo_setting_up::THE_FOUR`, in its
//! order, each with the name and the one line it declares — and
//! `alo-choosing` owns **the choice**: what the person's `settings.toml` says,
//! and the doors that change it. Setup is asked once; everything it decided is
//! changed afterwards through `alo_choosing::Choosing::answered_by`, which is
//! *a person changing their mind in Settings* and leaves setup's own answer as
//! it was. So this section never calls setup's `answer`.
//!
//! # Only what can be chosen is offered
//!
//! Under each of the four are the choices the person's settings already hold:
//! weights they brought, the model of a provider they chose — and each model a
//! provider they added is known to offer, where their settings know any, which
//! today they do not keep — and each machine they paired with whose pairing
//! lets its models be asked. One of the
//! four with nothing under it is **absent** rather than drawn and disabled —
//! a provider nobody has added is not a choice, and a greyed line would be the
//! advertisement ADR 0009 rejects. *Not at all* needs nothing, so it is always
//! offered. Adding a provider, bringing weights and pairing are flows of their
//! own, in the crates that own them, and none is reachable from this list.
//!
//! # A file that did not read is not written over
//!
//! `alo_choosing::Choosing::at` refuses settings that are there and do not
//! read, for exactly the reason ADR 0038 gives: a surface that read a typo as
//! *nothing chosen* and then wrote would take away the edit that was about to
//! fix it. That refusal is drawn in its own words and nothing is offered.

use std::path::Path;
use std::time::SystemTime;

use alo_choosing::{Choosing, Chosen, NotSet, Picked, Which};
use alo_nearby::{MayAskIts, Pairings};
use alo_setting_up::{Offered, THE_FOUR};
use alo_strings::Strings;

use crate::settings_paired::PairedAsked;

/// One thing a person may choose to answer their questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsChoice {
    /// Nothing answers: no model, no provider and no agent.
    NotAtAll,
    /// Weights the person brought, by the name they are kept under.
    Brought(String),
    /// A model a provider the person added offers.
    Provider {
        /// The provider, by its name.
        provider: String,
        /// The model, as the provider names it.
        model: String,
    },
    /// A machine the person paired with, by its identity.
    Paired(String),
}

/// What choosing did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsAnswered {
    /// The person's settings say it now.
    Written,
    /// It was not written, and the section says why in `alo-choosing`'s words.
    NotWritten,
    /// There are no settings to write it into: the session has no folder, or
    /// the file there did not read and is not written over.
    NowhereToWrite,
}

/// The section, as it stands.
#[derive(Debug)]
pub(crate) struct AnsweringSection {
    /// The person's settings, when the session has a folder: the door that
    /// changes them, or why the file there did not read.
    choosing: Option<Result<Choosing, NotSet>>,
    /// What the section says about the last choice.
    told: Option<String>,
}

impl AnsweringSection {
    /// The section at sign-in, from the person's settings at `at`.
    pub(crate) fn at_sign_in(at: Option<&Path>) -> Self {
        Self {
            choosing: at.map(Choosing::at),
            told: None,
        }
    }

    /// Why the person's settings did not read, while they do not.
    pub(crate) fn not_read(&self) -> Option<&NotSet> {
        match &self.choosing {
            Some(Err(why)) => Some(why),
            Some(Ok(_)) | None => None,
        }
    }

    /// What the section says about the last choice.
    pub(crate) fn told(&self) -> Option<&str> {
        self.told.as_deref()
    }

    /// The four that have anything to choose, in setup's order, each with what
    /// can be chosen under it. Empty when there are no settings to change.
    pub(crate) fn offered(
        &self,
        pairings: &Pairings,
        now: SystemTime,
    ) -> Vec<(Offered, Vec<SettingsChoice>)> {
        let Some(Ok(choosing)) = &self.choosing else {
            return Vec::new();
        };
        let settings = choosing.settings();
        THE_FOUR
            .into_iter()
            .map(|offered| {
                let choices = match offered {
                    Offered::OnThisMachine => settings
                        .brought()
                        .for_the_agent()
                        .into_iter()
                        .map(|weights| SettingsChoice::Brought(weights.id.clone()))
                        .collect(),
                    Offered::OnAMachineOnThisNetwork => pairings
                        .every()
                        .iter()
                        .filter(|pairing| pairing.permits(MayAskIts::Models, now))
                        .map(|pairing| SettingsChoice::Paired(pairing.with().as_str().to_owned()))
                        .collect(),
                    Offered::FromAProvider => {
                        let mut choices: Vec<SettingsChoice> = settings
                            .providers()
                            .configured
                            .iter()
                            .flat_map(|provider| {
                                provider
                                    .models
                                    .iter()
                                    .map(|model| SettingsChoice::Provider {
                                        provider: provider.name.clone(),
                                        model: model.clone(),
                                    })
                            })
                            .collect();
                        if let Some(chosen @ SettingsChoice::Provider { .. }) = self.chosen()
                            && !choices.contains(&chosen)
                        {
                            choices.push(chosen);
                        }
                        choices
                    }
                    Offered::NotAtAll => vec![SettingsChoice::NotAtAll],
                };
                (offered, choices)
            })
            .filter(|(_, choices): &(Offered, Vec<SettingsChoice>)| !choices.is_empty())
            .collect()
    }

    /// Which of the four the person's settings are, when they say.
    pub(crate) fn chosen_way(&self) -> Option<Offered> {
        let Some(Ok(choosing)) = &self.choosing else {
            return None;
        };
        let settings = choosing.settings();
        match settings.chosen() {
            Some(Picked::OnThisMachine(_)) => Some(Offered::OnThisMachine),
            Some(Picked::FromAPairedMachine(_)) => Some(Offered::OnAMachineOnThisNetwork),
            Some(Picked::FromAProvider { .. }) => Some(Offered::FromAProvider),
            None if settings.setup().is_answered() => Some(Offered::NotAtAll),
            None => None,
        }
    }

    /// Which choice the person's settings are, when it is one this section
    /// lists.
    pub(crate) fn chosen(&self) -> Option<SettingsChoice> {
        let Some(Ok(choosing)) = &self.choosing else {
            return None;
        };
        let settings = choosing.settings();
        match settings.chosen() {
            Some(Picked::OnThisMachine(chosen)) => (chosen.which() == Which::Brought)
                .then(|| SettingsChoice::Brought(chosen.model().to_owned())),
            Some(Picked::FromAPairedMachine(machine)) => {
                Some(SettingsChoice::Paired(machine.machine().to_owned()))
            }
            Some(Picked::FromAProvider { provider, model }) => Some(SettingsChoice::Provider {
                provider: provider.clone(),
                model: model.clone(),
            }),
            None => settings
                .setup()
                .is_answered()
                .then_some(SettingsChoice::NotAtAll),
        }
    }

    /// The person chose `choice`, through the door `alo-choosing` has for it.
    pub(crate) fn choose(
        &mut self,
        choice: &SettingsChoice,
        pairings: &Pairings,
        now: SystemTime,
        strings: &Strings,
    ) -> SettingsAnswered {
        let Some(Ok(choosing)) = &mut self.choosing else {
            return SettingsAnswered::NowhereToWrite;
        };
        let written = match choice {
            SettingsChoice::NotAtAll => choosing.answered_by(None),
            SettingsChoice::Brought(name) => match Chosen::of(Which::Brought, name) {
                Ok(chosen) => choosing.answered_by(Some(Picked::OnThisMachine(chosen))),
                Err(_) => return SettingsAnswered::NowhereToWrite,
            },
            SettingsChoice::Provider { provider, model } => {
                match Picked::from_a_provider(provider, model) {
                    Ok(picked) => choosing.answered_by(Some(picked)),
                    Err(_) => return SettingsAnswered::NowhereToWrite,
                }
            }
            SettingsChoice::Paired(machine) => {
                choosing.answered_by_a_paired_machine(machine, &PairedAsked(pairings), now)
            }
        };
        match written {
            Ok(()) => {
                self.told = None;
                SettingsAnswered::Written
            }
            Err(refused) => {
                self.told = Some(refused.said(strings).into_text());
                SettingsAnswered::NotWritten
            }
        }
    }
}
