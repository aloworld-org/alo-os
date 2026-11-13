//! **The person signed in to GitHub, or out of it, from their own desktop** —
//! or tried, and what came of it.
//!
//! The owner's decision of 2026-11-13, on the GitHub sign-in sheet and Settings
//! → Accounts: *every sign-in, sign-out and failure is recorded.* The desktop
//! runs GitHub's own command, `gh`, under the person's account
//! (`gh auth login --web`, `gh auth logout --hostname github.com`), and tells
//! `alo-agentd` — this record's one writer — what came of it, on the person's
//! own door (`github-sign-in`, `docs/contracts/daemon-protocol.md`).
//!
//! **No agent, and no field for one**: the person acted. There is no verb that
//! signs in to GitHub, and an agent cannot send the request.
//!
//! **Identities, never sentences.** What is kept is *which* thing happened —
//! signed in, signed out, which of four reasons a sign-in did not finish — and
//! never an English sentence about it, so a person reading their record in
//! Greek reads it in Greek. The two pieces of text kept are facts that are not
//! this machine's to word: the **login** GitHub knows the person by, which is
//! a name rather than a sentence and is held to GitHub's own shape for one, and
//! **the last line `gh` printed** when it failed, which is that program's own
//! account and is quoted rather than translated.
//!
//! **Never a token, a code or a password.** The one-time code `gh` shows is not
//! kept, and there is no field it could go in.
//!
//! **Not egress, as this machine counts it.** `gh` reached GitHub under the
//! person's own account, from their own desktop; nothing this service or an
//! agent caused left, so [`crate::Happened::caused_egress`] is false.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::happened::Happened;
use crate::line::Line;

/// The longest login kept.
///
/// GitHub's own limit is 39 characters; an account an organisation manages
/// adds an underscore and its short name, so the bound here is generous rather
/// than GitHub's exact one, and still short enough that a login cannot be a
/// paragraph.
pub const LONGEST_LOGIN: usize = 100;

/// What happened at GitHub, as the person's desktop reported it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AtGitHub {
    /// They signed in, and GitHub knows them by this login.
    SignedIn {
        /// The login, as GitHub writes it.
        login: Line,
    },
    /// They signed out.
    SignedOut {
        /// The login that was signed out — absent when the desktop did not
        /// know which.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        login: Option<Line>,
    },
    /// They began to sign in, and it did not finish.
    DidNotSignIn {
        /// Why not.
        why: WhyNotSignedIn,
    },
    /// They asked to sign out, and it did not work.
    DidNotSignOut {
        /// The last line `gh` printed — absent when it printed nothing.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        said: Option<Line>,
    },
}

/// Why a sign-in to GitHub did not finish.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhyNotSignedIn {
    /// `gh` is not on this computer, so nothing could be started.
    GhIsNotHere,
    /// `gh` ended with an error.
    GhFailed {
        /// The last line it printed — absent when it printed nothing.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        said: Option<Line>,
    },
    /// `gh` never showed the one-time code the person types at GitHub.
    NoCodeArrived,
    /// The person cancelled it.
    TheyCancelled,
}

impl AtGitHub {
    /// They signed in as `login` — or [`None`] when `login` is not one.
    #[must_use]
    pub fn signed_in(login: &str) -> Option<Self> {
        Some(Self::SignedIn {
            login: a_login(login)?,
        })
    }

    /// They signed out, of `login` when the desktop knew which — or [`None`]
    /// when what it gave is not a login.
    #[must_use]
    pub fn signed_out(login: Option<&str>) -> Option<Self> {
        Some(Self::SignedOut {
            login: match login {
                Some(login) => Some(a_login(login)?),
                None => None,
            },
        })
    }

    /// A sign-in that did not finish, and why.
    #[must_use]
    pub const fn did_not_sign_in(why: WhyNotSignedIn) -> Self {
        Self::DidNotSignIn { why }
    }

    /// A sign-out that did not work, with the last line `gh` printed.
    #[must_use]
    pub fn did_not_sign_out(said: Option<&str>) -> Self {
        Self::DidNotSignOut {
            said: what_gh_said(said),
        }
    }

