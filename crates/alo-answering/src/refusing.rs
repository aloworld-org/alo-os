//! Why this crate says no to an answer a person gave.
//!
//! There is one of these, and it is here rather than in `failed.rs` for the
//! reason every `refusing.rs` in this repository exists: the sentences a person
//! reads when something is refused are worth being able to find in one place,
//! by somebody reviewing what this machine tells people.
//!
//! The other refusal this crate makes — [`crate::NotWhatFailed`] — is
//! deliberately **not** here. It refuses what an adapter reported rather than
//! what a person answered, it keeps its English and its `Display`, and its
//! reader is whoever is writing that adapter. `alo_capability::VerbError` and
//! `alo_shortcuts::DefaultsError` are the same distinction two crates over.
//!
//! # It carries the failure back
//!
//! A dialogue that has been open a while can be answered with an offer from an
//! older question. That is a person's ordinary mistake rather than a
//! programming one, and losing what they were looking at would be this crate
//! punishing them for it — so the refusal holds the failure and
//! [`NotOffered::back`] returns it. `alo_capability::Refused` carries the call
//! it refused for the same reason: a refusal that threw away what it refused
//! could only say that something was stopped.

use alo_strings::{Filling, Said, Strings};

use crate::failed::Failed;
use crate::words;

/// An offer that was not the one this failure made.
///
/// Boxed, because every [`crate::Failed::take`] returns it in the `Err` and the
/// path that succeeds should not carry a failure it never reads —
/// `alo_capability::Refused`'s boxing, and clippy's `result_large_err`, one
/// crate on.
#[derive(Debug, PartialEq, Eq)]
pub struct NotOffered {
    /// What was refused, kept whole.
    failed: Box<Failed>,
}

impl NotOffered {
    /// Made by [`crate::Failed::take`] and by nothing else.
    pub(crate) fn of(failed: Failed) -> Self {
        Self {
            failed: Box::new(failed),
        }
    }

    /// The failure this was refused for, so the same person can be shown the
    /// same dialogue rather than an empty one.
    #[must_use]
    pub fn back(self) -> Failed {
        *self.failed
    }

    /// What was refused, without taking it.
    #[must_use]
    pub fn failed(&self) -> &Failed {
        &self.failed
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        words::NOT_ON_OFFER
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::answering_words`] answers with the key, marked. **What was
    /// refused never depends on the string table** — nothing was sent before
    /// this was called, and calling it cannot change that.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        an_offer_from_another_failure, failing, here, hosted, in_english, translated,
    };
    use crate::wrong::WentWrong;

    /// The sentence says the thing the person needs first: their question did
    /// not go anywhere.
    #[test]
    fn the_refusal_says_nothing_was_sent_before_it_says_what_to_do() {
        let refused = failing(here(), WentWrong::NothingAnswered, &[hosted()])
            .take(&an_offer_from_another_failure())
            .unwrap_err();
        let said = refused.said(&in_english());
        assert_eq!(
            said.text(),
            "that was offered for a different question, so nothing was sent — ask again"
        );
    }

    /// The failure can be read without being taken back, and taken back
    /// afterwards — a shell showing the dialogue again needs both.
    #[test]
    fn what_was_refused_can_be_read_and_then_taken_back() {
        let refused = failing(hosted(), WentWrong::TookTooLong, &[])
            .take(&an_offer_from_another_failure())
            .unwrap_err();
        assert_eq!(refused.failed().source(), &hosted());
        assert_eq!(refused.failed().why(), WentWrong::TookTooLong);
        assert_eq!(refused.back().source(), &hosted());
    }

    /// **A refusal without the words still names the key**, so whoever forgot
    /// to declare this crate's vocabulary finds out from the sentence rather
    /// than from a blank dialogue.
    #[test]
    fn a_refusal_without_the_words_still_names_the_key() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let refused = failing(here(), WentWrong::NothingAnswered, &[hosted()])
            .take(&an_offer_from_another_failure())
            .unwrap_err();
        let said = refused.said(&nothing);
        assert!(said.is_a_bug());
        assert!(said.text().contains("answering.not-on-offer"), "{said}");
    }

    /// It is read in the reader's own language, like everything else this crate
    /// says.
    #[test]
    fn the_refusal_is_said_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NOT_ON_OFFER,
            "das galt einer anderen Frage, es wurde nichts gesendet — fragen Sie noch einmal",
        )]);
        let refused = failing(here(), WentWrong::NothingAnswered, &[hosted()])
            .take(&an_offer_from_another_failure())
            .unwrap_err();
        let said = refused.said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("nichts gesendet"), "{said}");
    }
}
