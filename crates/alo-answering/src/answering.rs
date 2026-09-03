//! The only type meaning *this question may be answered here*, and the two
//! doors into it.
//!
//! It is the same shape as `alo_capability::Authorised`,
//! `alo_files::Touching`, `alo_applications::Reaching` and
//! `alo_egress::Departing`: something a caller has to be holding before it may
//! act, which cannot be made without the question having been asked. What is
//! different is what it is about — those four are about an agent reaching this
//! machine, and this is about somebody's question reaching a model.
//!
//! # Two doors, and there is no third
//!
//! [`Answering::chosen`] is the place a person set their machine to use, and it
//! is refused where the rule their organisation set does not permit it —
//! `alo_models::Trying::under`'s decision, applied to the questions somebody
//! asks all day rather than to a provider being tested once.
//!
//! [`crate::Failed::take`] is the other, and it exists only after a person has
//! answered an offer. **There is no door from a failure to an `Answering`
//! without somebody saying yes**, and that absence is the whole of ADR 0008's
//! *never a silent fallback*: a machine that fell back would need no new type,
//! only a second call in the same function.
//!
//! # One attempt
//!
//! [`Answering::did_not_answer`] takes `self` and an `Answering` is not
//! `Clone`, so the thing that means *may be asked here* is spent by finding out
//! that it could not be. A retry at the same place is a new decision, made
//! where a person can see it, rather than a loop nobody wrote down.
//!
//! # What it does not carry
//!
//! **The question.** Not in this type, not anywhere in this crate. What is
//! decided here is *where*, and a crate that held *what* would be one entry
//! away from being a place somebody's questions accumulate — which is what
//! `alo-record` refuses in ADR 0001 §7 and `alo-context` refuses in §4.

use alo_models::{InferenceSource, NotAllowed, SourcePolicy};

use crate::elsewhere::Elsewhere;
use crate::failed::Failed;
use crate::wrong::{NotWhatFailed, WentWrong};

/// A question that may be put to this place.
///
/// Deliberately **not** `Clone` and deliberately not serialisable: one of these
/// means one attempt, and one read back off a disk would mean a machine that
/// asks a provider because of a file rather than because of a person.
#[derive(Debug, PartialEq, Eq)]
pub struct Answering {
    /// Where the question may go.
    source: InferenceSource,
}

impl Answering {
    /// The place a person chose, if the rule on this machine permits it.
    ///
    /// # Errors
    /// [`NotAllowed`], carrying the rule that refused and the place it refused,
    /// in whichever language the person turns out to read. Nothing is attempted
    /// and nothing is sent.
    pub fn chosen(source: InferenceSource, policy: &SourcePolicy) -> Result<Self, NotAllowed> {
        match policy.refusal(&source) {
            Some(refusal) => Err(refusal),
            None => Ok(Self { source }),
        }
    }

    /// The place a person approved an offer for.
    ///
    /// `pub(crate)`: the only caller is [`crate::Failed::take`], which is
    /// reached only from an offer the policy permitted and a person answered.
    pub(crate) fn new(source: InferenceSource) -> Self {
        Self { source }
    }

    /// Where the question may go.
    #[must_use]
    pub fn source(&self) -> &InferenceSource {
        &self.source
    }

    /// Whether putting the question here sends anything off this machine.
    ///
    /// True for a machine in the next room as well as a provider. Whatever is
    /// holding this and about to ask needs an `alo_egress::Departing` first,
    /// which is where law 1's indicator fires; this answers the question a
    /// caller asks *before* going to get one.
    #[must_use]
    pub fn causes_egress(&self) -> bool {
        self.source.causes_egress()
    }