    /// The login this is about, where it names one.
    #[must_use]
    pub fn login(&self) -> Option<&Line> {
        match self {
            Self::SignedIn { login } => Some(login),
            Self::SignedOut { login } => login.as_ref(),
            Self::DidNotSignIn { .. } | Self::DidNotSignOut { .. } => None,
        }
    }

    /// The last line `gh` printed, where it failed and printed one.
    #[must_use]
    pub fn what_gh_said(&self) -> Option<&Line> {
        match self {
            Self::DidNotSignIn {
                why: WhyNotSignedIn::GhFailed { said },
            }
            | Self::DidNotSignOut { said } => said.as_ref(),
            Self::SignedIn { .. } | Self::SignedOut { .. } | Self::DidNotSignIn { .. } => None,
        }
    }

    /// Whether what the person asked for did not happen.
    ///
    /// A sign-in the person cancelled is not counted: nothing refused them,
    /// and *they changed their mind* is not a failure a review is looking for.
    #[must_use]
    pub const fn did_not_happen(&self) -> bool {
        matches!(
            self,
            Self::DidNotSignOut { .. }
                | Self::DidNotSignIn {
                    why: WhyNotSignedIn::GhIsNotHere
                        | WhyNotSignedIn::GhFailed { .. }
                        | WhyNotSignedIn::NoCodeArrived,
                }
        )
    }

    /// This, made again from text that may not have been through [`Line::of`]
    /// — what arrived on a wire — or [`None`] when a login in it is not one.
    ///
    /// A [`Line`] read back by `serde` is whatever the text was, so anything
    /// that reads this off a socket comes through here before it is kept.
    #[must_use]
    pub fn checked(self) -> Option<Self> {
        Some(match self {
            Self::SignedIn { login } => Self::signed_in(login.as_str())?,
            Self::SignedOut { login } => Self::signed_out(login.as_ref().map(Line::as_str))?,
            Self::DidNotSignIn {
                why: WhyNotSignedIn::GhFailed { said },
            } => Self::did_not_sign_in(WhyNotSignedIn::gh_failed(said.as_ref().map(Line::as_str))),
            Self::DidNotSignIn { why } => Self::DidNotSignIn { why },
            Self::DidNotSignOut { said } => Self::did_not_sign_out(said.as_ref().map(Line::as_str)),
        })
    }
}

impl WhyNotSignedIn {
    /// `gh` ended with an error, and this was the last line it printed.
    #[must_use]
    pub fn gh_failed(said: Option<&str>) -> Self {
        Self::GhFailed {
            said: what_gh_said(said),
        }
    }
}

impl Entry {
    /// The person signed in to GitHub or out of it, or tried — see
    /// [`AtGitHub`].
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn at_github(what: AtGitHub, at: SystemTime) -> Self {
        Self::new(at, Happened::AtGitHub { what })
    }
}

/// `text` as a login, or [`None`] when it is not shaped like one.
///
/// Letters, digits, hyphens and underscores, not beginning with a hyphen, and
/// no longer than [`LONGEST_LOGIN`]. Exact rather than cleaned up: a login with
/// a space or a newline in it is not a login with the space taken out.
fn a_login(text: &str) -> Option<Line> {
    let shaped = !text.is_empty()
        && text.chars().count() <= LONGEST_LOGIN
        && !text.starts_with('-')
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    shaped.then(|| Line::of(text))
}

