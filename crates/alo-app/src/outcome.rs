//! **What came of one act**, in the two shapes the record keeps
//! (`docs/contracts/terminal-commands.md`, section 4).
//!
//! An act either reached the application — and was done, refused by it, or
//! sent with no answer — or it was refused before it reached anything. The
//! difference is what the record may keep: **a refusal before the application
//! keeps the ability and why, and not the arguments**, because what never
//! validated is whatever arrived.

use alo_record::{Answered, Entry, Line, Road, SentDuring};
use alo_strings::Said;

use crate::ended::Ended;

/// What came of one act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// It was carried to the application.
    Carried {
        /// Which road carried it.
        road: Road,
        /// What it was given, by name, after validation.
        arguments: Vec<(String, String)>,
        /// What the application, or the machine, answered.
        answered: Answered,
        /// How it ends.
        ended: Ended,
        /// What the person is told.
        said: Said,
    },
    /// It was refused before anything reached the application.
    NotAsked {
        /// How it ends.
        ended: Ended,
        /// What the person is told.
        said: Said,
    },
}

impl Outcome {
    /// Refused before anything reached the application.
    #[must_use]
    pub const fn not_asked(ended: Ended, said: Said) -> Self {
        Self::NotAsked { ended, said }
    }

    /// Refused before anything reached the application, boxed for a `Result`
    /// whose success is small.
    #[must_use]
    pub fn refusing(ended: Ended, said: Said) -> Box<Self> {
        Box::new(Self::NotAsked { ended, said })
    }

    /// Carried, and refused by the application in this sentence.
    #[must_use]
    pub fn refused_by_it(
        road: Road,
        arguments: Vec<(String, String)>,
        ended: Ended,
        said: Said,
    ) -> Self {
        Self::Carried {
            road,
            arguments,
            answered: Answered::Refused {
                said: Line::of(said.text()),
            },
            ended,
            said,
        }
    }

    /// How it ends.
    #[must_use]
    pub const fn ended(&self) -> Ended {
        match self {
            Self::Carried { ended, .. } | Self::NotAsked { ended, .. } => *ended,
        }
    }

    /// What the person is told.
    #[must_use]
    pub const fn said(&self) -> &Said {
        match self {
            Self::Carried { said, .. } | Self::NotAsked { said, .. } => said,
        }
    }

    /// The record's entry for it.
    ///
    /// `sent` is what the application sent while the act ran; it is kept only
    /// where the act was carried, because a refusal sent nothing on its behalf.
    #[must_use]
    pub fn entry(
        &self,
        application: &str,
        ability: &str,
        sent: SentDuring,
        kept_as: Option<&str>,
        at: std::time::SystemTime,
    ) -> Entry {
        match self {
            Self::Carried {
                road,
                arguments,
                answered,
                ..
            } => {
                let entry = Entry::asked_an_application(
                    application,
                    ability,
                    *road,
                    arguments,
                    answered.clone(),
                    sent,
                    at,
                );
                match kept_as {
                    Some(place) => entry.kept_as(place),
                    None => entry,
                }
            }
            Self::NotAsked { said, .. } => {
                Entry::an_application_was_not_asked(application, ability, said.text(), at)
            }
        }
    }
}
