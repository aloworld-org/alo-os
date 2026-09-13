//! The seven ways nothing can be measured at all.
//!
//! Every one of these is a refusal of the whole question rather than a gap in
//! the answer. A number the kernel would not give for one process is a
//! [`crate::Number`] saying so in that process's row, and a folder that could
//! not be read is a [`crate::Counted`] on that folder's node; this is for when
//! there is no list to put a row in and no tree to put a node in. The last
//! two are met only by [`crate::Measured::of`], which is handed a permitted
//! call: one that was not this crate's, or one missing an argument it declares.

use std::path::PathBuf;

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Why what is running, or what is filling a folder, could not be measured.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotMeasured {
    /// This is not a Linux host, so there is no `/proc` to read.
    ///
    /// The crate compiles everywhere so that its types and its arithmetic
    /// are tested everywhere, and measures only where the kernel it reads is
    /// the kernel that is running. A reading of nothing would be a reading.
    #[error("what is running can only be read on alo OS itself")]
    NotOnThisHost,

    /// A file the kernel keeps for the whole machine could not be read.
    ///
    /// `/proc` itself, or `/proc/stat`, or `/proc/meminfo`: without those
    /// there is no list and no processor to give a share of. A file for one
    /// process failing to read is never this — that process is gone, or its
    /// number is withheld, and the list goes on.
    #[error("what is running could not be read from {at}: {why}")]
    Unreadable {
        /// The file.
        at: PathBuf,
        /// What the operating system said about it.
        why: String,
    },

    /// Two readings were given with no time between them.
    ///
    /// A rate is a difference divided by an interval, and an interval of
    /// nothing is a mistake in the code that asked rather than a rate of
    /// infinity.
    #[error("two readings need time between them")]
    NoInterval,

    /// Two readings that are the same moment.
    ///
    /// The kernel's own count of processor time, in `/proc/stat`, did not
    /// move between them: the same reading given twice, or two taken within
    /// one tick. Nothing can be said about a rate over no time.
    #[error("both readings are from the same moment")]
    SameMoment,

    /// The folder asked about could not be counted: it is not there, it is a
    /// file rather than a folder, or the machine would not read it.
    ///
    /// Only ever the folder the caller named. A folder found *inside* it
    /// that cannot be read is a node in the answer saying so, because the
    /// rest of the answer is still true.
    #[error("what is filling {} could not be counted", at.display())]
    NotCounted {
        /// The folder, as it was named.
        at: PathBuf,
        /// What the file half said about it, in its own words.
        why: alo_files::Failed,
    },

    /// A permitted call of a verb that is not this crate's to answer.
    ///
    /// Only ever met at [`crate::Measured::of`], which is handed a call that
    /// the capability model already permitted: the call was somebody else's
    /// to carry out, and nothing was measured.
    #[error("nothing here measures {verb}")]
    NotThisCrates {
        /// The verb as it was called.
        verb: String,
    },

    /// A verb performed without an argument it declares: a verb to write
    /// again rather than a call to make again.
    #[error("{verb} was performed without {argument}")]
    Missing {
        /// The verb as it was called.
        verb: String,
        /// The argument that did not arrive.
        argument: String,
    },
}

impl NotMeasured {
    /// The word this refusal is said with.
    #[must_use]
    pub fn word(&self) -> &'static Word {
        match self {
            Self::NotOnThisHost => &words::NOT_ON_THIS_HOST,
            Self::Unreadable { .. } => &words::UNREADABLE,
            Self::NoInterval => &words::NO_INTERVAL,
            Self::SameMoment => &words::SAME_MOMENT,
            Self::NotCounted { .. } => &words::NOT_COUNTED,
            Self::NotThisCrates { .. } => &words::NOT_THIS_CRATES,
            Self::Missing { .. } => &words::MISSING,
        }
    }

    /// This refusal, in the language the person reads.
    ///
    /// For [`Self::NotCounted`] the sentence carries the file half's own,
    /// which `alo-files` words; a vocabulary that holds only this crate's
    /// list shows that half as a key, marked as such, rather than as English
    /// nobody offered to translate.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Unreadable { at, .. } => Filling::of("at", at.display().to_string()),
            Self::NotCounted { at, why } => {
                Filling::of("at", at.display().to_string()).and_said("why", &why.said(strings))
            }
            Self::NotThisCrates { verb } => Filling::of("verb", verb.clone()),
            Self::Missing { verb, argument } => {
                Filling::of("verb", verb.clone()).and("argument", argument.clone())
            }
            Self::NotOnThisHost | Self::NoInterval | Self::SameMoment => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every refusal has a sentence, none of them is a key on somebody's
    /// screen, and the ones with a file or a folder in them name it.
    #[test]
    fn every_refusal_is_said_in_a_sentence_a_person_reads() {
        let mut vocabulary = crate::measuring_words().unwrap();
        alo_files::words::declare_into(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        let unreadable = NotMeasured::Unreadable {
            at: PathBuf::from("/proc/stat"),
            why: "no such file".to_owned(),
        };
        let not_counted = NotMeasured::NotCounted {
            at: PathBuf::from("Documents"),
            why: alo_files::Failed::Gone {
                path: "Documents".to_owned(),
            },
        };
        let not_this_crates = NotMeasured::NotThisCrates {
            verb: "list_folder".to_owned(),
        };
        let missing = NotMeasured::Missing {
            verb: "what_is_running".to_owned(),
            argument: "proc".to_owned(),
        };
        for refusal in [
            NotMeasured::NotOnThisHost,
            unreadable.clone(),
            NotMeasured::NoInterval,
            NotMeasured::SameMoment,
            not_counted.clone(),
            not_this_crates.clone(),
            missing.clone(),
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(!said.text().starts_with("measuring."), "{said}");
        }
        assert!(
            unreadable.said(&strings).text().contains("/proc/stat"),
            "the file is named"
        );
        let said = not_counted.said(&strings);
        assert!(said.text().contains("Documents"), "{said}");
        assert!(
            !said.text().contains("files."),
            "the file half's sentence is a sentence, not a key: {said}"
        );
        assert!(
            not_this_crates
                .said(&strings)
                .text()
                .contains("list_folder"),
            "the verb is named"
        );
        let said = missing.said(&strings).into_text();
        assert!(
            said.contains("what_is_running") && said.contains("proc"),
            "{said}"
        );
    }
}
