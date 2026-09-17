//! One menu: what the thing under the pointer offers, and what choosing does.
//!
//! # A menu is built, never handed in
//!
//! [`Menu::over`] is the only way to make one, and what it makes is decided by
//! the subject and by whether this machine has an agent at all. Nothing takes a
//! list of entries from anywhere, so there is no road by which an application, a
//! model or a settings file puts a row in front of a person.
//!
//! # With no agent, the entry is absent rather than greyed out
//!
//! ADR 0009: *the agent's surfaces disappear rather than nag* — **a greyed-out
//! feature is an advertisement.** A person who declined an agent at setup, or
//! whose subscription lapsed, or who is offline, does not meet a menu row about
//! it. [`TheAgent::NotOnThisMachine`] takes the entry out of the list entirely,
//! and the test that says so walks every subject.
//!
//! # Choosing is answered, and only one answer reaches the agent
//!
//! [`Menu::chosen`] hands back a [`crate::Chosen`], and the variant that offers
//! anything to the agent comes from exactly one entry: the one whose words say
//! that is what it does. Everything else is done on this machine.

use crate::action::Action;
use crate::choosing::Chosen;
use crate::refusing::NotChosen;
use crate::subject::Subject;

/// Whether this machine has an agent to offer anything to.
///
/// Not a preference and not a setting read here: the answer a shell already
/// holds, handed in, so that this crate has no opinion about where it came from
/// — declined at setup, no model downloaded, the money ran out, or the machine
/// is offline (ADR 0009's six reasons, only one of which is a choice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAgent {
    /// There is an agent, and a person can offer it something.
    OnThisMachine,
    /// There is not. No menu says a word about one.
    NotOnThisMachine,
}

/// The menu the thing under the pointer offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    /// What it is about.
    subject: Subject,
    /// What it offers, in the order it is read out.
    entries: Vec<Action>,
}

impl Menu {
    /// The menu for what is under the pointer, on a machine with or without an
    /// agent.
    #[must_use]
    pub fn over(subject: Subject, agent: TheAgent) -> Self {
        let mut entries = what_is_offered(subject);
        if subject.can_be_offered_to_the_agent() && agent == TheAgent::OnThisMachine {
            entries.push(Action::AskTheAgentAboutThis);
        }
        Self { subject, entries }
    }

    /// What this menu is about.
    #[must_use]
    pub const fn subject(&self) -> Subject {
        self.subject
    }

    /// What it offers, in the order a person reads them.
    #[must_use]
    pub fn entries(&self) -> &[Action] {
        &self.entries
    }

    /// Whether this menu offers that action.
    #[must_use]
    pub fn offers(&self, action: Action) -> bool {
        self.entries.contains(&action)
    }

    /// The person chose an entry.
    ///
    /// # Errors
    /// [`NotChosen::NotOnThisMenu`] for an action this menu does not offer —
    /// the stale menu, drawn over a file that has since been deleted, or a
    /// shell asking for something that was never on it. **Nothing happens on
    /// one**, and in particular nothing is offered to the agent: an action that
    /// is not on the menu was not one the person chose.
    pub fn chosen(&self, action: Action) -> Result<Chosen, NotChosen> {
        if !self.offers(action) {
            return Err(NotChosen::NotOnThisMenu { action });
        }
        if action.reaches_the_agent() {
            return Ok(Chosen::OfferedToTheAgent(action));
        }
        Ok(Chosen::DoneOnThisMachine(action))
    }
}

/// What each subject offers, before the agent's entry is considered.
///
/// A plain `match` over a closed list of subjects, so a subject added here is a
/// subject whose menu somebody had to write.
fn what_is_offered(subject: Subject) -> Vec<Action> {
    match subject {
        Subject::AFile => vec![
            Action::Open,
            Action::OpenWith,
            Action::Copy,
            Action::Cut,
            Action::Rename,
            Action::MoveToTheWastebasket,
        ],
        Subject::SomeText => vec![Action::Copy, Action::Cut, Action::Paste, Action::SelectAll],
        Subject::AWindow => vec![
            Action::TheLeftHalf,
            Action::TheRightHalf,
            Action::CloseTheWindow,
        ],
        Subject::TheDesktop => vec![Action::ChangeTheBackground, Action::OpenSettings],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A menu is the subject's list, and choosing something on it does it here.
    #[test]
    fn a_menu_offers_what_its_subject_offers() {
        let menu = Menu::over(Subject::AFile, TheAgent::OnThisMachine);
        assert_eq!(menu.subject(), Subject::AFile);
        assert!(menu.offers(Action::Open));
        assert!(!menu.offers(Action::ChangeTheBackground));
        assert_eq!(
            menu.chosen(Action::Open),
            Ok(Chosen::DoneOnThisMachine(Action::Open))
        );
    }

    /// **An action that is not on the menu was not chosen**, and nothing
    /// happens — including, and especially, nothing being offered to the agent.
    #[test]
    fn an_action_that_is_not_on_the_menu_does_nothing() {
        let menu = Menu::over(Subject::AWindow, TheAgent::OnThisMachine);
        assert_eq!(
            menu.chosen(Action::AskTheAgentAboutThis),
            Err(NotChosen::NotOnThisMenu {
                action: Action::AskTheAgentAboutThis
            })
        );
        assert_eq!(
            menu.chosen(Action::Open),
            Err(NotChosen::NotOnThisMenu {
                action: Action::Open
            })
        );
    }

    /// The agent's entry is last where it appears at all: a person reading down
    /// a menu meets what they came for first.
    #[test]
    fn the_agents_entry_comes_last() {
        let menu = Menu::over(Subject::SomeText, TheAgent::OnThisMachine);
        assert_eq!(menu.entries().last(), Some(&Action::AskTheAgentAboutThis));
    }
}
