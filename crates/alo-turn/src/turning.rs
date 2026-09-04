//! One turn, and the five doors into it.
//!
//! [`alo_context::Turn`] is what an invocation offered and the grant it made.
//! This is what happens next: a turn an agent can ask things of, and a person
//! can answer, from the moment the key was pressed until the grant goes away
//! again.
//!
//! # The doors, and what each one is for
//!
//! | Door | ADR 0001 | What it answers with |
//! |---|---|---|
//! | [`Turning::reading`] | §5, a read runs inside the turn | what the machine found |
//! | [`Turning::proposing`] | §5, a change waits | the number of a question a person is being asked |
//! | [`Turning::approving`] | §5, one approval is one execution | what the machine did |
//! | [`Turning::declining`] | §5, and *no* is a whole answer | nothing, and an entry saying the person said no |
//! | [`Turning::ending`] | §3, a grant that expires *and* is revoked | whether a grant was taken back |
//!
//! There is a sixth, [`Turning::asking`], and it is in [`crate::asking`] rather
//! than in this file because it is the one that may reach off this machine: it
//! puts a question to a model, shows what leaves while it leaves, and asks the
//! grants nothing at all. `alo-agentd`'s protocol reaches it beside the five
//! here, and law 1 rather than ADR 0001 §5 is what shapes it.
//!
//! **There is no door that takes a call.** Both of the two that start something
//! take a verb's name and what was given for each argument, and put them
//! through [`alo_capability::Verbs::call`] against the closed list this machine
//! offers. Law 2 is carried by there being nowhere else to come in: a caller
//! cannot hand over something it validated itself, because the type that means
//! *validated* is made here or not at all.
//!
//! # A read and a change cannot be swapped
//!
//! [`Turning::reading`] refuses a change and [`Turning::proposing`] refuses a
//! read, and neither refusal is written in this file: the first is
//! `alo_capability::Authorised::read` and the second is
//! `alo_capability::Proposal::checked`. A turn that decided this itself would
//! be a second answer to a question ADR 0001 §5 already answers, and the two
//! could disagree.
//!
//! # The grants are passed in, not held
//!
//! Every door takes the machine's grants at the moment it is called, so a grant
//! revoked between an approval and the execution it authorises stops that
//! execution. A turn that borrowed the list once at its beginning would be a
//! turn nobody could revoke anything during — which is item 3's rule, and the
//! whole of what makes *a revoked grant takes effect immediately* true rather
//! than eventual.
//!
//! # And a turn that could not write something down does nothing else
//!
//! Every door writes its entry before it answers, and a door that could not
//! closes the turn: [`Turning::is_closed`] is true from then on and every door
//! answers [`NotDone::TurnClosed`]. See [`crate`] for what that does and does
//! not close.
//!
//! # Two doors run something, and both run it inside a boundary
//!
//! [`Turning::reading`] and [`Turning::approving`] are the two, and neither
//! knows anything about how a boundary is imposed: `carrying.rs` has the order
//! and [`crate::Bounding`] is what the machine was made with. What is here is
//! the two things a turn is left holding when there was none —
//! [`NotDone::NotBounded`], which nothing writes down because nothing happened,
//! and [`Turning::a_thread_is_lost`], which is the one a service stops over.

use std::time::{Duration, SystemTime};

use alo_capability::{
    AnswerError, Approvals, Authorised, Call, Given, GrantError, GrantId, Grantee, Grants,
    Proposal, ProposalId, Waiting,
};
use alo_context::{Context, Turn};
use alo_files::Answer;
use alo_record::Entry;

use crate::carrying::carrying_out;
use crate::machine::Machine;
use crate::refusing::NotDone;

/// A turn that is under way.
///
/// Deliberately not `Clone`, like the [`Turn`] inside it: a turn that can be
/// copied is a turn that can be ended twice, and the second ending would revoke
/// a grant the first had already given back.
#[derive(Debug)]
pub struct Turning<'a, 'm> {
    /// What was offered at the invocation, and the grant it made.
    turn: Turn,
    /// The verbs, the words, the resolver and the record.
    machine: &'a mut Machine<'m>,
    /// The changes this turn has put to a person and that nobody has answered.
    ///
    /// The turn's own, not the machine's: a change is asked about during the
    /// turn that proposed it, and one nobody answered goes away with that turn
    /// rather than standing over into the next one.
    approvals: Approvals,
    /// Whether something that happened could not be written down.
    closed: bool,
    /// Whether a thread of this service went into a boundary and stayed there.
    ///
    /// Separate from `closed`, because they are two different things wrong with
    /// two different parts of the machine and a caller does two different
    /// things about them: a closed turn is a machine that has stopped keeping
    /// evidence, and this is a service that has lost a thread to a grant that
    /// is over. Neither can be told from the other by reading a sentence.
    lost_a_thread: bool,
}

