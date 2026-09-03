//! The door that does not leave: a question, and a model on this machine.
//!
//! [`crate::asking`] is four steps because something leaves; this is two,
//! because nothing does. The permission has to name this machine, and then the
//! runtime is asked. There is no indicator in this file, no
//! `alo_egress::Departing`, no `alo_egress::EgressPolicy` and no destination —
//! **not as an omission but as the shape of the path**, and
//! `alo_egress::Leaving::asking` refuses to make a departure for
//! `alo_models::InferenceSource::ThisMachine` if anybody ever tries.
//!
//! That is `docs/features.md`'s *a working day with a local model produces zero
//! inference egress* as far as a type can carry it: the door takes no
//! `alo_egress::Indicator`, so there is no line to show and no line that could
//! have been forgotten.
//!
//! # Nothing here can refuse it, and that is checked rather than assumed
//!
//! [`crate::Asking::to_a_provider`] asks `alo_models::SourcePolicy` a second
//! time at the moment the socket would open, because a rule tightened in
//! between is the rule that counts. This asks nothing, because there is no rule
//! that forbids a machine answering its own question — every `SourcePolicy`
//! permits `ThisMachine`, including `ThisMachineOnly`, and a refusal path that
//! cannot be reached is a branch nobody could test.
//!
//! It is a test rather than a sentence: `no_rule_can_stop_this_machine_
//! answering_its_own_question` walks every policy. A fifth variant that
//! forbade it would fail that test, which is where somebody would have to
//! decide what this door does about it.
//!
//! # ADR 0008 runs both ways, and this is the direction that was missing
//!
//! *A local model that fails does not quietly become an API call.* This is the
//! door where that could have been written, in one line, as a convenience — and
//! what it does instead is hand back `alo_answering::Failed`, whose only way
//! onward is an offer somebody answers. A machine that fell back would need no
//! new type here either: only a second call in this function.
//!
//! # An OpenAI-compatible service somebody pointed at loopback
//!
//! The question item 18a came with, and the answer is in two halves.
//!
//! **What it means**: it is this machine. `alo_models::Provider::source`
//! already says so for any loopback address, nothing leaves, law 1 shows
//! nothing, and an answer from it says *on this machine*. Which door a question
//! takes is decided by whether it leaves and by nothing else — not by what the
//! far end speaks.
//!
//! **What it is**: not the runtime. alo OS cannot list, fetch, load or remove
//! models on a service it did not install, so an
//! `alo_models::ModelRuntime` implementation for one would be four methods that
//! only refuse — a stub wearing an interface, which law 3 forbids. So this door
//! takes the runtime alo OS ships, and reaching a local service somebody else
//! runs is a second local shape and a queue item of its own. It is not built
//! here, and this file says so rather than leaving a person to find out.
//!
//! Loopback is taken at face value, here and in `alo-models`: a proxy listening
//! on `127.0.0.1` that forwards off the machine would be believed by every type
//! in this repository. `docs/quirks.md` records it, and the enforcement that
//! catches it is at the network boundary rather than in any of this.

use alo_answering::WentWrong;
use alo_models::{InferenceSource, ModelRuntime, RuntimeError};

use crate::answer::Answer;
use crate::asking::Asking;
use crate::question::Question;
use crate::refusing::{Miswired, NotAnswered};

