//! Everything a context menu can offer, as a closed list.
//!
//! **Closed is the whole design.** A menu whose entries an application could
//! write would be a list of sentences alo OS did not author, shown in the
//! product's own chrome, in a language nobody translated — and the entry that
//! read *send this to the assistant* would be one an application had put in
//! front of a person rather than one the system offers. So the list is here, it
//! is fourteen long, and a fifteenth is a string this crate declares, which is a
//! change somebody makes deliberately.
//!
//! # Every entry has another road, and that is checked rather than believed
//!
//! ADR 0009: *anything an agent verb can do, a person must also be able to do by
//! hand* — and the rule read the other way is what a menu is for. A context menu
//! is a **convenience**, not a capability: everything in it can be reached
//! somewhere else, so a person who never right-clicks, or who cannot, loses
//! speed and never function. [`Action::also_reached_by`] is where each entry
//! says where else, the compiler holds every action to answering, and
//! `tests/a_menu_is_a_closed_list_of_actions.rs` reads the answers.
//!
//! # And one of them, and only one, reaches the agent
//!
//! [`Action::AskTheAgentAboutThis`] is the entry that says what it does. Nothing
//! else in this list sends anything anywhere, which is
//! [`Action::reaches_the_agent`], and which the tests assert by choosing every
//! other entry and finding that nothing was offered.

use alo_shortcuts::Action as Shortcut;
use alo_strings::{Filling, Said, Strings};

use crate::also_by::AlsoBy;
use crate::words::{self, Word};

/// One thing a context menu can offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    /// Open the file.
    Open,
    /// Open the file with something other than the usual application.
    OpenWith,
    /// Give the file another name.
    Rename,
    /// Put the file in the wastebasket.
    MoveToTheWastebasket,
    /// Copy what is under the pointer.
    Copy,
    /// Cut what is under the pointer.
    Cut,
    /// Paste what was copied.
    Paste,
    /// Select everything.
    SelectAll,
    /// Close the window.
    CloseTheWindow,
    /// Put the window on the left half of the screen.
    TheLeftHalf,
    /// Put the window on the right half of the screen.
    TheRightHalf,
    /// Change the desktop background.
    ChangeTheBackground,
    /// Open Settings.
    OpenSettings,
    /// Offer this to the agent, for one question.
    ///
    /// **The one entry in this crate that reaches the agent**, and it says so in
    /// the words a person reads before they choose it.
    AskTheAgentAboutThis,
}

impl Action {
    /// Everything a context menu can offer, in the order this file declares it.
    ///
    /// Written down so that a test can walk the list rather than a person
    /// remembering to add one — `alo_shortcuts::Action::ALL`'s shape, copied
    /// rather than re-decided.
    pub const ALL: &'static [Self] = &[
        Self::Open,
        Self::OpenWith,
        Self::Rename,
        Self::MoveToTheWastebasket,
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::SelectAll,
        Self::CloseTheWindow,
        Self::TheLeftHalf,
        Self::TheRightHalf,
        Self::ChangeTheBackground,
        Self::OpenSettings,
        Self::AskTheAgentAboutThis,
    ];

    /// Whether choosing this offers anything to the agent.
    ///
    /// One entry answers yes, and it is the one whose words say so. A menu
    /// entry that quietly sent a person's document somewhere would be the
    /// background reader ADR 0001 §4 calls a bug, wearing a right-click.
    #[must_use]
    pub const fn reaches_the_agent(self) -> bool {
        matches!(self, Self::AskTheAgentAboutThis)
    }

    /// Where else a person reaches this, without a context menu.
    ///
    /// Every entry answers, and the compiler is what holds it to that: a
    /// `match` over a closed list cannot leave one out. An action with nowhere
    /// else to be reached would be a capability that lived in a right-click —
    /// unreachable by somebody driving the machine from the keyboard, and
    /// unreachable by a screen reader that does not open menus.
    #[must_use]
    pub const fn also_reached_by(self) -> AlsoBy {
        match self {
            Self::Open | Self::OpenWith | Self::Rename | Self::MoveToTheWastebasket => {
                AlsoBy::TheFilesWindow
            }
            Self::Copy | Self::Cut | Self::Paste | Self::SelectAll => {
                AlsoBy::TheApplicationsOwnMenus
            }
            Self::CloseTheWindow => AlsoBy::AShortcut(Shortcut::CloseWindow),
            Self::TheLeftHalf => AlsoBy::AShortcut(Shortcut::SnapLeft),
            Self::TheRightHalf => AlsoBy::AShortcut(Shortcut::SnapRight),
            Self::ChangeTheBackground | Self::OpenSettings => AlsoBy::Settings,
            Self::AskTheAgentAboutThis => AlsoBy::AShortcut(Shortcut::TheAgent),
        }
    }

    /// The string this crate declares for it: the row a person reads.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Open => words::OPEN,
            Self::OpenWith => words::OPEN_WITH,
            Self::Rename => words::RENAME,
            Self::MoveToTheWastebasket => words::MOVE_TO_THE_WASTEBASKET,
            Self::Copy => words::COPY,
            Self::Cut => words::CUT,
            Self::Paste => words::PASTE,
            Self::SelectAll => words::SELECT_ALL,
            Self::CloseTheWindow => words::CLOSE_THE_WINDOW,
            Self::TheLeftHalf => words::THE_LEFT_HALF,
            Self::TheRightHalf => words::THE_RIGHT_HALF,
            Self::ChangeTheBackground => words::CHANGE_THE_BACKGROUND,
            Self::OpenSettings => words::OPEN_SETTINGS,
            Self::AskTheAgentAboutThis => words::ASK_THE_AGENT_ABOUT_THIS,
        }
    }

    /// What this entry says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// The list is the list: every action is in it once, and nothing is in it
    /// twice.
    #[test]
    fn everything_a_menu_can_offer_is_named_once() {
        let named: BTreeSet<Action> = Action::ALL.iter().copied().collect();
        assert_eq!(named.len(), Action::ALL.len());
    }

    /// **Exactly one entry reaches the agent**, and it is the one whose words
    /// say what it does.
    #[test]
    fn one_entry_reaches_the_agent_and_says_so() {
        let reaching: Vec<Action> = Action::ALL
            .iter()
            .copied()
            .filter(|action| action.reaches_the_agent())
            .collect();
        assert_eq!(reaching, [Action::AskTheAgentAboutThis]);

        let said = Action::AskTheAgentAboutThis.said(&in_english());
        assert!(said.text().to_lowercase().contains("agent"), "{said}");
    }

    /// Every entry reads as something of its own, with nothing left to fill in.
    #[test]
    fn every_entry_reads_as_something_of_its_own() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for action in Action::ALL {
            let said = action.said(&strings);
            assert!(!said.is_a_bug(), "{action:?}");
            assert!(said.unfilled().is_empty(), "{action:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{action:?}");
        }
    }
}
