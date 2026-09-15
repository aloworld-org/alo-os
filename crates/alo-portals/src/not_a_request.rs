//! Why what arrived was never a request at all.
//!
//! A portal request comes from outside — from an application, through the
//! backend task 5 of the applications plan builds — and is checked at the
//! boundary before anything is judged, the way `alo_capability::Verbs::call`
//! checks a verb's arguments before `Grants::permitting` is asked. A refusal
//! here is not a *no* from the grants: nothing was asked of them, because there
//! was no well-formed question to ask.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a request could not be made.
///
/// No `Display`, for `alo_capability::GrantError`'s reason: the one road to
/// words is [`NotARequest::said`], which takes the strings the reader reads.
///
/// Written into the answers file by its kebab-case name, `not-an-identifier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotARequest {
    /// No application was named.
    NoApplication,
    /// An application's identifier with a space, a control character or too
    /// many bytes in it.
    NotAnIdentifier,
    /// A portal about a file, asked without one.
    NeedsAPath,
    /// A portal that is not about a file, asked with one — or open-with asked
    /// to name an application for a portal that is not open-with.
    NotOverAPath,
    /// A relative path.
    NotAFullPath,
    /// A path with `..` in it.
    CouldLeadElsewhere,
}

impl NotARequest {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::NoApplication => words::NO_APPLICATION,
            Self::NotAnIdentifier => words::NOT_AN_IDENTIFIER,
            Self::NeedsAPath => words::NEEDS_A_PATH,
            Self::NotOverAPath => words::NOT_OVER_A_PATH,
            Self::NotAFullPath => words::NOT_A_FULL_PATH,
            Self::CouldLeadElsewhere => words::COULD_LEAD_ELSEWHERE,
        }
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}
