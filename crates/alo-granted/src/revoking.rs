//! What revoking a row comes back as: done and already stopped, or a stale
//! row that changed nothing.
//!
//! There are exactly two, and the second is not an error code. A list is a
//! picture of a moment, grants expire on their own, and a machine has more
//! than one place a revocation can come from — so a row outliving its grant
//! is an ordinary thing that happens to careful people, and the machine owes
//! them a sentence about it rather than silence or a lie.
//!
//! What there deliberately is not: a *partly revoked*, a *revoked but still
//! usable until…*, or any state in between. `alo_capability::Grants::revoke`
//! takes effect on the next question asked, because there is no cache in
//! front of the grants and no decision made ahead of time; the two answers
//! here are the whole truth table of that call.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What became of a revocation asked for through the list.
///
/// Made by [`crate::Seen::revoke`] and readable by anybody; holding one grants
/// nothing and revokes nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revoked {
    /// The grant was removed, and it has already stopped: the next question
    /// asked of the grants is answered without it.
    Now,
    /// The row was stale — the grant had expired or was already revoked — and
    /// the machine's grants are exactly as they were.
    AlreadyGone,
}

impl Revoked {
    /// Whether this act removed a grant.
    #[must_use]
    pub const fn took_effect(self) -> bool {
        matches!(self, Self::Now)
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Now => words::REVOKED,
            Self::AlreadyGone => words::ALREADY_GONE,
        }
    }

    /// What to put in front of the person, in the language they read.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::granted_words`] answers with the key, marked — the honest
    /// answer to *the shell forgot to declare what this crate can say*, and
    /// never a blank line.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Both answers, so no test below quietly skips one.
    const BOTH: [Revoked; 2] = [Revoked::Now, Revoked::AlreadyGone];

    /// **Each answer reads, and no two read the same** — two states sharing a
    /// sentence would be one state wearing two names.
    #[test]
    fn both_answers_read_and_differently() {
        let strings = in_english();
        let now = Revoked::Now.said(&strings);
        let gone = Revoked::AlreadyGone.said(&strings);
        assert!(!now.is_a_bug(), "{now}");
        assert!(!gone.is_a_bug(), "{gone}");
        assert_ne!(now.text(), gone.text());
        assert!(Revoked::Now.took_effect());
        assert!(!Revoked::AlreadyGone.took_effect());
    }

    /// **A machine that never declared these words says so** rather than
    /// answering a revocation with a blank line.
    #[test]
    fn a_machine_that_never_declared_these_words_says_it_is_a_bug() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        for answer in BOTH {
            let said = answer.said(&nothing);
            assert!(said.is_a_bug(), "{answer:?} answered from nowhere");
            assert!(
                said.text().contains(answer.word().named()),
                "{answer:?} does not name the string nobody declared"
            );
        }
    }

    /// **The answer arrives in the language the person reads.**
    #[test]
    fn the_answer_is_read_in_the_readers_own_language() {
        let strings = translated(&[(
            words::ALREADY_GONE,
            "Diese Gewährung besteht nicht mehr: sie ist abgelaufen oder wurde bereits \
             widerrufen. Nichts wurde geändert — die Liste hier war veraltet, sehen Sie sie \
             erneut an",
        )]);
        let said = Revoked::AlreadyGone.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("Nichts wurde geändert"), "{said}");

        // The one nobody translated is still English, and says it is.
        let now = Revoked::Now.said(&strings);
        assert!(!now.is_translated());
        assert!(!now.is_a_bug());
    }
}
