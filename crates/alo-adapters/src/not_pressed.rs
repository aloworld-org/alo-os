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
pub enum NotPressed {
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

/// **Whether a switch the person can turn on would let this control be
/// pressed** — what the sentence for [`NotPressed::CannotBePressed`] says.
///
/// An application drawn by Chromium, started while *Let alo work fully inside
/// apps like VSCodium* is off, describes its page in a mode whose controls
/// offer no press that works (`alo_launching::describing_themselves`). Telling
/// the person only *it cannot be pressed* would hide a choice they have, so the
/// sentence names the switch. The caller decides which this is, because it is
/// the caller that knows the application and reads the person's settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingFully {
    /// Nothing the person can turn on would offer more: the switch is on, or
    /// the application is not one it reaches.
    NothingMoreToTurnOn,
    /// The switch is off, and the application is one it reaches.
    CouldBeTurnedOn,
}

impl WorkingFully {
    /// Which this is, for an application that is or is not one the switch
    /// reaches, where the person has or has not turned it on.
    #[must_use]
    pub const fn for_one(reached_by_the_switch: bool, turned_on: bool) -> Self {
        if reached_by_the_switch && !turned_on {
            Self::CouldBeTurnedOn
        } else {
            Self::NothingMoreToTurnOn
        }
    }
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
    /// What a person is told — naming *Let alo work fully inside apps like
    /// VSCodium* where the control offers no press and that switch would give
    /// it one.
    pub(crate) fn said(
        self,
        control: TheControl<'_>,
        fully: WorkingFully,
        strings: &Strings,
    ) -> Said {
        let word = match (self, fully) {
            (Self::NoSuchControl, _) => words::NO_SUCH_CONTROL,
            (Self::MoreThanOne, _) => words::MORE_THAN_ONE,
            (Self::TooMuchToBeSure, _) => words::TOO_MUCH_TO_BE_SURE,
            (Self::CannotBeUsedNow, _) => words::CANNOT_BE_USED_NOW,
            (Self::CannotBePressed, WorkingFully::NothingMoreToTurnOn) => words::CANNOT_BE_PRESSED,
            (Self::CannotBePressed, WorkingFully::CouldBeTurnedOn) => {
                words::CANNOT_BE_PRESSED_UNTIL_ALO_WORKS_FULLY
            }
            (Self::DidNotPress, _) => words::DID_NOT_PRESS,
        };
        strings.say(&word.key(), &control.filling(strings))
    }

    /// What a person is told about the `kind` called `name` in `application`
    /// — the same sentence the agent's road says, for a person's own command.
    #[must_use]
    pub fn said_of(
        self,
        application: &str,
        kind: Pressable,
        name: &str,
        fully: WorkingFully,
        strings: &Strings,
    ) -> Said {
        self.said(
            TheControl {
                application,
                kind,
                name,
            },
            fully,
            strings,
        )
    }
}
