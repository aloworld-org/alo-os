//! The door that reaches off this machine: a question, and the model that
//! answers it.
//!
//! Every other door in [`crate::turning`] happens on this machine. This is the
//! one that may not, and it is cut from the rest on the line law 1 draws.
//!
//! # A turn does not decide where a question goes
//!
//! Not one step of it. The place is the person's setting, and it arrives twice
//! over: as an `alo_answering::Answering`, which is the only type meaning *this
//! question may be answered here*, and as a [`crate::Answers`], which is the
//! thing that actually answers there. A turn spends the first on the second and
//! adds three things around them — the order, the indicator, and the record.
//!
//! **That is why the permission comes in rather than being made here.** A turn
//! that built its own would ask `alo_models::SourcePolicy` about the place
//! before the egress rule ever saw it, and the two refusals are not
//! interchangeable: only the second makes an `alo_egress::NotPermitted`, and
//! `alo_record::Entry::held_back` is made from one of those and from nothing
//! else (item 5a). A machine whose rule was tightened between the person
//! choosing and the agent asking would then have refused the question with no
//! record of having refused it. So the rule is asked where its refusal can be
//! written down, which is inside `alo-asking`, at the moment the socket would
//! open.
//!
//! It is not law 2's *there is no door that takes a call* being weakened
//! either. That rule is about what an **agent** may ask for, and an agent does
//! not choose where its question is answered — ADR 0008 says the person does.
//! A permission arriving here is the person's decision arriving, in the way the
//! grants arrive at every other door.
//!
//! # What is written down, and when
//!
//! Before the caller is told anything, as everywhere else in this crate:
//!
//! | Where it went | What is written |
//! |---|---|
//! | A provider, answered | `left` — the departure, from the token the door hands back |
//! | A provider, no reply | `left` — it went, and a record of only the answered ones would report a quieter day than the machine had |
//! | A provider, refused by the rule | `held back`, in the rule's own words |
//! | This machine, answered | `answered here` — who asked, and nothing else |
//! | This machine, no reply | nothing: `crate::unanswered` has the four absences and the argument for each |
//!
//! # The line comes off the indicator whatever happens
//!
//! Including when the record cannot be written. The indicator is a statement
//! about **now** — a connection that has ended is not leaving — so leaving a
//! line up because a disk was full would make law 1's surface wrong in order to
//! signal that the record is. What breaks instead is the turn: it closes, every
//! door afterwards refuses, and the answer says whether the question had
//! already left, because *the record broke* and *somebody's question went to a
//! provider and there is no evidence of it* are two different mornings for
//! whoever reads the machine.

use std::time::SystemTime;

use alo_answering::Answering;
use alo_asking::{Answer, Asked, Asking, DidNotAnswer, NotAnswered, NotAsked, Question};
use alo_capability::Grantee;
use alo_keeping::NotKept;
use alo_record::Entry;

use crate::answers::Answers;
use crate::places::Places;
use crate::turning::Turning;
use crate::unanswered::NoAnswer;

