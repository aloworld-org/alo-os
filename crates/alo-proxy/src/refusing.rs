//! Why a road out was not taken, in words a person can act on.
//!
//! One refusal, with three reasons inside it, and all three say the same thing
//! to the person in front of the machine: **nothing was sent**. That is the
//! sentence that matters, because the alternative — a road that quietly went
//! straight out when the proxy could not be worked out — is a machine sending a
//! company's traffic around the company's own rule, and nobody would ever be
//! told.
//!
//! `alo-egress` refuses a destination its policy cannot permit rather than
//! reaching it anyway; this is the same rule one step earlier, about the road
//! rather than about the destination. The constraint in
//! `docs/autonomy/v0-5-software-and-the-web-plan.md` states it in one line: *a
//! policy that cannot be evaluated refuses.*
//!
//! # What the person is told, and what is kept for whoever fixes it
//!
//! The sentence says nothing was sent and to ask whoever set the network up.
//! What the evaluator actually said, and what it answered that nothing
//! understood, are kept on [`crate::NotEvaluated`] for whoever administers the
//! machine — never shown. It came from outside this machine, and a sentence
//! quoting it would be a sentence somebody outside helped compose;
//! `alo_software::NotDone::DidNotAnswer` keeps its own the same way.

use alo_strings::{Filling, Said, Strings};

use crate::automatic::NotEvaluated;
use crate::words;

/// Why a road out was not taken.
///
/// **No `Display`**, so the only road to words is [`NotOnTheRoad::said`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotOnTheRoad {
    /// The machine's proxy is an automatic configuration, and this machine
    /// could not work out from it where this road goes.
    CouldNotBeWorkedOut(NotEvaluated),
}

impl NotOnTheRoad {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self {
            Self::CouldNotBeWorkedOut(NotEvaluated::NothingEvaluatesIt) => {
                words::NOTHING_WORKS_OUT_THE_PROXY
            }
            Self::CouldNotBeWorkedOut(
                NotEvaluated::DidNotAnswer { .. } | NotEvaluated::NotUnderstood { .. },
            ) => words::THE_PROXY_COULD_NOT_BE_WORKED_OUT,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::proxy_words`] answers with the
    /// key, marked, and `Said::is_a_bug`. **The road was refused before this
    /// was called** — worded afterwards, as `alo-egress` words its own, so a
    /// machine whose translations failed to load refuses exactly what it
    /// refused before.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// What the evaluator said, for whoever administers the machine.
    ///
    /// Never shown to a person: it came from outside this machine.
    #[must_use]
    pub const fn because(&self) -> &NotEvaluated {
        match self {
            Self::CouldNotBeWorkedOut(why) => why,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use crate::words;

    /// **Both sentences say nothing was sent**, which is the fact a person has
    /// to act on, and neither quotes what came from outside the machine.
    #[test]
    fn every_refusal_says_nothing_was_sent_and_quotes_nothing() {
        let strings = in_english();
        for refusal in [
            NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NothingEvaluatesIt),
            NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::DidNotAnswer {
                said: "the program crashed at line 4".to_owned(),
            }),
            NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NotUnderstood {
                answered: "SOCKS proxy.example.com:1080".to_owned(),
            }),
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
            assert!(said.text().contains("nothing was sent"), "{said}");
            assert!(!said.text().contains("SOCKS"), "{said}");
            assert!(!said.text().contains("line 4"), "{said}");
        }
    }

    /// What the evaluator said is kept, and it is kept for whoever administers
    /// the machine rather than for the person.
    #[test]
    fn what_the_evaluator_said_is_kept_for_whoever_fixes_the_machine() {
        let refusal = NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::DidNotAnswer {
            said: "the program crashed at line 4".to_owned(),
        });
        assert_eq!(
            refusal.because(),
            &NotEvaluated::DidNotAnswer {
                said: "the program crashed at line 4".to_owned()
            }
        );
    }

    /// A machine with nothing that works one out says so as its own sentence,
    /// because it is the one with a different thing to do about it.
    #[test]
    fn a_machine_with_nothing_that_works_one_out_has_its_own_sentence() {
        let strings = in_english();
        let nothing =
            NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NothingEvaluatesIt).said(&strings);
        let failed = NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NotUnderstood {
            answered: "nonsense".to_owned(),
        })
        .said(&strings);
        assert_ne!(nothing.text(), failed.text());
    }

    /// **The refusal reaches a person in their own language**, and says so —
    /// this is the sentence somebody meets when their machine has stopped
    /// reaching anything, and English is not what everybody reads.
    #[test]
    fn a_refusal_a_person_meets_is_read_in_their_own_language() {
        let strings = translated(&[(
            words::THE_PROXY_COULD_NOT_BE_WORKED_OUT,
            "Dieser Rechner konnte nicht ermitteln, wie das zu erreichen ist. Es wurde nichts \
             gesendet",
        )]);
        let said = NotOnTheRoad::CouldNotBeWorkedOut(NotEvaluated::NotUnderstood {
            answered: "nonsense".to_owned(),
        })
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("nichts"), "{said}");
    }
}
