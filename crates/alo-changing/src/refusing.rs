//! The two ways a change does not happen, and the words for each.
//!
//! Both happen **before the knock**: a change that was refused never reaches
//! the disk, and a change that never reached the disk is nothing to tell a
//! daemon about. That ordering is [`crate::Changing`]'s and is measured
//! there; this file owns what is said.
//!
//! [`NotChanged::NotAGrant`] keeps `alo-capability`'s own sentence, because
//! the words a person reads about a grant belong to the crate that decides
//! what a grant is — the same rule `alo-picking` follows for the same
//! refusals. [`NotChanged::NotKept`] keeps `alo-remembering`'s English inside
//! it for whoever is looking at the machine, and hands the person one
//! declared sentence, because there is one thing for them to know about every
//! way a file fails to be written: nothing moved.

use alo_capability::GrantError;
use alo_remembering::NotRemembered;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a change to the grants did not happen.
///
/// In every case the file, the list in memory and the running daemon are
/// exactly as they were: the change is applied to a copy, the copy is written
/// before it replaces anything, and the knock happens after both or not at
/// all.
#[derive(Debug, thiserror::Error)]
pub enum NotChanged {
    /// The machine refused the grant itself — an agent with no name, a
    /// duration of zero or of no end. `alo-capability`'s refusal, carried
    /// whole. Deliberately not `#[from]`-with-`source`: a `GrantError` has no
    /// English of its own — nobody using a machine reads one raw — and this
    /// crate keeps it that way rather than writing English for it here.
    #[error("the machine refused the grant, in its own words")]
    NotAGrant(GrantError),

    /// The file would not take the change. `alo-remembering`'s English names
    /// the file and what was wrong with it, for whoever is standing at the
    /// machine; the person is told one sentence, [`NotChanged::said`].
    #[error("the grants were not kept: {0}")]
    NotKept(#[from] NotRemembered),
}

impl From<GrantError> for NotChanged {
    fn from(why: GrantError) -> Self {
        Self::NotAGrant(why)
    }
}

impl NotChanged {
    /// What to say to the person, in the language they read.
    ///
    /// A refused grant is `alo-capability`'s own sentence. A write that
    /// failed is this crate's one sentence about it — nothing was granted and
    /// nothing was revoked — because the detail is the machine's log's, not
    /// the person's problem.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAGrant(why) => why.said(strings),
            Self::NotKept(_) => strings.say(&words::NOT_KEPT.key(), &Filling::nothing()),
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

    /// A vocabulary holding both this crate's words and the grants' own.
    fn in_english() -> Strings {
        let mut vocabulary = crate::changing_words().unwrap();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **A refused grant keeps the grants' own sentence** — the words a
    /// person reads about a grant belong to the crate that decides what one
    /// is.
    #[test]
    fn a_refused_grant_is_told_in_the_grants_own_words() {
        let said = NotChanged::from(GrantError::NoTime).said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
    }

    /// **A write that failed is one declared sentence**, whichever way the
    /// file failed — and it says that nothing moved.
    #[test]
    fn a_write_that_failed_is_one_declared_sentence() {
        let why = NotRemembered::NotWritten {
            at: std::path::PathBuf::from("/var/lib/alo/grants.toml"),
            why: "No space left on device".to_owned(),
        };
        let said = NotChanged::from(why).said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("nothing was granted"), "{said}");
        assert!(said.text().contains("nothing was revoked"), "{said}");
    }

    /// **The machine's own detail survives inside the refusal**, in English,
    /// for whoever reads the log — naming the file, which is where they go
    /// next.
    #[test]
    fn the_english_for_the_log_names_the_file() {
        let refused = NotChanged::from(NotRemembered::NotWritten {
            at: std::path::PathBuf::from("/var/lib/alo/grants.toml"),
            why: "No space left on device".to_owned(),
        });
        let written = refused.to_string();
        assert!(written.contains("/var/lib/alo/grants.toml"), "{written}");
        assert!(written.contains("No space left on device"), "{written}");
    }
}
