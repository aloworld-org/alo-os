//! A machine this person is paired with, chosen to answer their questions.
//!
//! A place ADR 0008 names that a person could not pick until the local network
//! could carry a question: *a machine without a GPU discovers the one with it,
//! and the agents just work.* ADR 0003 settles what makes that allowed — a
//! pairing two people made, which permits asking that machine's models, and
//! which has not ended — and this file is where a person's **choice** is held
//! to it.
//!
//! # Chosen by identity, never by address or by name
//!
//! What is written down is the other machine's identity, which is what the
//! pairing names. An address is what discovery measures at the moment of asking
//! and is never typed or kept (ADR 0003: nothing typed is ever dialled); a name
//! is what the person here called it, which is the shell's to keep and can
//! change without the choice changing. The identity is kept **exactly as it was
//! written** and is not judged here: something that is not an identity is paired
//! with nothing, and is refused as exactly that.
//!
//! # This crate does not hold the pairings, and asks whoever does
//!
//! A settings store has no road off the machine and depends on nothing that has
//! one (`tests/a_grade_travels_nowhere.rs` refuses `alo-nearby` by name), so the
//! pairings — held by the daemon behind one lock — are not reachable from here.
//! What is here is the question put to them, [`WhoMayBeAsked`], answered by
//! whoever holds them. The daemon answers it with the very list that refuses the
//! next question from a revoked machine, so *at choosing* and *at every
//! question* are one answer asked twice.
//!
//! # Refused at choosing, and again at every question
//!
//! [`AMachine::permitted`] is the only door a surface has to this choice, and it
//! asks at the moment somebody picks: a machine that is merely on the network,
//! one whose pairing ended, one whose pairing was revoked and one whose pairing
//! permits something other than its models are all [`NotPairedToAnswer`], the
//! same fact about this machine. A pairing is not forever, though, and the file
//! outlives it — so a choice read back off the disk is **not** checked against
//! anything (a settings file is not wrong because an expiry passed overnight),
//! and whatever puts the question asks [`AMachine::still_permitted`] again at the
//! moment it does.
//!
//! # And which model answers there is not this person's to choose
//!
//! The machine down the corridor puts a question to the model **its** person
//! chose (`docs/contracts/local-network-wire.md`) and names that model in its
//! answer. So this choice carries no model, and the name a question down the
//! corridor is put with is [`WHAT_THAT_MACHINE_CHOSE`] — honest on a screen,
//! and set aside by the machine that reads it.

use std::time::SystemTime;

use alo_models::InferenceSource;

/// The model a question to a paired machine is put with.
///
/// Which model answers there is that machine's person's setting (ADR 0008), so
/// this names no model: it is what the question carries because a question
/// names *a* model, and what the other machine checks and does not use.
pub const WHAT_THAT_MACHINE_CHOSE: &str = "what-that-machine-chose";

/// Whoever holds this machine's pairings, asked whether one permits asking a
/// machine's models.
///
/// One method, answered at a moment the caller names. `alo-agentd` answers it
/// from the pairings it holds; a test answers it from a list of its own.
pub trait WhoMayBeAsked {
    /// Whether a pairing with the machine of this identity permits asking its
    /// models at `now`. Anything that is not an identity is paired with
    /// nothing.
    fn may_ask_the_models_of(&self, machine: &str, now: SystemTime) -> bool;
}

/// A machine this person is paired with, chosen to answer their questions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AMachine {
    /// Its identity, exactly as it was written.
    machine: String,
}

/// No pairing with this machine permits asking its models at the moment.
///
/// Not a refusal with words of its own, for [`crate::NoModel`]'s reason: the
/// sentence belongs to whoever has the rest of it — [`crate::NotWritten`] names
/// the settings file a choice was not written to, and whatever puts a question
/// says that nothing was sent.
///
/// One value for never paired, expired, revoked and paired for something else,
/// for `alo_asking::Miswired::NotPairedWithIt`'s reason: telling which kind of
/// not-paired a machine is would be telling somebody how to become paired.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotPairedToAnswer {
    /// The machine that was chosen, exactly as it was written.
    machine: String,
}

