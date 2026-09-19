//! What one check answered.
//!
//! Where the machine stands, the build it stood on when it was asked, why it
//! was asked and when. The first is `alo_keeping_up::Standing` and is not
//! decided again here; the rest is what makes an answer readable later without
//! being read *wrongly* later.
//!
//! # Why the build it was about is carried
//!
//! Because *an update is ready* is a sentence about a particular machine at a
//! particular moment, and the moment ends. A machine that applied an update, or
//! went back, or was updated by somebody else at a root shell, is no longer the
//! machine the answer was about — and a surface that showed the kept sentence
//! anyway would be telling somebody an update is ready for a system they are no
//! longer running. So the answer names the build it was about, and
//! [`crate::TheAnswer::said`] refuses rather than saying something stale.

use std::time::SystemTime;

use alo_keeping_up::{Digest, Standing};
use alo_strings::{Said, Strings};

use crate::because::Because;

/// What one check answered.
///
/// Not `Deserialize`, for `alo_keeping_up::Offered`'s reason: an answer read
/// back off a disk would be a check nobody was shown making.
/// [`crate::TheAnswer`] is what a kept answer becomes, and it is deliberately
/// a different type with less in it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Found {
    /// The build this machine was running when it asked.
    about: Digest,
    /// Where it stands.
    standing: Standing,
    /// Why it asked.
    because: Because,
    /// When it asked.
    at: SystemTime,
}

impl Found {
    /// What a check answered.
    ///
    /// Made by [`crate::look`] alone, which is what holds it to having been on
    /// the indicator: the `Standing` it carries can only have come from an
    /// `Offered`, and an `Offered` only from an errand that was shown.
    pub(crate) fn of(about: Digest, standing: Standing, because: Because, at: SystemTime) -> Self {
        Self {
            about,
            standing,
            because,
            at,
        }
    }

    /// The build this machine was running when it asked.
    #[must_use]
    pub fn about(&self) -> &Digest {
        &self.about
    }

    /// Where this machine stands.
    #[must_use]
    pub fn standing(&self) -> &Standing {
        &self.standing
    }

    /// Whether there is an update.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.standing.is_ready()
    }

    /// Why this machine asked.
    #[must_use]
    pub fn because(&self) -> Because {
        self.because
    }

    /// When it asked.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// What a person is told: that an update is ready, or that the machine is
    /// up to date. `alo_keeping_up::Standing::said`, unchanged.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        self.standing.said(strings)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_moment, build, in_english};

    /// An answer says where the machine stands, which build it was about, why
    /// it was asked and when — and the sentence is the one that already
    /// existed.
    #[test]
    fn an_answer_says_where_the_machine_stands_and_what_it_was_about() {
        let found = Found::of(
            build("aa"),
            Standing::UpToDate,
            Because::ThePersonAsked,
            a_moment(),
        );
        assert_eq!(found.about(), &build("aa"));
        assert!(!found.is_ready());
        assert_eq!(found.because(), Because::ThePersonAsked);
        assert_eq!(found.at(), a_moment());
        assert_eq!(
            found.said(&in_english()).text(),
            "This machine is up to date"
        );
    }

    /// **It is written down without a person's name on it and without an
    /// agent's**, because a check is neither: it is something the machine did.
    #[test]
    fn an_answer_written_down_names_no_agent() {
        let written = serde_json::to_string(&Found::of(
            build("aa"),
            Standing::UpToDate,
            Because::ThisMachineStarted,
            a_moment(),
        ))
        .unwrap();
        assert!(written.contains("this-machine-started"), "{written}");
        assert!(!written.contains("agent"), "{written}");
        assert!(!written.contains("grantee"), "{written}");
    }
}
