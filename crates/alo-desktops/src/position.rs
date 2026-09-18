//! Where a desktop sits in the person's order, counted the way a person counts.
//!
//! **One-based, and not an index.** A person's first desktop is desktop 1: it is
//! what the shortcut `Super+1` reaches, what a settings list numbers, and what
//! an unnamed desktop is called ([`crate::words::DESKTOP_NUMBERED`]). A type
//! that was an index would put the off-by-one in every one of those places
//! instead of in the one method below.

use std::num::NonZeroU16;

/// Which desktop, counted from one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(NonZeroU16);

impl Position {
    /// The first desktop, which every display always has.
    #[must_use]
    pub const fn first() -> Self {
        Self(NonZeroU16::MIN)
    }

    /// The desktop a person would call this number.
    ///
    /// `None` for zero, which is nobody's first desktop.
    #[must_use]
    pub const fn numbered(number: u16) -> Option<Self> {
        match NonZeroU16::new(number) {
            Some(number) => Some(Self(number)),
            None => None,
        }
    }

    /// The number a person reads.
    #[must_use]
    pub const fn number(self) -> u16 {
        self.0.get()
    }

    /// The place in a list of desktops, counted from zero.
    #[must_use]
    pub const fn index(self) -> usize {
        (self.0.get() as usize) - 1
    }

    /// The position of the desktop at `index` in a list.
    ///
    /// `None` when the list is longer than a person could ever count, which is
    /// beyond [`crate::MOST_DESKTOPS`] and so cannot happen to a list this
    /// crate made.
    pub(crate) fn at(index: usize) -> Option<Self> {
        u16::try_from(index)
            .ok()
            .and_then(|index| index.checked_add(1))
            .and_then(Self::numbered)
    }

    /// The position one along, in the direction a switch asked for.
    pub(crate) fn along(self, forwards: bool) -> Option<Self> {
        if forwards {
            self.0.checked_add(1).map(Self)
        } else {
            Self::numbered(self.0.get() - 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A person counts from one and a list counts from zero, and this is the
    /// only place the two meet.
    #[test]
    fn a_person_counts_from_one_and_the_list_from_zero() {
        assert_eq!(Position::first().number(), 1);
        assert_eq!(Position::first().index(), 0);
        assert_eq!(Position::numbered(0), None);
        assert_eq!(Position::numbered(4).map(Position::index), Some(3));
        assert_eq!(Position::at(0), Some(Position::first()));
        assert_eq!(Position::at(7).map(Position::number), Some(8));
    }

    /// One along in either direction, and nothing before the first.
    #[test]
    fn nothing_lies_before_the_first_desktop() {
        let first = Position::first();
        assert_eq!(first.along(false), None);
        assert_eq!(first.along(true).map(Position::number), Some(2));
        assert_eq!(
            Position::numbered(2).and_then(|at| at.along(false)),
            Some(first)
        );
        // And nothing beyond what the counter can hold, which is far beyond
        // `MOST_DESKTOPS` and so is unreachable through this crate.
        assert_eq!(
            Position::numbered(u16::MAX).and_then(|at| at.along(true)),
            None
        );
    }
}
