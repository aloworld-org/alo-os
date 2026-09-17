//! What choosing an entry did.
//!
//! Two answers, and the difference between them is the promise this crate
//! makes: **no menu entry sends anything to the agent without the person
//! choosing the entry that says so.**
//!
//! [`Chosen::OfferedToTheAgent`] comes from exactly one
//! [`crate::Action`], and `menu.rs` is where that is decided — by asking the
//! action itself, which answers from a `match` the compiler holds to every
//! variant. A new entry that reached the agent would have to say so in
//! [`crate::Action::reaches_the_agent`], and a new entry that did it quietly
//! would have to be written by somebody editing the one line that decides.
//!
//! # Offered is not sent, and it is not a grant either
//!
//! What this answers is that the person chose to offer something. What is then
//! offered, and for how long, is `alo-context`'s and `alo-handing`'s — one turn,
//! no grant, ADR 0001 §§3 and 4. Nothing is carried in this value: a menu knows
//! what was under the pointer, not what is in it.

use crate::action::Action;

/// What happened when the person chose an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chosen {
    /// The machine, or the application, does it. Nothing is sent anywhere.
    DoneOnThisMachine(Action),
    /// The person chose the entry that says so, and this is offered to the
    /// agent — for one question, and as a grant of nothing.
    OfferedToTheAgent(Action),
}

impl Chosen {
    /// Which entry was chosen.
    #[must_use]
    pub const fn action(self) -> Action {
        match self {
            Self::DoneOnThisMachine(action) | Self::OfferedToTheAgent(action) => action,
        }
    }

    /// Whether anything was offered to the agent.
    ///
    /// The question a reviewer asks about a context menu, answerable rather than
    /// argued.
    #[must_use]
    pub const fn reached_the_agent(self) -> bool {
        matches!(self, Self::OfferedToTheAgent(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of the two answers reaches the agent, and it carries the entry the
    /// person actually chose.
    #[test]
    fn only_one_of_the_two_answers_reaches_the_agent() {
        let here = Chosen::DoneOnThisMachine(Action::Copy);
        let offered = Chosen::OfferedToTheAgent(Action::AskTheAgentAboutThis);

        assert!(!here.reached_the_agent());
        assert_eq!(here.action(), Action::Copy);
        assert!(offered.reached_the_agent());
        assert_eq!(offered.action(), Action::AskTheAgentAboutThis);
    }
}
