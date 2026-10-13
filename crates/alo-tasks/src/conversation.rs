//! What was said, in the order it was said.
//!
//! The owner's words, 2026-10-12: *Conversation shows the request, alo's
//! replies and questions, and the person's answers.* Four kinds of turn, and
//! the file's shape is one rule: **who said a thing is never inferred.**
//!
//! # Why the speaker is a variant and not a flag
//!
//! A person reading a conversation has to be able to tell, at a glance and
//! without colour, which words are theirs and which are the machine's. A
//! surface that worked that out — from position, from whether a turn has a
//! question mark, from whatever came before — would get it wrong somewhere, and
//! the place it gets it wrong is a person believing they said something they
//! did not.
//!
//! So each turn carries its speaker, and no `match` on one can compile without
//! saying which it is handling.
//!
//! # The proposed action is a turn here, and that is the owner's design
//!
//! *When alo needs permission, show the specific proposed action **in the
//! conversation** with Allow once and Deny.* It is not a dialogue over the top
//! and not a row in the Permissions area: it is a thing alo said, in the place
//! everything else it said appears, so a person reads it in the order it
//! happened. [`crate::ThePermissions`] holds what answering it does.

/// One turn in a task's conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhoSaidIt {
    /// What the person asked for, or answered.
    ThePerson,
    /// What alo replied.
    Alo,
    /// What alo asked the person.
    ///
    /// **Separate from a reply**, because a question is waiting for something
    /// and a reply is not — and a person scanning a conversation for *what does
    /// it need from me* is looking for exactly these.
    AloAsking,
    /// An action alo proposes, which a person allows once or denies.
    ///
    /// Carries no buttons: what answering does is
    /// [`crate::ThePermissions::allow_once`] and `deny`, and a turn that owned
    /// its own answering would be a second place deciding what approval means.
    AloProposing,
}

/// One thing somebody said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    /// Who said it.
    who: WhoSaidIt,
    /// What they said, in the words they read or wrote.
    said: String,
}

impl Turn {
    /// A thing the person said.
    #[must_use]
    pub fn from_the_person(said: impl Into<String>) -> Self {
        Self {
            who: WhoSaidIt::ThePerson,
            said: said.into(),
        }
    }

    /// A thing alo replied.
    #[must_use]
    pub fn from_alo(said: impl Into<String>) -> Self {
        Self {
            who: WhoSaidIt::Alo,
            said: said.into(),
        }
    }

    /// A question alo asked.
    #[must_use]
    pub fn alo_asking(said: impl Into<String>) -> Self {
        Self {
            who: WhoSaidIt::AloAsking,
            said: said.into(),
        }
    }

    /// An action alo proposes, in the sentence a person approves.
    #[must_use]
    pub fn alo_proposing(said: impl Into<String>) -> Self {
        Self {
            who: WhoSaidIt::AloProposing,
            said: said.into(),
        }
    }

    /// Who said it.
    #[must_use]
    pub const fn who(&self) -> &WhoSaidIt {
        &self.who
    }

    /// What was said.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// Whether this turn is waiting for the person.
    ///
    /// **A question and a proposal both are**, and nothing else is. Named here
    /// rather than worked out by each reader, because *is this task waiting on
    /// me* is the question a person asks of a minimised task from across the
    /// room — and two readers answering it differently is two answers to one
    /// question.
    #[must_use]
    pub const fn is_waiting_for_the_person(&self) -> bool {
        matches!(self.who, WhoSaidIt::AloAsking | WhoSaidIt::AloProposing)
    }
}

/// Everything said in one task, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheConversation {
    /// The turns, oldest first — the order a person reads them in.
    turns: Vec<Turn>,
}

impl TheConversation {
    /// A task that has begun with this request.
    ///
    /// **A conversation starts with the person**, always: a task exists because
    /// somebody asked for something, and one that began with alo speaking would
    /// be the agent starting a conversation nobody invited.
    #[must_use]
    pub fn began_with(request: impl Into<String>) -> Self {
        Self {
            turns: vec![Turn::from_the_person(request)],
        }
    }

    /// And then this was said.
    pub fn and_then(&mut self, turn: Turn) {
        self.turns.push(turn);
    }

    /// Every turn, oldest first.
    #[must_use]
    pub fn each(&self) -> &[Turn] {
        &self.turns
    }

    /// What the person first asked for.
    ///
    /// The request a task is named by, and the one turn that is always there.
    #[must_use]
    pub fn the_request(&self) -> Option<&Turn> {
        self.turns.first()
    }

    /// Whether the task is waiting for the person to say something.
    ///
    /// **The last turn decides it**, and only the last: a question alo asked
    /// and the person answered is not still waiting, and a conversation that
    /// scanned for any unanswered question would keep a task looking blocked
    /// for the rest of its life.
    #[must_use]
    pub fn is_waiting_for_the_person(&self) -> bool {
        self.turns
            .last()
            .is_some_and(Turn::is_waiting_for_the_person)
    }
}

#[cfg(test)]
#[path = "conversation_tests.rs"]
mod tests;
