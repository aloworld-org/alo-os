//! Why nothing was opened, and what a person is told.
//!
//! Every one of these means **nothing was opened**. An application that did
//! not answer may have opened it, and is said beside an opening recorded as
//! having run (`crate::opening::Opened`).

use alo_strings::{Filling, Said, Strings};

use crate::fallback_words::{APPLICATION, NAME};
use crate::opening_words as words;

/// Why nothing was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotOpened {
    /// No item of that name is on the screen.
    NoSuchItem,
    /// More than one is, and which was meant is not guessed.
    MoreThanOne,
    /// The window shows more than is read at once.
    TooMuchToBeSure,
    /// It is greyed out.
    CannotBeUsedNow,
    /// It offers no way to be opened from outside the application.
    CannotBeOpened,
    /// The application answered that it did not open it.
    DidNotOpen,
}

impl NotOpened {
    /// What a person is told about `name` in `application`.
    #[must_use]
    pub fn said(self, application: &str, name: &str, strings: &Strings) -> Said {
        let word = match self {
            Self::NoSuchItem => words::NO_SUCH_ITEM,
            Self::MoreThanOne => words::MORE_THAN_ONE,
            Self::TooMuchToBeSure => words::TOO_MUCH_TO_BE_SURE,
            Self::CannotBeUsedNow => words::CANNOT_BE_USED_NOW,
            Self::CannotBeOpened => words::CANNOT_BE_OPENED,
            Self::DidNotOpen => words::DID_NOT_OPEN,
        };
        strings.say(&word.key(), &filling(application, name))
    }
}

/// The gaps every sentence about an opening fills.
pub(crate) fn filling(application: &str, name: &str) -> Filling {
    Filling::of(APPLICATION, application.to_owned()).and(NAME, name.to_owned())
}
