//! Every way the greeter is stopped by the machine rather than by a password,
//! and the two readers each of them has.
//!
//! Both types here carry **English for whoever maintains the machine** and a
//! **sentence for whoever is standing at the screen**, and they carry both
//! rather than choosing: a library that printed the English itself would be
//! picking somebody's log for them, and a library that dropped it would leave
//! the one person who could fix the machine with a sentence written for
//! somebody who cannot.
//!
//! Nothing in either of them can carry a password, a name typed at a prompt,
//! or a hash. [`NotReadable`] names a path and quotes `alo-accounts`' own
//! complaint about it; [`NotAnswered`] names a socket and quotes the machine's
//! complaint about that. Neither has anywhere for anything a person typed.

use alo_strings::{Filling, Said, Strings};

use crate::words::{ACCOUNTS_UNREADABLE, NOTHING_LISTENING, NOTHING_SAID, Word};

/// This machine's accounts are there and would not be believed.
///
/// Told apart from *there is no store* deliberately: no store at all is the
/// first-boot state and is [`crate::Standing::MakeAnAccount`], while a store
/// that will not read is a machine somebody has to go and look at. Making one
/// of them look like the other would either hide a broken machine or offer to
/// make a second account on a machine that already has one.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("the accounts at {at} would not be read, so nobody can sign in here: {why}")]
pub struct NotReadable {
    /// Where the store is.
    pub at: String,
    /// What `alo-accounts` said about it, in its own words.
    pub why: String,
}

impl NotReadable {
    /// What a person standing at the screen is told, in the language they
    /// read.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&ACCOUNTS_UNREADABLE.key(), &Filling::nothing())
    }
}

/// Every way the conversation with the opener fails to happen.
///
/// It matters that this is four values and not a silence. Somebody standing at
/// a sign-in that did nothing cannot tell a refusal from a machine that is not
/// running, and the two call for opposite things: retyping a password, and
/// going to look at a service. Each of these says which, in the person's own
/// language ([`NotAnswered::said`]) and in the maintainer's ([`Display`]).
///
/// [`Display`]: std::fmt::Display
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum NotAnswered {
    /// Nothing is listening at the door at all.
    ///
    /// The service is not running, or its socket is not where it is expected.
    /// The one of these whoever maintains the machine can act on immediately.
    #[error(
        "nothing is listening at {at}, so no session could be asked for: {why} — alo-sessiond is \
         the service that opens one (ADR 0024)"
    )]
    NobodyThere {
        /// The door that was knocked on.
        at: String,
        /// What the machine said about reaching it.
        why: String,
    },

    /// The door was reached and the knock could not be delivered.
    #[error("the knock at {at} could not be delivered: {why}")]
    WentAway {
        /// The door that was knocked on.
        at: String,
        /// What the machine said.
        why: String,
    },

    /// The knock was delivered and nothing came back.
    ///
    /// A door that closed without answering, or one that took longer than the
    /// patience a person standing at a screen has.
    #[error("the door at {at} took the knock and said nothing: {why}")]
    NothingCameBack {
        /// The door that was knocked on.
        at: String,
        /// What the machine said, or what was noticed about the silence.
        why: String,
    },

    /// Something came back and it is not an answer.
    ///
    /// Read by whoever wrote the other end: `alo-sessiond` says `opened`, or
    /// `refused` and one key, and nothing else is read leniently.
    #[error("the door at {at} answered something that is not an answer: {why}")]
    NotAnAnswer {
        /// The door that was knocked on.
        at: String,
        /// What `alo-sessiond`'s own reader said about the line.
        why: String,
    },
}

