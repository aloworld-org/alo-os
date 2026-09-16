//! The states of one thing in an application's window: whether it is on the
//! screen, whether a person could use it, and whether it is on.
//!
//! `AtspiStateType` numbers each state by its bit, and the rented tree answers
//! with two 32-bit words, low word first. Five states are read here; the rest
//! say nothing an agent reading or pressing a control needs.

use crate::role::Role;

/// The states of one thing, as the tree reports them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct States(u64);

impl States {
    /// `ATSPI_STATE_CHECKED`.
    const CHECKED: u32 = 4;
    /// `ATSPI_STATE_ENABLED`.
    const ENABLED: u32 = 8;
    /// `ATSPI_STATE_SENSITIVE`.
    const SENSITIVE: u32 = 24;
    /// `ATSPI_STATE_SHOWING`.
    const SHOWING: u32 = 25;
    /// `ATSPI_STATE_CHECKABLE`.
    const CHECKABLE: u32 = 41;

    /// The states in the two words the tree answers with, low word first.
    #[must_use]
    pub fn of_the_tree(words: &[u32]) -> Self {
        let low = words.first().copied().unwrap_or(0);
        let high = words.get(1).copied().unwrap_or(0);
        Self(u64::from(low) | (u64::from(high) << 32))
    }

    /// The two words the tree answers with, low word first — what a tree a test
    /// builds reports.
    #[must_use]
    pub const fn as_the_tree_says(self) -> [u32; 2] {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "each half of a 64-bit set of states is one 32-bit word, by construction"
        )]
        let words = [self.0 as u32, (self.0 >> 32) as u32];
        words
    }

    /// The bits a showing thing a person could use has — what a tree a test
    /// builds starts from.
    #[must_use]
    pub const fn showing_and_usable() -> Self {
        Self((1 << Self::SHOWING) | (1 << Self::ENABLED) | (1 << Self::SENSITIVE))
    }

    /// These states, and checked.
    #[must_use]
    pub const fn and_checked(self) -> Self {
        Self(self.0 | (1 << Self::CHECKED))
    }

    /// These states, greyed out.
    #[must_use]
    pub const fn greyed_out(self) -> Self {
        Self(self.0 & !(1 << Self::SENSITIVE) & !(1 << Self::ENABLED))
    }

    /// These states, off the screen.
    #[must_use]
    pub const fn hidden(self) -> Self {
        Self(self.0 & !(1 << Self::SHOWING))
    }

    /// Whether the bit numbered `state` is set.
    const fn has(self, state: u32) -> bool {
        self.0 & (1 << state) != 0
    }

    /// Whether it is on the screen now.
    #[must_use]
    pub const fn showing(self) -> bool {
        self.has(Self::SHOWING)
    }

    /// Whether a person could use it now — neither greyed out nor disabled.
    #[must_use]
    pub const fn usable(self) -> bool {
        self.has(Self::ENABLED) && self.has(Self::SENSITIVE)
    }

    /// On or off, for a thing that can be either; [`None`] for one that cannot.
    #[must_use]
    pub const fn on(self, role: Role) -> Option<bool> {
        let checkable = self.has(Self::CHECKABLE)
            || matches!(role, Role::CheckBox | Role::RadioButton | Role::Switch);
        if checkable {
            Some(self.has(Self::CHECKED))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_states_the_tree_reports_are_read_from_both_words() {
        // A check box a GTK 3 window reported: enabled, focusable, sensitive,
        // showing, visible.
        let states = States::of_the_tree(&[1_124_075_776, 0]);
        assert!(states.showing());
        assert!(states.usable());
        assert_eq!(states.on(Role::CheckBox), Some(false));
        assert_eq!(states.on(Role::Button), None);
        // Checkable is in the high word.
        let checkable = States::of_the_tree(&[0, 1 << 9]);
        assert_eq!(checkable.on(Role::MenuItem), Some(false));
        assert!(!States::showing_and_usable().greyed_out().usable());
        assert!(!States::showing_and_usable().hidden().showing());
        assert_eq!(
            States::showing_and_usable().and_checked().on(Role::Switch),
            Some(true)
        );
    }
}
