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
/// Seven, and an eighth belongs here only if it is a different thing to be
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
/// [`SentSomewhereElse`]: WentWrong::SentSomewhereElse
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
    /// It answered, and what it answered was that it was having trouble.
    ///
    /// Carries the status it answered with, which is an identifier and not a
    /// count.
    HavingTrouble(u16),
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
        }
    }

    /// Whether this is something that could have happened at this place.
    ///
    /// Everything can happen everywhere except being refused a key, which
    /// needs somewhere that was given one — and the one place in ADR 0008 that
    /// nothing here can give a key to is a machine on this network, because
    /// nothing here reaches one at all.
    #[must_use]
    pub fn can_happen(&self, source: &InferenceSource) -> bool {
        match self {
            Self::KeyNotAccepted => !matches!(source, InferenceSource::PairedMachine { .. }),
            Self::NothingAnswered
            | Self::TookTooLong
            | Self::NothingUsable
            | Self::NoModelThere
            | Self::SentSomewhereElse
            | Self::HavingTrouble(_) => true,
        }
    }

    /// The same question, answered as a refusal whoever reported the failure
    /// can act on.
    pub(crate) fn checked(self, source: &InferenceSource) -> Result<Self, NotWhatFailed> {
        if self.can_happen(source) {
            Ok(self)
        } else {
            Err(NotWhatFailed::NoKeyThere)
        }
    }
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

    /// The refusal is read by whoever wrote the adapter that reported the
    /// failure, so it says what to do about it rather than naming a state.
    #[test]
    fn the_refusal_tells_whoever_reported_it_what_to_do() {
        let said = NotWhatFailed::NoKeyThere.to_string();
        assert!(said.contains("report what actually went wrong"), "{said}");
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
        ];
        let mut keys: Vec<String> = every.iter().map(|w| w.word().named().to_owned()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), every.len());
    }
}