impl<'a, 'm> Turning<'a, 'm> {
    /// Begin a turn from what an invocation offered.
    ///
    /// The document the person had open becomes a grant in the machine's own
    /// list, for the length of the turn; the window in front of them and the
    /// text they had selected grant nothing. `alo_context::turn` is where that
    /// is decided and why.
    ///
    /// **A turn cannot begin on a machine with no agent** (ADR 0009), and no
    /// check here says so: this needs the machine's grants, and a machine where
    /// the person declined has none to lend — `alo_capability::Agent::grants_mut`
    /// answers `None`.
    ///
    /// # Errors
    /// [`GrantError`], carried whole from `alo-capability` rather than reworded,
    /// so the words a person reads about a grant that could not be made are the
    /// grants' own wherever it happened.
    pub fn beginning(
        context: Context,
        agent: &str,
        lasting: Duration,
        grants: &mut Grants,
        machine: &'a mut Machine<'m>,
    ) -> Result<Self, GrantError> {
        Ok(Self {
            turn: Turn::beginning(context, agent, lasting, grants)?,
            machine,
            approvals: Approvals::default(),
            closed: false,
            lost_a_thread: false,
        })
    }

    /// A read, which answers inside the turn.
    ///
    /// The verb is looked up on the list this machine offers, the arguments are
    /// validated, the grants are asked, every path is resolved and asked about
    /// again where it really leads, and then the machine does it. A read needs
    /// no approval and still needs its grant: running inside the turn is about
    /// *approval*, never about *reach*.
    ///
    /// # Errors
    /// [`NotDone`], and the entry that says so is written before this answers —
    /// [`NotDone::TurnedAway`] if nothing formed, [`NotDone::Refused`] if the
    /// capability model said no, [`NotDone::MachineCouldNot`] if it said yes and
    /// the disk did not.
    pub fn reading(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Answer, NotDone> {
        self.still_open()?;
        let call = self.calling(verb, given, now)?;
        let authorised = match Authorised::read(&call, self.turn.grantee(), grants, now) {
            Ok(authorised) => authorised,
            Err(refused) => return self.stopped_at_the_moment(refused, now),
        };
        let (entry, outcome) = self.inside_a_boundary(authorised, grants)?;
        self.writing_down(entry)?;
        outcome
    }

    /// A change, put to a person in one sentence.
    ///
    /// Nothing happens here and nothing is written down: a proposal is a
    /// question, and what the record keeps is its answer. The grants are asked
    /// now as well as at the moment of execution, so a change nobody could have
    /// run never interrupts anybody — and *that* is written down, because being
    /// refused before anybody was asked is a thing that happened.
    ///
    /// # Errors
    /// [`NotDone::TurnedAway`] if nothing formed, and [`NotDone::NeverAsked`] if
    /// the grants refused it or a read was offered where only a change waits.
    pub fn proposing(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        standing: Duration,
        now: SystemTime,
    ) -> Result<ProposalId, NotDone> {
        self.still_open()?;
        let call = self.calling(verb, given, now)?;
        let proposal = match Proposal::checked(&call, self.turn.grantee(), grants, now, standing) {
            Ok(proposal) => proposal,
            Err(why) => {
                let said = why.said(self.machine.strings()).into_text();
                let entry = Entry::never_asked(
                    &call,
                    self.turn.grantee(),
                    &said,
                    self.machine.strings(),
                    now,
                );
                self.writing_down(entry)?;
                return Err(NotDone::NeverAsked(why));
            }
        };
        Ok(self.approvals.propose(proposal))
    }

    /// The person approved it, so it runs — once.
    ///
    /// The approval is spent by being answered, so approving the same number
    /// twice answers [`NotDone::NotAnswered`] the second time; the grants are
    /// asked again here, which is where one revoked since the question was
    /// asked takes effect.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] if that number is not waiting or the question
    /// stood too long, [`NotDone::Refused`] if the grants said no at this
    /// moment, [`NotDone::MachineCouldNot`] if the disk did.
    pub fn approving(
        &mut self,
        id: ProposalId,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Answer, NotDone> {
        self.still_open()?;
        let approved = self.approvals.approve(id, now)?;
        let authorised = match approved.redeem(grants, now) {
            Ok(authorised) => authorised,
            Err(refused) => return self.stopped_at_the_moment(refused, now),
        };
        let (entry, outcome) = self.inside_a_boundary(authorised, grants)?;
        self.writing_down(entry)?;
        outcome
    }

    /// The person said no.
    ///
    /// Nothing is kept about why: *no* is the whole answer, and a system that
    /// recorded a reason would be a system that asked for one.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] if that number is not waiting for an answer, and
    /// [`NotDone::NotRecorded`] if the refusal could not be written down.
    pub fn declining(&mut self, id: ProposalId, now: SystemTime) -> Result<(), NotDone> {
        self.still_open()?;
        let Some(proposal) = self.approvals.decline(id) else {
            return Err(NotDone::NotAnswered(AnswerError::NothingWaiting {
                number: id.as_u64(),
            }));
        };
        let entry = Entry::declined(&proposal, self.machine.strings(), now);
        self.writing_down(entry)
    }

    /// End the turn, taking the grant the invocation made back out of the list.
    ///
    /// Answers whether a grant was taken away — `false` when no document was
    /// offered, and when the person has already revoked it themselves. Changes
    /// this turn put to somebody and nobody answered go away with it.
    ///
    /// # Examples
    ///
    /// A turn cannot be ended twice, because ending it consumes it — and the
    /// second ending would revoke a grant the first had already given back, on
    /// a list where the handle may since have been given to something else:
    ///
    /// ```compile_fail
    /// use alo_capability::Grants;
    /// use alo_context::Context;
    /// use alo_egress::Indicator;
    /// use alo_files::OnThisMachine;
    /// use alo_record::Record;
    /// use alo_strings::{Strings, Vocabulary};
    /// use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, Turning};
    /// use std::time::{Duration, SystemTime};
    ///
    /// // A machine with nothing in front of a turn, which is not a machine alo
    /// // OS ships: `alo_turn::bounding` says why there is no such thing in any
    /// // library here, and why what a test needs is these four lines.
    /// struct NothingIsBounded;
    /// impl Bounding for NothingIsBounded {
    ///     fn carrying_out(
    ///         &mut self,
    ///         _reaching: &alo_files::Reaching,
    ///         doing: Doing<'_>,
    ///     ) -> Result<Done, NoBoundary> {
    ///         Ok(doing.done())
    ///     }
    /// }
    ///
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let strings = Strings::of(Vocabulary::empty());
    /// let mut indicator = Indicator::default();
    /// let mut record = Record::default();
    /// let mut bounding = NothingIsBounded;
    /// let mut machine = Machine::carrying_out_file_verbs(
    ///     &strings,
    ///     &OnThisMachine,
    ///     &mut bounding,
    ///     &mut indicator,
    ///     &mut record,
    /// )
    /// .unwrap();
    /// let mut grants = Grants::default();
    /// let turning = Turning::beginning(
    ///     Context::at_invocation(now),
    ///     "@files",
    ///     Duration::from_secs(300),
    ///     &mut grants,
    ///     &mut machine,
    /// )
    /// .unwrap();
    ///
    /// turning.ending(&mut grants);
    /// turning.ending(&mut grants);
    /// ```
    ///
    /// Checked by unmarking it: it fails with **E0382, use of moved value**,
    /// and not on a typo. The twin that passes is the same code with the second
    /// line removed, which is
    /// `what_the_invocation_offered_is_the_only_thing_it_granted` below.
    #[must_use]
    pub fn ending(self, grants: &mut Grants) -> bool {
        self.turn.ending(grants)
    }

    /// The agent this turn is for.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        self.turn.grantee()
    }

    /// What was offered at the invocation this turn began at.
    #[must_use]
    pub fn context(&self) -> &Context {
        self.turn.context()
    }

    /// When the turn is over.
    #[must_use]
    pub fn ends(&self) -> SystemTime {
        self.turn.ends()
    }

    /// The handle the grant this invocation made went into the machine's list
    /// under, and nothing when no document was offered.
    #[must_use]
    pub fn granted(&self) -> Option<GrantId> {
        self.turn.granted()
    }

    /// The changes this turn is waiting for a person to answer.
    ///
    /// What a shell draws, and what it draws them from: each carries the number
    /// to answer with and the sentence the person is being asked about.
    pub fn waiting_at(&self, now: SystemTime) -> impl Iterator<Item = &Waiting> {
        self.approvals.waiting_at(now)
    }

    /// What is leaving this machine right now.
    ///
    /// The machine's indicator, lent out while a turn holds the machine — a
    /// shell that could not draw law 1's surface during a turn would be a shell
    /// that cannot draw it at the one moment it matters. Only
    /// [`Turning::asking`] and alo OS's own errands put anything on it.
    #[must_use]
    pub fn showing(&self) -> &alo_egress::Indicator {
        self.machine.showing()
    }

    /// Whether this turn has stopped because something could not be written
    /// down.
    ///
    /// A daemon reads this to know it has a machine to stop rather than a turn
    /// to retry: what is missing is evidence, and nothing further will be done
    /// under this turn.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Whether a thread of this service went into a boundary and stayed there.
    ///
    /// A daemon reads this for the reason it reads [`Turning::is_closed`], and
    /// it does the same thing about it: there is a thread in this process that
    /// is refused everything outside a grant that has ended, so the service is
    /// over. It is asked first of the two, because a service that has lost a
    /// thread has not stopped keeping evidence and must not report that it has.
    #[must_use]
    pub fn a_thread_is_lost(&self) -> bool {
        self.lost_a_thread
    }

    /// A name and some values, made into a call against the closed list — or
    /// turned away, and written down as having been.
    ///
    /// The one door in, so that everything this crate can run has been through
    /// `alo_capability::Verbs::call`. What is written down carries the verb as
    /// it was asked for and the refusal, and **no arguments at all**: they are
    /// whatever a model was persuaded to send, and `alo_record` keeps none of
    /// them.
    fn calling(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        now: SystemTime,
    ) -> Result<Call, NotDone> {
        match self.machine.verbs().call(verb, given) {
            Ok(call) => Ok(call),
            Err(why) => {
                let said = why.said(self.machine.strings()).into_text();
                let entry = Entry::turned_away(verb, &said, self.turn.grantee(), now);
                self.writing_down(entry)?;
                Err(NotDone::TurnedAway(why))
            }
        }
    }

    /// Carry the call out inside the boundary this machine imposes, or say
    /// there was none.
    ///
    /// Both doors that run something come through here, and it is the whole of
    /// what a boundary changes about them: everything else on either side of it
    /// is what it always was. Nothing is written down on the refusing road —
    /// `carrying.rs` says why there is nothing true to write — and a
    /// thread left inside is remembered on the turn, because the service that
    /// holds it has to be able to ask.
    fn inside_a_boundary(
        &mut self,
        authorised: Authorised,
        grants: &Grants,
    ) -> Result<(Entry, Result<Answer, NotDone>), NotDone> {
        match carrying_out(self.machine, authorised, grants) {
            Ok(both) => Ok(both),
            Err(no_boundary) => {
                if no_boundary.a_thread_is_still_inside() {
                    self.lost_a_thread = true;
                }
                Err(NotDone::NotBounded(no_boundary))
            }
        }
    }

    /// The capability model said no where it is asked last, written down and
    /// handed back.
    ///
    /// Both doors that run something end here when the grants refuse, and they
    /// share it because it is one fact: a properly formed call, stopped at the
    /// moment it would have run.
    fn stopped_at_the_moment<T>(
        &mut self,
        refused: alo_capability::Refused,
        now: SystemTime,
    ) -> Result<T, NotDone> {
        let entry = Entry::refused(&refused, self.turn.grantee(), self.machine.strings(), now);
        self.writing_down(entry)?;
        Err(NotDone::Refused(refused))
    }

    /// Write one thing that happened down, and close the turn if it could not
    /// be written.
    ///
    /// The only road to the record in this crate. A door that answered without
    /// coming through here would be a door the gate's *every execution and every
    /// refusal leaves a record* is not true of.
    fn writing_down(&mut self, entry: Entry) -> Result<(), NotDone> {
        self.keeping(entry).map_err(NotDone::NotRecorded)
    }

    /// The same, answering with what the record said rather than with a turn's
    /// refusal.
    ///
    /// `pub(crate)` and the only thing in this crate that touches the record.
    /// [`crate::asking`] needs the failure itself, because a question has one
    /// more thing to say about it than a verb does — whether it had already
    /// left the machine when the record broke — and that is not a distinction
    /// [`NotDone`] has anywhere to put.
    pub(crate) fn keeping(&mut self, entry: Entry) -> Result<(), alo_keeping::NotKept> {
        match self.machine.kept().keep(entry) {
            Ok(()) => Ok(()),
            Err(why) => {
                self.closed = true;
                Err(why)
            }
        }
    }

    /// The verbs, the words, the resolver, the indicator and the record.
    ///
    /// `pub(crate)`, for [`crate::asking`]: a question reaches the indicator
    /// and the record, and both of those are the machine's rather than the
    /// turn's.
    pub(crate) fn machine(&mut self) -> &mut Machine<'m> {
        self.machine
    }

