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
//! # The bound, and a conversation over it says so
//!
//! [`TheConversationSoFar::THE_MOST_SENT`] bytes of what was said. Over it, the
//! **oldest** turns are left out whole — never cut in the middle, which would
//! change what a sentence says — and the model is told how many were. A turn
//! that is on its own longer than the bound is the one exception: it is cut,
//! and the cut is written where it happened. The person's window says so too,
//! which is the shell's half.
//!
//! # Where the model is shown it, and why the graded words do not move
//!
//! Between what the turn may reach and the request, in the request half of
//! the text, exactly where the language clause and a turn's steps already sit.
//! The instructions a grade names by digest
//! ([`crate::Instructions::SHOWN_TO_A_TURN`]) are untouched, so no grade is
//! set aside. What that costs is the same as the language clause's: a grade
//! earned on a first request is not proof about a follow-up. ADR 0136 says so.

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
    /// The words of this turn, which is what the bound counts.
    #[must_use]
    pub fn said(&self) -> &str {
        match self {
            Self::ThePersonSaid(said) | Self::AloAnswered(said) => said,
            Self::AloProposed { sentence, .. } => sentence,
        }
    }

    /// The same turn with its words replaced, keeping who said it.
    fn saying(&self, said: String) -> Self {
        match self {
            Self::ThePersonSaid(_) => Self::ThePersonSaid(said),
            Self::AloAnswered(_) => Self::AloAnswered(said),
            Self::AloProposed { answer, .. } => Self::AloProposed {
                sentence: said,
                answer: *answer,
            },
        }
    }
}

/// The conversation before a follow-up, already held to the bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheConversationSoFar {
    /// What was said, oldest first.
    turns: Vec<AnEarlierTurn>,
    /// How many of the oldest turns were left out.
    left_out: usize,
    /// Whether the oldest turn kept was cut because it alone was over the
    /// bound.
    cut: bool,
}

/// Said where a turn that alone was over the bound was cut.
pub const WHERE_IT_WAS_CUT: &str = " (this was longer and was cut here)";

impl TheConversationSoFar {
    /// **How much of a conversation is sent with a follow-up**, in bytes of
    /// what was said.
    ///
    /// About two thousand words — room for a request, a listing and two or
    /// three answers, which is what *and the April one too* needs. Not more,
    /// because the whole of it goes to the model with every follow-up, and to
    /// a provider that is the whole of it leaving again each time. A model
    /// with a small context is the other reason: the instructions and the
    /// verbs come first and must still fit.
    pub const THE_MOST_SENT: usize = 8000;

    /// Nothing was said before: a first request.
    #[must_use]
    pub const fn nothing() -> Self {
        Self {
            turns: Vec::new(),
            left_out: 0,
            cut: false,
        }
    }

    /// **The conversation, held to [`Self::THE_MOST_SENT`].**
    ///
    /// The newest turns are kept and the oldest left out whole, because what a
    /// follow-up refers to is nearly always what was just said. A newest turn
    /// that alone is over the bound is cut on a character boundary and says so.
    #[must_use]
    pub fn of(turns: Vec<AnEarlierTurn>) -> Self {
        let mut kept: Vec<AnEarlierTurn> = Vec::new();
        let mut used = 0_usize;
        let mut cut = false;
        for turn in turns.iter().rev() {
            let length = turn.said().len();
            if used + length <= Self::THE_MOST_SENT {
                used += length;
                kept.push(turn.clone());
                continue;
            }
            if kept.is_empty() {
                let room = Self::THE_MOST_SENT.saturating_sub(WHERE_IT_WAS_CUT.len());
                let mut at = room.min(length);
                while at > 0 && !turn.said().is_char_boundary(at) {
                    at -= 1;
                }
                let shown = turn.said().get(..at).unwrap_or("");
                kept.push(turn.saying(format!("{shown}{WHERE_IT_WAS_CUT}")));
                cut = true;
            }
            break;
        }
        let left_out = turns.len() - kept.len();
        kept.reverse();
        Self {
            turns: kept,
            left_out,
            cut,
        }
    }

    /// Rebuilt from parts a reader has already checked against the bound.
    pub(crate) const fn from_parts(turns: Vec<AnEarlierTurn>, left_out: usize, cut: bool) -> Self {
        Self {
            turns,
            left_out,
            cut,
        }
    }

    /// What was said, oldest first.
    #[must_use]
    pub fn turns(&self) -> &[AnEarlierTurn] {
        &self.turns
    }

    /// How many of the oldest turns were left out.
    #[must_use]
    pub const fn left_out(&self) -> usize {
        self.left_out
    }

    /// Whether a turn was cut because it alone was over the bound.
    #[must_use]
    pub const fn was_cut(&self) -> bool {
        self.cut
    }

    /// **Whether anything of the conversation was not sent**, which the person
    /// is told in their window.
    #[must_use]
    pub const fn is_shortened(&self) -> bool {
        self.left_out > 0 || self.cut
    }

    /// Whether there is anything at all: a first request has nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.turns.is_empty() && self.left_out == 0
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
        if self.left_out > 0 {
            text.push_str(&format!(
                "\n({} earlier part(s) of this conversation were left out because it is long.)",
                self.left_out
            ));
        }
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

impl Default for TheConversationSoFar {
    fn default() -> Self {
        Self::nothing()
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]
#[path = "the_conversation_so_far_tests.rs"]
mod tests;