impl Asking<'_> {
    /// Put the question to a model on this machine.
    ///
    /// Takes `self`, as [`Asking::to_a_provider`] does and for the same reason:
    /// an `alo_answering::Answering` means one attempt, and this is where that
    /// attempt happens.
    ///
    /// Answers with an [`Answer`] and nothing beside it. The other door hands
    /// back a departure as well, because `alo_record::Entry::left` can be made
    /// from one and from nothing else — there is no equivalent here, since
    /// `alo_record::Entry::answered_here` needs only the agent and the moment,
    /// both of which the caller already has. **Nothing this door could hand
    /// back would make that entry any more certain to be written**, and
    /// inventing a token to imply otherwise would be a guarantee in the shape
    /// of one rather than a guarantee.
    ///
    /// One permission is one attempt here as well:
    ///
    /// ```compile_fail
    /// use alo_answering::Answering;
    /// use alo_asking::{Asking, Question};
    /// use alo_capability::Grantee;
    /// use alo_models::{InferenceSource, ModelRuntime, SourcePolicy};
    ///
    /// # fn main() {
    /// fn ask(mail: &Grantee, runtime: &dyn ModelRuntime) {
    ///     let question = Question::asked("may the tenant sublet?", "a-model")
    ///         .expect("a question");
    ///     let answering = Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
    ///         .expect("nothing forbids answering here");
    ///     let asking = Asking::by(mail, answering, &[], &SourcePolicy::Anywhere);
    ///
    ///     let _once = asking.to_this_machine(&question, runtime);
    ///     let _twice = asking.to_this_machine(&question, runtime);
    /// }
    /// # }
    /// ```
    ///
    /// Checked by unmarking it, as every `compile_fail` in this workspace is:
    /// it fails with **E0382, use of moved value**, and not on a typo. The twin
    /// that passes is `one_permission_is_one_attempt` below.
    ///
    /// # Errors
    /// [`NotAnswered`], which is two things rather than four: law 1's two
    /// refusals do not exist on this path. [`crate::refusing`] has the table.
    pub fn to_this_machine(
        self,
        question: &Question,
        runtime: &dyn ModelRuntime,
    ) -> Result<Answer, NotAnswered> {
        let source = self.answering.source().clone();
        match &source {
            InferenceSource::ThisMachine => {}
            // The person chose a provider. Answering them from a model on this
            // machine would give them a different answer wearing the same face,
            // which is the half of ADR 0008 that was missing from it until
            // somebody pointed out that it read as though only one direction
            // mattered.
            InferenceSource::Hosted { .. } => return Err(Miswired::NotTheRuntime.into()),
            InferenceSource::PairedMachine { .. } => {
                return Err(Miswired::NoPathToAPairedMachine.into());
            }
        }

        // No policy is asked, no indicator is shown and no departure is made.
        // There is nothing here for any of the three to be about.
        match runtime.answers(question.text(), question.of()) {
            Ok(said) => Ok(Answer::new(said, source, question.of().to_owned())),
            Err(why) => {
                match self
                    .answering
                    .did_not_answer(what_went_wrong(why), self.others, self.policy)
                {
                    Ok(failed) => Err(NotAnswered::DidNotAnswer(Box::new(failed))),
                    // A reason that could not have happened where it is
                    // reported. This door cannot reach it — the only refusal
                    // `alo-answering` makes is about a key, and no key is ever
                    // sent here — but a reporter does not get to assume it
                    // passes its own reader's check.
                    Err(reported) => Err(reported.into()),
                }
            }
        }
    }
}