    /// It did not answer. What the person may be told, and what they may be
    /// asked.
    ///
    /// `others` is every place this machine has, in the order the person set
    /// them up; the one that just failed is left out of the offers, and so is a
    /// place the rule forbids. `policy` is asked again here rather than
    /// remembered from [`chosen`](Self::chosen), because the rule that matters
    /// is the one in force now — the same reason `alo-capability` asks the
    /// grants at the moment of execution rather than caching the answer.
    ///
    /// Takes `self`, so one attempt is one attempt:
    ///
    /// ```compile_fail
    /// use alo_answering::{Answering, WentWrong};
    /// use alo_models::{InferenceSource, SourcePolicy};
    ///
    /// # fn main() {
    /// let here = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
    ///     .expect("nothing forbids answering here");
    /// let _first = here.did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere);
    /// let _again = here.did_not_answer(WentWrong::TookTooLong, &[], &SourcePolicy::Anywhere);
    /// # }
    /// ```
    ///
    /// # Errors
    /// [`NotWhatFailed`] when the reason could not have happened at this place
    /// — a key refused where no key is ever sent. It is a mistake in whatever
    /// reported the failure, and it is refused rather than shown to somebody.
    pub fn did_not_answer(
        self,
        why: WentWrong,
        others: &[InferenceSource],
        policy: &SourcePolicy,
    ) -> Result<Failed, NotWhatFailed> {
        let why = why.checked(&self.source)?;
        let elsewhere = Elsewhere::of(&self.source, others, policy);
        Ok(Failed::of(self.source, why, elsewhere))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{here, hosted, in_english, paired};

    /// A question goes only where the rule permits, and the refusal is the
    /// rule's own — so a machine cannot explain one rule two ways.
    #[test]
    fn a_question_is_only_ever_put_somewhere_the_rule_permits() {
        assert!(Answering::chosen(hosted(), &SourcePolicy::Anywhere).is_ok());

        let refused = Answering::chosen(hosted(), &SourcePolicy::InTheBuilding).unwrap_err();
        assert_eq!(
            Some(&refused),
            SourcePolicy::InTheBuilding.refusal(&hosted()).as_ref()
        );
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("keep questions in the building")
        );
    }

    /// The strictest rule there is still lets a machine answer its own
    /// questions — which is the working day law 1 is about.
    #[test]
    fn the_strictest_rule_still_lets_this_machine_answer_itself() {
        let asking = Answering::chosen(here(), &SourcePolicy::ThisMachineOnly).unwrap();
        assert!(!asking.causes_egress());
        assert_eq!(asking.source(), &here());
    }

    /// **The corridor is egress too**, so a question going to the machine in
    /// the next room says so before it goes.
    #[test]
    fn a_question_going_to_the_next_room_says_it_is_leaving() {
        let asking = Answering::chosen(paired(), &SourcePolicy::InTheBuilding).unwrap();
        assert!(asking.causes_egress());
    }

    /// **The rule is asked again when the question fails**, not remembered from
    /// when it was chosen. A rule that changed in between — an organisation
    /// tightening it, a machine being enrolled — decides what may be offered
    /// now.
    #[test]
    fn the_rule_in_force_now_decides_what_may_be_offered() {
        let asking = Answering::chosen(here(), &SourcePolicy::Anywhere).unwrap();
        let failed = asking
            .did_not_answer(
                WentWrong::NothingAnswered,
                &[hosted()],
                &SourcePolicy::InTheBuilding,
            )
            .unwrap();
        assert!(failed.elsewhere().is_nowhere());
        assert!(!failed.elsewhere().nothing_else());
    }

    /// The passing twin of the `compile_fail` above: one attempt is a program,
    /// and this is what it looks like — so the doctest cannot quietly become a
    /// test that a typo does not compile. Both `compile_fail`s in this crate
    /// were checked by unmarking them, and both fail with **E0382, use of moved
    /// value**.
    #[test]
    fn one_attempt_is_a_program() {
        let asking = Answering::chosen(here(), &SourcePolicy::Anywhere).unwrap();
        let failed = asking
            .did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere)
            .unwrap();
        assert_eq!(failed.source(), &here());
    }
}
