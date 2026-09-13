//! What can go wrong when a question is put somewhere, and what cannot go
//! wrong there.
//!
//! A closed list, for the reason every closed list in this repository is one:
//! the alternative is a `String` an adapter wrote, one `to_string()` from a
//! screen, in English, in a file whose author had no reason to think about
//! language. `alo_models::RuntimeError` was that shape once and item 9f
//! changed it; this list is written the way that one ended up.
//!
//! # It holds no text anybody else wrote
//!
//! Not the question, not the model's name, not what a provider said about
//! itself. The only thing here that came from outside is [`HavingTrouble`]'s
//! number, which is an identifier rather than a sentence.
//!
//! That is a deliberate refusal of something convenient. A variant carrying
//! *what the provider said* would make the most useful-looking failure line in
//! the product, and it would be a line somebody else's service composed,
//! arriving on a person's screen wearing alo OS's voice — which is
//! `alo_models::NotTried`'s rule, and `alo-files`' rule about a filename, met
//! at the one moment a person is least likely to be reading carefully.
//!
//! [`HavingTrouble`]: WentWrong::HavingTrouble
//!
//! # And a reason has to be possible where it is said to have happened
//!
//! [`WentWrong::KeyNotAccepted`] cannot happen on a paired machine, because
//! nothing in this repository reaches one at all, let alone with a key. A
//! person told *the key for this provider was not accepted* about a machine in
//! the next room would go looking for a key that does not exist — which is
//! exactly the confusion `alo-models`' *needs a key* and *key not accepted*
//! pair was written to prevent. So it is refused where it is impossible, at the
//! moment the failure is reported rather than at the moment it is shown.
//!
//! [`WentWrong::RanOut`] is refused in the same place and for the same shape of
//! reason, twice over: nothing reaches a machine on this network, and a machine
//! in the next room bills nobody. Two of the nine reasons are about an
//! arrangement with the far end rather than about the answer — a **key**
//! somebody pasted and an **account** somebody pays for — and a paired machine
//! has neither.
//!
//! # And one reason is about this machine rather than the far end
//!
//! [`WentWrong::NoWayThere`] is the ninth, and it is the only one that says
//! nothing about the place a question was put: the operating system answered
//! that there is **no route** from this machine to that address, so no packet
//! was sent and nothing at the far end was reached. It exists for the office
//! `docs/features.md` promises *works with no internet at all* — where a
//! question to a hosted provider must fail with a true sentence rather than
//! *nothing answered by Mistral*, which would send somebody to check on a
//! provider that is fine. It is reported only when the kernel said so
//! (`alo_asking::openai` reads it off the socket error and off nothing else),
//! and it does not say the machine is *offline*, which no machine can know
//! about itself: a machine on an office network with no internet reaches
//! everything in the building and nothing beyond it, and the sentence says
//! exactly that much.
//!
//! **This machine used to be on that list and no longer is, and the reason is
//! item 18b.** A key was impossible here while the only thing on this machine
//! that could answer was the runtime alo OS ships, which is never given one.
//! Since a person can point alo OS at an OpenAI-compatible service they run
//! themselves — vLLM started with `--api-key`, and it is `InferenceSource::
//! ThisMachine` because nothing leaves — a refused key here is an ordinary
//! thing that really happens, and refusing to report it would send somebody to
//! look at a service that is working for a key that is wrong.
//!
//! What kept the runtime's half of the guarantee is not this check: it is that
//! `alo_asking::locally` translates `alo_models::RuntimeError` into this list
//! and has no arm that can produce [`WentWrong::KeyNotAccepted`] at all, which
//! is a test in that file. A guarantee carried by the absence of a branch is
//! the stronger of the two, and it is where the runtime path actually is.

use alo_models::InferenceSource;

use crate::words;