/// What a runtime failure is, said as the thing a person is told.
///
/// `alo_models::RuntimeError` is written for somebody managing models and
/// `alo_answering::WentWrong` for somebody who asked a question, so the mapping
/// is a translation between two audiences rather than a rename. It lives here
/// for [`crate::hosted`]'s reason: the crate that joins two decisions up is
/// where they are joined.
///
/// **The two download reasons cannot happen when a question is asked**, and
/// they are still mapped rather than left to a wildcard: a runtime that answered
/// a question with *there is not enough disk* has answered with something that
/// is not an answer, which is exactly what `NothingUsable` says.
fn what_went_wrong(why: RuntimeError) -> WentWrong {
    match why {
        RuntimeError::Unreachable => WentWrong::NothingAnswered,
        RuntimeError::TookTooLong => WentWrong::TookTooLong,
        // The weights are not there, or this runtime does not have that model.
        // One sentence to the person who asked: what was to answer this was not
        // there.
        RuntimeError::NotInstalled(_) | RuntimeError::NotOffered(_) => WentWrong::NoModelThere,
        RuntimeError::Unusable
        | RuntimeError::NotEnoughDisk { .. }
        | RuntimeError::DownloadIncomplete => WentWrong::NothingUsable,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{Stub, in_english, mistral_source, translated};
    use alo_answering::Answering;
    use alo_capability::Grantee;
    use alo_egress::Indicator;
    use alo_models::{Region, SourcePolicy};

    fn mail() -> Grantee {
        Grantee::named("@mail")
    }

    fn question() -> Question {
        Question::asked("may the tenant sublet?", "mistral-7b-instruct").unwrap()
    }

    fn here() -> Answering {
        Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere).unwrap()
    }

    /// The failure, if it was asked and did not answer.
    fn did_not_answer(not_answered: NotAnswered) -> Option<alo_answering::Failed> {
        match not_answered {
            NotAnswered::DidNotAnswer(failed) => Some(*failed),
            NotAnswered::Miswired(_) => None,
        }
    }

    /// The wiring mistake, if that is what this was.
    fn miswired(not_answered: NotAnswered) -> Option<Miswired> {
        match not_answered {
            NotAnswered::Miswired(miswired) => Some(miswired),
            NotAnswered::DidNotAnswer(_) => None,
        }
    }

    /// **The whole path, in order** — and the shape of it is the promise: the
    /// question goes to the runtime, the answer comes back knowing it came from
    /// this machine, and there is no indicator anywhere in the call.
    #[test]
    fn a_question_is_answered_here_and_the_answer_knows_it_never_left() {
        let runtime = Stub::answering("No, not without written consent.");
        let answer = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_this_machine(&question(), &runtime)
            .unwrap();

        assert_eq!(answer.text(), "No, not without written consent.");
        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert_eq!(answer.model(), "mistral-7b-instruct");
        assert_eq!(answer.came_from(&in_english()).text(), "on this machine");
        // The question and the model reached the runtime as they were written.
        assert_eq!(
            runtime.asked(),
            Some((
                "may the tenant sublet?".to_owned(),
                "mistral-7b-instruct".to_owned()
            ))
        );
    }

    /// **Zero inference egress, as far as a type can carry it.** An indicator
    /// held beside a whole local turn is quiet at the end of it, because this
    /// door has no parameter to be given one and no way to make a departure.
    #[test]
    fn a_working_day_on_this_machine_puts_nothing_on_the_indicator() {
        let indicator = Indicator::default();
        let runtime = Stub::answering("…");
        let mail = mail();
        for _ in 0..8 {
            let asking = Asking::by(&mail, here(), &[], &SourcePolicy::ThisMachineOnly);
            assert!(asking.to_this_machine(&question(), &runtime).is_ok());
        }
        assert!(indicator.is_quiet());
        assert_eq!(indicator.showing().len(), 0);
        // And law 1 refuses to make a departure for this place at all, which is
        // the guarantee underneath the absence of a parameter.
        assert!(alo_egress::Leaving::asking(&mail, &InferenceSource::ThisMachine).is_err());
    }

    /// **No rule can stop a machine answering its own question**, so this door
    /// asks none — and the list is walked rather than sampled, so a policy
    /// added later that did forbid it fails here instead of being permitted by
    /// a door that never asks.
    #[test]
    fn no_rule_can_stop_this_machine_answering_its_own_question() {
        let mail = mail();
        for policy in [
            SourcePolicy::Anywhere,
            SourcePolicy::InTheBuilding,
            SourcePolicy::InRegion("Switzerland".to_owned()),
            SourcePolicy::ThisMachineOnly,
        ] {
            assert!(policy.permits(&InferenceSource::ThisMachine), "{policy:?}");
            let runtime = Stub::answering("it answered");
            let asking = Asking::by(
                &mail,
                Answering::chosen(InferenceSource::ThisMachine, &policy).unwrap(),
                &[],
                &policy,
            );
            let answer = asking.to_this_machine(&question(), &runtime).unwrap();
            assert_eq!(answer.text(), "it answered", "{policy:?}");
        }
    }

    /// **Never a silent fallback, in the direction ADR 0008 was missing.** The
    /// machine has a provider it could ask and does not: the other place is an
    /// *offer* nobody has answered, and the person is told outright that
    /// nothing was sent anywhere.
    #[test]
    fn a_local_failure_asks_no_provider_and_the_person_is_told_so() {
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let others = [mistral_source()];
        let not_answered = Asking::by(&mail(), here(), &others, &SourcePolicy::Anywhere)
            .to_this_machine(&question(), &runtime)
            .unwrap_err();

        // Nothing was asked of the provider: the stub is the only thing that
        // was asked anything, and it was asked once.
        assert_eq!(runtime.times_asked(), 1);
        let failed = did_not_answer(not_answered).unwrap();
        assert_eq!(failed.why(), WentWrong::NothingAnswered);
        assert_eq!(failed.source(), &InferenceSource::ThisMachine);
        assert_eq!(failed.elsewhere().offers().len(), 1);
        assert_eq!(
            failed.nothing_was_sent(&in_english()).text(),
            "nothing was sent anywhere, and nothing will be unless you say so"
        );
        assert_eq!(
            failed.said(&in_english()).text(),
            "nothing answered on this machine"
        );
    }

    /// **Every way a runtime can fail reaches the person as what it means to
    /// them**, and the list is walked so that a reason added to `RuntimeError`
    /// later cannot quietly become whatever the last arm said.
    #[test]
    fn each_runtime_failure_reaches_a_person_as_the_thing_it_actually_means() {
        for (why, expected, reads) in [
            (
                RuntimeError::Unreachable,
                WentWrong::NothingAnswered,
                "nothing answered on this machine",
            ),
            (
                RuntimeError::TookTooLong,
                WentWrong::TookTooLong,
                "nothing answered on this machine within the time this machine waits",
            ),
            (
                RuntimeError::NotInstalled("mistral-7b-instruct".to_owned()),
                WentWrong::NoModelThere,
                "the model this question needed was not there to answer on this machine",
            ),
            (
                RuntimeError::NotOffered("something".to_owned()),
                WentWrong::NoModelThere,
                "the model this question needed was not there to answer on this machine",
            ),
            (
                RuntimeError::Unusable,
                WentWrong::NothingUsable,
                "but not with anything this machine could use",
            ),
            (
                RuntimeError::NotEnoughDisk {
                    needed_bytes: 5,
                    free_bytes: 1,
                },
                WentWrong::NothingUsable,
                "but not with anything this machine could use",
            ),
            (
                RuntimeError::DownloadIncomplete,
                WentWrong::NothingUsable,
                "but not with anything this machine could use",
            ),
        ] {
            assert_eq!(what_went_wrong(why.clone()), expected, "{why:?}");
            let runtime = Stub::failing(why.clone());
            let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
                .to_this_machine(&question(), &runtime)
                .unwrap_err();
            let failed = did_not_answer(not_answered).unwrap();
            assert_eq!(failed.why(), expected, "{why:?}");
            assert!(
                failed.said(&in_english()).text().contains(reads),
                "{why:?}: {}",
                failed.said(&in_english())
            );
        }
    }

    /// **A permission for somewhere else asks the runtime nothing**, and the
    /// refusal names the door that place is behind rather than offering this
    /// one as a substitute for it.
    #[test]
    fn a_permission_for_somewhere_else_asks_the_runtime_nothing() {
        for (permitted_place, expected) in [
            (mistral_source(), Miswired::NotTheRuntime),
            (
                InferenceSource::Hosted {
                    provider: "alo".to_owned(),
                    region: Region::Unknown,
                },
                Miswired::NotTheRuntime,
            ),
            (
                InferenceSource::PairedMachine {
                    machine: "the studio workstation".to_owned(),
                },
                Miswired::NoPathToAPairedMachine,
            ),
        ] {
            let runtime = Stub::answering("this should never be reached");
            let not_answered = Asking::by(
                &mail(),
                Answering::chosen(permitted_place.clone(), &SourcePolicy::Anywhere).unwrap(),
                &[],
                &SourcePolicy::Anywhere,
            )
            .to_this_machine(&question(), &runtime)
            .unwrap_err();

            assert_eq!(
                miswired(not_answered),
                Some(expected),
                "{permitted_place:?}"
            );
            assert_eq!(runtime.times_asked(), 0, "{permitted_place:?}");
        }
    }

    /// **The failure a person reads is in their own language**, and *on this
    /// machine* inside it is in that language too rather than an English clause
    /// in the middle of a German line.
    #[test]
    fn what_went_wrong_here_is_read_in_the_readers_own_language() {
        let strings = translated(&[
            (alo_models::words::ON_THIS_MACHINE, "auf diesem Rechner"),
            (
                alo_answering::words::NOTHING_ANSWERED,
                "{source} hat nicht geantwortet",
            ),
        ]);
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_this_machine(&question(), &runtime)
            .unwrap_err();
        let said = did_not_answer(not_answered).unwrap().said(&strings);
        assert!(said.is_translated(), "{said}");
        assert_eq!(said.text(), "auf diesem Rechner hat nicht geantwortet");
    }

    /// The passing twin of the `compile_fail` on this door: one permission is
    /// one attempt, and this is what spending it looks like.
    #[test]
    fn one_permission_is_one_attempt() {
        let runtime = Stub::failing(RuntimeError::Unusable);
        let not_answered = Asking::by(&mail(), here(), &[], &SourcePolicy::Anywhere)
            .to_this_machine(&question(), &runtime)
            .unwrap_err();
        assert_eq!(
            did_not_answer(not_answered).map(|failed| failed.why()),
            Some(WentWrong::NothingUsable)
        );
    }
}
