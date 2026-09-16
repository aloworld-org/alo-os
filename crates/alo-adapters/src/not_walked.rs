//! Why an application's windows could not be walked, and what a person is told.
//!
//! Every one of these means **nothing was read and nothing was pressed**: the
//! walk either did not start or stopped before anything it read was handed
//! back.

use alo_strings::{Filling, Said, Strings};

use crate::accessibility_tree::TreeFault;
use crate::fallback_words as words;

/// Why nothing was walked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotWalked {
    /// The application is not running, or has no window on the screen.
    NoWindow,
    /// The tree could not be asked.
    Tree(TreeFault),
}

impl NotWalked {
    /// What a person is told.
    pub(crate) fn said(self, application: &str, strings: &Strings) -> Said {
        let word = match self {
            // A tree that lost the application's root while it was listed is
            // an application that is no longer there.
            Self::NoWindow | Self::Tree(TreeFault::Vanished) => words::NO_WINDOW,
            Self::Tree(TreeFault::NotThere) => words::NOT_READABLE_HERE,
            Self::Tree(TreeFault::DidNotAnswer) => words::DID_NOT_ANSWER_READING,
        };
        strings.say(
            &word.key(),
            &Filling::of(words::APPLICATION, application.to_owned()),
        )
    }
}
