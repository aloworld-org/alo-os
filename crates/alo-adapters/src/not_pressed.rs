//! Why an approved control was not pressed, and what a person is told.
//!
//! Every one of these means **nothing was pressed** — the one outcome that is
//! not here is an application that did not answer, which may have pressed it
//! and is said as not known (`crate::activating::Pressed`).

use alo_strings::{Filling, Said, Strings};

use crate::fallback_words as words;
use crate::pressable::Pressable;

/// Why nothing was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NotPressed {
    /// No control of that kind and name is on the screen.
    NoSuchControl,
    /// More than one is, and which was meant is not guessed.
    MoreThanOne,
    /// The application shows more than is read at once, so the control being
    /// the only one of its name is not known.
    TooMuchToBeSure,
    /// It is greyed out.
    CannotBeUsedNow,
    /// It offers no way to be pressed from outside the application.
    CannotBePressed,
    /// The application answered that it did not press it.
    DidNotPress,
}

/// The control a person approved pressing, for the words about it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheControl<'a> {
    /// Its application.
    pub(crate) application: &'a str,
    /// Its kind.
    pub(crate) kind: Pressable,
    /// Its name.
    pub(crate) name: &'a str,
}

impl TheControl<'_> {
    /// The gaps every sentence about it fills.
    pub(crate) fn filling(self, strings: &Strings) -> Filling {
        Filling::of(words::APPLICATION, self.application.to_owned())
            .and_said(
                words::KIND,
                &strings.say(&self.kind.words().key(), &Filling::nothing()),
            )
            .and(words::NAME, self.name.to_owned())
    }
}

impl NotPressed {
    /// What a person is told.
    pub(crate) fn said(self, control: TheControl<'_>, strings: &Strings) -> Said {
        let word = match self {
            Self::NoSuchControl => words::NO_SUCH_CONTROL,
            Self::MoreThanOne => words::MORE_THAN_ONE,
            Self::TooMuchToBeSure => words::TOO_MUCH_TO_BE_SURE,
            Self::CannotBeUsedNow => words::CANNOT_BE_USED_NOW,
            Self::CannotBePressed => words::CANNOT_BE_PRESSED,
            Self::DidNotPress => words::DID_NOT_PRESS,
        };
        strings.say(&word.key(), &control.filling(strings))
    }
}