impl Turning<'_, '_> {
    /// Put a question to the place the person's machine is set to use.
    ///
    /// The question is made here rather than handed in, for the reason the call
    /// is: what arrives is what somebody typed and the name of a model, and
    /// `alo_asking::Question::asked` is the one thing that decides whether
    /// those are a question. Nothing of either is kept anywhere — not by this
    /// crate, not by the record.
    ///
    /// `answering` is the person's setting, spent by being used, and `answers`
    /// is what actually replies there. When they name different places the door
    /// refuses it as [`NoAnswer::Miswired`]; nothing is sent, and it is a fault
    /// in whatever wired them rather than a thing that happened to the person.
    ///
    /// **The answer to a taken offer comes back in here.** A question that was
    /// not answered hands back an `alo_answering::Failed`, and the only way on
    /// from one is an offer a person answered, which yields another
    /// `Answering` — so a second attempt is this same door, called again, shown
    /// and written down exactly as the first was. There is no shorter road, and
    /// that absence is ADR 0008's *never a silent fallback*.
    ///
    /// **One permission is one attempt**, at this door as at `alo-asking`'s.
    /// The permission is taken by value, so asking twice with one setting is
    /// not a program that compiles:
    ///
    /// ```compile_fail
    /// use alo_answering::Answering;
    /// use alo_capability::Grants;
    /// use alo_context::Context;
    /// use alo_egress::Indicator;
    /// use alo_files::OnThisMachine;
    /// use alo_models::{InferenceSource, ModelRuntime, SourcePolicy};
    /// use alo_record::Record;
    /// use alo_strings::{Strings, Vocabulary};
    /// use alo_turn::{Answers, Machine, Places, Turning};
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() {
    /// fn ask_twice(runtime: &dyn ModelRuntime) {
    ///     let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    ///     let strings = Strings::of(Vocabulary::empty());
    ///     let mut indicator = Indicator::default();
    ///     let mut record = Record::default();
    ///     let mut machine = Machine::carrying_out_file_verbs(
    ///         &strings,
    ///         &OnThisMachine,
    ///         &mut indicator,
    ///         &mut record,
    ///     )
    ///     .unwrap();
    ///     let mut grants = Grants::default();
    ///     let mut turning = Turning::beginning(
    ///         Context::at_invocation(now),
    ///         "@mail",
    ///         Duration::from_secs(300),
    ///         &mut grants,
    ///         &mut machine,
    ///     )
    ///     .unwrap();
    ///
    ///     let policy = SourcePolicy::Anywhere;
    ///     let permitted =
    ///         Answering::chosen(InferenceSource::ThisMachine, &policy).unwrap();
    ///     let here = Answers::Runtime(runtime);
    ///     let places = Places::under(&policy);
    ///
    ///     let _once = turning.asking("is it?", "a-model", permitted, &here, &places, now);
    ///     let _twice = turning.asking("is it?", "a-model", permitted, &here, &places, now);
    /// }
    /// # }
    /// ```
    ///
    /// Checked by unmarking it, as every `compile_fail` in this workspace is:
    /// it fails with **E0382, use of moved value**, and not on a typo. The twin
    /// that passes is `an_offer_a_person_took_is_asked_through_the_same_door`
    /// below, which is what a second attempt really looks like — two
    /// permissions, and a person between them.
    ///
    /// # Errors
    /// [`NoAnswer`], which is seven different things to do rather than one
    /// sentence — [`crate::unanswered`] has the table, and says which of them
    /// leave a record.
    pub fn asking(
        &mut self,
        asked: &str,
        of_model: &str,
        answering: Answering,
        answers: &Answers<'_>,
        places: &Places<'_>,
        now: SystemTime,
    ) -> Result<Answer, NoAnswer> {
        if self.is_closed() {
            return Err(NoAnswer::TurnClosed);
        }
        let question = Question::asked(asked, of_model)?;
        // Cloned rather than borrowed: the indicator and the record are reached
        // through the same machine this borrows from, and an agent's name is a
        // short string beside a question that is about to cross a network.
        let agent = self.grantee().clone();
        let asking = Asking::by(&agent, answering, places.everywhere_else(), places.policy());

        match answers {
            Answers::Provider(hosted) => {
                let outcome =
                    asking.to_a_provider(&question, hosted, self.machine().indicator(), now);
                self.what_a_provider_did(outcome, now)
            }
            Answers::Runtime(runtime) => {
                let outcome = asking.to_this_machine(&question, *runtime);
                self.what_this_machine_did(outcome, &agent, now)
            }
            Answers::Service(served) => {
                let outcome = asking.to_a_service_on_this_machine(&question, served);
                self.what_this_machine_did(outcome, &agent, now)
            }
        }
    }

    /// What came back from a provider, written down and then handed over.
    ///
    /// Both roads where something left end with the departure written before
    /// the line comes off the indicator and before the caller hears anything:
    /// `alo_record::Entry::left` is made from the token the door hands back and
    /// from nothing else, which is what makes an egress the indicator showed an
    /// entry that can be written.
    fn what_a_provider_did(
        &mut self,
        outcome: Result<Asked, NotAsked>,
        now: SystemTime,
    ) -> Result<Answer, NoAnswer> {
        match outcome {
            Ok(asked) => {
                let kept = self.keeping(Entry::left(asked.departing()));
                let answer = asked.ended(self.machine().indicator());
                match kept {
                    Ok(()) => Ok(answer),
                    Err(why) => Err(after_it_left(why)),
                }
            }
            Err(NotAsked::DidNotAnswer(unanswered)) => self.what_a_provider_did_not_do(*unanswered),
            Err(NotAsked::HeldBack(refused)) => {
                let strings = self.machine().strings();
                let entry = Entry::held_back(&refused, strings, now);
                match self.keeping(entry) {
                    Ok(()) => Err(NoAnswer::HeldBack(refused)),
                    Err(why) => Err(nothing_left(why)),
                }
            }
            // Neither of these addressed anything or sent anything, and
            // `crate::unanswered` has the argument for why neither is an entry.
            Err(NotAsked::CannotBeShown(why)) => Err(NoAnswer::CannotBeShown(why)),
            Err(NotAsked::Miswired(why)) => Err(why.into()),
        }
    }

    /// It went to a provider and nothing came back.
    ///
    /// A road of its own because it is the one a record of successes would
    /// lose: the question left, so law 1's *what left this machine today* has
    /// to find it, and the failure the person is shown is handed on afterwards
    /// without this crate reading a word of it.
    fn what_a_provider_did_not_do(&mut self, unanswered: DidNotAnswer) -> Result<Answer, NoAnswer> {
        let kept = self.keeping(Entry::left(unanswered.departing()));
        let failed = unanswered.ended(self.machine().indicator());
        match kept {
            Ok(()) => Err(NoAnswer::DidNotAnswer(Box::new(failed))),
            Err(why) => Err(after_it_left(why)),
        }
    }

    /// What came back from the runtime or from a service running here.
    ///
    /// One road for both, because what is written down is the same: an answer
    /// from this machine is `alo_record::Entry::answered_here`, which names who
    /// asked and has nowhere to name anything else. There is no departure on
    /// either, so there is nothing to end and nothing to take off the
    /// indicator.
    fn what_this_machine_did(
        &mut self,
        outcome: Result<Answer, NotAnswered>,
        agent: &Grantee,
        now: SystemTime,
    ) -> Result<Answer, NoAnswer> {
        match outcome {
            Ok(answer) => match self.keeping(Entry::answered_here(agent, now)) {
                Ok(()) => Ok(answer),
                Err(why) => Err(nothing_left(why)),
            },
            // Nothing left and nothing answered, so there is nothing an entry
            // could truthfully say. `alo-answering` settled it and
            // `crate::unanswered` quotes the reasoning.
            Err(NotAnswered::DidNotAnswer(failed)) => Err(NoAnswer::DidNotAnswer(failed)),
            Err(NotAnswered::Miswired(why)) => Err(why.into()),
        }
    }
}

/// The record broke while nothing had gone anywhere.
fn nothing_left(why: NotKept) -> NoAnswer {
    NoAnswer::NotRecorded {
        why,
        after_it_left: false,
    }
}

/// The record broke after somebody's question had already left the machine.
fn after_it_left(why: NotKept) -> NoAnswer {
    NoAnswer::NotRecorded {
        why,
        after_it_left: true,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::kept::Kept;
    use crate::machine::Machine;
    use crate::testing::{
        AN_ANSWER, Stub, a_provider, a_service, far_away, files, hour, in_english, mistral_source,
        noon, permitting, serving,
    };
    use alo_asking::{Hosted, Served};
    use alo_capability::Grants;
    use alo_context::Context;
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_keeping::NotKept;
    use alo_models::{InferenceSource, RuntimeError, SourcePolicy};
    use alo_record::{Asking as AskingAbout, Only, Record};

    /// What was asked, in every one of these.
    const SUBLET: &str = "may the tenant sublet?";

    /// A record that cannot be written to, which is what a full disk looks like
    /// from inside a turn.
    #[derive(Default)]
    struct ANoSpaceLeftDisk {
        /// How many entries it was handed and refused.
        refused: usize,
    }

    impl Kept for ANoSpaceLeftDisk {
        fn keep(&mut self, _entry: Entry) -> Result<(), NotKept> {
            self.refused += 1;
            Err(NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    /// Nothing here is shortened: these tests are about what a question leaves
    /// behind when the record cannot be written to.
    impl crate::Shortening for ANoSpaceLeftDisk {
        fn shorten(
            &mut self,
            _keeping: alo_keeping::Keeping,
            _now: std::time::SystemTime,
        ) -> Result<crate::Shortened, NotKept> {
            Ok(crate::Shortened::NotOnADisk)
        }
    }

    /// One turn on a machine with this record and this indicator, ending when
    /// the closure is done.
    ///
    /// Written as a closure for `turning.rs`' reason: a `Turning` borrows the
    /// `Machine` and the machine borrows the indicator and the record, so all
    /// of them have to live in one frame.
    fn on_a_machine<T>(
        kept: &mut dyn crate::Shortening,
        indicator: &mut Indicator,
        doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
    ) -> T {
        let strings = in_english();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, indicator, kept).unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let outcome = doing(&mut turning);
        // A question needs no grant, so the turn made none and takes none back.
        assert!(!turning.ending(&mut grants));
        outcome
    }

    /// How many entries there are of this kind.
    fn how_many(record: &Record, only: Only) -> usize {
        record
            .answering(&AskingAbout::anything().only(only))
            .count()
    }

    /// **The whole path a question that leaves takes.** It goes, the person is
    /// shown it going, the answer comes back knowing where it came from, the
    /// departure is in the record, and the indicator is quiet again afterwards.
    #[test]
    fn a_question_that_leaves_is_written_down_as_having_left() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = a_provider(&url);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "mistral-small-latest",
                permitting(mistral_source()),
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap();

        assert_eq!(answer.text(), "No, not without written consent.");
        assert_eq!(answer.source(), &mistral_source());
        assert!(server.join().unwrap().contains(SUBLET));

        // Law 1's second half: it left, and the record says so, under the
        // agent's name and nobody else's.
        assert_eq!(how_many(&record, Only::Egress), 1);
        assert_eq!(how_many(&record, Only::OnItsOwn), 0);
        assert_eq!(
            record
                .answering(&AskingAbout::anything().by(files().as_str()))
                .count(),
            1
        );
        // And the line came off, because the connection is over.
        assert!(indicator.is_quiet());
    }

    /// **A working day with a local model produces zero inference egress**, and
    /// this is that sentence as a test: nothing on the indicator at any point,
    /// nothing in the record that left, and an entry naming who asked.
    #[test]
    fn a_question_answered_on_this_machine_leaves_nothing_at_all() {
        let runtime = Stub::answering("No, not without written consent.");
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap();

        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert_eq!(
            runtime.asked(),
            Some((SUBLET.to_owned(), "a-model".to_owned()))
        );
        assert!(indicator.is_quiet());
        assert_eq!(how_many(&record, Only::Egress), 0);
        assert_eq!(how_many(&record, Only::ByAnAgent), 1);
    }

    /// A service somebody runs here is this machine in the only sense law 1
    /// cares about: the entry says *answered here* and nothing left.
    #[test]
    fn a_service_on_this_machine_is_answered_here_and_shows_nothing() {
        let (url, server) = serving(AN_ANSWER, 200);
        let service = a_service(&url);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Service(Served::at(&service, None).unwrap()),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap();

        assert_eq!(answer.source(), &InferenceSource::ThisMachine);
        assert!(server.join().unwrap().contains(SUBLET));
        assert!(indicator.is_quiet());
        assert_eq!(how_many(&record, Only::Egress), 0);
        assert_eq!(how_many(&record, Only::ByAnAgent), 1);
    }

    /// **The rule in force refuses it, and the refusal is written down** — in
    /// the rule's own words, from the value the policy made, because
    /// `Entry::held_back` can be made from nothing else.
    #[test]
    fn a_question_the_rule_will_not_let_leave_is_refused_and_recorded() {
        let provider = far_away();
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "mistral-small-latest",
                permitting(mistral_source()),
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&SourcePolicy::ThisMachineOnly),
                noon(),
            )
        })
        .unwrap_err();

