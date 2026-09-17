//! Why nothing happened when an entry was chosen.
//!
//! One refusal, and it is the one that matters: an action that is **not on the
//! menu** was not one the person chose. It happens for an ordinary reason — a
//! menu left open over a file that has since been deleted, or over a window that
//! has closed — and the answer has to be *nothing has happened* rather than the
//! action carried out against whatever is there now.
//!
//! It matters most for the one entry that reaches the agent. A shell that could
//! ask for [`crate::Action::AskTheAgentAboutThis`] against a menu that never
//! offered it would be a road by which something was offered to the agent
//! without a person choosing the entry that says so — so the check is on the
//! menu, at the moment of choosing, rather than on what the caller believes the
//! menu held.
//!
//! Like `alo-shortcuts`' refusals, a [`NotChosen`] has no `Display`: the only
//! road to words is [`NotChosen::said`], in the language the person reads.

use alo_strings::{Filling, Said, Strings};

use crate::action::Action;
use crate::words::{self, Word};

/// Why choosing an entry did nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotChosen {
    /// This menu does not offer that action.
    NotOnThisMenu {
        /// What was asked for.
        action: Action,
    },
}

impl NotChosen {
    /// The action that was asked for.
    #[must_use]
    pub const fn action(self) -> Action {
        match self {
            Self::NotOnThisMenu { action } => action,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotOnThisMenu { .. } => words::NOT_ON_THIS_MENU,
        }
    }

    /// What this says, in the language the person reads.
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

    /// The refusal says something declared, with nothing left to fill in, and
    /// keeps the action for whoever is reading a log rather than a screen.
    #[test]
    fn the_refusal_says_something_of_its_own() {
        let refused = NotChosen::NotOnThisMenu {
            action: Action::Open,
        };
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug());
        assert!(said.unfilled().is_empty(), "{said}");
        assert_eq!(refused.action(), Action::Open);
    }
}
