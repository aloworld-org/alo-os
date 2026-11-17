//! Why nothing was chosen, and what a person is told.
//!
//! Every one of these means **nothing was chosen**. The two outcomes that are
//! not here — an application that did not answer, and one that answered but
//! whose window does not show the choice made — may have chosen it, and are
//! said beside a choice recorded as having run (`crate::choosing::Chosen`).

use alo_strings::{Filling, Said, Strings};

use crate::choosing_words as words;
use crate::fallback_words::{APPLICATION, NAME};

/// Why nothing was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotChosen {
    /// No drop-down of that name is on the screen.
    NoSuchDropDown,
    /// No item of that name is on the screen.
    NoSuchItem,
    /// More than one is, and which was meant is not guessed.
    MoreThanOne,
    /// The window shows more than is read at once, so the one named being the
    /// only one is not known.
    TooMuchToBeSure,
    /// It is greyed out.
    CannotBeUsedNow,
    /// The drop-down lists no option of that name.
    NoSuchOption,
    /// The drop-down lists that option more than once.
    MoreThanOneOption,
    /// The application offers no way to choose it from outside.
    CannotBeChosen,
    /// The application answered that it did not choose it.
    DidNotChoose,
}

impl NotChosen {
    /// What a person is told about `name` — and `option`, for a drop-down —
    /// in `application`.
    #[must_use]
    pub fn said(
        self,
        application: &str,
        name: &str,
        option: Option<&str>,
        strings: &Strings,
    ) -> Said {
        let word = match self {
            Self::NoSuchDropDown => words::NO_SUCH_DROP_DOWN,
            Self::NoSuchItem => words::NO_SUCH_ITEM,
            Self::MoreThanOne => words::MORE_THAN_ONE,
            Self::TooMuchToBeSure => words::TOO_MUCH_TO_BE_SURE,
            Self::CannotBeUsedNow => words::CANNOT_BE_USED_NOW,
            Self::NoSuchOption => words::NO_SUCH_OPTION,
            Self::MoreThanOneOption => words::MORE_THAN_ONE_OPTION,
            Self::CannotBeChosen => words::CANNOT_BE_CHOSEN,
            Self::DidNotChoose => words::DID_NOT_CHOOSE,
        };
        strings.say(&word.key(), &filling(application, name, option))
    }
}

/// The gaps every sentence about a choice fills.
pub(crate) fn filling(application: &str, name: &str, option: Option<&str>) -> Filling {
    Filling::of(APPLICATION, application.to_owned())
        .and(NAME, name.to_owned())
        .and(words::OPTION, option.unwrap_or_default().to_owned())
}
