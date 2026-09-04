//! Everything a person has said about their own machine, as one value.
//!
//! Two things, and
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! says why they are one store rather than two: which model answers their
//! questions and which language they read have the same owner, the same
//! lifetime, and are both read before a turn can say anything. Inventing a
//! second store for one owner is how a settings system becomes six settings
//! systems.
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

use std::path::Path;

use alo_strings::Language;

use crate::chosen::Chosen;
use crate::refusing::NotSet;
use crate::written::read;

/// What this person has said about their machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// What they chose to answer their questions, where they have chosen.
    chosen: Option<Chosen>,
    /// The languages they read, best first.
    languages: Vec<Language>,
}

impl Settings {
    /// Settings holding what somebody chose.
    #[must_use]
    pub const fn of(chosen: Option<Chosen>, languages: Vec<Language>) -> Self {
        Self { chosen, languages }
    }

    /// A machine nobody has configured.
    ///
    /// What a person who has never opened a settings panel has, and what a
    /// missing file answers with. Nothing answers questions here, and the
    /// machine says so.
    #[must_use]
    pub const fn untouched() -> Self {
        Self {
            chosen: None,
            languages: Vec::new(),
        }
    }

    /// What answers this person's questions, where they have chosen.
    #[must_use]
    pub fn chosen(&self) -> Option<&Chosen> {
        self.chosen.as_ref()
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
    use crate::chosen::Which;

    /// **A machine nobody has configured has chosen nothing and said nothing**,
    /// which is the state most machines are in the first time somebody asks
    /// their agent a question.
    #[test]
    fn a_machine_nobody_has_configured_has_chosen_nothing() {
        let settings = Settings::untouched();
        assert!(settings.chosen().is_none());
        assert!(settings.languages().is_empty());
    }

    /// What went in comes back out, which is what the daemon is handed.
    #[test]
    fn settings_say_back_what_somebody_chose() {
        let settings = Settings::of(
            Some(Chosen::of(Which::Brought, "my-finetune").unwrap()),
            vec![Language::written("pt-BR").unwrap()],
        );
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
