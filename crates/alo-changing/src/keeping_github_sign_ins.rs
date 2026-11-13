//! The desktop asks the daemon to keep a GitHub sign-in in the record.
//!
//! The owner's decision of 2026-11-13: *every sign-in, sign-out and failure is
//! recorded.* The desktop runs GitHub's own command, `gh`, and watches what
//! comes of it; it is not the record's writer — `alo-agentd` is — so it sends
//! what happened on the person's own door, as `github-sign-in`, and the daemon
//! keeps it. One connection, one line, one answer, exactly as the knock is.
//!
//! # Every failure says which, and none is read as kept
//!
//! A knock that is not heard is answered by the next sign-in, because the
//! change it announces is already on the disk. A sign-in is not: if the daemon
//! did not keep it, nothing did, so every way the conversation fails comes back
//! as its own [`GitHubSignInNotKept`] rather than being flattened into *fine*.
//! What the desktop does about it — say so, try again — is the desktop's; this
//! only never pretends.

use alo_protocol::{AtGitHub, FromAPerson, ToAPerson};

use crate::door::{TheDaemonsDoor, Unheard};

/// Why what happened at GitHub was not kept in the record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitHubSignInNotKept {
    /// Nobody was listening at the door: no daemon is running for this
    /// person, so nothing was written.
    NobodyThere,
    /// The door was reached and no readable answer came back in time, or the
    /// answer was not one this request can earn — so nobody knows whether it
    /// was written.
    NotAnswered,
    /// The daemon refused it, in its own words, in the person's language — a
    /// login that was not one, or a request it could not read.
    Refused {
        /// What the daemon said.
        told: String,
    },
}

impl TheDaemonsDoor {
    /// Ask the daemon to keep `what` in the record: what happened when the
    /// person signed in to GitHub or out of it, or tried.
    ///
    /// `Ok(())` only when the daemon answered that it was written down.
    ///
    /// # Errors
    ///
    /// [`GitHubSignInNotKept`], saying which of the three ways it was not.
    pub fn keep_a_github_sign_in(&self, what: &AtGitHub) -> Result<(), GitHubSignInNotKept> {
        match self.asked(&FromAPerson::GitHubSignIn(what.clone())) {
            Ok(ToAPerson::GitHubSignInKept) => Ok(()),
            Ok(ToAPerson::Refused(wording)) => Err(GitHubSignInNotKept::Refused {
                told: wording.text().to_owned(),
            }),
            Err(Unheard::NobodyThere) => Err(GitHubSignInNotKept::NobodyThere),
            // Any other answer is not one this request can earn, and says
            // nothing about whether it was kept.
            Ok(_) | Err(Unheard::NotAnswered) => Err(GitHubSignInNotKept::NotAnswered),
        }
    }
}
