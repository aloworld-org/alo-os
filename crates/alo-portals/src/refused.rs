//! Why a portal request was refused, and what a person is told.
//!
//! Two refusals, and the order they are reached in is the point.
//!
//! **Nothing granted at all** comes first. An application nobody has granted
//! anything is refused before any ask is looked at and before any dialog is
//! imagined: a portal backend that turned such a request into a question on the
//! person's screen would let any application put a question there, as often as
//! it liked, until the person answered yes to make it stop.
//!
//! **Not allowed this** is `alo-capability`'s own refusal, carried whole —
//! the grant that expired, or the one never made — with the portal it was asked
//! of beside it. Its words are that crate's, so a refusal of a portal request
//! and the grant a person reads in their list are worded by one crate.

use alo_capability::{Applicant, NotAllowed};
use alo_strings::{Filling, Said, Strings};

use crate::portal::Portal;
use crate::words;

/// Why a request was refused.
///
/// A value, worded when somebody shows it or writes it down, so the screen and
/// the record cannot be two accounts of one refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// The application holds no grant at all, so nothing was asked of the
    /// grants and nothing was put to the person.
    NothingGranted {
        /// The application that asked.
        application: Applicant,
        /// The portal it asked.
        portal: Portal,
    },
    /// The application holds grants, and none of them covers this.
    NotAllowed {
        /// The portal it asked.
        portal: Portal,
        /// The grants' own refusal.
        why: NotAllowed,
    },
}

impl Refused {
    /// The application that asked.
    #[must_use]
    pub fn application(&self) -> &Applicant {
        match self {
            Self::NothingGranted { application, .. } => application,
            Self::NotAllowed { why, .. } => why.application(),
        }
    }

    /// The portal it asked.
    #[must_use]
    pub const fn portal(&self) -> Portal {
        match self {
            Self::NothingGranted { portal, .. } | Self::NotAllowed { portal, .. } => *portal,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// For [`Refused::NotAllowed`] this is `alo-capability`'s sentence, not one
    /// written again here.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NothingGranted { application, .. } => strings.say(
                &words::NOTHING_GRANTED.key(),
                &Filling::of("application", application.as_str().to_owned()),
            ),
            Self::NotAllowed { why, .. } => why.said(strings),
        }
    }
}
