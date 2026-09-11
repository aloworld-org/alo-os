//! Where a change stands with the running daemon, once it is on the disk.
//!
//! Every value of this type is about a change that has **already been kept**:
//! the file was replaced whole before any of the three below could exist, so
//! none of them is a failure of the change itself — that is
//! [`crate::NotChanged`], and it happens before there is anything to knock
//! about.
//!
//! What the three tell apart is the daemon. A person revoking something
//! worrying is owed the difference between *the running agent has already
//! been told* ([`Stood::Heard`]), *nothing is running, so it takes effect at
//! the next sign-in* ([`Stood::AtTheNextSignIn`]) and *a daemon is running
//! and could not read the list again, so it is still serving under the old
//! one* ([`Stood::TurnedAway`]) — the last of which is the one a machine must
//! never dress up as either of the others, because it is the only one with
//! something left to look at.

use alo_strings::{Filling, Strings};

use crate::words;

/// Where a change that is already on the disk stands with the running daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stood {
    /// The running daemon read the list again and serves under it now.
    Heard {
        /// How many grants are in force there after the reading — the
        /// daemon's own count, carried rather than recomputed, so the surface
        /// and the service cannot disagree about what the knock came to.
        holding: u64,
    },
    /// Nobody was at the door. The change stands on the disk and applies at
    /// the next sign-in, which is the state this repository shipped with
    /// before the knock existed — not an error, and nothing more to do.
    AtTheNextSignIn,
    /// A daemon answered and could not read the list again, so it is still
    /// serving under the list it had. The change stands on the disk; what to
    /// show is the daemon's own sentence, carried rather than reworded,
    /// because a machine with two accounts of one moment is a machine a
    /// person cannot check.
    TurnedAway {
        /// What the daemon said, in the language the person reads.
        told: String,
    },
}

impl Stood {
    /// Whether the running daemon already serves under the change.
    #[must_use]
    pub const fn was_heard(&self) -> bool {
        matches!(self, Self::Heard { .. })
    }

    /// The sentence to put beside the outcome, when there is one.
    ///
    /// Nothing for [`Stood::Heard`] — the outcome's own sentence (the grant
    /// made, the revocation done) is the whole story there. The other two
    /// each have one thing to add: the declared sentence for a change that
    /// waits for the next sign-in, and the daemon's own words when it turned
    /// the re-reading away.
    #[must_use]
    pub fn explained(&self, strings: &Strings) -> Option<String> {
        match self {
            Self::Heard { .. } => None,
            Self::AtTheNextSignIn => Some(
                strings
                    .say(&words::AT_THE_NEXT_SIGN_IN.key(), &Filling::nothing())
                    .text()
                    .to_owned(),
            ),
            Self::TurnedAway { told } => Some(told.clone()),
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

    /// This crate's own words, for the sentence below.
    fn in_english() -> Strings {
        Strings::of(crate::changing_words().unwrap())
    }

    /// **A change the daemon heard needs no sentence beside it** — the
    /// outcome's own is the whole story — and it says it was heard.
    #[test]
    fn a_change_that_was_heard_has_nothing_to_add() {
        let stood = Stood::Heard { holding: 3 };
        assert!(stood.was_heard());
        assert_eq!(stood.explained(&in_english()), None);
    }

    /// **A change with nobody at the door says when it takes effect and that
    /// there is nothing more to do** — in the declared sentence, not in
    /// hardcoded English.
    #[test]
    fn a_change_for_the_next_sign_in_is_explained_in_the_declared_sentence() {
        let stood = Stood::AtTheNextSignIn;
        assert!(!stood.was_heard());
        let explained = stood.explained(&in_english()).unwrap();
        assert!(explained.contains("next sign-in"), "{explained}");
        assert!(explained.contains("nothing more to do"), "{explained}");
    }

    /// **A daemon that turned the re-reading away is answered in its own
    /// words**, carried rather than reworded — and never read as heard.
    #[test]
    fn a_daemon_that_turned_it_away_keeps_its_own_sentence() {
        let stood = Stood::TurnedAway {
            told: "Was gewährt ist, wurde nicht neu gelesen".to_owned(),
        };
        assert!(!stood.was_heard());
        assert_eq!(
            stood.explained(&in_english()).unwrap(),
            "Was gewährt ist, wurde nicht neu gelesen"
        );
    }
}