/// Why the place a question was put did not answer it.
///
/// Nine, and a tenth belongs here only if it is a different thing to be
/// **told** — not a different thing to have happened. *The runtime crashed* and
/// *the runtime was not running* are one sentence to the person reading them.
///
/// [`SentSomewhereElse`] is the seventh, added by `alo-asking` when there was
/// finally something that put a question anywhere, and it passes that bar in a
/// way the others do not: it is not a failure at the far end at all. It is a
/// **refusal alo OS made** — the address answered by pointing somewhere nobody
/// agreed to, and the question was not carried there — and telling somebody
/// *nothing usable came back* would hide the one thing that happened, which is
/// that their machine stopped it. `alo_models::NotTried::Redirected` is the
/// same call about testing a provider, made where the stakes were smaller.
///
/// [`RanOut`] is the eighth, and it passes the bar more plainly than any of
/// them: it is the only one here that is **not a fault**. Nothing is broken,
/// nothing is misconfigured and there is nothing to retry — an account somebody
/// pays for has nothing left in it, which is an ordinary state of an ordinary
/// account. It was reported as [`KeyNotAccepted`] or as
/// [`HavingTrouble`]`(402)` until ADR 0009's *since it was accepted* section
/// named that as the wrong answer twice over: the first sends a person to check
/// a key that is perfectly correct, and the second hands them a number.
///
/// [`NoWayThere`] is the ninth, and it passes the bar because it is the only
/// one that is **about this machine** rather than the far end: the kernel
/// found no route to the address, nothing was sent, and the place was never
/// reached — so *nothing answered there* would be a sentence about a service
/// that was never asked. It is measured, never inferred: only a socket error
/// that says *no route* becomes it, and a refused connection, a timeout or a
/// name that did not resolve stays what it was.
///
/// [`SentSomewhereElse`]: WentWrong::SentSomewhereElse
/// [`RanOut`]: WentWrong::RanOut
/// [`KeyNotAccepted`]: WentWrong::KeyNotAccepted
/// [`HavingTrouble`]: WentWrong::HavingTrouble
/// [`NoWayThere`]: WentWrong::NoWayThere
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WentWrong {
    /// Nothing was listening, or nothing was running.
    NothingAnswered,
    /// Something is there, and it did not answer inside the time this machine
    /// waits.
    TookTooLong,
    /// It answered, and not with anything that could be used — a corrupt
    /// download, or a service answering in a shape this system does not know.
    NothingUsable,
    /// The model itself was not there to answer: the weights are not on this
    /// machine any more, or the provider no longer offers it.
    NoModelThere,
    /// The key was refused. Only somewhere that was given a key can do this,
    /// which is a hosted provider or a service on this machine that somebody
    /// configured one for.
    KeyNotAccepted,
    /// The address answered by sending this machine somewhere else, and the
    /// question was not carried to an address nobody agreed to.
    SentSomewhereElse,
    /// The account the question would have been answered on has run out.
    ///
    /// Not a fault and not something to retry: it works again when somebody
    /// pays, and nothing else about the machine has changed. Only somewhere
    /// with an account can do this, which is a hosted provider or a service
    /// somebody put a budget on.
    RanOut,
    /// It answered, and what it answered was that it was having trouble.
    ///
    /// Carries the status it answered with, which is an identifier and not a
    /// count.
    HavingTrouble(u16),
    /// This machine has no route to that address, so nothing was sent and
    /// nothing there was reached.
    ///
    /// The one reason here that is about this machine's own connection rather
    /// than about the far end, and the one an office with no internet meets
    /// every time a question is bound for a provider. Only the operating
    /// system can say it — *no route to host* or *network unreachable* on the
    /// socket — and nothing here guesses it from a failure that could have
    /// another cause.
    NoWayThere,
}