        assert!(refused.nothing_left(), "{refused:?}");
        assert!(matches!(refused, NoAnswer::HeldBack(_)), "{refused:?}");
        assert!(
            refused
                .said(&in_english())
                .is_some_and(|said| !said.is_a_bug())
        );
        // It was refused, and it is not findable as something that left.
        assert_eq!(how_many(&record, Only::Refusals), 1);
        assert_eq!(how_many(&record, Only::Egress), 0);
        assert!(indicator.is_quiet());
    }

    /// **A question that failed still left the machine**, so the record has to
    /// say so — a machine that wrote down only the answered ones would report a
    /// quieter day than it had.
    #[test]
    fn a_question_that_left_and_did_not_answer_is_still_written_down_as_having_left() {
        let provider = far_away();
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let no_answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "mistral-small-latest",
                permitting(mistral_source()),
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();

        assert!(!no_answer.nothing_left(), "{no_answer:?}");
        assert!(
            matches!(no_answer, NoAnswer::DidNotAnswer(_)),
            "{no_answer:?}"
        );
        assert_eq!(how_many(&record, Only::Egress), 1);
        assert!(indicator.is_quiet());
    }

    /// **And one this machine could not answer leaves nothing**, which is
    /// `alo-answering`'s decision followed rather than made again: nothing
    /// left, nothing answered, and an entry per failure would build a log of
    /// somebody's questions failing one honest entry at a time.
    #[test]
    fn a_question_this_machine_could_not_answer_leaves_no_entry_at_all() {
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let no_answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();

        assert!(no_answer.nothing_left(), "{no_answer:?}");
        assert_eq!(
            record.len(),
            0,
            "a failure here is not a thing that happened"
        );
        assert!(indicator.is_quiet());
    }

    /// **The whole of ADR 0008, as a test.** The place a person chose does not
    /// answer, there is somewhere else set up, and the turn asks it nothing —
    /// the offer comes back for a person, and nothing in this crate can take
    /// one.
    #[test]
    fn a_turn_takes_no_offer_of_its_own() {
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let elsewhere = Stub::answering("something else entirely");
        let others = [mistral_source()];
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let no_answer = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere).and_everywhere_else(&others),
                noon(),
            )
        })
        .unwrap_err();

        let NoAnswer::DidNotAnswer(failed) = no_answer else {
            unreachable!("a runtime that is not answering is a failure")
        };
        // There is somewhere to offer, and it was offered rather than tried.
        assert_eq!(failed.elsewhere().offers().len(), 1);
        assert_eq!(runtime.times_asked(), 1);
        assert_eq!(elsewhere.times_asked(), 0);
        assert!(indicator.is_quiet());
        assert_eq!(record.len(), 0);
    }

    /// **And the answer to an offer a person took comes back in at the same
    /// door**, shown and written down exactly as a first attempt would be. That
    /// is the whole of what a turn does with a failure: hand it back, and be
    /// here when somebody says yes.
    #[test]
    fn an_offer_a_person_took_is_asked_through_the_same_door() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = a_provider(&url);
        let runtime = Stub::failing(RuntimeError::Unreachable);
        let others = [mistral_source()];
        let places = Places::under(&SourcePolicy::Anywhere).and_everywhere_else(&others);
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let answer = on_a_machine(&mut record, &mut indicator, |turning| {
            let no_answer = turning
                .asking(
                    SUBLET,
                    "a-model",
                    permitting(InferenceSource::ThisMachine),
                    &Answers::Runtime(&runtime),
                    &places,
                    noon(),
                )
                .unwrap_err();
            let NoAnswer::DidNotAnswer(failed) = no_answer else {
                unreachable!("a runtime that is not answering is a failure")
            };

            // The person says yes to the one place they were offered. This is
            // the only road from a failure to another attempt there is.
            let offer = failed.elsewhere().offers().first().cloned().unwrap();
            let taken = failed.take(&offer).unwrap();

            turning.asking(
                SUBLET,
                "mistral-small-latest",
                taken,
                &Answers::Provider(Hosted::provider(&provider, None)),
                &places,
                noon(),
            )
        })
        .unwrap();

        assert_eq!(answer.source(), &mistral_source());
        assert!(server.join().unwrap().contains(SUBLET));
        // One entry: the first attempt went nowhere, and the second left.
        assert_eq!(record.len(), 1);
        assert_eq!(how_many(&record, Only::Egress), 1);
        assert!(indicator.is_quiet());
    }

    /// A permission for one place and a question put to another sends nothing
    /// at all — and it is the one refusal here that is not said to anybody,
    /// because nobody using the machine can cause it.
    #[test]
    fn a_permission_and_a_place_that_disagree_send_nothing() {
        let runtime = Stub::answering("this should never be reached");
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                // The person's machine is set to a provider, and the runtime
                // was asked. `alo-asking` owns that check; nothing here repeats
                // it.
                permitting(mistral_source()),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();

        assert!(matches!(refused, NoAnswer::Miswired(_)), "{refused:?}");
        assert!(refused.nothing_left());
        assert!(refused.said(&in_english()).is_none());
        assert_eq!(runtime.times_asked(), 0);
        assert_eq!(record.len(), 0);
        assert!(indicator.is_quiet());
    }

    /// Nothing was typed, so nothing is asked and nothing is written down: what
    /// a person typed is the one thing ADR 0001 §7 never keeps, so an entry
    /// could only say that somebody had pressed a key.
    #[test]
    fn nothing_is_asked_when_there_is_no_question() {
        let runtime = Stub::answering("this should never be reached");
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let refused = on_a_machine(&mut record, &mut indicator, |turning| {
            turning.asking(
                "   ",
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();

        assert!(matches!(refused, NoAnswer::NotAQuestion(_)), "{refused:?}");
        assert_eq!(runtime.times_asked(), 0);
        assert_eq!(record.len(), 0);
    }

    /// **A question that left with no record of it closes the turn**, and the
    /// answer says which of the two mornings this is: something went to a
    /// provider and there is now no evidence of it.
    #[test]
    fn a_departure_that_could_not_be_written_down_closes_the_turn_and_says_it_left() {
        let (url, server) = serving(AN_ANSWER, 200);
        let provider = a_provider(&url);
        let runtime = Stub::answering("this should never be reached");
        let mut indicator = Indicator::default();
        let mut disk = ANoSpaceLeftDisk::default();

        let (first, second) = on_a_machine(&mut disk, &mut indicator, |turning| {
            let first = turning.asking(
                SUBLET,
                "mistral-small-latest",
                permitting(mistral_source()),
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            );
            assert!(turning.is_closed());
            let second = turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            );
            (first.err(), second.err())
        });

        let first = first.unwrap();
        assert!(
            matches!(
                first,
                NoAnswer::NotRecorded {
                    after_it_left: true,
                    ..
                }
            ),
            "{first:?}"
        );
        assert!(!first.nothing_left());
        assert!(first.is_the_end_of_the_turn());

        // And nothing else happens under this turn.
        let second = second.unwrap();
        assert!(matches!(second, NoAnswer::TurnClosed), "{second:?}");
        assert_eq!(runtime.times_asked(), 0);

        // The question really did go, and the line still came off the
        // indicator: it is a statement about now, and the connection is over.
        assert!(server.join().unwrap().contains(SUBLET));
        assert_eq!(disk.refused, 1);
        assert!(indicator.is_quiet());
    }

    /// The same disk, with nothing having left: the turn closes just the same,
    /// and the answer says so — because *the record broke* and *somebody's
    /// question left with no evidence of it* are two different mornings.
    #[test]
    fn a_record_that_broke_with_nothing_leaving_says_nothing_left() {
        let runtime = Stub::answering("No, not without written consent.");
        let mut indicator = Indicator::default();
        let mut disk = ANoSpaceLeftDisk::default();

        let refused = on_a_machine(&mut disk, &mut indicator, |turning| {
            turning.asking(
                SUBLET,
                "a-model",
                permitting(InferenceSource::ThisMachine),
                &Answers::Runtime(&runtime),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
        })
        .unwrap_err();

        assert!(
            matches!(
                refused,
                NoAnswer::NotRecorded {
                    after_it_left: false,
                    ..
                }
            ),
            "{refused:?}"
        );
        assert!(refused.nothing_left());
        assert!(refused.is_the_end_of_the_turn());
        assert!(indicator.is_quiet());
    }

    /// **A question asks the grants nothing**, on a machine that granted
    /// nothing at all: an agent putting a question to a model is what an agent
    /// is, not a capability it was given.
    #[test]
    fn a_question_needs_no_grant() {
        let runtime = Stub::answering("No, not without written consent.");
        let mut indicator = Indicator::default();
        let mut record = Record::default();

        let strings = in_english();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, &mut indicator, &mut record)
                .unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();

        assert!(grants.is_empty(), "nothing was granted to anybody");
        assert!(
            turning
                .asking(
                    SUBLET,
                    "a-model",
                    permitting(InferenceSource::ThisMachine),
                    &Answers::Runtime(&runtime),
                    &Places::under(&SourcePolicy::Anywhere),
                    noon(),
                )
                .is_ok()
        );
        assert!(grants.is_empty(), "asking a question granted something");
    }
}
