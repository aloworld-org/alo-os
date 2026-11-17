//! Why nothing was moved, and what a person is told.
//!
//! Every one of these means **nothing was moved**. The two outcomes not here —
//! an application that did not answer, and one whose control reads back as
//! another number — may have moved it, and are said beside a setting recorded
//! as having run (`crate::setting::Set`).

use alo_strings::{Filling, Said, Strings};

use crate::accessibility_tree::Bounds;
use crate::fallback_words::{APPLICATION, NAME};
use crate::setting_words as words;

/// Why nothing was moved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotSet {
    /// No slider or number box of that name is on the screen.
    NoSuchControl,
    /// More than one is, and which was meant is not guessed.
    MoreThanOne,
    /// The window shows more than is read at once.
    TooMuchToBeSure,
    /// It is greyed out.
    CannotBeUsedNow,
    /// It is a progress bar or a meter, which only shows a number.
    OnlyShowsANumber,
    /// It says no range, so it cannot be moved from outside.
    CannotBeMoved,
    /// The number is outside the range it says.
    OutsideItsRange(Bounds),
    /// The application refused the number.
    DidNotTake,
}

impl NotSet {
    /// What a person is told about moving `name` in `application` to `value`.
    #[must_use]
    pub fn said(self, application: &str, name: &str, value: i64, strings: &Strings) -> Said {
        let word = match self {
            Self::NoSuchControl => words::NO_SUCH_CONTROL,
            Self::MoreThanOne => words::MORE_THAN_ONE,
            Self::TooMuchToBeSure => words::TOO_MUCH_TO_BE_SURE,
            Self::CannotBeUsedNow => words::CANNOT_BE_USED_NOW,
            Self::OnlyShowsANumber => words::ONLY_SHOWS_A_NUMBER,
            Self::CannotBeMoved => words::CANNOT_BE_MOVED,
            Self::OutsideItsRange(_) => words::OUTSIDE_ITS_RANGE,
            Self::DidNotTake => words::DID_NOT_TAKE,
        };
        let filling = filling(application, name, value);
        let filling = match self {
            Self::OutsideItsRange(bounds) => filling
                .and(words::LEAST, written(bounds.least))
                .and(words::MOST, written(bounds.most)),
            _ => filling,
        };
        strings.say(&word.key(), &filling)
    }
}

/// The gaps every sentence about a setting fills.
pub(crate) fn filling(application: &str, name: &str, value: i64) -> Filling {
    Filling::of(APPLICATION, application.to_owned())
        .and(NAME, name.to_owned())
        .and(words::VALUE, value.to_string())
}

/// A number as a person reads it: the shortest decimal that is the number.
fn written(number: f64) -> String {
    number.to_string()
}
