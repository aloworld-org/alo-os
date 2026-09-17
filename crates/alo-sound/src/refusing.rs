//! Why a machine could not be asked, or could not be told.
//!
//! Two failures, kept apart on purpose. **Asking** fails when the record cannot
//! be had: there is no media server, it would not answer, or it answered
//! something this crate does not recognise. **Telling** fails when the machine
//! was asked to change something and did not.
//!
//! Neither is a sentence a person reads. They are for whoever is fixing the
//! machine, so they name the tool and what it said — and that is exactly why
//! they are not in [`crate::words`]: a sentence naming the rented server is a
//! sentence this plan may not show anybody.

/// Why this machine's sound could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotHeard {
    /// There is no media server on this machine to ask.
    #[error("nothing on this machine handles sound: {said}")]
    NothingHandlesSound {
        /// What trying to ask said.
        said: String,
    },
    /// It is there and it did not answer.
    #[error("this machine's media server was asked about sound and did not answer: {said}")]
    NoAnswer {
        /// What it said instead.
        said: String,
    },
    /// It answered something this crate cannot read.
    #[error("this machine's media server answered something this crate cannot read: {said}")]
    NotUnderstood {
        /// What was wrong with the answer.
        said: String,
    },
}

impl NotHeard {
    /// What to put in front of whoever is fixing the machine.
    #[must_use]
    pub fn diagnosis(&self) -> String {
        self.to_string()
    }
}

/// Why the machine was not told.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotDone {
    /// The device is not here.
    #[error("{identity} is not a device this machine has now")]
    NotHere {
        /// The device that was asked for.
        identity: String,
    },
    /// The record could not be read first, so nothing was attempted.
    #[error(transparent)]
    NotHeard(#[from] NotHeard),
    /// There is no tool on this machine to tell it with.
    #[error("nothing on this machine changes sound settings: {said}")]
    NothingToTellItWith {
        /// What trying to run it said.
        said: String,
    },
    /// It was told and it refused.
    #[error("this machine's media server refused to {doing}: {said}")]
    Refused {
        /// What it was asked to do.
        doing: String,
        /// What it said.
        said: String,
    },
}