    /// Whether anything more may happen under this turn.
    fn still_open(&self) -> Result<(), NotDone> {
        if self.closed {
            return Err(NotDone::TurnClosed);
        }
        Ok(())
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
    use crate::testing::{
        NoBoundaryAtAll, NothingIsBounded, a_folder_of_our_own, as_given, files, granting, hour,
        in_english, noon, offering,
    };
    use alo_capability::{Agent, Ask, CallError, ProposalError};
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_keeping::NotKept;
    use alo_record::{Asking, Only, Record};
    use alo_strings::{Strings, Vocabulary};
    use std::fs;
    use std::path::{Path, PathBuf};

    /// A record that cannot be written to, which is what a full disk looks like
    /// from inside a turn.
    ///
    /// It keeps what it was handed anyway, so a test can assert that the entry
    /// the turn tried to write is the entry it should have written: the failure
    /// is in the keeping, not in the making.
    #[derive(Default)]
    struct ANoSpaceLeftDisk {
        /// What the turn tried to write down.
        tried: Vec<Entry>,
    }

    impl Kept for ANoSpaceLeftDisk {
        fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
            self.tried.push(entry);
            Err(NotKept::NotAddedTo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "no space left on device".to_owned(),
            })
        }
    }

    /// A disk with no space on it is still a disk, and nothing here is asked to
    /// shorten anything: these tests are about what a turn does when it cannot
    /// write.
    impl crate::Shortening for ANoSpaceLeftDisk {
        fn shorten(
            &mut self,
            _keeping: alo_keeping::Keeping,
            _now: std::time::SystemTime,
        ) -> Result<crate::Shortened, NotKept> {
            Ok(crate::Shortened::NotOnADisk)
        }
    }

    /// A folder with one file in it, and the file's path.
    fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
        let folder = a_folder_of_our_own(what);
        let invoice = folder.join("march.pdf");
        fs::write(&invoice, "March, 4180.00").unwrap();
        (folder, invoice)
    }

    /// The arrangement most of these tests need: a machine that offers the six,
    /// grants over one folder, and an invocation that offered the file in it.
    ///
    /// Written as a closure taking the turn rather than a function returning
    /// one, because a [`Turning`] borrows the [`Machine`] and the machine
    /// borrows the record — so all three have to live in the caller's frame.
    fn on_a_machine<T>(
        what: &str,
        kept: &mut dyn crate::Shortening,
        doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Path, &Path) -> T,
    ) -> T {
        on_a_machine_bounded_by(what, kept, &mut NothingIsBounded, doing)
    }

    /// The same, on a machine whose boundary is the one this test is about.
    ///
    /// Separate from [`on_a_machine`] rather than a parameter on it, because
    /// what a boundary does is what three tests here are for and what every
    /// other test in this file takes for granted.
    fn on_a_machine_bounded_by<T>(
        what: &str,
        kept: &mut dyn crate::Shortening,
        bounding: &mut dyn crate::Bounding,
        doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &Path, &Path) -> T,
    ) -> T {
        let strings = in_english();
        let (folder, invoice) = a_folder_with_an_invoice(what);
        let mut indicator = Indicator::default();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            bounding,
            &mut indicator,
            kept,
        )
        .unwrap();
        let mut grants = granting(&[&folder]);
        let mut turning = Turning::beginning(
            offering(&invoice),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        doing(&mut turning, &mut grants, &folder, &invoice)
    }

    /// What the one change these tests make is asked for with.
    fn renaming(invoice: &Path) -> Vec<(&'static str, Given)> {
        vec![
            ("file", as_given(invoice)),
            ("name", Given::text("march-final.pdf")),
        ]
    }

    /// **A read answers inside the turn, and the record says it ran.** The
    /// ordinary path in one test: a name and a value, validated against the
    /// closed list, permitted by a grant, resolved, done, written down.
    #[test]
    fn a_read_answers_inside_the_turn_and_is_written_down() {
        let mut record = Record::default();
        on_a_machine("a-read", &mut record, |turning, grants, folder, _| {
            let answer = turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    grants,
                    noon(),
                )
                .unwrap();
            assert_eq!(answer.listed().unwrap().things().len(), 1);
        });

        assert_eq!(record.len(), 1);
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().ran());
        assert_eq!(
            entry.happened().from_approval(),
            None,
            "a read ran from an approval nobody gave"
        );
    }

    /// **A change waits for one person, and one approval is one execution.**
    /// The question is not an entry; what the person answered is.
    #[test]
    fn a_change_waits_for_an_approval_and_then_runs_once() {
        let mut record = Record::default();
        let renamed = on_a_machine("a-change", &mut record, |turning, grants, _, invoice| {
            let id = turning
                .proposing("rename_file", &renaming(invoice), grants, hour(), noon())
                .unwrap();
            assert_eq!(turning.waiting_at(noon()).count(), 1);

            let answer = turning.approving(id, grants, noon()).unwrap();
            let now_at = answer.now_at().unwrap().to_path_buf();

            // The approval is spent. A second one is not a second execution.
            let again = turning.approving(id, grants, noon()).unwrap_err();
            assert!(matches!(
                again,
                NotDone::NotAnswered(AnswerError::NothingWaiting { .. })
            ));
            now_at
        });

        assert!(renamed.ends_with("march-final.pdf"));
        assert!(renamed.is_file(), "the file did not move on the disk");
        assert_eq!(
            record.len(),
            1,
            "either a question was recorded, or an approval ran twice"
        );
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().ran());
        assert!(entry.happened().from_approval().is_some());
    }

    /// **A change offered as a read is refused, and a read offered for approval
    /// is too** — and neither refusal is written in this crate. ADR 0001 §5 is
    /// two refusals rather than a convention, and both are recorded.
    #[test]
    fn a_read_and_a_change_cannot_be_swapped() {
        let mut record = Record::default();
        let still_there = on_a_machine(
            "swapped",
            &mut record,
            |turning, grants, folder, invoice| {
                let as_a_read = turning
                    .reading("rename_file", &renaming(invoice), grants, noon())
                    .unwrap_err();
                assert!(as_a_read.was_refused(), "{as_a_read:?}");

                let as_a_change = turning
                    .proposing(
                        "list_folder",
                        &[("folder", as_given(folder))],
                        grants,
                        hour(),
                        noon(),
                    )
                    .unwrap_err();
                assert!(matches!(
                    as_a_change,
                    NotDone::NeverAsked(ProposalError::ReadDoesNotWait { .. })
                ));
                invoice.is_file()
            },
        );

        assert!(still_there, "a change ran without anybody approving it");
        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            2
        );
    }

    /// **A verb that is not on the list never becomes a call**, and what is
    /// written down carries no arguments — they are whatever a model was
    /// persuaded to send.
    #[test]
    fn a_verb_nobody_declared_is_turned_away_with_nothing_kept_about_it() {
        let mut record = Record::default();
        on_a_machine("no-such-verb", &mut record, |turning, grants, _, _| {
            let turned_away = turning
                .reading(
                    "delete_everything",
                    &[("path", Given::text("/home/anna/.ssh/id_ed25519"))],
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(
                turned_away,
                NotDone::TurnedAway(CallError::NoSuchVerb { .. })
            ));
        });

        assert_eq!(record.len(), 1);
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().was_stopped());
        assert!(
            entry.what().is_none(),
            "a call that never formed has arguments kept against it"
        );
        assert!(
            !format!("{record:?}").contains("id_ed25519"),
            "an unvalidated argument reached the record"
        );
    }

    /// **A path nobody granted is refused before the disk is touched**, and the
    /// refusal is recorded as carefully as an execution.
    #[test]
    fn a_folder_nobody_granted_is_refused_and_recorded() {
        let mut record = Record::default();
        let somewhere_else = a_folder_of_our_own("not-granted");
        on_a_machine("not-granted-turn", &mut record, |turning, grants, _, _| {
            let refused = turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(&somewhere_else))],
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(refused.was_refused());
            let said = refused.said(&in_english()).into_text();
            assert!(said.contains("has not been granted"), "{said}");
        });

        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            1
        );
    }

    /// **Every path a call names is asked about**, not only the first one. A
    /// move out of a granted folder into one nobody granted never becomes a
    /// question, so nobody is interrupted about a change that could not run.
    #[test]
    fn a_change_into_a_folder_nobody_granted_moves_nothing() {
        let mut record = Record::default();
        let somewhere_else = a_folder_of_our_own("into-not-granted");
        on_a_machine(
            "out-of-the-grant",
            &mut record,
            |turning, grants, _, invoice| {
                let never_asked = turning
                    .proposing(
                        "move_file",
                        &[
                            ("file", as_given(invoice)),
                            ("into", as_given(&somewhere_else)),
                        ],
                        grants,
                        hour(),
                        noon(),
                    )
                    .unwrap_err();
                assert!(matches!(never_asked, NotDone::NeverAsked(_)));
                assert_eq!(
                    turning.waiting_at(noon()).count(),
                    0,
                    "somebody was interrupted about a change that could never have run"
                );
                assert!(invoice.is_file());
            },
        );

        assert!(
            !somewhere_else.join("march.pdf").exists(),
            "a file moved into a folder nobody granted"
        );
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().was_stopped());
    }

    /// **A grant taken away between the question and the answer stops it.**
    /// Item 3's rule walked end to end: the grants are asked last, and last is
    /// the moment the approval is spent.
    #[test]
    fn a_grant_taken_away_after_the_question_stops_the_change() {
        let mut record = Record::default();
        on_a_machine("revoked", &mut record, |turning, grants, _, invoice| {
            let id = turning
                .proposing("rename_file", &renaming(invoice), grants, hour(), noon())
                .unwrap();

            // The person changes their mind between the question and the answer.
            let held: Vec<_> = grants.active_at(noon()).map(|held| held.id).collect();
            for grant in held {
                assert!(grants.revoke(grant));
            }

            let stopped = turning.approving(id, grants, noon()).unwrap_err();
            assert!(matches!(stopped, NotDone::Refused(_)), "{stopped:?}");
            assert!(invoice.is_file(), "the file moved after the grant was gone");
        });

        let entry = record.everything().next().unwrap();
        assert!(entry.happened().was_stopped());
    }

    /// **A person who says no is answered, and it is written down.** Nothing is
    /// kept about why, because nothing was asked.
    #[test]
    fn a_change_the_person_declined_is_recorded_and_nothing_runs() {
        let mut record = Record::default();
        on_a_machine("declined", &mut record, |turning, grants, _, invoice| {
            let id = turning
                .proposing("rename_file", &renaming(invoice), grants, hour(), noon())
                .unwrap();
            turning.declining(id, noon()).unwrap();
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            assert!(invoice.is_file(), "a declined change ran anyway");

            // And it cannot be approved afterwards.
            let gone = turning.approving(id, grants, noon()).unwrap_err();
            assert!(matches!(gone, NotDone::NotAnswered(_)));
        });

        assert_eq!(record.len(), 1);
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().was_stopped());
        assert_eq!(
            entry.happened().why_stopped(),
            None,
            "the record kept a reason nobody was asked for"
        );
    }

    /// **A question nobody answered leaves no entry**, which is the one refusal
    /// here with nothing written behind it: what the record keeps is what the
    /// agent did, and a person who did not answer did not act.
    #[test]
    fn a_question_that_stood_too_long_is_not_written_down() {
        let mut record = Record::default();
        on_a_machine("lapsed", &mut record, |turning, grants, _, invoice| {
            let id = turning
                .proposing(
                    "rename_file",
                    &renaming(invoice),
                    grants,
                    Duration::from_secs(60),
                    noon(),
                )
                .unwrap();
            let too_late = turning
                .approving(id, grants, noon() + Duration::from_secs(120))
                .unwrap_err();
            assert!(matches!(
                too_late,
                NotDone::NotAnswered(AnswerError::Lapsed { .. })
            ));
        });

        assert!(
            record.is_empty(),
            "a question nobody answered became an entry"
        );
    }

    /// **Nothing is handed back that has not been written down, and a turn that
    /// could not write does nothing else.** The read really happened on the
    /// disk; the caller is told the record failed rather than told what it
    /// found.
    #[test]
    fn a_turn_that_could_not_write_something_down_does_nothing_else() {
        let mut disk = ANoSpaceLeftDisk::default();
        on_a_machine("no-space", &mut disk, |turning, grants, folder, invoice| {
            // A question is not an entry, so this one gets through — and gives
            // the closed doors below a number to be refused with.
            let id = turning
                .proposing("rename_file", &renaming(invoice), grants, hour(), noon())
                .unwrap();

            let not_recorded = turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(folder))],
                    grants,
                    noon(),
                )
                .unwrap_err();
            assert!(matches!(not_recorded, NotDone::NotRecorded(_)));
            assert!(not_recorded.is_the_end_of_the_turn());
            assert!(turning.is_closed());

            // Every door afterwards, including the ones that would only refuse.
            let closed = [
                turning
                    .reading(
                        "list_folder",
                        &[("folder", as_given(folder))],
                        grants,
                        noon(),
                    )
                    .unwrap_err(),
                turning
                    .proposing("delete_everything", &[], grants, hour(), noon())
                    .unwrap_err(),
                turning.approving(id, grants, noon()).unwrap_err(),
                turning.declining(id, noon()).unwrap_err(),
            ];
            for door in closed {
                assert_eq!(door, NotDone::TurnClosed);
            }
            assert!(invoice.is_file(), "a change ran under a closed turn");
        });

        assert_eq!(
            disk.tried.len(),
            1,
            "the turn went on making entries after it stopped being able to keep them"
        );
        assert!(
            disk.tried
                .first()
                .is_some_and(|entry| entry.happened().ran())
        );
    }

    /// **A turn that could not be bounded does nothing at all**, and that is
    /// ADR 0015's rule met at the door: the file is still where it was, the
    /// person is told in their own language, and it is not a refusal — nothing
    /// was refused, because nothing was asked.
    #[test]
    fn a_turn_that_could_not_be_bounded_does_nothing_and_says_so() {
        let mut record = Record::default();
        let still_there = on_a_machine_bounded_by(
            "no-boundary",
            &mut record,
            &mut NoBoundaryAtAll::kernel_would_not_take_it(),
            |turning, grants, _, invoice| {
                let id = turning
                    .proposing("rename_file", &renaming(invoice), grants, hour(), noon())
                    .unwrap();
                let not_bounded = turning.approving(id, grants, noon()).unwrap_err();

                assert!(
                    matches!(not_bounded, NotDone::NotBounded(_)),
                    "{not_bounded:?}"
                );
                assert!(
                    !not_bounded.was_refused(),
                    "a machine that could not bound a turn was reported as the grants refusing it"
                );
                assert!(!not_bounded.is_the_end_of_the_turn());
                assert!(!turning.a_thread_is_lost());

                let said = not_bounded.said(&in_english()).into_text();
                assert!(said.contains("nothing was done"), "{said}");
                invoice.is_file()
            },
        );

        assert!(still_there, "a change ran with no boundary around it");
        assert!(
            record.is_empty(),
            "a turn that never ran wrote something down about having run"
        );
    }

    /// **A read is bounded too**, which is the half somebody would be tempted to
    /// leave out: a read touches a disk, so a verb with a bug in it reads
    /// whatever it names, and ADR 0013 is about exactly that.
    #[test]
    fn a_read_that_could_not_be_bounded_answers_nothing() {
        let mut record = Record::default();
        on_a_machine_bounded_by(
            "no-boundary-read",
            &mut record,
            &mut NoBoundaryAtAll::kernel_would_not_take_it(),
            |turning, grants, folder, _| {
                let not_bounded = turning
                    .reading(
                        "list_folder",
                        &[("folder", as_given(folder))],
                        grants,
                        noon(),
                    )
                    .unwrap_err();
                assert!(
                    matches!(not_bounded, NotDone::NotBounded(_)),
                    "{not_bounded:?}"
                );
            },
        );

        assert!(record.is_empty(), "a read that never ran left an entry");
    }

    /// **A thread that could not be brought back is a service that stops**, and
    /// the turn is what a daemon asks: a thread of this process is inside a
    /// boundary belonging to a turn that is over, refused everything outside a
    /// grant that no longer exists.
    ///
    /// It is deliberately *not* a closed turn. Nothing has gone wrong with the
    /// record, and a service that reported this as *nothing is written down*
    /// would send whoever reads it to look at a disk that is fine.
    #[test]
    fn a_thread_left_inside_a_boundary_is_a_service_that_stops() {
        let mut record = Record::default();
        on_a_machine_bounded_by(
            "thread-lost",
            &mut record,
            &mut NoBoundaryAtAll::a_thread_is_still_inside(),
            |turning, grants, folder, _| {
                let not_bounded = turning
                    .reading(
                        "list_folder",
                        &[("folder", as_given(folder))],
                        grants,
                        noon(),
                    )
                    .unwrap_err();

                assert!(
                    matches!(not_bounded, NotDone::NotBounded(_)),
                    "{not_bounded:?}"
                );
                assert!(turning.a_thread_is_lost());
                assert!(
                    !turning.is_closed(),
                    "a lost thread was reported as a machine that has stopped keeping evidence"
                );
            },
        );

        assert!(record.is_empty());
    }

    /// **The document the invocation offered is reachable and the folder around
    /// it is not**, and when the turn ends the grant goes with it.
    #[test]
    fn what_the_invocation_offered_is_the_only_thing_it_granted() {
        let strings = in_english();
        let mut record = Record::default();
        let (folder, invoice) = a_folder_with_an_invoice("offered");
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut grants = Grants::default();
        let turning = {
            let mut turning = Turning::beginning(
                offering(&invoice),
                "@files",
                hour(),
                &mut grants,
                &mut machine,
            )
            .unwrap();
            assert!(
                turning
                    .reading(
                        "read_file",
                        &[("file", as_given(&invoice))],
                        &grants,
                        noon()
                    )
                    .is_ok()
            );
            let refused = turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(&folder))],
                    &grants,
                    noon(),
                )
                .unwrap_err();
            assert!(refused.was_refused());
            turning
        };

        assert!(turning.ending(&mut grants));
        assert!(grants.is_empty());
        assert!(!grants.permits(&files(), &Ask::path(&invoice), noon()));
        assert_eq!(record.len(), 2);
    }

    /// **A turn cannot begin on a machine with no agent** (ADR 0009), and no
    /// check in this crate says so: beginning one needs the machine's grants,
    /// and a machine where the person declined has none to lend.
    #[test]
    fn a_turn_cannot_begin_where_somebody_declined_an_agent() {
        let mut declined = Agent::declined();
        assert!(
            declined.grants_mut().is_none(),
            "a declined machine lent a grant list to begin a turn with"
        );

        let strings = in_english();
        let mut record = Record::default();
        let (_folder, invoice) = a_folder_with_an_invoice("declined-agent");
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut present = Agent::present();
        let turning = Turning::beginning(
            offering(&invoice),
            "@files",
            hour(),
            present.grants_mut().unwrap(),
            &mut machine,
        )
        .unwrap();
        assert_eq!(turning.grantee(), &files());
        assert_eq!(turning.ends(), noon() + hour());
        assert!(turning.granted().is_some());
        assert!(!turning.is_closed());
        assert!(turning.context().document().is_some());
    }

    /// A turn belonging to nobody is not a turn, and the refusal is the grants'
    /// own rather than a sentence this crate wrote.
    #[test]
    fn a_turn_has_an_agent() {
        let strings = in_english();
        let mut record = Record::default();
        let (_folder, invoice) = a_folder_with_an_invoice("anonymous");
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut grants = Grants::default();
        let refused =
            Turning::beginning(offering(&invoice), "  ", hour(), &mut grants, &mut machine)
                .unwrap_err();
        assert_eq!(refused, GrantError::Anonymous);
        assert!(grants.is_empty());
        assert!(record.is_empty(), "a turn that never began wrote an entry");
    }

    /// **A machine that could not is not a refusal.** Asking to read a folder
    /// is granted, resolved and attempted, and the machine says no — so the
    /// entry says it ran, and a security review reading for what was *stopped*
    /// does not find it among the things the grants turned down.
    #[test]
    fn a_machine_that_could_not_is_not_a_refusal() {
        let mut record = Record::default();
        on_a_machine("could-not", &mut record, |turning, grants, folder, _| {
            let could_not = turning
                .reading("read_file", &[("file", as_given(folder))], grants, noon())
                .unwrap_err();
            assert!(!could_not.was_refused(), "{could_not:?}");
            assert!(matches!(
                could_not,
                NotDone::MachineCouldNot(alo_files::Failed::NotAFile { .. })
            ));
        });

        assert_eq!(
            record
                .answering(&Asking::anything().only(Only::Refusals))
                .count(),
            0,
            "the record called a machine that could not a refusal"
        );
        assert_eq!(record.len(), 1);
        assert!(
            record
                .everything()
                .next()
                .is_some_and(|entry| entry.happened().ran())
        );
    }

    /// **A path that is not there is refused by the crate that went looking**,
    /// in that crate's words, and it never reaches the disk-acting half at all.
    ///
    /// Worth its own test because it reads like the one above and is a
    /// different fact: `alo-files` asks the grants about a path *as written*
    /// before it resolves it (item 6), so a refusal cannot tell an agent
    /// whether a file it may not reach exists — and once the grants have said
    /// yes, *there is nothing there* is answered by the resolver rather than by
    /// the machine doing the work.
    #[test]
    fn a_path_that_is_not_there_is_refused_before_anything_is_opened() {
        let mut record = Record::default();
        let said = on_a_machine("gone", &mut record, |turning, grants, folder, _| {
            let missing = folder.join("nothing-here.pdf");
            turning
                .reading("read_file", &[("file", as_given(&missing))], grants, noon())
                .unwrap_err()
                .said(&in_english())
                .into_text()
        });

        assert!(said.contains("there is nothing at"), "{said}");
        assert!(said.contains("nothing-here.pdf"), "{said}");
        assert_eq!(record.len(), 1);
        assert!(
            record
                .everything()
                .next()
                .is_some_and(|entry| entry.happened().was_stopped())
        );
    }

    /// **What a person was told is what the record kept**, one rendering rather
    /// than two: the turn hands the refusal the machine's own strings.
    #[test]
    fn what_a_person_was_told_is_what_the_record_kept() {
        let mut record = Record::default();
        let somewhere_else = a_folder_of_our_own("one-rendering-elsewhere");
        let said = on_a_machine("one-rendering", &mut record, |turning, grants, _, _| {
            turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(&somewhere_else))],
                    grants,
                    noon(),
                )
                .unwrap_err()
                .said(&in_english())
                .into_text()
        });

        let entry = record.everything().next().unwrap();
        assert_eq!(
            entry.happened().why_stopped().map(|why| why.as_str()),
            Some(said.as_str())
        );
    }

    /// **Deciding never depends on a vocabulary having been read.** A machine
    /// whose shell loaded no words at all permits and refuses exactly the same
    /// things, and says so with a marked key rather than with a sentence.
    #[test]
    fn a_machine_that_loaded_no_words_refuses_the_same_things() {
        let mut record = Record::default();
        let strings = Strings::of(Vocabulary::empty());
        let folder = a_folder_of_our_own("no-words");
        let somewhere_else = a_folder_of_our_own("no-words-elsewhere");
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut grants = granting(&[&folder]);
        {
            let mut turning = Turning::beginning(
                Context::at_invocation(noon()),
                "@files",
                hour(),
                &mut grants,
                &mut machine,
            )
            .unwrap();
            assert!(
                turning
                    .reading(
                        "list_folder",
                        &[("folder", as_given(&folder))],
                        &grants,
                        noon()
                    )
                    .is_ok()
            );
            let refused = turning
                .reading(
                    "list_folder",
                    &[("folder", as_given(&somewhere_else))],
                    &grants,
                    noon(),
                )
                .unwrap_err();
            assert!(refused.was_refused());
            assert!(
                refused.said(&strings).is_a_bug(),
                "a vocabulary nobody loaded answered with a sentence"
            );
        }
        assert_eq!(record.len(), 2);
    }
}
