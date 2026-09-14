//! A question from a paired machine, answered by this machine's own model,
//! inside no turn of the asking machine's.
//!
//! *A machine without a GPU discovers the one with it, and the agents just
//! work.* `alo-asking`'s corridor is the asking side of that sentence, and
//! [`crate::Arriving`] says in its own header that a remote turn puts no
//! question: a remote turn is verbs on this machine's grants, and a question
//! is not a verb. So this is a door on the [`Machine`] — the one thing on this
//! machine that outlives every turn — and it is the door the daemon that owns
//! the port answers a proven, permitted question through.
//!
//! # This machine's own model, and nothing else
//!
//! The question is put to a runtime on this machine and to nothing else. There
//! is no parameter for a provider, no [`crate::Answers`] to hand in, and no
//! road from here to `alo_asking::Asking::to_a_provider`: the one door it
//! reaches is `alo_asking::Asking::to_this_machine`, which refuses a
//! permission for anywhere else as miswired before it asks anything. ADR 0008
//! runs both ways, and a machine whose person chose a provider is a machine
//! that answers *for nobody* — the daemon says so before this door is reached,
//! and this door has no way to disagree with it. Which model answers is the
//! **person's** choice on this machine, read where the daemon reads it for the
//! person's own questions; the model named in the question that arrived is
//! not consulted, because the asking machine does not choose what runs here.
//!
//! # What is written down, and when
//!
//! Before the caller hears anything, as everywhere in this crate:
//!
//! | What happened | What is written |
//! |---|---|
//! | The model answered | `answered for another machine`, with the origin named ([`alo_record::Entry::answered_for`]) |
//! | The answer went back | `left`, stamped with the origin, from the departure the indicator made |
//! | The rule refused the answer going back | `held back`, in the rule's own words, stamped with the origin |
//! | The model did not answer | nothing: [`crate::unanswered`] has the argument, and it is the same here |
//!
//! **What was asked is not written and there is nowhere for it to go**, which
//! is [`alo_record::Happened::AnsweredForAnotherMachine`]'s own rule.
//!
//! # And the answer leaves under this machine's indicator
//!
//! An answer to a question from the machine down the corridor is this
//! machine's words leaving it, and law 1 is not suspended for the length of a
//! corridor. [`Machine::answering_for`] asks this machine's egress rule about
//! the answer going back to the origin machine and puts it on this machine's
//! indicator, exactly as [`crate::Arriving::departing`] does for an answer to
//! a verb; what it hands back holds the [`Departing`] that is the only thing
//! meaning *this may leave*, and the daemon has no road to its socket without
//! one. [`Machine::answer_returned`] is what it is spent on once the bytes
//! have gone: the departure is written before the line comes off, and the
//! line comes off whatever the record said, because the indicator is a
//! statement about now.
//!
//! # And while a remote turn holds the machine
//!
//! A question from a paired machine can arrive while another paired machine's
//! turn — or the same machine's — holds this machine for its verbs. The
//! record and the indicator are behind that turn, so [`Arriving`] has the same
//! two doors, forwarding to the machine it holds. The question is still put
//! inside no turn: nothing about the turn is asked, nothing on it is spent,
//! and the entries are the machine's own rather than stamped with that turn's
//! origin, because the machine whose turn it is caused nothing about them.

use std::time::SystemTime;

use alo_answering::Answering;
use alo_asking::{Answer, Asking, NotAnswered, Question};
use alo_capability::Grantee;
use alo_egress::{Departing, Destination, EgressPolicy, Leaving, Why};
use alo_models::{ModelRuntime, SourcePolicy};
use alo_nearby::Origin;
use alo_record::Entry;

use crate::arriving::Arriving;
use crate::machine::Machine;
use crate::unanswered::NoAnswer;

/// An answer for a paired machine, and the departure it leaves under.
///
/// Made only by [`Machine::answering_for`], once the model has answered and
/// the indicator shows the answer leaving. Holding one is evidence of both.
#[derive(Debug)]
pub struct AnsweredFor {
    /// What the model said, and which model.
    answer: Answer,
    /// The answer going back, on this machine's indicator.
    departing: Departing,
}

impl AnsweredFor {
    /// What the model said.
    #[must_use]
    pub const fn answer(&self) -> &Answer {
        &self.answer
    }

    /// The answer going back, as the indicator shows it.
    #[must_use]
    pub const fn departing(&self) -> &Departing {
        &self.departing
    }

    /// The answer, and the departure to spend on [`Machine::answer_returned`]
    /// once the bytes have gone.
    #[must_use]
    pub fn into_parts(self) -> (Answer, Departing) {
        (self.answer, self.departing)
    }
}