impl WentWrong {
    /// The string this crate declares for this failure.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::NothingAnswered => words::NOTHING_ANSWERED,
            Self::TookTooLong => words::TOOK_TOO_LONG,
            Self::NothingUsable => words::NOTHING_USABLE,
            Self::NoModelThere => words::NO_MODEL_THERE,
            Self::KeyNotAccepted => words::KEY_NOT_ACCEPTED,
            Self::SentSomewhereElse => words::SENT_SOMEWHERE_ELSE,
            Self::HavingTrouble(_) => words::HAVING_TROUBLE,
            Self::RanOut => words::RAN_OUT,
            Self::NoWayThere => words::NO_WAY_THERE,
        }
    }

    /// Whether this is something that could have happened at this place.
    ///
    /// Everything can happen everywhere except the two reasons that are about
    /// an arrangement with the far end rather than about the answer, and the
    /// one place in ADR 0008 that has neither arrangement is a machine on this
    /// network.
    #[must_use]
    pub fn can_happen(&self, source: &InferenceSource) -> bool {
        !neither_a_key_nor_an_account(source) || self.needs_a_key_or_an_account().is_none()
    }

    /// The refusal this reason is met with where the thing it is about does not
    /// exist, if it is about such a thing at all.
    ///
    /// Two of the nine are: a **key** somebody pasted, and an **account**
    /// somebody pays for. The other seven are things this machine can observe
    /// about any place at all, so they are refused nowhere — including
    /// [`NoWayThere`](Self::NoWayThere), which is about this machine's route
    /// to a place and can be true of any address, loopback included, on a
    /// machine whose loopback interface is down. The list is walked rather
    /// than wildcarded, so a reason added later has to answer this question
    /// rather than inherit an answer.
    fn needs_a_key_or_an_account(&self) -> Option<NotWhatFailed> {
        match self {
            Self::KeyNotAccepted => Some(NotWhatFailed::NoKeyThere),
            Self::RanOut => Some(NotWhatFailed::NoAccountThere),
            Self::NothingAnswered
            | Self::TookTooLong
            | Self::NothingUsable
            | Self::NoModelThere
            | Self::SentSomewhereElse
            | Self::HavingTrouble(_)
            | Self::NoWayThere => None,
        }
    }

    /// The same question, answered as a refusal whoever reported the failure
    /// can act on.
    pub(crate) fn checked(self, source: &InferenceSource) -> Result<Self, NotWhatFailed> {
        match self.needs_a_key_or_an_account() {
            Some(refusal) if neither_a_key_nor_an_account(source) => Err(refusal),
            _ => Ok(self),
        }
    }
}

/// Whether this is a place that has neither a key nor an account.
///
/// One of ADR 0008's three is both, and it is a machine on this network:
/// nothing in this repository reaches one at all, let alone with a credential,
/// and a machine in the next room bills nobody. A person told either thing
/// about the workstation down the corridor would go looking for something that
/// does not exist.
fn neither_a_key_nor_an_account(source: &InferenceSource) -> bool {
    matches!(source, InferenceSource::PairedMachine { .. })
}

