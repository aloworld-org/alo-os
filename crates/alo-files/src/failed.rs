//! Why the machine could not do something it was allowed to do.
//!
//! **This is not a refusal, and keeping the two apart is the point of the
//! file.** [`alo_capability::Refused`] means the capability model said no: no
//! grant covered it, nobody approved it, the grants changed under it. A
//! [`Failed`] means everything said yes and the disk could not — the file was
//! gone, the folder was a file, the machine denied it, there was already
//! something at that name.
//!
//! A record that called a full disk a refusal would tell a security review that
//! the grants stopped something they did not. So the two are different types,
//! they leave [`crate::Did`] by different doors, and only one of them is
//! evidence about the capability model.
//!
//! Every message says what to do next, because every one of them is read by
//! somebody holding a call that did not happen. "IO error 13" tells that person
//! nothing they can act on.
//!
//! # There is no way to turn one of these into English by accident
//!
//! A [`Failed`] has no `Display`. The only way to words is [`Failed::said`],
//! which takes the strings the person in front of the machine reads, and
//! answers with a `Said` that says whether anybody translated it. That is
//! deliberate and it is the point of item 9b: a `Display` here would be an
//! English sentence one `to_string()` away from a screen, in a shell whose
//! author had no reason to think about it, and *hardcoded English is a bug*
//! rather than a preference. What is lost is `std::error::Error` — these are
//! not errors a programmer handles, they are sentences a person reads.

use alo_strings::{Counting, Filling, Said, Strings};

use crate::words;

/// Why the machine could not do it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failed {
    /// A verb that is not one of the six.
    NotAFileVerb {
        /// The verb that was asked for.
        verb: String,
    },

    /// A verb was performed without an argument it declares.
    ///
    /// Unreachable for a call that came through [`alo_capability::Verbs`],
    /// which refuses a call missing an argument — and still a `Result` rather
    /// than an `unwrap`, because a library that panics on a verb somebody
    /// declared wrongly is a library that takes the daemon with it.
    Missing {
        /// The verb.
        verb: String,
        /// The argument it declared and did not get.
        argument: String,
    },

    /// Something is there, and it is not a folder.
    NotAFolder {
        /// What was named.
        path: String,
    },

    /// Something is there, and it is not a file.
    NotAFile {
        /// What was named.
        path: String,
    },

    /// It was there when it was checked, and it is not there now.
    Gone {
        /// What was named.
        path: String,
    },

    /// A file too large to answer with.
    ///
    /// The one thing this crate says that counts something, so it is the one
    /// answered with `alo_strings::Strings::count`: *one byte* and *4 000 000
    /// bytes* are two sentences in English, three in Polish and five in Irish.
    TooBig {
        /// What was named.
        path: String,
        /// How big it is.
        bytes: u64,
        /// The most a read answers with.
        most: u64,
    },

    /// A file that is not text.
    NotText {
        /// What was named.
        path: String,
    },

    /// Something is already at the name a change would create.
    AlreadyThere {
        /// Where the change would have put something.
        path: String,
    },

    /// A file asked to move into the folder it is already in.
    AlreadyIn {
        /// The file.
        path: String,
    },

    /// An archive asked to be written inside the folder it is an archive of.
    IntoItself {
        /// The folder being archived.
        folder: String,
    },

    /// An archive asked for under a name that does not say what it is.
    NotAZipName {
        /// The name that was asked for.
        name: String,
    },

    /// More in a folder than one archive holds.
    TooMany {
        /// The folder.
        folder: String,
        /// The most one archive holds.
        most: usize,
    },

    /// More bytes than one archive holds.
    TooMuch {
        /// The folder.
        folder: String,
        /// The most one archive holds.
        most: u64,
    },

    /// The machine said no, in its own words.
    TheMachineSaidNo {
        /// What was being touched.
        path: String,
        /// What was being done to it, in one word.
        doing: String,
        /// What the machine said.
        why: String,
    },
}