impl Machine<'_> {
    /// Put a question from `origin` to this machine's own model, and put the
    /// answer going back on the indicator.
    ///
    /// `question` is what arrived, already read as one and already naming
    /// the model the person here chose (`alo_asking::a_question_off_the_wire`
    /// is the one reader, and it is handed that model); `answering` is the
    /// permission for this machine, spent here; `runtime` is what answers;
    /// `policy` is the rule in force, asked once about the answer leaving.
    /// `origin` was made by [`alo_nearby::Origin::proven`] and by nothing
    /// else, so a question cannot be answered here for a machine this one is
    /// not paired with.
    ///
    /// # Errors
    ///
    /// [`NoAnswer`], as [`crate::Turning::asking`] answers it, less the arms
    /// no local model can reach: [`NoAnswer::Miswired`] when the permission
    /// is for somewhere other than this machine (nothing was asked),
    /// [`NoAnswer::DidNotAnswer`] when the model did not (nothing written),
    /// [`NoAnswer::CannotBeShown`] when the origin's name cannot be put on
    /// the indicator (nothing left, nothing written), [`NoAnswer::HeldBack`]
    /// when the rule refuses the answer going back (written down as such),
    /// and [`NoAnswer::NotRecorded`] with `after_it_left` false when either
    /// entry could not be written.
    pub fn answering_for(
        &mut self,
        origin: &Origin,
        question: &Question,
        answering: Answering,
        runtime: &dyn ModelRuntime,
        policy: &SourcePolicy,
        now: SystemTime,
    ) -> Result<AnsweredFor, NoAnswer> {
        let grantee = Grantee::named(&origin.principal());
        // No other place is offered: an offer is a sentence for a person, and
        // the person on the machine that asked is not this machine's to ask.
        let asking = Asking::by(&grantee, answering, &[], policy);
        let answer = match asking.to_this_machine(question, runtime) {
            Ok(answer) => answer,
            Err(NotAnswered::DidNotAnswer(failed)) => return Err(NoAnswer::DidNotAnswer(failed)),
            Err(NotAnswered::Miswired(why)) => return Err(why.into()),
        };
        self.kept()
            .keep(Entry::answered_for(origin.called(), now))
            .map_err(nothing_left)?;

        let destination = Destination::paired(origin.called()).map_err(NoAnswer::CannotBeShown)?;
        let leaving = Leaving::because(&grantee, Why::Sending, destination);
        match self
            .indicator()
            .beginning(&EgressPolicy::from(policy), leaving, now)
        {
            Ok(departing) => Ok(AnsweredFor { answer, departing }),
            Err(refused) => {
                let entry = Entry::held_back(&refused, self.strings(), now)
                    .from_another_machine(origin.called());
                match self.kept().keep(entry) {
                    Ok(()) => Err(NoAnswer::HeldBack(refused)),
                    Err(why) => Err(nothing_left(why)),
                }
            }
        }
    }

    /// The answer has gone back to `origin`: write the departure down, and
    /// take the line off the indicator.
    ///
    /// In that order, as [`crate::Arriving::returned`] keeps it, and the line
    /// comes off whatever the record said.
    ///
    /// # Errors
    ///
    /// [`NoAnswer::NotRecorded`] with `after_it_left` true: the answer went
    /// back and there is no evidence of it, which is law 1's second half
    /// failing, and the service that holds this machine stops for it.
    pub fn answer_returned(
        &mut self,
        origin: &Origin,
        departing: Departing,
    ) -> Result<(), NoAnswer> {
        let kept = self
            .kept()
            .keep(Entry::left(&departing).from_another_machine(origin.called()));
        self.indicator().ended(departing);
        kept.map_err(|why| NoAnswer::NotRecorded {
            why,
            after_it_left: true,
        })
    }
}

impl Arriving<'_, '_> {
    /// [`Machine::answering_for`], while this remote turn holds the machine.
    ///
    /// Nothing about this turn is asked or spent: the question is inside no
    /// turn, and the entries are the machine's own.
    ///
    /// # Errors
    ///
    /// As [`Machine::answering_for`].
    pub fn answering_for(
        &mut self,
        origin: &Origin,
        question: &Question,
        answering: Answering,
        runtime: &dyn ModelRuntime,
        policy: &SourcePolicy,
        now: SystemTime,
    ) -> Result<AnsweredFor, NoAnswer> {
        self.machine()
            .answering_for(origin, question, answering, runtime, policy, now)
    }

    /// [`Machine::answer_returned`], while this remote turn holds the machine.
    ///
    /// # Errors
    ///
    /// As [`Machine::answer_returned`].
    pub fn answer_returned(
        &mut self,
        origin: &Origin,
        departing: Departing,
    ) -> Result<(), NoAnswer> {
        self.machine().answer_returned(origin, departing)
    }
}

