//! Everything a person has said about their own machine, as one value.
//!
//! Three things, and
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! says why they are one store rather than three: which model answers their
//! questions, which weights they brought to this machine themselves and which
//! language they read have the same owner, the same lifetime, and are all read
//! before a turn can say anything. Inventing a second store for one owner is how
//! a settings system becomes six settings systems.
//!
//! # Only what somebody changed is here
//!
//! `alo-shortcuts` settled this in item 7 and `alo-appearance` followed it: the
//! difference is stored and the rest lives in the code, so a release can improve
//! what an untouched machine does. Here the untouched machine has **no model and
//! no language**, and that is not a value waiting to be improved — it is the
//! honest state of a machine nobody has configured, and the thing a person is
//! told when they ask it a question.
//!
//! There is deliberately no `Default`. [`Settings::untouched`] says the same
//! thing with a name on it, and a `Default` would put *nothing chosen* one
//! `..Default::default()` away from being the fallback for a file that failed
//! to read — which is exactly the silence this crate exists to prevent.
//!
//! # The weights somebody brought are here, and the choice cannot outrun them
//!
//! [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md)
//! puts `alo_models::Brought` in this file rather than in a store of its own:
//! the weights are the person's, they fetched them, the licence they accepted
//! is theirs, and ADR 0016 already put *which model answers* here. A second
//! store for one owner is how a settings system becomes six.
//!
//! What that costs is a rule, and it is carried by [`Settings::of`] rather than
//! by whoever remembers to check: **a choice naming the brought list must name
//! an entry on it.** Settings that could say *my questions are answered by
//! weights that are not on my list* would be a machine asking a runtime for a
//! name nobody could look up, and the sentence a person got back would be about
//! a runtime rather than about their own file. So the pair is refused where it
//! is made, both roads into it — the file, and a settings panel calling this
//! directly.

use std::path::Path;

use alo_models::{Brought, Weights};
use alo_strings::Language;

use crate::chosen::{Chosen, Which};
use crate::refusing::NotSet;
use crate::written::read;

/// A choice naming weights this person's own list does not have.
///
/// Not a refusal with words of its own, for [`crate::NoModel`]'s reason:
/// whoever needs a sentence about it is reading a file, and
/// [`crate::NotSet::NotBrought`] is the one that can name the file it is in.
/// This says only that the two halves of a settings value have to agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoSuchWeights(String);

impl NoSuchWeights {
    /// What the choice named, exactly as it was written.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.0
    }
}

/// What this person has said about their machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// What they chose to answer their questions, where they have chosen.
    chosen: Option<Chosen>,
    /// The weights they brought to this machine themselves.
    brought: Brought,
    /// The languages they read, best first.
    languages: Vec<Language>,
}

impl Settings {
    /// Settings holding what somebody chose, brought and reads.
    ///
    /// # Errors
    /// [`NoSuchWeights`] when the choice names the brought list and nothing on
    /// that list answers to the name. This file's header says why that is a
    /// refusal here rather than a `None` somewhere later.
    pub fn of(
        chosen: Option<Chosen>,
        brought: Brought,
        languages: Vec<Language>,
    ) -> Result<Self, NoSuchWeights> {
        if let Some(chosen) = &chosen
            && chosen.which() == Which::Brought
            && brought.get(chosen.model()).is_none()
        {
            return Err(NoSuchWeights(chosen.model().to_owned()));
        }
        Ok(Self {
            chosen,
            brought,
            languages,
        })
    }

    /// A machine nobody has configured.
    ///
    /// What a person who has never opened a settings panel has, and what a
    /// missing file answers with. Nothing answers questions here, nothing was
    /// brought here, and the machine says so.
    #[must_use]
    pub fn untouched() -> Self {
        Self {
            chosen: None,
            brought: Brought::default(),
            languages: Vec::new(),
        }
    }

    /// What answers this person's questions, where they have chosen.
    #[must_use]
    pub fn chosen(&self) -> Option<&Chosen> {
        self.chosen.as_ref()
    }

    /// The weights this person brought to this machine themselves.
    ///
    /// Empty is a machine nobody has pointed at weights of their own, which is
    /// most of them. It is a list rather than a question because what is on it
    /// is read by a settings panel offering somebody a choice, and
    /// `alo_models::Brought::for_the_agent` is what that panel offers from.
    #[must_use]
    pub fn brought(&self) -> &Brought {
        &self.brought
    }

    /// The weights this person's choice names, where it names their own list.
    ///
    /// [`None`] for a person who chose nothing and for one who chose from the
    /// catalogue — **and never for one who chose weights**, because
    /// [`Settings::of`] refuses a choice its own list cannot answer. That is
    /// the guarantee this method exists to hand on: whoever holds it does not
    /// have to decide what to do about a name nobody can look up.
    ///
    /// What it is for is the entry rather than the name: the grade a
    /// measurement earned and what the weights cost on this machine are on it,
    /// and both are read beside the name a runtime is asked for.
    #[must_use]
    pub fn weights(&self) -> Option<&Weights> {
        let chosen = self.chosen.as_ref()?;
        if chosen.which() != Which::Brought {
            return None;
        }
        self.brought.get(chosen.model())
    }

