//! Where a change stands with the running daemon, once it has been made.
//!
//! Every value of this type is about a change that **happened**. For a grant
//! the file was replaced whole before any of the first three below could
//! exist; for a pairing the daemon said it took the pairing away before either
//! of the last two could. None of them is a failure of the change itself —
//! that is [`crate::NotChanged`], and it happens before there is anything to
//! say about a daemon.
//!
//! What the first three tell apart is the daemon. A person revoking something
//! worrying is owed the difference between *the running agent has already
//! been told* ([`Stood::Heard`]), *nothing is running, so it takes effect at
//! the next sign-in* ([`Stood::AtTheNextSignIn`]) and *a daemon is running
//! and could not read the list again, so it is still serving under the old
//! one* ([`Stood::TurnedAway`]) — the last of which is the one a machine must
//! never dress up as either of the others, because it is the only one with
//! something left to look at.
//!
//! The last two are a pairing's, whose file the daemon writes itself. The
//! daemon took it away at once and wrote that down ([`Stood::KeptByTheDaemon`]),
//! or took it away at once and could not write it down
//! ([`Stood::UntilARestart`]) — which is said, because a pairing that returns
//! after a restart is a pairing the person has to revoke again, and a machine
//! that let them believe otherwise would be lying by omission.

use alo_strings::{Filling, Strings};

use crate::words;

/// Where a change that has been made stands with the running daemon.
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
    /// The running daemon made the change itself, at once, and wrote it down
    /// in the file it alone writes. A pairing's revocation, done.
    KeptByTheDaemon,
    /// The running daemon made the change at once and could not write it
    /// down: it holds until this machine restarts, and after a restart the
    /// file the daemon reads says what it said before.
    UntilARestart,
}

impl Stood {
    /// Whether the running daemon already serves under the change.
    #[must_use]
    pub const fn was_heard(&self) -> bool {
        matches!(
            self,
            Self::Heard { .. } | Self::KeptByTheDaemon | Self::UntilARestart
        )
    }

    /// The sentence to put beside the outcome, when there is one.
    ///
    /// Nothing for [`Stood::Heard`] and [`Stood::KeptByTheDaemon`] — the
    /// outcome's own sentence (the grant made, the revocation done) is the
    /// whole story there. The others each have one thing to add: the declared
    /// sentence for a change that waits for the next sign-in, the daemon's own
    /// words when it turned the re-reading away, and the declared sentence for
    /// a revocation that lasts only until a restart.
    #[must_use]
    pub fn explained(&self, strings: &Strings) -> Option<String> {
        match self {
            Self::Heard { .. } | Self::KeptByTheDaemon => None,
            Self::UntilARestart => Some(
                strings
                    .say(&words::UNTIL_A_RESTART.key(), &Filling::nothing())
                    .text()
                    .to_owned(),
            ),
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

    /// **A pairing the daemon revoked and wrote down needs no sentence
    /// beside it**, and the running daemon already serves under it.
    #[test]
    fn a_pairing_the_daemon_kept_has_nothing_to_add() {
        let stood = Stood::KeptByTheDaemon;
        assert!(stood.was_heard());
        assert_eq!(stood.explained(&in_english()), None);
    }

    /// **A revocation that lasts only until a restart says so**, in the
    /// declared sentence — the one case where the person has something left
    /// to do after a restart.
    #[test]
    fn a_revocation_until_a_restart_says_it_comes_back() {
        let stood = Stood::UntilARestart;
        assert!(stood.was_heard(), "the running daemon did take it away");
        let explained = stood.explained(&in_english()).unwrap();
        assert!(explained.contains("restart"), "{explained}");
        assert!(explained.contains("again"), "{explained}");
    }
}
