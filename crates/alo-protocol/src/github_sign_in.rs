//! What the person's desktop says happened at GitHub, as the wire spells it.
//!
//! `github-sign-in` carries one of the things `alo_record::AtGitHub` can keep —
//! signed in, signed out, a sign-in that did not finish and why, a sign-out
//! that did not work — and nothing else. Its shape here is the record's shape,
//! field for field, with two differences that are the reason this file exists:
//!
//! - **a field nobody declared is not a request**, as everywhere on either
//!   door, so a desktop that also sent a token, a code or a moment is refused
//!   rather than quietly half-read;
//! - **what arrives is text, not a record's line**, so it is made into one by
//!   `alo-record`'s own constructors — a login held to the shape of a login,
//!   and gh's last line put through `alo_record::Line::of` — and a login that
//!   is not one makes the whole message unreadable.
//!
//! Which is why the daemon is never handed something it would have to check
//! again: what [`crate::FromAPerson::GitHubSignIn`] carries was made by those
//! constructors and by nothing else.

use alo_record::{AtGitHub, WhyNotSignedIn};
use serde::{Deserialize, Serialize};

/// One thing that happened at GitHub, as it crosses the socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum OnTheWire {
    /// They signed in, as this login.
    SignedIn {
        /// The login, as GitHub writes it.
        login: String,
    },
    /// They signed out, of this login when the desktop knew which.
    SignedOut {
        /// The login, or absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        login: Option<String>,
    },
    /// A sign-in that did not finish.
    DidNotSignIn {
        /// Why not.
        why: WhyNotOnTheWire,
    },
    /// A sign-out that did not work.
    DidNotSignOut {
        /// The last line gh printed, or absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        said: Option<String>,
    },
}

/// Why a sign-in did not finish, as it crosses the socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum WhyNotOnTheWire {
    /// gh is not on this computer.
    GhIsNotHere,
    /// gh ended with an error.
    GhFailed {
        /// The last line it printed, or absent.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        said: Option<String>,
    },
    /// gh never showed the one-time code.
    NoCodeArrived,
    /// The person cancelled it.
    TheyCancelled,
}

impl OnTheWire {
    /// What the record can keep, or [`None`] when a login here is not one.
    pub(crate) fn kept(self) -> Option<AtGitHub> {
        match self {
            Self::SignedIn { login } => AtGitHub::signed_in(&login),
            Self::SignedOut { login } => AtGitHub::signed_out(login.as_deref()),
            Self::DidNotSignIn { why } => Some(AtGitHub::did_not_sign_in(match why {
                WhyNotOnTheWire::GhIsNotHere => WhyNotSignedIn::GhIsNotHere,
                WhyNotOnTheWire::GhFailed { said } => WhyNotSignedIn::gh_failed(said.as_deref()),
                WhyNotOnTheWire::NoCodeArrived => WhyNotSignedIn::NoCodeArrived,
                WhyNotOnTheWire::TheyCancelled => WhyNotSignedIn::TheyCancelled,
            })),
            Self::DidNotSignOut { said } => Some(AtGitHub::did_not_sign_out(said.as_deref())),
        }
    }

    /// What the record keeps, as the wire spells it.
    pub(crate) fn of(what: &AtGitHub) -> Self {
        let text = |line: &alo_record::Line| line.as_str().to_owned();
        match what {
            AtGitHub::SignedIn { login } => Self::SignedIn { login: text(login) },
            AtGitHub::SignedOut { login } => Self::SignedOut {
                login: login.as_ref().map(text),
            },
            AtGitHub::DidNotSignIn { why } => Self::DidNotSignIn {
                why: match why {
                    WhyNotSignedIn::GhIsNotHere => WhyNotOnTheWire::GhIsNotHere,
                    WhyNotSignedIn::GhFailed { said } => WhyNotOnTheWire::GhFailed {
                        said: said.as_ref().map(text),
                    },
                    WhyNotSignedIn::NoCodeArrived => WhyNotOnTheWire::NoCodeArrived,
                    WhyNotSignedIn::TheyCancelled => WhyNotOnTheWire::TheyCancelled,
                },
            },
            AtGitHub::DidNotSignOut { said } => Self::DidNotSignOut {
                said: said.as_ref().map(text),
            },
        }
    }
}

#[cfg(test)]
#[path = "github_sign_in_tests.rs"]
mod tests;
