//! Why nothing changed, and what the person is told.
//!
//! **Every refusal leaves the desktops exactly as they were.** Nothing here
//! removes a desktop and then discovers it should not have, or takes a window
//! off one before finding it had nowhere to go: each change is checked before
//! any of it is applied, and the tests beside each operation are what say so.
//!
//! Like `alo-dividing`'s and `alo-shortcuts`' refusals, a [`Refused`] has no
//! `Display`: the only road to words is [`Refused::said`], in the language the
//! person reads. And like `alo-dividing`'s, **no sentence here names a desktop,
//! a window or a shortcut**. A refusal carries the thing it is about so the
//! shell can mark it — [`Refused::desktop`], [`Refused::window`] — and a
//! sentence that carried a name would be one this crate had to be handed a name
//! for, which is the title task 2 of this plan keeps out of an arrangement.
//!
//! The refusals a *person* never meets are elsewhere and are English:
//! [`crate::NotADisplay`] and [`crate::TwoPromises`] both mean the shell and
//! this crate disagree about what exists, which is alo OS's own bug.

use alo_dividing::WindowId;
use alo_shortcuts::Action;
use alo_strings::{Filling, Said, Strings};

use crate::always::Always;
use crate::desktop::DesktopId;
use crate::position::Position;
use crate::switching::Switch;
use crate::words::{self, Word};

/// Why a person's desktops were left as they were.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// No desktop of this display has that identity.
    NoSuchDesktop(DesktopId),
    /// This display has no desktop at that number.
    NoSuchPosition(Position),
    /// The only desktop a display has cannot be removed, because a display
    /// always has one.
    TheLastDesktop,
    /// A display already holds [`crate::MOST_DESKTOPS`].
    TooManyDesktops,
    /// There is no desktop the way the switch asked.
    NoDesktopThatWay(Switch),
    /// Another desktop of this display is already called that, and which one.
    NameIsTaken(DesktopId),
    /// This window is one of the three that are on every desktop at once, and
    /// cannot be put on one, pinned, or closed from here.
    APromise(Always),
    /// This window is on no desktop of this display.
    NoSuchWindow(WindowId),
    /// One of the person's system shortcuts already has that chord.
    ChordIsTaken(Action),
    /// Another desktop switch already has that chord.
    ChordIsASwitch(Switch),
}

impl Refused {
    /// The desktop this is about, when it is about one — so the shell can mark
    /// it beside the sentence, which never names one.
    #[must_use]
    pub const fn desktop(self) -> Option<DesktopId> {
        match self {
            Self::NoSuchDesktop(desktop) | Self::NameIsTaken(desktop) => Some(desktop),
            Self::NoSuchPosition(_)
            | Self::TheLastDesktop
            | Self::TooManyDesktops
            | Self::NoDesktopThatWay(_)
            | Self::APromise(_)
            | Self::NoSuchWindow(_)
            | Self::ChordIsTaken(_)
            | Self::ChordIsASwitch(_) => None,
        }
    }

    /// The window this is about, when it is about one.
    ///
    /// `None` for [`Refused::APromise`], whose window is a surface of alo OS's
    /// own rather than one of the person's — what to say about it is the
    /// promise's name, [`Always::said`].
    #[must_use]
    pub const fn window(self) -> Option<WindowId> {
        match self {
            Self::NoSuchWindow(window) => Some(window),
            Self::NoSuchDesktop(_)
            | Self::NoSuchPosition(_)
            | Self::TheLastDesktop
            | Self::TooManyDesktops
            | Self::NoDesktopThatWay(_)
            | Self::NameIsTaken(_)
            | Self::APromise(_)
            | Self::ChordIsTaken(_)
            | Self::ChordIsASwitch(_) => None,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NoSuchDesktop(_) => words::NO_SUCH_DESKTOP,
            Self::NoSuchPosition(_) => words::NO_SUCH_POSITION,
            Self::TheLastDesktop => words::THE_LAST_DESKTOP,
            Self::TooManyDesktops => words::TOO_MANY_DESKTOPS,
            Self::NoDesktopThatWay(_) => words::NO_DESKTOP_THAT_WAY,
            Self::NameIsTaken(_) => words::NAME_IS_TAKEN,
            Self::APromise(_) => words::A_PROMISE,
            Self::NoSuchWindow(_) => words::NO_SUCH_WINDOW,
            Self::ChordIsTaken(_) => words::CHORD_IS_TAKEN,
            Self::ChordIsASwitch(_) => words::CHORD_IS_A_SWITCH,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::TooManyDesktops => Filling::of("most", crate::MOST_DESKTOPS.to_string()),
            Self::NoSuchDesktop(_)
            | Self::NoSuchPosition(_)
            | Self::TheLastDesktop
            | Self::NoDesktopThatWay(_)
            | Self::NameIsTaken(_)
            | Self::APromise(_)
            | Self::NoSuchWindow(_)
            | Self::ChordIsTaken(_)
            | Self::ChordIsASwitch(_) => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{every_refusal, in_english};
    use std::collections::BTreeSet;

    /// Every refusal says something of its own, declared, with nothing left to
    /// fill in — a person told the wrong reason would try the same thing again.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for refused in every_refusal() {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "two are both {said}");
        }
    }

    /// A refusal carries the thing it is about, and **no sentence names it**:
    /// two refusals about two different desktops read identically, which is
    /// what makes the shell's marking the only place a name could appear.
    #[test]
    fn a_refusal_carries_what_it_is_about_and_never_names_it() {
        let strings = in_english();
        let mut desktops = 0_usize;
        let mut windows = 0_usize;
        for refused in every_refusal() {
            desktops += usize::from(refused.desktop().is_some());
            windows += usize::from(refused.window().is_some());
        }
        assert_eq!(desktops, 2, "no such desktop, and the name already taken");
        assert_eq!(windows, 1, "a window on no desktop here");

        let (first, second) = crate::testing::two_desktops();
        assert_eq!(
            Refused::NoSuchDesktop(first).said(&strings),
            Refused::NoSuchDesktop(second).said(&strings)
        );
        assert_eq!(
            Refused::NameIsTaken(first).said(&strings),
            Refused::NameIsTaken(second).said(&strings)
        );
        assert_eq!(
            Refused::NoSuchWindow(WindowId::from_compositor(1)).said(&strings),
            Refused::NoSuchWindow(WindowId::from_compositor(2)).said(&strings)
        );
        assert_eq!(
            Refused::ChordIsTaken(Action::Launcher).said(&strings),
            Refused::ChordIsTaken(Action::TheAgent).said(&strings)
        );
    }

    /// The one sentence with something to fill in says the cap it is about.
    #[test]
    fn the_cap_is_in_the_sentence_about_it() {
        let said = Refused::TooManyDesktops.said(&in_english());
        assert!(
            said.text().contains(&crate::MOST_DESKTOPS.to_string()),
            "{said}"
        );
    }
}
