//! Moving to another desktop: the closed set of ways a person can ask.
//!
//! Three, and there will not be a fourth without a reason: the next one, the
//! previous one, and one by number. A gesture asks for the first two (task 5 of
//! this crate's plan, which decides a three- or four-finger swipe into one of
//! these) and a chord can ask for any of them ([`crate::DesktopChords`]).
//!
//! **Switching does not wrap.** Swiping past the last desktop does not land on
//! the first: it is refused with [`crate::Refused::NoDesktopThatWay`], and the
//! shell shows the edge of the row the way every other end of a list is shown.
//! Wrapping reads as a bug the first time it happens to somebody with two
//! desktops, and a swipe is the one gesture a person makes without looking.
//!
//! **A switch is a request, never an effect.** [`crate::OnADisplay::switch`] is
//! the only thing that carries one out, and the same value is what a gesture and
//! a chord both hand it — so there is one road to another desktop and not one
//! per input device.

use alo_strings::{Filling, Said, Strings};

use crate::position::Position;
use crate::words::{self, Word};

/// A way a person asks for another desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Switch {
    /// The desktop after this one.
    Next,
    /// The desktop before this one.
    Previous,
    /// The desktop a person counts as this one.
    To(Position),
}

impl Switch {
    /// The string this crate declares for it.
    ///
    /// [`Switch::To`] is read as the desktop it reaches — *Desktop 4* — which is
    /// the same string an unnamed desktop is read as, because it is the same
    /// sentence about the same thing.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Next => words::NEXT_DESKTOP,
            Self::Previous => words::PREVIOUS_DESKTOP,
            Self::To(_) => words::DESKTOP_NUMBERED,
        }
    }

    /// What this asks for, in the language the person reads — the row a
    /// shortcuts panel draws beside the chord.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::To(at) => Filling::of("number", at.number().to_string()),
            Self::Next | Self::Previous => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// Each way of asking says something of its own, with nothing left to fill
    /// in, and the numbered one names the desktop it reaches.
    #[test]
    fn each_way_of_asking_says_what_it_reaches() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for switch in [
            Switch::Next,
            Switch::Previous,
            Switch::To(Position::numbered(4).unwrap()),
        ] {
            let said = switch.said(&strings);
            assert!(!said.is_a_bug(), "{switch:?}");
            assert!(said.unfilled().is_empty(), "{switch:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "two are both {said}");
        }
        assert_eq!(
            Switch::To(Position::numbered(4).unwrap())
                .said(&strings)
                .text(),
            "Desktop 4"
        );
    }

    /// Two switches asking for the same desktop by number are the same switch,
    /// so a chord bound to one is bound to the other.
    #[test]
    fn the_same_desktop_by_number_is_the_same_switch() {
        let fourth = Position::numbered(4).unwrap();
        assert_eq!(
            Switch::To(fourth),
            Switch::To(Position::numbered(4).unwrap())
        );
        assert_ne!(Switch::To(fourth), Switch::To(Position::first()));
        assert_ne!(Switch::Next, Switch::Previous);
    }
}
