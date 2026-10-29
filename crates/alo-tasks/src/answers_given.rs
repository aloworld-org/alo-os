//! What the person answered to each change a task proposed, in order.
//!
//! The conversation shows the proposal and the card shows Allow once and Deny,
//! and **nothing kept which one was pressed** — the card comes down and the
//! answer goes to the daemon. A follow-up sent with the conversation so far
//! needs it ([ADR 0137](../../../docs/decisions/0137-a-follow-up-is-sent-with-the-conversation-so-far.md)):
//! a model told *alo proposed deleting april.pdf* and not told the person said
//! no would read the file as gone.
//!
//! **A list of answers, not a list of grants.** It records that one sentence was
//! answered one way, once; nothing reads it to decide whether anything may run,
//! and nothing could — an approval is the daemon's, by number, and this holds
//! no number.

use crate::permissions::Answered;

/// Every answer the person gave in this task, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheAnswersGiven {
    /// The sentence answered, and how.
    given: Vec<(String, Answered)>,
}

impl TheAnswersGiven {
    /// No answer given yet.
    #[must_use]
    pub fn nothing_yet() -> Self {
        Self::default()
    }

    /// The person answered this sentence this way.
    ///
    /// **A press that answered nothing is not kept**: a double-click on a card
    /// already answered is not a second answer the person gave.
    pub(crate) fn given(&mut self, sentence: impl Into<String>, answered: Answered) {
        if answered != Answered::NothingWasAsked {
            self.given.push((sentence.into(), answered));
        }
    }

    /// Every answer, oldest first.
    #[must_use]
    pub fn each(&self) -> &[(String, Answered)] {
        &self.given
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Answers are kept in the order given, and a press that answered
    /// nothing is not one of them.**
    #[test]
    fn answers_are_kept_in_order_and_an_empty_press_is_not_an_answer() {
        let mut given = TheAnswersGiven::nothing_yet();
        given.given("Move march.pdf", Answered::AllowedOnce);
        given.given("Move march.pdf", Answered::NothingWasAsked);
        given.given("Delete april.pdf", Answered::Denied);
        assert_eq!(
            given.each(),
            [
                ("Move march.pdf".to_owned(), Answered::AllowedOnce),
                ("Delete april.pdf".to_owned(), Answered::Denied),
            ]
        );
    }
}