impl NotAnswered {
    /// The string this crate declares for this failure.
    ///
    /// Two sentences for four values, and the split is by what the reader can
    /// do: *nothing is running there* against *something is and did not
    /// finish*. Four sentences would be four ways of saying a thing the person
    /// cannot act on; one would throw away the only distinction that sends a
    /// maintainer to a different place.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NobodyThere { .. } => NOTHING_LISTENING,
            Self::WentAway { .. } | Self::NothingCameBack { .. } | Self::NotAnAnswer { .. } => {
                NOTHING_SAID
            }
        }
    }

    /// What a person standing at the screen is told, in the language they
    /// read.
    ///
    /// Never fails and never panics: a `Strings` that was never given this
    /// crate's words answers with the key, marked as the bug it is, and
    /// nobody was signed in either way.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        // No gap in either sentence, deliberately: a gap is the one road text
        // from a machine — a socket path, a library's complaint — could take
        // into a sentence somebody reads at a sign-in prompt.
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// Which door this was about.
    #[must_use]
    pub fn at(&self) -> &str {
        match self {
            Self::NobodyThere { at, .. }
            | Self::WentAway { at, .. }
            | Self::NothingCameBack { at, .. }
            | Self::NotAnAnswer { at, .. } => at,
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

    /// This crate's own words, for the sentences below.
    fn in_english() -> Strings {
        Strings::of(crate::greeting_words().unwrap())
    }

    /// One of each, so a test can walk every way this can go wrong.
    fn every_way() -> [NotAnswered; 4] {
        [
            NotAnswered::NobodyThere {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "No such file or directory".to_owned(),
            },
            NotAnswered::WentAway {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "Broken pipe".to_owned(),
            },
            NotAnswered::NothingCameBack {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "the door closed without answering".to_owned(),
            },
            NotAnswered::NotAnAnswer {
                at: "/run/alo-sessiond/sign-in.sock".to_owned(),
                why: "`hello` is not an answer".to_owned(),
            },
        ]
    }

    /// **Every way the conversation can fail to happen is told in words**, and
    /// none of them reaches a person as a key. Silence here is the failure
    /// this type exists to prevent: somebody standing at a sign-in that did
    /// nothing cannot tell a refusal from a machine that is not running.
    #[test]
    fn every_way_it_can_fail_says_something_a_person_can_read() {
        let strings = in_english();
        for why in every_way() {
            let said = why.said(&strings);
            assert!(!said.is_a_bug(), "{} reached a person as a key", why);
            assert!(
                said.text().contains("Nothing you typed was wrong"),
                "{said}"
            );
        }
    }

    /// **Nothing running there and something that did not finish are two
    /// sentences**, because they send whoever maintains the machine to two
    /// different places.
    #[test]
    fn nothing_listening_and_nothing_said_are_two_sentences() {
        let strings = in_english();
        let [nobody, went_away, nothing_back, not_an_answer] = every_way();
        assert_eq!(nobody.word().named(), NOTHING_LISTENING.named());
        for why in [went_away, nothing_back, not_an_answer] {
            assert_eq!(why.word().named(), NOTHING_SAID.named());
        }
        assert_ne!(
            nobody.said(&strings).text(),
            every_way()[1].said(&strings).text()
        );
    }

    /// **The English keeps the detail the sentence deliberately drops**, and
    /// names the door — whoever reads it has a service to look at.
    #[test]
    fn the_english_names_the_door_and_what_the_machine_said() {
        for why in every_way() {
            let english = why.to_string();
            assert!(
                english.contains("/run/alo-sessiond/sign-in.sock"),
                "{english}"
            );
            assert_eq!(why.at(), "/run/alo-sessiond/sign-in.sock");
        }
        assert!(every_way()[0].to_string().contains("ADR 0024"));
    }

    /// **A store that will not read is told as itself**, naming the file for
    /// whoever has to fix it and saying nothing about it to the person.
    #[test]
    fn a_store_that_will_not_read_names_the_file_for_the_maintainer() {
        let refused = NotReadable {
            at: "/etc/alo/accounts.toml".to_owned(),
            why: "it is a symbolic link".to_owned(),
        };
        assert!(refused.to_string().contains("/etc/alo/accounts.toml"));
        assert!(refused.to_string().contains("symbolic link"));

        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(!said.text().contains("/etc/alo"), "{said}");
        assert!(said.text().contains("Nobody can sign in here"), "{said}");
    }
}
