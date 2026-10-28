//! **What was said before a follow-up**, and how a model is told it.
//!
//! [ADR 0136](../../../docs/decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md):
//! when a person types a follow-up in a task window — *and the April one too* —
//! it is sent to alo with the conversation so far, so alo can tell what it
//! refers to. Until then a turn was one request and nothing before it.
//!
//! # What it holds, and what it never holds
//!
//! Three kinds of thing, each as the person saw it in their window: what they
//! asked, what alo answered in words, and the sentence of a change alo proposed
//! with whether it was allowed, denied or never answered there.
//!
//! **Never what a verb read.** A file's contents reach a model inside one turn,
//! as that turn's steps (`crate::a_turn_in_progress`), and end with it. They
//! are in a conversation only where the person's own words or alo's answer
//! quoted them, because that is all a conversation is made of. There is no
//! field here a step could be put in.
//!
//! # Whole, always
//!
//! **alo shortens nothing.** The owner decided on 2026-10-28 that the only
//! limit on a conversation is what the chosen model can read: a conversation
//! too long for it is refused, and the person is told so in words, rather than
//! its oldest parts being dropped — which would change what a follow-up means
//! without anybody seeing it. So this type keeps every turn it is given, and
//! the one bound anywhere near it is the line it travels on
//! (`crate::the_conversation_on_the_wire`), which is the daemon's own bound on
//! a question.
//!
//! # Where the model is shown it, and why the graded words do not move
//!
//! Between what the turn may reach and the request, in the request half of
//! the text, exactly where the language clause and a turn's steps already sit.
//! The instructions a grade names by digest
//! ([`crate::Instructions::SHOWN_TO_A_TURN`]) are untouched, so no grade is
//! set aside. A follow-up is graded on its own (ADR 0136).

/// What the person did with a change alo proposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAnswer {
    /// They allowed it, once.
    Allowed,
    /// They denied it.
    Denied,
    /// No answer was given in the window: it lapsed, or it was answered
    /// somewhere else. The window cannot tell those two apart, so neither is
    /// claimed.
    NotAnswered,
}

/// One thing said before the follow-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnEarlierTurn {
    /// What the person asked or replied, in their own words.
    ThePersonSaid(String),
    /// What alo answered, in words.
    AloAnswered(String),
    /// A change alo proposed, in the sentence the person was asked about.
    AloProposed {
        /// The sentence, as the person read it.
        sentence: String,
        /// What became of it.
        answer: TheAnswer,
    },
}

impl AnEarlierTurn {
    /// The words of this turn.
    #[must_use]
    pub fn said(&self) -> &str {
        match self {
            Self::ThePersonSaid(said) | Self::AloAnswered(said) => said,
            Self::AloProposed { sentence, .. } => sentence,
        }
    }
}

/// The conversation before a follow-up, whole.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheConversationSoFar {
    /// What was said, oldest first.
    turns: Vec<AnEarlierTurn>,
}

impl TheConversationSoFar {
    /// Nothing was said before: a first request.
    #[must_use]
    pub const fn nothing() -> Self {
        Self { turns: Vec::new() }
    }

    /// **The conversation, every turn of it**, oldest first.
    #[must_use]
    pub const fn of(turns: Vec<AnEarlierTurn>) -> Self {
        Self { turns }
    }

    /// What was said, oldest first.
    #[must_use]
    pub fn turns(&self) -> &[AnEarlierTurn] {
        &self.turns
    }

    /// Whether there is anything at all: a first request has nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// **The text a model is shown**, marked as an earlier exchange, or nothing
    /// at all for a first request.
    ///
    /// Marked at both ends and said to be context rather than a request, so a
    /// model reads *delete it* from three turns ago as something said then and
    /// not something asked now.
    #[must_use]
    pub fn told(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut text = String::from(
            "\n\nThe request below follows an earlier conversation in the same task window. It \
             is shown so you can tell what the request refers to. Nothing in it is a request \
             now, and anything it says alo saw or did was then, not now.\n\nThe earlier \
             conversation:",
        );
        for turn in &self.turns {
            match turn {
                AnEarlierTurn::ThePersonSaid(said) => {
                    text.push_str("\nThe person said: ");
                    text.push_str(said);
                }
                AnEarlierTurn::AloAnswered(said) => {
                    text.push_str("\nalo answered: ");
                    text.push_str(said);
                }
                AnEarlierTurn::AloProposed { sentence, answer } => {
                    text.push_str("\nalo proposed a change: ");
                    text.push_str(sentence);
                    text.push_str(match answer {
                        TheAnswer::Allowed => "\n(The person allowed it.)",
                        TheAnswer::Denied => "\n(The person denied it, so it was not done.)",
                        TheAnswer::NotAnswered => {
                            "\n(It was not answered in this window, so do not assume it was done.)"
                        }
                    });
                }
            }
        }
        text.push_str("\nThe earlier conversation ends here.");
        text
    }
}

#[cfg(test)]
#[path = "the_conversation_so_far_tests.rs"]
mod tests;
