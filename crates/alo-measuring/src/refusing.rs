//! The four ways nothing can be measured at all.
//!
//! Every one of these is a refusal of the whole question rather than a gap in
//! the answer. A number the kernel would not give for one process is a
//! [`crate::Number`] saying so in that process's row; this is for when there is
//! no list to put a row in.

use std::path::PathBuf;

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Why what is running could not be measured.
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
        }
    }

    /// This refusal, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Unreadable { at, .. } => Filling::of("at", at.display().to_string()),
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
    /// screen, and the one with a file in it names the file.
    #[test]
    fn every_refusal_is_said_in_a_sentence_a_person_reads() {
        let strings = Strings::of(crate::measuring_words().unwrap());
        let unreadable = NotMeasured::Unreadable {
            at: PathBuf::from("/proc/stat"),
            why: "no such file".to_owned(),
        };
        for refusal in [
            NotMeasured::NotOnThisHost,
            unreadable.clone(),
            NotMeasured::NoInterval,
            NotMeasured::SameMoment,
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
    }
}
