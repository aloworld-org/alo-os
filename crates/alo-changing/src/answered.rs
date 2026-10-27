//! What answering a waiting change came back as.
//!
//! ADR 0001 §5: *any change to the machine is proposed with a sentence
//! describing it and waits for one approval. What a person approves is that
//! sentence.* This is the other half of that sentence — what came back once
//! they answered — and it is deliberately only two cases.
//!
//! # It decides nothing and words nothing
//!
//! **One approval is one execution**, and that rule is not kept here. It is
//! `alo_capability::Approved::redeem`'s, and a second answer to the same
//! number is refused there rather than guarded against on this side. A door
//! that tried to keep it would be a second place to get it wrong, and the one
//! that matters would still be the daemon's.
//!
//! **A refusal arrives already worded.** The daemon says why in the language
//! the person reads, and [`Answered::Refused`] carries that text unchanged. A
//! surface shows it; nothing here shortens it, heads it, or decides what it
//! meant. That is `crate::refusing`'s rule for every other request on this
//! door, kept here too.
//!
//! # Why `Done` is boxed
//!
//! `alo_protocol::Done` holds a folder's whole listing or a file's contents,
//! and is much larger than a refusal's sentence. Unboxed, every `Answered`
//! would be as big as the largest thing a change can come back with, including
//! the ones that come back with almost nothing.

use alo_protocol::Done;

/// What the daemon said when a person answered a waiting change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answered {
    /// It was carried out, and this is what it did.
    It(Box<Done>),
    /// It was not, and this is the daemon's own reason, in the language the
    /// person reads.
    Refused {
        /// What to show them, unchanged.
        told: String,
    },
    /// The person said no, and the daemon recorded it: nothing was done.
    ///
    /// **A case of its own since 2026-10-26.** The daemon has always answered a
    /// decline with `ToAPerson::Declined`, and this door read it as no answer
    /// at all — so a surface could not tell *you said no and it was kept* from
    /// *this machine never replied*, which are opposite things to tell a
    /// person.
    Declined,
}

impl Answered {
    /// What the change did, when it was carried out.
    #[must_use]
    pub fn did(&self) -> Option<&Done> {
        match self {
            Self::It(done) => Some(done),
            Self::Refused { .. } | Self::Declined => None,
        }
    }

    /// Why it was not, in the daemon's words.
    #[must_use]
    pub fn told(&self) -> Option<&str> {
        match self {
            Self::It(_) | Self::Declined => None,
            Self::Refused { told } => Some(told),
        }
    }

    /// Whether the change was carried out.
    #[must_use]
    pub const fn was_done(&self) -> bool {
        matches!(self, Self::It(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A refusal to answer with.
    fn refused() -> Answered {
        Answered::Refused {
            told: String::from("that number is not waiting for you"),
        }
    }

    /// The two cases are told apart by every reader, and neither answers for
    /// the other — which is the whole reason this is an enum and not a
    /// `Result<Done, String>` somebody could flatten.
    #[test]
    fn a_refusal_and_a_change_that_ran_are_never_confused() {
        let done = Answered::It(Box::new(Done::Renamed { now_at: None }));
        assert!(done.was_done());
        assert!(done.did().is_some());
        assert_eq!(done.told(), None);

        assert!(!refused().was_done());
        assert_eq!(refused().did(), None);
        assert_eq!(refused().told(), Some("that number is not waiting for you"));
    }

    /// **The refusal is carried unchanged.** A surface shows the daemon's own
    /// words, so a door that trimmed or rephrased them would be inventing a
    /// reason nobody gave.
    #[test]
    fn a_refusals_words_are_the_daemons_own() {
        let said = "  that number is not waiting for you  ";
        let answered = Answered::Refused {
            told: said.to_owned(),
        };
        assert_eq!(answered.told(), Some(said));
    }
}
