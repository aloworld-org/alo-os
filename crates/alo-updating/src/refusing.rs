//! Every way applying an update, reading what runs, or writing down that the
//! machine updated can fail — each with the sentence a person reads, and what
//! the machine said kept beside it for whoever administers it.
//!
//! **Nothing here has a `Display` a person could be shown.** The sentences are
//! `alo-keeping-up`'s and `alo-keeping`'s, in the vocabulary `alo-saying`
//! collects; the program's own words and paths are fields, for a journal, and
//! never a sentence on a screen.

use alo_keeping::NotKept;
use alo_keeping_up::{NotStaged, words};
use alo_strings::{Filling, Said, Strings};

/// The base's program did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnswered {
    /// It could not be run at all.
    NotStarted {
        /// Which program.
        program: String,
        /// What the machine said.
        why: String,
    },
    /// It ran and exited unsuccessfully.
    SaidNo {
        /// Which program.
        program: String,
        /// Its exit code, when it exited rather than being stopped by a signal.
        code: Option<i32>,
        /// The end of what it said.
        said: String,
    },
}

/// Which build is running could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRead {
    /// The base did not answer.
    NotAnswered(NotAnswered),
    /// It answered with something that is not its status, or that names half
    /// a build.
    NotUnderstood {
        /// Why, as the parser said it.
        why: String,
    },
    /// Its status names no build the machine booted from an image.
    NotRunningABuild,
}

/// An update was not applied, and nothing on the machine changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotApplied {
    /// What is running could not be read, so nothing was decided.
    NotRead(NotRead),
    /// Refused before anything ran.
    Refused(NotStaged),
    /// The base was told and refused, or failed — the download did not
    /// complete, or the build was not one the signature policy accepts.
    TheBaseDidNotStageIt(NotAnswered),
}

/// Whether the machine updated could not be written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRecorded {
    /// What is running could not be read.
    NotRead(NotRead),
    /// The build last known could not be read, so there is no *from*.
    ///
    /// Refused rather than read as *nothing known*: that would write no entry
    /// for an update that happened, and a record with a hole where a change
    /// used to be is worse than a machine that says it could not write it.
    LastKnownNotRead {
        /// Where it is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// The build running now could not be kept as the last known.
    LastKnownNotKept {
        /// Where it is kept.
        path: String,
        /// What the machine said.
        why: String,
    },
    /// The record would not take the entry.
    NotKept(NotKept),
}

impl NotRead {
    /// What a person reads: that which version runs could not be read, and so
    /// nothing was changed.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::RUNNING_NOT_KNOWN.key(), &Filling::nothing())
    }
}

impl NotApplied {
    /// What a person reads, in the vocabulary.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotRead(not_read) => not_read.said(strings),
            Self::Refused(refused) => refused.said(strings),
            Self::TheBaseDidNotStageIt(_) => {
                strings.say(&words::NOT_PREPARED.key(), &Filling::nothing())
            }
        }
    }
}

impl NotRecorded {
    /// What a person reads, in the vocabulary.
    ///
    /// A record that would not take the entry says so in `alo-keeping`'s own
    /// words, because what is wrong is the record rather than the update.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotRead(not_read) => not_read.said(strings),
            Self::NotKept(not_kept) => not_kept.said(strings),
            Self::LastKnownNotRead { .. } | Self::LastKnownNotKept { .. } => {
                strings.say(&words::NOT_WRITTEN_DOWN.key(), &Filling::nothing())
            }
        }
    }
}