impl AMachine {
    /// This machine, chosen, if a pairing permits asking its models now.
    ///
    /// # Errors
    /// [`NotPairedToAnswer`] when no pairing permits asking that machine's
    /// models at `now`. Nothing is chosen.
    pub fn permitted(
        machine: &str,
        pairings: &dyn WhoMayBeAsked,
        now: SystemTime,
    ) -> Result<Self, NotPairedToAnswer> {
        let chosen = Self {
            machine: machine.to_owned(),
        };
        chosen.still_permitted(pairings, now)?;
        Ok(chosen)
    }

    /// The choice as a settings file holds it, checked against nothing.
    ///
    /// `pub(crate)`, and its one caller is the reader: a file is not refused
    /// because a pairing ended since it was written, and the question is
    /// refused instead, in words, by whoever asks [`AMachine::still_permitted`].
    pub(crate) const fn as_written(machine: String) -> Self {
        Self { machine }
    }

    /// Whether a pairing still permits what this choice needs.
    ///
    /// # Errors
    /// [`NotPairedToAnswer`] when no pairing permits asking that machine's
    /// models at `now` — revoked or expired since it was chosen.
    pub fn still_permitted(
        &self,
        pairings: &dyn WhoMayBeAsked,
        now: SystemTime,
    ) -> Result<(), NotPairedToAnswer> {
        if pairings.may_ask_the_models_of(&self.machine, now) {
            Ok(())
        } else {
            Err(NotPairedToAnswer {
                machine: self.machine.clone(),
            })
        }
    }

    /// The machine chosen, by its identity.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }

    /// Where a question put to this choice is answered, named by its identity.
    ///
    /// Whoever puts the question names it by what the person here called it
    /// where there is such a name; this crate keeps none, so it says what it
    /// has. Every rule an organisation can set decides on the kind of place
    /// alone, so the name changes nothing about what is permitted.
    #[must_use]
    pub fn source(&self) -> InferenceSource {
        InferenceSource::PairedMachine {
            machine: self.machine.clone(),
        }
    }
}

impl NotPairedToAnswer {
    /// The machine that was chosen, exactly as it was written.
    #[must_use]
    pub fn machine(&self) -> &str {
        &self.machine
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::testing::{PairedFor, a_moment, the_studio};

    /// **A machine whose pairing permits asking its models can be chosen**, and
    /// the choice says it is answered in the building rather than here.
    #[test]
    fn a_machine_paired_to_answer_can_be_chosen() {
        let chosen =
            AMachine::permitted(the_studio(), &PairedFor::a_day(the_studio()), a_moment()).unwrap();
        assert_eq!(chosen.machine(), the_studio());
        assert!(chosen.source().causes_egress());
        assert!(chosen.source().stays_in_the_building());
    }

    /// **Choosing is refused when no pairing permits asking its models**: a
    /// machine never paired with, a pairing with another machine, and a pairing
    /// that has ended are one refusal naming the machine chosen.
    #[test]
    fn a_machine_not_paired_to_answer_cannot_be_chosen() {
        let a_week_later = a_moment() + Duration::from_secs(7 * 86_400);
        for (pairings, now) in [
            (PairedFor::nothing(), a_moment()),
            (
                PairedFor::a_day("11112222333344445555666677778888"),
                a_moment(),
            ),
            (PairedFor::a_day(the_studio()), a_week_later),
        ] {
            let refused = AMachine::permitted(the_studio(), &pairings, now).unwrap_err();
            assert_eq!(refused.machine(), the_studio());
        }
    }

    /// **And a choice already made is asked again**: a pairing revoked after
    /// choosing refuses it at the next question.
    #[test]
    fn a_choice_whose_pairing_was_revoked_is_no_longer_permitted() {
        let chosen =
            AMachine::permitted(the_studio(), &PairedFor::a_day(the_studio()), a_moment()).unwrap();
        assert!(
            chosen
                .still_permitted(&PairedFor::nothing(), a_moment())
                .is_err()
        );
    }
}
