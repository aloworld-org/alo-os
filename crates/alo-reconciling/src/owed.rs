//! What is still owed on a promise, and how much of an answer that has to be.
//!
//! Most v0.01 promises are partly kept: the code is finished and the machine has
//! never been asked, or four of the five things a sentence names are built and
//! the fifth needs a screen. **Both halves go in the ledger**, because a promise
//! recorded as *shown* when a third of it is owed is how a release convinces
//! itself it is finished — and because `ROADMAP.md` already found that failure
//! once, in a line ticked outright whose own footnote said the machine was still
//! owed.
//!
//! Nothing mechanical can judge whether a sentence is honest. What can be held
//! is a floor beneath one, so that `soon` and `todo` are refused; the reader of
//! the ledger is the real check, and this is what stops them reading a shrug.

/// How much of an answer *what is still owed* has to be, in characters.
///
/// `not yet` is seven and passes nothing; the shortest honest entry in this
/// repository's ledger names a thing and why it is not done, which does not fit
/// in forty. Deliberately a floor rather than a judge: a sentence that clears it
/// and says nothing is caught by whoever reads the ledger, and this catches the
/// one nobody would read at all.
pub const AN_ANSWER: usize = 40;

/// What is still owed on a promise, as the ledger words it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owed {
    /// The sentence.
    sentence: String,
}

impl Owed {
    /// What is owed, if the ledger said anything.
    ///
    /// Whitespace-only is nothing said. A dash — which is how a table would say
    /// *none* — is nothing said too, and is deliberately not a way to record an
    /// empty answer as an answer.
    #[must_use]
    pub fn said(sentence: &str) -> Option<Self> {
        let sentence = sentence.trim().trim_matches('—').trim();
        (!sentence.is_empty()).then(|| Self {
            sentence: sentence.to_owned(),
        })
    }

    /// The sentence, as it will be read.
    #[must_use]
    pub fn sentence(&self) -> &str {
        &self.sentence
    }

    /// Whether it is an answer rather than a shrug.
    #[must_use]
    pub fn is_an_answer(&self) -> bool {
        self.sentence.chars().count() >= AN_ANSWER
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Nothing said is nothing said, however it was written.
    #[test]
    fn an_empty_answer_is_not_an_answer() {
        assert_eq!(Owed::said(""), None);
        assert_eq!(Owed::said("   \n "), None);
        assert_eq!(
            Owed::said("—"),
            None,
            "a dash was read as something being owed, so a promise with nothing \
             behind it could be reconciled by a punctuation mark"
        );
    }

    /// And a shrug is refused, while an answer of the shortest honest kind is
    /// not.
    #[test]
    fn a_shrug_is_refused_and_an_answer_is_not() {
        assert!(
            !Owed::said("not yet").unwrap().is_an_answer(),
            "`not yet` was accepted as what is still owed on a release promise"
        );
        assert!(
            !Owed::said("todo").unwrap().is_an_answer(),
            "`todo` was accepted as what is still owed on a release promise"
        );
        assert!(
            Owed::said("no certified machine has booted this, which is task 12")
                .unwrap()
                .is_an_answer(),
            "a sentence naming the thing and why it is not done was refused, so \
             the floor is a wall"
        );
    }
}