/// A failure reported at a place it could not have happened.
///
/// **Keeps its English and its `Display`**, and is not read by whoever is using
/// the machine. It refuses what an adapter *reported*, the way
/// `alo_capability::VerbError` refuses what an adapter *declared*, and its
/// reader is the person writing that adapter. An enum with one variant rather
/// than a struct, because the next impossible pairing is added here.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum NotWhatFailed {
    /// A key was refused somewhere that is never given one.
    #[error(
        "nothing in this repository reaches a machine on this network, let alone with a key, so a \
         question answered on a paired one cannot have been refused for one — report what actually \
         went wrong"
    )]
    NoKeyThere,
    /// An account ran out somewhere that nobody has an account with.
    #[error(
        "a machine on this network bills nobody, and nothing in this repository reaches one to be \
         billed by, so a question answered on a paired one cannot have run out of anything — \
         report what actually went wrong"
    )]
    NoAccountThere,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{here, hosted, paired};

    /// Everything except a refused key can happen anywhere, and the list is
    /// walked rather than sampled so that a variant added later is a test that
    /// fails rather than a case nobody thought about.
    #[test]
    fn almost_anything_can_go_wrong_almost_anywhere() {
        for went_wrong in [
            WentWrong::NothingAnswered,
            WentWrong::TookTooLong,
            WentWrong::NothingUsable,
            WentWrong::NoModelThere,
            WentWrong::SentSomewhereElse,
            WentWrong::HavingTrouble(503),
            WentWrong::NoWayThere,
        ] {
            for source in [here(), paired(), hosted()] {
                assert!(went_wrong.can_happen(&source), "{went_wrong:?} {source:?}");
                assert_eq!(went_wrong.checked(&source), Ok(went_wrong));
            }
        }
    }

    /// **A person told a machine in the next room refused their key would go
    /// looking for a key that does not exist.** The two sentences about keys
    /// are the pair `alo-models` already found people confuse, so the
    /// impossible one is refused where it is reported rather than corrected
    /// where it is shown.
    #[test]
    fn a_key_cannot_have_been_refused_where_nothing_can_send_one() {
        assert!(!WentWrong::KeyNotAccepted.can_happen(&paired()));
        assert_eq!(
            WentWrong::KeyNotAccepted.checked(&paired()),
            Err(NotWhatFailed::NoKeyThere)
        );
    }

    /// **And this machine is not that place, since item 18b.** A service
    /// somebody runs on their own machine — vLLM started with `--api-key` — is
    /// `InferenceSource::ThisMachine` because nothing leaves, and it really can
    /// refuse a key. Reporting that as something else would send a person to
    /// look at a service that is working.
    ///
    /// The runtime alo OS ships is still never given a key, and what keeps that
    /// true is `alo_asking::locally`, which has no arm that can produce this
    /// reason at all.
    #[test]
    fn a_service_on_this_machine_can_refuse_a_key_and_say_so() {
        assert!(WentWrong::KeyNotAccepted.can_happen(&here()));
        assert_eq!(
            WentWrong::KeyNotAccepted.checked(&here()),
            Ok(WentWrong::KeyNotAccepted)
        );
        assert!(WentWrong::KeyNotAccepted.can_happen(&hosted()));
    }

    /// **Running out is refused where there is nothing to run out of**, which is
    /// the same shape as the key and a different sentence: a person told *the
    /// account has run out* about the workstation down the corridor would go
    /// looking for a bill nobody sends.
    #[test]
    fn an_account_cannot_have_run_out_where_nobody_has_one() {
        assert!(!WentWrong::RanOut.can_happen(&paired()));
        assert_eq!(
            WentWrong::RanOut.checked(&paired()),
            Err(NotWhatFailed::NoAccountThere)
        );
    }

    /// **And a service somebody put a budget on is not that place.** A gateway
    /// on this machine or a provider outside it can both answer that the money
    /// is gone, and reporting either as something else would send a person to
    /// look at a service that is working perfectly.
    #[test]
    fn somewhere_with_an_account_can_say_the_money_is_gone() {
        for source in [here(), hosted()] {
            assert!(WentWrong::RanOut.can_happen(&source), "{source:?}");
            assert_eq!(
                WentWrong::RanOut.checked(&source),
                Ok(WentWrong::RanOut),
                "{source:?}"
            );
        }
    }

    /// **Having no way there is a fact about this machine, and this machine
    /// can have no way to any address**, so it is refused nowhere — a paired
    /// machine on a link that went down, a provider from an office with no
    /// internet, and even this machine's own address on a machine whose
    /// loopback is down all read as the true thing that happened.
    #[test]
    fn having_no_way_there_can_happen_towards_any_place_at_all() {
        for source in [here(), paired(), hosted()] {
            assert!(WentWrong::NoWayThere.can_happen(&source), "{source:?}");
            assert_eq!(
                WentWrong::NoWayThere.checked(&source),
                Ok(WentWrong::NoWayThere),
                "{source:?}"
            );
        }
    }

    /// The two impossible pairings are two sentences, because they send whoever
    /// wrote the adapter to two different mistakes.
    #[test]
    fn the_refusal_tells_whoever_reported_it_what_to_do() {
        for refusal in [NotWhatFailed::NoKeyThere, NotWhatFailed::NoAccountThere] {
            let said = refusal.to_string();
            assert!(said.contains("report what actually went wrong"), "{said}");
        }
        assert_ne!(
            NotWhatFailed::NoKeyThere.to_string(),
            NotWhatFailed::NoAccountThere.to_string()
        );
    }

    /// Every reason has a string, and no two share one — otherwise two
    /// different failures would read identically.
    #[test]
    fn every_reason_says_something_of_its_own() {
        let every = [
            WentWrong::NothingAnswered,
            WentWrong::TookTooLong,
            WentWrong::NothingUsable,
            WentWrong::NoModelThere,
            WentWrong::KeyNotAccepted,
            WentWrong::SentSomewhereElse,
            WentWrong::HavingTrouble(503),
            WentWrong::RanOut,
            WentWrong::NoWayThere,
        ];
        let mut keys: Vec<String> = every.iter().map(|w| w.word().named().to_owned()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), every.len());
    }
}
