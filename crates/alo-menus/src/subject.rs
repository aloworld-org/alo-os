//! What is under the pointer when a person asks for a menu.
//!
//! Four things, and a menu is one of them and its list. There is no fifth that
//! means *anything else*: a subject this crate does not know is a subject with
//! no menu, and a right-click on it does nothing rather than offering a list
//! somebody invented on the spot.
//!
//! # The subject decides the entries, and the application does not
//!
//! An application is not asked what to put in the menu. What it can do is refuse
//! to draw one at all, which is its own window's business; what it cannot do is
//! add an entry, reword one, or put its own sentence where a person expects the
//! system's. `action.rs` is why.

/// What a person asked for a menu about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    /// A file — in the files window, or on the desktop.
    AFile,
    /// Text a person has selected.
    SomeText,
    /// A window, by its title bar or its entry in the dock.
    AWindow,
    /// The desktop itself, with nothing on it under the pointer.
    TheDesktop,
}

impl Subject {
    /// Everything a menu can be about, in the order this file declares it.
    pub const ALL: &'static [Self] =
        &[Self::AFile, Self::SomeText, Self::AWindow, Self::TheDesktop];

    /// Whether a person can hand this to the agent at all.
    ///
    /// A file and a selection are things a person can offer; a window and the
    /// desktop are not. The distinction is `alo-context`'s: what a person had
    /// **open** or had **selected** is something they chose, and the rest of
    /// what happened to be in front of them is not — so a menu over a window
    /// offers no road to the agent, because there would be nothing in it that a
    /// person had decided to hand over.
    #[must_use]
    pub const fn can_be_offered_to_the_agent(self) -> bool {
        matches!(self, Self::AFile | Self::SomeText)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Everything a menu can be about is named once.
    #[test]
    fn everything_a_menu_can_be_about_is_named_once() {
        let named: BTreeSet<Subject> = Subject::ALL.iter().copied().collect();
        assert_eq!(named.len(), Subject::ALL.len());
    }

    /// What a person can offer is what they chose: a file they were pointing at
    /// and text they had selected, and never a window they happened to be
    /// looking at.
    #[test]
    fn only_what_a_person_chose_can_be_offered() {
        assert!(Subject::AFile.can_be_offered_to_the_agent());
        assert!(Subject::SomeText.can_be_offered_to_the_agent());
        assert!(!Subject::AWindow.can_be_offered_to_the_agent());
        assert!(!Subject::TheDesktop.can_be_offered_to_the_agent());
    }
}