    /// The languages this person reads, best first.
    ///
    /// Empty is a person who has not said, which is not the same as a person
    /// who reads English: `alo-strings` holds English as the **source** rather
    /// than as a default, so a machine with no language named shows the source
    /// and every string on it answers `Said::is_translated` with `false`.
    #[must_use]
    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    /// The settings in the file at this path.
    ///
    /// **A file that is not there is a person who has not chosen**, and comes
    /// back as [`untouched`](Self::untouched) rather than as a refusal. That is
    /// deliberately the opposite of `alo_keeping::Reading`, where a missing
    /// record is refused: a record is evidence alo OS wrote and its absence is
    /// something to answer for, while a settings file is one somebody may
    /// simply never have made.
    ///
    /// **Nothing about who owns the file is asked here.** It is under the
    /// person's own home directory and is read by a service running as them
    /// (ADR 0001 §2), so the question `alo-agentd`'s `trusting` asks about the
    /// organisation's description — *could somebody else have written this* —
    /// has a different answer and a different owner. Where that is checked at
    /// all, it is checked by whoever knows what a login is, which is not this
    /// crate.
    ///
    /// # Errors
    ///
    /// [`NotSet`], which is every way a file that **is** there is not settings.
    /// Nothing in it is honoured in any of them.
    pub fn at(path: &Path) -> Result<Self, NotSet> {
        match std::fs::read_to_string(path) {
            Ok(said) => read(&said, path),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(Self::untouched()),
            Err(why) => Err(NotSet::NotRead {
                at: path.to_owned(),
                why: why.to_string(),
            }),
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
    use alo_models::Driving;

    /// One set of weights on somebody's own list, measured well enough to be
    /// given a turn.
    fn theirs(id: &str) -> Brought {
        let mut brought = Brought::default();
        brought
            .add(
                Weights::checked(id, 4_000_000_000)
                    .unwrap()
                    .measured(Driving::Reliably),
            )
            .unwrap();
        brought
    }

    /// **A machine nobody has configured has chosen nothing and said nothing**,
    /// which is the state most machines are in the first time somebody asks
    /// their agent a question.
    #[test]
    fn a_machine_nobody_has_configured_has_chosen_nothing() {
        let settings = Settings::untouched();
        assert!(settings.chosen().is_none());
        assert!(settings.languages().is_empty());
        assert!(settings.brought().weights.is_empty());
        assert!(settings.weights().is_none());
    }

    /// What went in comes back out, which is what the daemon is handed.
    #[test]
    fn settings_say_back_what_somebody_chose() {
        let settings = Settings::of(
            Some(Chosen::of(Which::Brought, "my-finetune").unwrap()),
            theirs("my-finetune"),
            vec![Language::written("pt-BR").unwrap()],
        )
        .unwrap();
        assert_eq!(settings.chosen().unwrap().model(), "my-finetune");
        assert_eq!(settings.chosen().unwrap().which(), Which::Brought);
        assert_eq!(
            settings
                .languages()
                .iter()
                .map(Language::tag)
                .collect::<Vec<_>>(),
            ["pt-BR"]
        );
    }

    /// **A choice of brought weights resolves to the entry itself**, which is
    /// what carries the measurement and the cost — the name alone would be a
    /// second lookup somebody could get wrong.
    #[test]
    fn a_choice_of_brought_weights_comes_back_as_the_weights() {
        let settings = Settings::of(
            Some(Chosen::of(Which::Brought, "my-finetune").unwrap()),
            theirs("my-finetune"),
            Vec::new(),
        )
        .unwrap();
        let weights = settings.weights().unwrap();
        assert_eq!(weights.id, "my-finetune");
        assert!(weights.can_be_the_agent());
    }

    /// **A settings value cannot say its questions are answered by weights that
    /// are not on its list.** The refusal names what was asked for, so whoever
    /// is holding the file can find the line.
    #[test]
    fn a_choice_naming_weights_nobody_brought_is_not_settings() {
        let refused = Settings::of(
            Some(Chosen::of(Which::Brought, "my-finetune").unwrap()),
            theirs("something-else"),
            Vec::new(),
        )
        .unwrap_err();
        assert_eq!(refused.named(), "my-finetune");

        // And an empty list is the same answer, which is the shape it really
        // arrives in: somebody wrote the choice and never wrote the list.
        assert!(
            Settings::of(
                Some(Chosen::of(Which::Brought, "my-finetune").unwrap()),
                Brought::default(),
                Vec::new(),
            )
            .is_err()
        );
    }

    /// **The catalogue is not cross-checked here, and that is not an
    /// oversight.** The catalogue ships with the release rather than living in
    /// this file, and `alo_models::ModelRuntime::answers` has never been gated
    /// by it — a model already on somebody's own disk is theirs. So a
    /// catalogued choice is carried as written, and the one list this file can
    /// contradict itself about is the one this file holds.
    #[test]
    fn a_catalogued_choice_is_carried_as_written_and_names_no_weights() {
        let settings = Settings::of(
            Some(Chosen::of(Which::Catalogue, "a-model-nobody-here-lists").unwrap()),
            theirs("my-finetune"),
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            settings.chosen().unwrap().model(),
            "a-model-nobody-here-lists"
        );
        assert!(settings.weights().is_none());
        // The list is still theirs and is still here, which is what a settings
        // panel offering them a different choice reads.
        assert_eq!(settings.brought().weights.len(), 1);
    }

    /// **A file that is not there is not a refusal.** The path here cannot
    /// exist, and what comes back is a person who has not chosen.
    #[test]
    fn settings_nobody_has_written_read_as_a_person_who_has_not_chosen() {
        let nowhere = std::env::temp_dir()
            .join("alo-choosing-nothing-here")
            .join("settings.toml");
        assert_eq!(Settings::at(&nowhere).unwrap(), Settings::untouched());
    }
}
