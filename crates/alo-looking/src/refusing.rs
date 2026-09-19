//! Everything a check can answer other than an offer, and the sentence each
//! one is read as.
//!
//! **A closed set.** Five, and a sixth cannot arrive without this file
//! changing — which is deliberate, because the alternative is the shape every
//! updater ends up with: an error string from whatever client made the request,
//! shown to a person who then knows a status code and nothing about their
//! machine. What a person needs to know is which of five things happened and
//! that their machine is exactly as it was, and both of those are here.
//!
//! **Told apart by what a person can do about them**, not by what went wrong
//! underneath. *Nothing could be reached at all* and *it did not answer* are
//! two lines rather than one because the first is the machine's connection and
//! the second is the far end — and the first is the one said **once**
//! ([`crate::SaidOnce`]), because a machine with no way out would otherwise
//! repeat it at every asking for as long as the network was down.
//!
//! **One of them is not worded here.** [`NoAnswer::NotUnderstood`] is read as
//! `alo_keeping_up::words::ANSWER_NOT_UNDERSTOOD`, which already says exactly
//! that and is already in the machine's vocabulary. Writing a second sentence
//! for it would be two spellings of one fact, which is the drift every crate in
//! this workspace is arranged to avoid.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a check produced no offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoAnswer {
    /// Nothing could be reached at all: no road out of this machine, or
    /// nothing at the other end of it.
    ///
    /// The one that is said once rather than at every asking.
    #[error("there is no way out of this machine to the place its updates come from")]
    NoWayOut,
    /// The place was reached and no answer arrived before this machine gave up
    /// waiting.
    #[error("the place this machine's updates come from did not answer")]
    NothingCameBack,
    /// It answered, and what it answered was a refusal rather than an answer.
    #[error("the place this machine's updates come from refused this machine")]
    ItRefused,
    /// It answered properly, and offers no version of this system at all.
    #[error("the place this machine's updates come from offers no version of this system")]
    NothingIsOffered,
    /// It answered with something this machine could not read as an answer.
    #[error("the answer about whether there is an update could not be understood")]
    NotUnderstood,
}

impl NoAnswer {
    /// Every one of them, in the order this file declares them.
    ///
    /// What the test per refusal walks, so that a sixth added here arrives in
    /// the acceptance rather than quietly beside it.
    pub const EVERY: [Self; 5] = [
        Self::NoWayOut,
        Self::NothingCameBack,
        Self::ItRefused,
        Self::NothingIsOffered,
        Self::NotUnderstood,
    ];

    /// The string this refusal is read as.
    ///
    /// Four are this crate's; the fifth is `alo-keeping-up`'s, for the reason
    /// this file's header gives.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::NoWayOut => words::NO_WAY_OUT,
            Self::NothingCameBack => words::NOTHING_CAME_BACK,
            Self::ItRefused => words::IT_REFUSED,
            Self::NothingIsOffered => words::NOTHING_IS_OFFERED,
            Self::NotUnderstood => alo_keeping_up::words::ANSWER_NOT_UNDERSTOOD,
        }
    }

    /// What a person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given this machine's vocabulary answers with
    /// the key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// Each of the five reads as its own sentence, and no two read alike.
    #[test]
    fn each_refusal_is_its_own_sentence() {
        let strings = in_english();
        let mut read = std::collections::BTreeSet::new();
        for refusal in NoAnswer::EVERY {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            read.insert(said.into_text());
        }
        assert_eq!(read.len(), NoAnswer::EVERY.len(), "two refusals read alike");
    }

    /// **The answer nobody could read is `alo-keeping-up`'s sentence**, not a
    /// second one saying the same thing.
    #[test]
    fn an_answer_that_could_not_be_read_is_the_sentence_that_already_existed() {
        assert_eq!(
            NoAnswer::NotUnderstood.word().named(),
            alo_keeping_up::words::ANSWER_NOT_UNDERSTOOD.named()
        );
        assert!(
            !words::EVERY_WORD
                .iter()
                .any(|word| word.named() == NoAnswer::NotUnderstood.word().named()),
            "this crate declares a second sentence for an answer it could not read"
        );
    }

    /// The Rust-facing text is for whoever reads a journal, and says nothing a
    /// person would be shown: it is never a sentence on a screen.
    #[test]
    fn what_a_journal_reads_is_not_what_a_person_reads() {
        let strings = in_english();
        for refusal in NoAnswer::EVERY {
            assert_ne!(refusal.to_string(), refusal.said(&strings).text());
        }
    }
}
