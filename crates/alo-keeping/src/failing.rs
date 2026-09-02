//! Why there is no record to write to, or none to read.
//!
//! Every one of these is read by somebody whose machine is not keeping evidence
//! of what its agent did, or is about to stop. So each says what state the
//! record is in and not only what went wrong: *nothing has been written to it*,
//! *the record is no longer all of what happened*, *nothing was removed*.
//!
//! **This is not a refusal**, in the sense `alo-capability` means. Nothing here
//! is the capability model saying no; nothing an agent asked for was stopped.
//! It is `alo-files`' distinction between `Failed` and `Refused` at one remove:
//! a record that could not be written is a machine problem, and a record that
//! said the grants refused something they did not would tell a security review
//! the opposite of the truth.
//!
//! # There is no way to turn one of these into English by accident
//!
//! No `Display`, since item 9b. The only road to words is [`NotKept::said`],
//! which takes the strings the person in front of the machine reads. What is
//! given up is `std::error::Error`, which these were never: they are sentences
//! a person reads, not errors a programmer matches on the text of.

use std::path::Path;

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why what happened is not being written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotKept {
    /// There is no record at that path.
    ///
    /// Read rather than repaired: a missing record is not an empty one, and
    /// making a fresh record over the top of a deleted one is how a deleted
    /// record becomes an innocent one.
    NotThere {
        /// Where it was looked for.
        path: String,
    },

    /// Something is at that path and it is not an alo OS record.
    NotARecord {
        /// What was named.
        path: String,
    },

    /// A record in a shape this version of alo OS does not know.
    FromANewerAlo {
        /// What was named.
        path: String,
        /// The shape it says it is in.
        format: u32,
    },

    /// A record with lines in it that cannot be read, which is never shortened.
    Damaged {
        /// What was named.
        path: String,
    },

    /// The record could not be opened at all.
    NotOpened {
        /// What was named.
        path: String,
        /// What the machine said.
        why: String,
    },

    /// One thing that happened could not be added to it.
    ///
    /// Almost always the disk. It also carries the one thing that cannot
    /// happen — an entry that could not be turned into a line — which is a
    /// `Result` rather than an unwrap for the reason `alo-files`' `Failed`
    /// keeps its unreachable variant: a library that panics inside the daemon
    /// takes the daemon with it, and this one runs on every execution.
    NotAddedTo {
        /// What was named.
        path: String,
        /// What the machine said.
        why: String,
    },

    /// The record could not be read back.
    NotRead {
        /// What was named.
        path: String,
        /// What the machine said.
        why: String,
    },

    /// The record could not be shortened, and nothing was removed.
    NotShortened {
        /// What was named.
        path: String,
        /// What the machine said.
        why: String,
    },
}

