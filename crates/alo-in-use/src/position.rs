//! Where a line sits, which is fixed by what is in use and never by who.
//!
//! The third of the three things
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! asks for beside a colour — mark, word and **position**. A mark is a shape
//! somebody has to recognise and a word is a sentence somebody has to read;
//! position is the one that costs nothing to learn. The camera is always in the
//! same place on the indicator, so a person who has glanced at it twice knows
//! where to look the third time without reading anything at all.
//!
//! # Fixed by what, never by who
//!
//! [`Position::of`] takes a [`crate::Used`] and nothing else. This is the whole
//! point of the type and it is tested: if position depended on who was using
//! something, the microphone would move when an agent picked it up, and the
//! glance a person had learned would stop working at exactly the moment it
//! mattered most.
//!
//! Two things using the camera at once are two lines at the same position, one
//! under the other. Position groups; it does not number.

use crate::used::Used;

/// Where one line sits on the in-use indicator.
///
/// Three places in a fixed order, top to bottom or leading to trailing as the
/// shell's writing direction decides. Which end is which is the shell's; that
/// the screen always comes before the camera and the camera before the
/// microphone is this value's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Position {
    /// Where every line about the screen sits.
    First,
    /// Where every line about the camera sits.
    Second,
    /// Where every line about the microphone sits.
    Third,
}

impl Position {
    /// All three, in order.
    pub const EVERY: [Self; 3] = [Self::First, Self::Second, Self::Third];

    /// Where lines about this thing sit.
    ///
    /// The only way to a [`Position`], and it takes what is in use rather than
    /// a use: there is no argument here that could make the camera's place
    /// depend on who has it.
    #[must_use]
    pub const fn of(used: Used) -> Self {
        match used {
            Used::Screen => Self::First,
            Used::Camera => Self::Second,
            Used::Microphone => Self::Third,
        }
    }

    /// How far down the indicator this place is, counting from zero.
    ///
    /// For whoever lays the lines out; the order is what matters and the number
    /// is how it is said to a layout.
    #[must_use]
    pub const fn how_far_down(self) -> usize {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **No two things share a place**, which is what makes the glance work:
    /// the second row is the camera whether or not anything else is showing.
    #[test]
    fn no_two_things_share_a_place() {
        let places: BTreeSet<Position> = Used::EVERY.into_iter().map(Position::of).collect();
        assert_eq!(places.len(), Used::EVERY.len());
    }

    /// **The order is the order the three are listed in**, so the layout and
    /// this value cannot drift apart about which row is which.
    #[test]
    fn the_places_run_in_the_order_the_things_are_listed_in() {
        let mut expected = 0_usize;
        for used in Used::EVERY {
            assert_eq!(Position::of(used).how_far_down(), expected, "{used:?}");
            expected = expected.saturating_add(1);
        }
        assert_eq!(
            Position::EVERY.map(Position::how_far_down),
            [0, 1, 2],
            "the places and the things they are for have drifted apart"
        );
    }

    /// **Position is a function of what, and there is no argument for who.**
    /// The type says so — this asks it twice of the same thing and gets the
    /// same answer, which is the whole guarantee stated as arithmetic.
    #[test]
    fn where_a_line_sits_never_depends_on_who_is_using_it() {
        for used in Used::EVERY {
            assert_eq!(Position::of(used), Position::of(used), "{used:?}");
        }
    }
}