/// The last line `gh` printed, through [`Line::of`] — and [`None`] when there
/// are no words in it.
fn what_gh_said(said: Option<&str>) -> Option<Line> {
    said.map(Line::of).filter(|line| !line.is_empty())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn every_kind() -> Vec<AtGitHub> {
        vec![
            AtGitHub::signed_in("anna").unwrap(),
            AtGitHub::signed_out(Some("anna")).unwrap(),
            AtGitHub::signed_out(None).unwrap(),
            AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
            AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some(
                "error connecting to api.github.com",
            ))),
            AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(None)),
            AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived),
            AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
            AtGitHub::did_not_sign_out(Some("not logged in to github.com")),
            AtGitHub::did_not_sign_out(None),
        ]
    }

    /// **Every kind survives being written to the record and read back**, as
    /// an entry, and names no agent.
    #[test]
    fn every_kind_is_kept_and_read_back_naming_nobody() {
        for what in every_kind() {
            let entry = Entry::at_github(what.clone(), SystemTime::UNIX_EPOCH);
            assert_eq!(entry.agent(), None);
            assert!(!entry.happened().caused_egress());
            assert!(!entry.happened().ran());
            let written = serde_json::to_string(&entry).unwrap();
            assert!(written.contains(r#""at-github":{"what":"#), "{written}");
            let back: Entry = serde_json::from_str(&written).unwrap();
            assert_eq!(back, entry, "{written}");
            assert_eq!(back.happened(), &Happened::AtGitHub { what });
        }
    }

    /// **What the file holds is identities, not sentences**: the reasons are
    /// tags a reader in any language can word, and the only text is the
    /// login and what `gh` itself printed.
    #[test]
    fn the_file_holds_identities_and_not_sentences() {
        let written =
            serde_json::to_string(&AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere)).unwrap();
        assert_eq!(written, r#"{"did-not-sign-in":{"why":"gh-is-not-here"}}"#);
        let written = serde_json::to_string(&AtGitHub::signed_in("anna").unwrap()).unwrap();
        assert_eq!(written, r#"{"signed-in":{"login":"anna"}}"#);
        let written = serde_json::to_string(&AtGitHub::signed_out(None).unwrap()).unwrap();
        assert_eq!(written, r#"{"signed-out":{}}"#);
    }

    /// **A login that is not one is refused**, rather than cleaned up into
    /// one nobody has — and never kept as a sentence wearing a login's name.
    #[test]
    fn a_login_that_is_not_one_is_refused() {
        let too_long = "a".repeat(LONGEST_LOGIN + 1);
        let long_enough = "a".repeat(LONGEST_LOGIN);
        for not_one in [
            "",
            "-anna",
            "anna smith",
            "anna\nsigned out of everything",
            "anna\u{1b}[2K",
            "ánna",
            too_long.as_str(),
        ] {
            assert_eq!(AtGitHub::signed_in(not_one), None, "{not_one:?}");
            assert_eq!(AtGitHub::signed_out(Some(not_one)), None, "{not_one:?}");
        }
        for one in [
            "anna",
            "anna-smith",
            "anna_acme",
            "A1",
            long_enough.as_str(),
        ] {
            assert!(AtGitHub::signed_in(one).is_some(), "{one:?}");
        }
    }

    /// **What `gh` printed cannot rewrite the record**, and a line with no
    /// words in it is kept as nothing rather than as an empty quotation.
    #[test]
    fn what_gh_printed_is_one_line_or_nothing() {
        let failed = AtGitHub::did_not_sign_out(Some("failed\r\nsigned in as root"));
        assert!(
            failed
                .what_gh_said()
                .is_some_and(|said| said.is("failed signed in as root"))
        );
        assert_eq!(
            AtGitHub::did_not_sign_out(Some(" \n\t")).what_gh_said(),
            None
        );
    }

    /// **What arrived on a wire is held to the same rules as what was made
    /// here**: a login read back by `serde` is checked again, and a quotation
    /// is put through [`Line::of`] again.
    #[test]
    fn what_arrived_on_a_wire_is_checked_again() {
        let arrived: AtGitHub =
            serde_json::from_str(r#"{"signed-in":{"login":"anna smith"}}"#).unwrap();
        assert_eq!(arrived.checked(), None);
        let arrived: AtGitHub =
            serde_json::from_str(r#"{"did-not-sign-out":{"said":"a\nb"}}"#).unwrap();
        assert_eq!(
            arrived.checked(),
            Some(AtGitHub::did_not_sign_out(Some("a b")))
        );
        for what in every_kind() {
            assert_eq!(what.clone().checked(), Some(what));
        }
    }

    /// **A failure is among the refusals and a cancelled sign-in is not**:
    /// something the person asked for did not happen in the one, and the
    /// person changed their mind in the other.
    #[test]
    fn a_failure_is_counted_and_a_cancel_is_not() {
        for what in every_kind() {
            let happened = Happened::AtGitHub { what: what.clone() };
            assert_eq!(happened.was_stopped(), what.did_not_happen(), "{what:?}");
            assert_eq!(happened.why_stopped(), what.what_gh_said(), "{what:?}");
        }
        assert!(!AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled).did_not_happen());
        assert!(AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived).did_not_happen());
        assert!(!AtGitHub::signed_in("anna").unwrap().did_not_happen());
    }
}