impl NotKept {
    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    /// A `Strings` that was never given [`crate::keeping_words`] answers with
    /// the key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotThere { path } => {
                strings.say(&words::NOT_THERE.key(), &Filling::of("path", path.clone()))
            }
            Self::NotARecord { path } => strings.say(
                &words::NOT_A_RECORD.key(),
                &Filling::of("path", path.clone()),
            ),
            Self::FromANewerAlo { path, format: _ } => strings.say(
                &words::FROM_A_NEWER_ALO.key(),
                &Filling::of("path", path.clone()),
            ),
            Self::Damaged { path } => {
                strings.say(&words::DAMAGED.key(), &Filling::of("path", path.clone()))
            }
            Self::NotOpened { path, why } => strings.say(
                &words::NOT_OPENED.key(),
                &Filling::of("path", path.clone()).and("why", why.clone()),
            ),
            Self::NotAddedTo { path, why } => strings.say(
                &words::NOT_ADDED_TO.key(),
                &Filling::of("path", path.clone()).and("why", why.clone()),
            ),
            Self::NotRead { path, why } => strings.say(
                &words::NOT_READ.key(),
                &Filling::of("path", path.clone()).and("why", why.clone()),
            ),
            Self::NotShortened { path, why } => strings.say(
                &words::NOT_SHORTENED.key(),
                &Filling::of("path", path.clone()).and("why", why.clone()),
            ),
        }
    }

    /// Whether the record still holds everything it held before this.
    ///
    /// True for every one of these but [`NotKept::NotAddedTo`], and that is the
    /// difference a daemon acts on: something that happened has not been
    /// written down, so the record is no longer complete and somebody has to be
    /// told rather than have it retried quietly.
    #[must_use]
    pub fn record_is_still_whole(&self) -> bool {
        !matches!(self, Self::NotAddedTo { .. })
    }

    /// The machine said no while doing this to the record.
    ///
    /// One place turns a `std::io::Error` into one of these, so that *it is not
    /// there* is the same answer wherever it happened rather than one answer
    /// per call site — `alo-files`' `Failed::machine` at one remove.
    pub(crate) fn opening(path: &Path, why: &std::io::Error) -> Self {
        Self::NotOpened {
            path: path.display().to_string(),
            why: why.to_string(),
        }
    }

    /// The machine said no while adding to the record.
    pub(crate) fn adding(path: &Path, why: &str) -> Self {
        Self::NotAddedTo {
            path: path.display().to_string(),
            why: why.to_owned(),
        }
    }

    /// The machine said no while reading the record — or there is none.
    pub(crate) fn reading(path: &Path, why: &std::io::Error) -> Self {
        if why.kind() == std::io::ErrorKind::NotFound {
            return Self::NotThere {
                path: path.display().to_string(),
            };
        }
        Self::NotRead {
            path: path.display().to_string(),
            why: why.to_string(),
        }
    }

    /// The machine said no while shortening the record.
    pub(crate) fn shortening(path: &Path, why: &std::io::Error) -> Self {
        Self::shortening_because(path, &why.to_string())
    }

    /// The same, where what stopped it was not the machine.
    pub(crate) fn shortening_because(path: &Path, why: &str) -> Self {
        Self::NotShortened {
            path: path.display().to_string(),
            why: why.to_owned(),
        }
    }

    /// Nothing at that path is a record.
    pub(crate) fn not_a_record(path: &Path) -> Self {
        Self::NotARecord {
            path: path.display().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{every_way_it_can_fail, in_english, said};
    use alo_strings::CameFrom;
    use std::io::{Error, ErrorKind};

    /// **A record that is not there is not a record that is empty.** A missing
    /// file is answered as missing, wherever the reading of it was attempted,
    /// and the sentence says why that is worth finding out about.
    #[test]
    fn a_record_that_is_not_there_says_so_rather_than_reporting_an_error_number() {
        let missing = NotKept::reading(
            Path::new("/var/lib/alo/record.jsonl"),
            &Error::new(ErrorKind::NotFound, "no such file or directory"),
        );
        assert!(matches!(missing, NotKept::NotThere { .. }));
        let message = said(&missing.said(&in_english()));
        assert!(message.contains("has done nothing"), "{message}");
    }

    /// Anything else keeps the machine's own words, because they are the only
    /// thing that says whether this is a full disk, a read-only mount or a
    /// permission somebody can change.
    #[test]
    fn anything_else_keeps_what_the_machine_said() {
        let denied = NotKept::opening(
            Path::new("/var/lib/alo/record.jsonl"),
            &Error::new(ErrorKind::PermissionDenied, "permission denied"),
        );
        let message = said(&denied.said(&in_english()));
        assert!(message.contains("permission denied"), "{message}");
        assert!(message.contains("nothing"), "{message}");
    }

    /// **Only one of these means the record has lost something**, and a daemon
    /// has to be able to tell which: a record that could not be read is intact,
    /// and a record something could not be added to is not.
    #[test]
    fn only_a_failure_to_add_means_the_record_is_no_longer_whole() {
        for failure in every_way_it_can_fail() {
            let whole = failure.record_is_still_whole();
            assert_eq!(
                whole,
                !matches!(failure, NotKept::NotAddedTo { .. }),
                "{failure:?}"
            );
        }
    }

    /// **Every way this crate can fail is a string it can say.** A variant
    /// added without a word for it would reach a person as its own key.
    #[test]
    fn every_failure_is_something_this_crate_can_say() {
        let strings = in_english();
        for failure in every_way_it_can_fail() {
            let said = failure.said(&strings);
            assert_eq!(
                said.came_from(),
                &CameFrom::TheSource,
                "{failure:?} has no words"
            );
            assert!(!said.is_a_bug(), "{failure:?}");
            assert!(
                said.unfilled().is_empty(),
                "{failure:?} left {:?} with nothing in it",
                said.unfilled()
            );
        }
    }

    /// **Every one of them names the record it is about**, because a person
    /// told only that something failed cannot tell whether it is their
    /// machine's record or a copy of somebody else's they were reading.
    #[test]
    fn every_failure_names_the_record_it_is_about() {
        let strings = in_english();
        for failure in every_way_it_can_fail() {
            let message = said(&failure.said(&strings));
            assert!(
                message.contains("/var/lib/alo/record.jsonl"),
                "{failure:?}: {message}"
            );
        }
    }

    /// The shape a record is in is a number, carried on the refusal and not
    /// written into the sentence — as `alo-egress` carries how long an address
    /// may be.
    #[test]
    fn the_shape_a_newer_record_is_in_is_carried_and_not_said() {
        let newer = NotKept::FromANewerAlo {
            path: "/var/lib/alo/record.jsonl".to_owned(),
            format: 4,
        };
        let message = said(&newer.said(&in_english()));
        assert!(!message.contains('4'), "{message}");
        assert!(message.contains("newer alo OS"), "{message}");
        assert!(matches!(newer, NotKept::FromANewerAlo { format: 4, .. }));
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug.
    #[test]
    fn a_failure_nobody_declared_the_words_for_says_so() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = NotKept::not_a_record(Path::new("/tmp/notes.txt")).said(&strings);
        assert!(said.is_a_bug());
        assert_eq!(said.text(), "«keeping.not-a-record»");
    }
}
