//! Every way applying an update, going back, reading what runs, or writing down that the
//! machine updated can fail — each with the sentence a person reads, and what
//! the machine said kept beside it for whoever administers it.
//!
//! **Nothing here has a `Display` a person could be shown.** The sentences are
//! `alo-keeping-up`'s and `alo-keeping`'s, in the vocabulary `alo-saying`
//! collects; the program's own words and paths are fields, for a journal, and
//! never a sentence on a screen.

use alo_keeping::NotKept;
use alo_keeping_up::{CannotGoBack, NotStaged, words};
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
    /// The base was told and refused, or failed: the download did not
    /// complete, or something else went wrong in preparing it.
    ///
    /// **Not the signature policy**, which is
    /// [`Self::TheBuildWasNotGenuine`]. The two were one member until
    /// 2026-09-19, and one sentence — and a person who reads *it could not be
    /// prepared* about a build their machine refused to trust has been told
    /// the least interesting half of what happened.
    TheBaseDidNotStageIt(NotAnswered),
    /// The base refused the build because its signature policy would not have
    /// it: this machine could not confirm the build is alo OS.
    ///
    /// Told apart from the member above by what the base said
    /// ([`crate::genuine::refused_for_its_signature`]), because the base
    /// gives no other sign. Nothing about that reading decides whether a build
    /// is staged — the base has already refused by the time this is made, and
    /// `--enforce-container-sigpolicy` is what made it refuse.
    TheBuildWasNotGenuine(NotAnswered),
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
    /// The build the person chose to go back to could not be read, so whether
    /// this start is a return cannot be told.
    ///
    /// Refused rather than read as *no return chosen*, which would write a
    /// return into the record as an update.
    GoingBackNotRead {
        /// Where it is kept.
        path: String,
        /// What was wrong with it.
        why: String,
    },
    /// The note of a return that is done could not be cleared. The entry and
    /// the last known build were already kept.
    GoingBackNotCleared {
        /// Where it is kept.
        path: String,
        /// What the machine said.
        why: String,
    },
    /// The record would not take the entry.
    NotKept(NotKept),
}

/// Going back was not set, and the next restart starts the build running now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotGoneBack {
    /// What is running could not be read, so nothing was decided.
    NotRead(NotRead),
    /// Refused before anything ran: the machine is not as the offer described,
    /// or going back is already set.
    Refused(CannotGoBack),
    /// The build to go back to could not be noted, so the base was told
    /// nothing — without the note the return would be recorded as an update.
    NotNoted {
        /// Where it is kept.
        path: String,
        /// What the machine said.
        why: String,
    },
    /// The base was told and refused, or failed.
    TheBaseDidNotSetIt(NotAnswered),
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
            Self::TheBuildWasNotGenuine(_) => {
                strings.say(&words::NOT_GENUINE.key(), &Filling::nothing())
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
            Self::LastKnownNotRead { .. }
            | Self::LastKnownNotKept { .. }
            | Self::GoingBackNotRead { .. }
            | Self::GoingBackNotCleared { .. } => {
                strings.say(&words::NOT_WRITTEN_DOWN.key(), &Filling::nothing())
            }
        }
    }
}

impl NotGoneBack {
    /// What a person reads, in the vocabulary.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotRead(not_read) => not_read.said(strings),
            Self::Refused(refused) => refused.said(strings),
            Self::NotNoted { .. } | Self::TheBaseDidNotSetIt(_) => {
                strings.say(&words::GOING_BACK_NOT_PREPARED.key(), &Filling::nothing())
            }
        }
    }
}