impl Failed {
    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put on the screen, and what there was to
    /// say about where it came from is on the [`Said`].
    ///
    /// A `Strings` that was never given [`crate::file_words`] answers with the
    /// key, marked, and `Said::is_a_bug` — which is the honest answer to *the
    /// shell forgot to declare what this crate can say*, and is not something
    /// this crate can paper over with a sentence of its own.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotAFileVerb { verb } => strings.say(
                &words::NOT_A_FILE_VERB.key(),
                &Filling::of("verb", verb.clone()),
            ),
            Self::Missing { verb, argument } => strings.say(
                &words::MISSING.key(),
                &Filling::of("verb", verb.clone()).and("argument", argument.clone()),
            ),
            Self::NotAFolder { path } => strings.say(
                &words::NOT_A_FOLDER.key(),
                &Filling::of("path", path.clone()),
            ),
            Self::NotAFile { path } => {
                strings.say(&words::NOT_A_FILE.key(), &Filling::of("path", path.clone()))
            }
            Self::Gone { path } => {
                strings.say(&words::GONE.key(), &Filling::of("path", path.clone()))
            }
            Self::TooBig { path, bytes, most } => strings.count(
                &words::TOO_BIG.key(),
                &Counting::of(*bytes),
                &Filling::of("path", path.clone()).and("most", most.to_string()),
            ),
            Self::NotText { path } => {
                strings.say(&words::NOT_TEXT.key(), &Filling::of("path", path.clone()))
            }
            Self::AlreadyThere { path } => strings.say(
                &words::ALREADY_THERE.key(),
                &Filling::of("path", path.clone()),
            ),
            Self::AlreadyIn { path } => {
                strings.say(&words::ALREADY_IN.key(), &Filling::of("path", path.clone()))
            }
            Self::IntoItself { folder } => strings.say(
                &words::INTO_ITSELF.key(),
                &Filling::of("folder", folder.clone()),
            ),
            // The name that was asked for is carried, and deliberately not
            // said: the sentence corrects it, and repeating it back reads like
            // an instruction to use it. `words.rs` says so beside the sentence,
            // where a translator meets it.
            Self::NotAZipName { name: _ } => {
                strings.say(&words::NOT_A_ZIP_NAME.key(), &Filling::nothing())
            }
            Self::TooMany { folder, most } => strings.say(
                &words::TOO_MANY.key(),
                &Filling::of("folder", folder.clone()).and("most", most.to_string()),
            ),
            Self::TooMuch { folder, most } => strings.say(
                &words::TOO_MUCH.key(),
                &Filling::of("folder", folder.clone()).and("most", most.to_string()),
            ),
            Self::TheMachineSaidNo { path, doing, why } => strings.say(
                &words::THE_MACHINE_SAID_NO.key(),
                &Filling::of("path", path.clone())
                    .and("doing", doing.clone())
                    .and("why", why.clone()),
            ),
        }
    }

    /// The machine said no while doing this to this path.
    ///
    /// One place turns an [`std::io::Error`] into words, so that *it went away*
    /// is always the same answer wherever it happened, rather than one answer
    /// per call site.
    pub(crate) fn machine(path: &std::path::Path, doing: &str, why: &std::io::Error) -> Self {
        if why.kind() == std::io::ErrorKind::NotFound {
            return Self::Gone {
                path: path.display().to_string(),
            };
        }
        Self::TheMachineSaidNo {
            path: path.display().to_string(),
            doing: doing.to_owned(),
            why: why.to_string(),
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
    use crate::testing::{every_failure, in_english, said};
    use alo_strings::{CameFrom, Language, Translation};
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    /// A file that went away is answered as having gone away, whatever it was
    /// being asked to do at the time. The alternative is three messages for one
    /// fact, differing only in which verb was unlucky.
    #[test]
    fn a_file_that_went_away_says_so_rather_than_reporting_an_error_number() {
        let gone = Failed::machine(
            Path::new("/home/anna/Invoices/march.pdf"),
            "read",
            &Error::new(ErrorKind::NotFound, "no such file or directory"),
        );
        assert!(
            matches!(gone, Failed::Gone { .. }),
            "a missing file is not a machine refusal"
        );
        assert!(said(&gone).contains("ask again"), "{}", said(&gone));
    }

    /// Anything else keeps the machine's own words, because they are the only
    /// thing that says whether this is a full disk, a read-only mount or a
    /// permission a person can change.
    #[test]
    fn anything_else_keeps_what_the_machine_said() {
        let denied = said(&Failed::machine(
            Path::new("/home/anna/Invoices"),
            "listed",
            &Error::new(ErrorKind::PermissionDenied, "permission denied"),
        ));
        assert!(denied.contains("permission denied"), "{denied}");
        assert!(denied.contains("listed"), "{denied}");
        assert!(denied.contains("nothing else was attempted"), "{denied}");
    }

    /// Every message says what to do about it. A refusal a person cannot act on
    /// is a refusal they will ask somebody else about.
    #[test]
    fn every_failure_says_what_to_do_next() {
        for failed in every_failure() {
            let message = said(&failed);
            assert!(
                message.contains(" — "),
                "{message}: a failure that does not say what to do is half a message"
            );
        }
    }

    /// **Every way this crate can fail is a string it can say.** A variant
    /// added without a word for it would reach a person as its own key, and
    /// this is what says so before anybody meets one — one example of every
    /// variant there is, through the lookup, asking where the answer came from.
    #[test]
    fn every_failure_is_something_this_crate_can_say() {
        let strings = in_english();
        for failed in every_failure() {
            let said = failed.said(&strings);
            assert_eq!(
                said.came_from(),
                &CameFrom::TheSource,
                "{failed:?} has no words"
            );
            assert!(!said.is_a_bug(), "{failed:?}");
            assert!(
                said.unfilled().is_empty(),
                "{failed:?} left {:?} with nothing in it",
                said.unfilled()
            );
        }
    }

    /// **The one that counts is counted the reader's way.** German has two
    /// forms as English does, and the sentence about one byte is not the
    /// sentence about four million — which as this crate wrote it before was
    /// "1 bytes".
    #[test]
    fn the_size_of_a_file_is_counted_and_not_stuck_into_a_sentence() {
        let mut strings = in_english();
        let german = strings
            .vocabulary()
            .check(
                Translation::into_language(Language::written("de").unwrap())
                    .says(
                        words::TOO_BIG.key().for_form(alo_strings::Form::One),
                        "{path} ist ein Byte groß, und ein Verb liest höchstens {most}",
                    )
                    .says(
                        words::TOO_BIG.key().for_form(alo_strings::Form::Other),
                        "{path} ist {bytes} Bytes groß, und ein Verb liest höchstens {most}",
                    ),
            )
            .unwrap();
        strings.speaks(german).unwrap();
        strings.prefers(&[Language::written("de").unwrap()]);

        let one = Failed::TooBig {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
            bytes: 1,
            most: 1_048_576,
        };
        let many = Failed::TooBig {
            path: "/home/anna/Invoices/scan.tiff".to_owned(),
            bytes: 200_000_000,
            most: 1_048_576,
        };
        assert_eq!(
            one.said(&strings).text(),
            "/home/anna/Invoices/march.pdf ist ein Byte groß, und ein Verb liest höchstens 1048576"
        );
        assert!(
            many.said(&strings).text().contains("ist 200000000 Bytes"),
            "{}",
            many.said(&strings).text()
        );
        assert!(one.said(&strings).is_translated());
    }

    /// A shell that never declared this crate's words shows the key and says it
    /// is a bug, rather than being handed an English sentence this crate kept
    /// for the purpose.
    #[test]
    fn a_failure_nobody_declared_the_words_for_says_so() {
        let strings = alo_strings::Strings::of(alo_strings::Vocabulary::empty());
        let said = Failed::NotAFolder {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
        }
        .said(&strings);
        assert!(said.is_a_bug());
        assert_eq!(said.text(), "«files.failed.not-a-folder»");
    }
}
