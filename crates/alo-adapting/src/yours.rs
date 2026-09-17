//! **How an adapted model is listed: yours, from these files, under this grant.**
//!
//! An adapted model is not one of ours. It was made on this machine, from
//! documents a person granted, and the catalogue must say so — a list where
//! *Qwen 2.5 0.5B* and *your Qwen, from your invoices* look alike is a list that
//! invites somebody to send the second one somewhere.
//!
//! It also says **what the machine will and will not do with it**. An adapted
//! model that does not drive the verbs is not given the agent, and a person
//! whose own model is not the agent deserves the reason rather than a silence
//! they have to guess at.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::adapter::Adapter;

/// **An adapted model, as a person sees it in the list.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Yours {
    /// The base it is composed over, as the catalogue names it.
    pub over: String,
    /// The folders it learned from.
    pub from_these_files: Vec<PathBuf>,
    /// **The grant it was made under** — what makes revocation mechanical: the
    /// machine finds the adapter by the grant, and deletes it.
    pub under_the_grant_to: String,
    /// The day it was made, as `YYYY-MM-DD`.
    pub made_on: String,
    /// What it earned on the fixed ten, and of how many attempts.
    pub drove: (u32, u32),
    /// Whether the machine will give it an agent turn.
    pub the_agent: TheAgent,
}

/// Whether an adapted model drives the machine, and what it is for if not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TheAgent {
    /// It cleared the same bar every model clears, and may be given the agent.
    ItDrivesTheVerbs,
    /// It did not. It answers questions; it does not act on the machine.
    ItAnswersButDoesNotDrive,
}

impl Yours {
    /// An entry for this adapter, with what the measurement found.
    #[must_use]
    pub fn of(adapter: &Adapter, made_on: &str, drove: u32, of: u32, bar_cleared: bool) -> Self {
        Self {
            over: adapter.applies_to().to_owned(),
            from_these_files: adapter.learned().folders.clone(),
            under_the_grant_to: adapter.learned().under_the_grant_to.clone(),
            made_on: made_on.to_owned(),
            drove: (drove, of),
            the_agent: if bar_cleared {
                TheAgent::ItDrivesTheVerbs
            } else {
                TheAgent::ItAnswersButDoesNotDrive
            },
        }
    }

    /// **Whether this is one of ours.** It never is: the type exists to make the
    /// question answerable by a list that holds both kinds.
    #[must_use]
    pub const fn is_one_of_ours(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::learned_from::LearnedFrom;

    fn an_adapter() -> Adapter {
        Adapter::of(
            std::path::Path::new("/home/anna/.alo/adapters/invoices.safetensors"),
            "qwen2.5-0.5b-instruct",
            "sha256-fdf756fa",
            LearnedFrom {
                folders: vec![PathBuf::from("/home/anna/Invoices")],
                under_the_grant_to: "@adapting".to_owned(),
                taken_at: SystemTime::UNIX_EPOCH,
                learned_from: 8,
                left_out: 0,
            },
            SystemTime::UNIX_EPOCH,
        )
    }

    /// **The entry says whose it is, what it learned from, and under which
    /// grant** — the grant because that is what makes revocation mechanical.
    #[test]
    fn an_adapted_model_is_listed_as_the_persons_own_with_its_grant() {
        let yours = Yours::of(&an_adapter(), "2026-09-17", 0, 20, false);
        assert!(!yours.is_one_of_ours());
        assert_eq!(yours.under_the_grant_to, "@adapting");
        assert_eq!(
            yours.from_these_files,
            vec![PathBuf::from("/home/anna/Invoices")]
        );
        assert_eq!(yours.made_on, "2026-09-17");
        assert_eq!(yours.over, "qwen2.5-0.5b-instruct");
    }

    /// **A model that does not drive the verbs says so, with both numbers**, and
    /// is not given the agent — the same bar, and no allowance for being the
    /// person's own.
    #[test]
    fn a_model_that_does_not_drive_the_verbs_is_not_the_agent_and_says_why() {
        let did_not = Yours::of(&an_adapter(), "2026-09-17", 0, 20, false);
        assert_eq!(did_not.drove, (0, 20));
        assert_eq!(did_not.the_agent, TheAgent::ItAnswersButDoesNotDrive);

        let cleared = Yours::of(&an_adapter(), "2026-09-17", 19, 20, true);
        assert_eq!(cleared.the_agent, TheAgent::ItDrivesTheVerbs);
    }
}