/// The record broke while nothing had gone anywhere.
fn nothing_left(why: alo_keeping::NotKept) -> NoAnswer {
    NoAnswer::NotRecorded {
        why,
        after_it_left: false,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_answering::Answering;
    use alo_asking::Question;
    use alo_capability::Grants;
    use alo_egress::{Destination, Indicator, Why};
    use alo_files::OnThisMachine;
    use alo_models::{InferenceSource, RuntimeError, SourcePolicy};
    use alo_nearby::{
        Deliberating, Keying, MachineId, MayAskIts, Origin, Pairings, Proof, Proposal, Seen, Side,
    };
    use alo_record::{Asking as AskingAbout, Happened, Only, Record};

    use super::AnsweredFor;
    use crate::arriving::Arriving;
    use crate::machine::Machine;
    use crate::testing::{NothingIsBounded, Stub, hour, in_english, noon};
    use crate::unanswered::NoAnswer;

    /// The machine down the corridor, which asked.
    fn the_reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// This machine, which answers.
    fn here() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// The reception machine, proven at this machine's door.
    fn origin() -> Origin {
        let at_reception = Keying::fresh().unwrap();
        let proposal = Proposal::checked(
            the_reception(),
            here(),
            &[MayAskIts::Models],
            hour(),
            at_reception.offer().clone(),
        )
        .unwrap();
        let on_here = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
        let on_reception = Deliberating::asking(proposal, at_reception)
            .unwrap()
            .answered_with(on_here.answered().unwrap().clone())
            .unwrap();
        let on_reception = on_reception
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(noon())
            .unwrap();
        let on_here = on_here
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(noon())
            .unwrap();
        let mut pairings = Pairings::none();
        pairings.keep(on_here);
        let proof = Proof::made(&on_reception, &the_reception(), b"a question", noon());
        Origin::proven(
            &pairings,
            &here(),
            &proof,
            b"a question",
            "the reception machine",
            noon(),
            &mut Seen::nothing(),
        )
        .unwrap()
    }

    /// What is asked, in every one of these.
    fn sublet() -> Question {
        Question::asked("may the tenant sublet?", "a-model").unwrap()
    }

    /// The permission to answer on this machine, which every rule grants.
    fn here_permitted() -> Answering {
        Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere).unwrap()
    }

    /// This machine, with this record and indicator, doing what `doing` says.
    fn on_this_machine<T>(
        record: &mut Record,
        indicator: &mut Indicator,
        doing: impl FnOnce(&mut Machine<'_>) -> T,
    ) -> T {
        let strings = in_english();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            indicator,
            record,
        )
        .unwrap();
        doing(&mut machine)
    }

    /// How many entries there are of this kind.
    fn how_many(record: &Record, only: Only) -> usize {
        record
            .answering(&AskingAbout::anything().only(only))
            .count()
    }

    /// **The whole road.** The question is put to this machine's runtime, the
    /// answer knows it came from here, the record says a question was answered
    /// for the reception machine, the answer going back is shown leaving to
    /// the reception machine under its principal, and once returned it is
    /// written down as left with the origin named and the line is off.
    #[test]
    fn a_question_from_a_paired_machine_is_answered_here_and_its_answer_leaves_under_a_departure() {
        let runtime = Stub::answering("No, not without written consent.");
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let origin = origin();
        on_this_machine(&mut record, &mut indicator, |machine| {
            let answered = machine
                .answering_for(
                    &origin,
                    &sublet(),
                    here_permitted(),
                    &runtime,
                    &SourcePolicy::Anywhere,
                    noon(),
                )
                .unwrap();
            assert_eq!(answered.answer().text(), "No, not without written consent.");
            assert_eq!(answered.answer().source(), &InferenceSource::ThisMachine);
            assert_eq!(answered.answer().model(), "a-model");
            assert_eq!(
                runtime.asked(),
                Some(("may the tenant sublet?".to_owned(), "a-model".to_owned()))
            );
            assert_eq!(answered.departing().why(), Why::Sending);
            assert_eq!(
                answered.departing().destination(),
                &Destination::paired("the reception machine").unwrap()
            );
            assert_eq!(answered.departing().agent().as_str(), origin.principal());
            assert_eq!(machine.showing().showing().len(), 1);

            let (_, departing) = answered.into_parts();
            machine.answer_returned(&origin, departing).unwrap();
            assert!(machine.showing().is_quiet());
        });

        assert_eq!(record.len(), 2, "{record:?}");
        let mut entries = record.everything();
        let answered = entries.next().unwrap();
        assert!(
            matches!(
                answered.happened(),
                Happened::AnsweredForAnotherMachine { .. }
            ),
            "{answered:?}"
        );
        assert!(
            answered
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        );
        let left = entries.next().unwrap();
        assert!(left.happened().caused_egress(), "{left:?}");
        assert!(
            left.origin()
                .is_some_and(|from| from.is("the reception machine"))
        );
        assert_eq!(how_many(&record, Only::Egress), 1);
    }

    /// **A model that does not answer leaves nothing**: no entry, nothing on
    /// the indicator, and the failure handed back whole for the daemon to word
    /// on the wire.
    #[test]
    fn a_model_that_does_not_answer_writes_nothing_and_shows_nothing() {
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        on_this_machine(&mut record, &mut indicator, |machine| {
            let why = machine
                .answering_for(
                    &origin(),
                    &sublet(),
                    here_permitted(),
                    &runtime,
                    &SourcePolicy::Anywhere,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(why, NoAnswer::DidNotAnswer(_)), "{why:?}");
            assert!(why.nothing_left());
            assert!(machine.showing().is_quiet());
        });
        assert!(record.is_empty(), "{record:?}");
    }

    /// **A permission for anywhere but this machine asks the runtime
    /// nothing.** There is no road from this door to a provider or to another
    /// paired machine: the permission is refused as miswired before anything
    /// is asked, and nothing is written.
    #[test]
    fn a_permission_for_anywhere_else_asks_this_machines_runtime_nothing() {
        let runtime = Stub::answering("this should never be reached");
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        for elsewhere in [
            InferenceSource::Hosted {
                provider: "Mistral".to_owned(),
                region: alo_models::Region::Declared("the EU".to_owned()),
            },
            InferenceSource::PairedMachine {
                machine: "a third machine".to_owned(),
            },
        ] {
            on_this_machine(&mut record, &mut indicator, |machine| {
                let why = machine
                    .answering_for(
                        &origin(),
                        &sublet(),
                        Answering::chosen(elsewhere.clone(), &SourcePolicy::Anywhere).unwrap(),
                        &runtime,
                        &SourcePolicy::Anywhere,
                        noon(),
                    )
                    .unwrap_err();
                assert!(matches!(why, NoAnswer::Miswired(_)), "{why:?}");
            });
        }
        assert_eq!(runtime.times_asked(), 0);
        assert!(record.is_empty());
        assert!(indicator.is_quiet());
    }

    /// **A rule that says nothing leaves holds the answer back**: the model
    /// answered and that is written down, the held-back departure is written
    /// down with the origin named, and nothing is on the indicator.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_answer_back_and_writes_it_down() {
        let runtime = Stub::answering("No, not without written consent.");
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        on_this_machine(&mut record, &mut indicator, |machine| {
            let why = machine
                .answering_for(
                    &origin(),
                    &sublet(),
                    Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::ThisMachineOnly)
                        .unwrap(),
                    &runtime,
                    &SourcePolicy::ThisMachineOnly,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(why, NoAnswer::HeldBack(_)), "{why:?}");
            assert!(why.nothing_left());
            assert!(machine.showing().is_quiet());
        });
        assert_eq!(record.len(), 2, "{record:?}");
        assert_eq!(how_many(&record, Only::Egress), 0);
        assert!(
            record
                .everything()
                .any(|entry| matches!(entry.happened(), Happened::HeldBack { .. }))
        );
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        }));
    }

    /// **The same door while a remote turn holds the machine**: the question
    /// is answered through the turn, nothing on the turn is spent — no change
    /// waits, no grant moves — and the entries are the machine's own rather
    /// than stamped with the turn's origin.
    #[test]
    fn a_question_is_answered_through_a_remote_turn_without_touching_the_turn() {
        let runtime = Stub::answering("Three are unpaid.");
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let origin = origin();
        on_this_machine(&mut record, &mut indicator, |machine| {
            let mut grants = Grants::default();
            let mut arriving =
                Arriving::beginning(&origin, hour(), noon(), &mut grants, machine).unwrap();
            let answered: AnsweredFor = arriving
                .answering_for(
                    &origin,
                    &Question::asked("how many are unpaid?", "a-model").unwrap(),
                    here_permitted(),
                    &runtime,
                    &SourcePolicy::Anywhere,
                    noon() + Duration::from_secs(1),
                )
                .unwrap();
            let (answer, departing) = answered.into_parts();
            assert_eq!(answer.text(), "Three are unpaid.");
            arriving.answer_returned(&origin, departing).unwrap();
            assert_eq!(arriving.waiting_at(noon()).count(), 0);
            assert!(!arriving.is_closed());
            assert!(grants.is_empty());
            arriving.ending(&mut grants);
        });
        assert_eq!(record.len(), 2, "{record:?}");
        assert!(record.everything().all(|entry| {
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        }));
        assert!(indicator.is_quiet());
    }
}
