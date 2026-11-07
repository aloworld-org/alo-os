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
pub enum NotWalked {
    /// The application is not running, or has no window on the screen.
    NoWindow,
    /// A person named one window with `--in`, and no window on the screen has
    /// that title — or, on alo's own tree, no surface showing has that name.
    NoSuchWindow,
    /// A person named one window with `--in`, and more than one has that title.
    MoreThanOneWindow,
    /// The tree could not be asked.
    Tree(TreeFault),
}

impl NotWalked {
    /// What a person is told, about `application` — and about `within`, the
    /// window they named, where they named one.
    #[must_use]
    pub fn said(self, application: &str, within: Option<&str>, strings: &Strings) -> Said {
        let word = match self {
            Self::NoWindow | Self::Tree(TreeFault::Vanished) => words::NO_WINDOW,
            Self::NoSuchWindow => words::NO_SUCH_WINDOW,
            Self::MoreThanOneWindow => words::MORE_THAN_ONE_WINDOW,
            Self::Tree(TreeFault::NotThere) => words::NOT_READABLE_HERE,
            Self::Tree(TreeFault::DidNotAnswer) => words::DID_NOT_ANSWER_READING,
        };
        let filling = Filling::of(words::APPLICATION, application.to_owned());
        let filling = match self {
            Self::NoSuchWindow | Self::MoreThanOneWindow => {
                filling.and(words::WINDOW, within.unwrap_or_default().to_owned())
            }
            Self::NoWindow | Self::Tree(_) => filling,
        };
        strings.say(&word.key(), &filling)
    }
}
