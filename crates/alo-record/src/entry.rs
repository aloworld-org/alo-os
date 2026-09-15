//! One entry: a moment, and what happened at it.
//!
//! There is one constructor for each point in the journey ADR 0001 §5 and §7
//! describe, and between them they cover all of it: a call that never formed, a
//! change that was never put to anybody, a change a person declined, a call
//! refused at the moment it would have run, a call that ran, and a question
//! answered on this machine.
//!
//! **Egress is the other half, and it is [`crate::departed`]'s.** What an agent
//! caused to leave is decided by a different crate, guaranteed by a different
//! type and changed for different reasons, so it is a file of its own — law 4.
//! Between the two there is nothing an agent causes that goes unrecorded.
//!
//! **Nothing here reads the clock**, as in [`alo_capability`] and for the same
//! reason: the moment is passed in, so a record can be written about a moment
//! that has been decided once rather than about whenever the writing happened.
//! [`Entry::ran`] is the exception that proves it — it takes its moment from
//! the [`Authorised`] itself, because the moment that matters is the one the
//! grants were asked at, and that moment already exists.
//!
//! **Every constructor that writes down a call takes the strings** (item 9g).
//! A call carries what names the sentence a person approves and the values that
//! fill it, and the words are asked for here, once, with the vocabulary that
//! person reads — the rule [`Entry::refused`] has kept since 9e, now covering
//! what ran as well as what did not.
//!
//! # Where it came from, when it came from another machine
//!
//! ADR 0003: *B records it, with A named as the origin.* A verb that arrived
//! from a paired machine walks the same journey as one an agent on this machine
//! asked for — the same constructors above, the same four answers — and one
//! thing more is true of it, which is which machine it came from. That is
//! [`Entry::from_another_machine`], a stamp put on an entry already made rather
//! than a second set of constructors, so that a remote turn cannot write down a
//! kind of event a local one could not: what it adds is an origin and nothing
//! else. [`Entry::origin`] answers it, and answers it for the question a
//! paired machine put to this one's models as well, which carried its origin
//! inside [`Happened::AnsweredForAnotherMachine`] before there was a field for
//! it.
//!
//! **The name is the one this machine's person gave the other when they
//! paired**, for the reason that variant gives: it is the only name here anybody
//! on this machine has reason to trust. It is additive, and `format` stays `1`:
//! an entry with nowhere to have come from carries no field, and a reader that
//! has never heard of one ignores it.

use std::time::SystemTime;

use alo_capability::{Authorised, Call, Grantee, Proposal, Refused};
use alo_strings::Strings;
use serde::{Deserialize, Serialize};

use crate::happened::{Happened, Stopped};
use crate::line::Line;
use crate::what::What;

/// One thing that happened, and when.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// The moment it happened.
    at: SystemTime,
    /// What happened.
    happened: Happened,
    /// The machine it was caused from, when it was caused from another one
    /// (ADR 0003), by the name this machine's person gave it when they paired.
    ///
    /// Absent — not present and empty — for everything caused on this
    /// machine, so a record written before there was such a thing as a remote
    /// turn reads back exactly as it was written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    origin: Option<Line>,
}

impl Entry {
    /// A moment and what happened at it.
    ///
    /// Crate-private, and shared with [`crate::departed`] so the egress
    /// constructors can live in a file of their own. There is no public way to
    /// make an arbitrary entry: every public constructor is a named point in
    /// the journey, which is what stops a record being handed something that
    /// never happened.
    pub(crate) fn new(at: SystemTime, happened: Happened) -> Self {
        Self {
            at,
            happened,
            origin: None,
        }
    }

    /// This entry, as something a paired machine caused (ADR 0003).
    ///
    /// `machine` is the name this machine's person gave the other when they
    /// paired with it — their own word for it, and the only name here anybody
    /// on this machine has reason to trust. It goes through [`Line`] like every
    /// other sentence the record keeps.
    ///
    /// A stamp rather than a constructor, on purpose: everything else about the
    /// entry was decided by the constructor that made it, from the same values
    /// a local turn would have handed over, and what a remote turn can add is
    /// where it came from and nothing beside it. The name a person gave a
    /// machine is not an authority and does not become one here — the entry's
    /// `agent` is still whose grants on **this** machine permitted or refused
    /// the call.
    #[must_use]
    pub fn from_another_machine(mut self, machine: &str) -> Self {
        self.origin = Some(Line::of(machine));
        self
    }

    /// A verb ran.
    ///
    /// The moment comes from the authorisation rather than from the caller: it
    /// is the moment the grants were asked, which is the moment the thing was
    /// allowed to happen. A record that stamped its own time would be recording
    /// when it got round to writing.
    #[must_use]
    pub fn ran(authorised: &Authorised, strings: &Strings) -> Self {
        Self::new(
            authorised.at(),
            Happened::Ran {
                agent: Line::of(authorised.under().as_str()),
                what: What::of(authorised.call(), strings),
                from_approval: authorised.from_approval().map(|from| from.as_u64()),
                against: authorised
                    .against()
                    .iter()
                    .map(|grant| grant.as_u64())
                    .collect(),
            },
        )
    }

    /// A call was refused at the moment it would have run.
    ///
    /// This is [`alo_capability::Authorised::read`] and
    /// [`alo_capability::Approved::redeem`] saying no — the grants asked last,
    /// or a change offered where only a read may go. The agent is passed in
    /// because a refusal is not an authority and does not carry one.
    ///
    /// **The strings are passed in rather than the words.** A refusal is a
    /// value until somebody asks it for a sentence, so the record asks it here
    /// with the vocabulary the person in front of the machine reads: what is
    /// written down is what they were told, and it cannot be a sentence about
    /// something else, because a caller has no way to hand one over.
    #[must_use]
    pub fn refused(refused: &Refused, agent: &Grantee, strings: &Strings, at: SystemTime) -> Self {
        Self::stopped(
            refused.call(),
            agent,
            Stopped::AtTheMoment(Line::of(refused.said(strings).text())),
            strings,
            at,
        )
    }

    /// A change was never put to a person.
    ///
    /// A read offered for approval, or a change the grants already refused —
    /// [`alo_capability::ProposalError`]. Nobody was interrupted, which is the
    /// intended behaviour and still a thing that happened.
    #[must_use]
    pub fn never_asked(
        call: &Call,
        agent: &Grantee,
        why: &str,
        strings: &Strings,
        at: SystemTime,
    ) -> Self {
        Self::stopped(
            call,
            agent,
            Stopped::BeforeAnybodyWasAsked(Line::of(why)),
            strings,
            at,
        )
    }

    /// A person declined a change.
    #[must_use]
    pub fn declined(proposal: &Proposal, strings: &Strings, at: SystemTime) -> Self {
        Self::stopped(
            proposal.call(),
            proposal.grantee(),
            Stopped::ByThePerson,
            strings,
            at,
        )
    }

    /// Something that never became a call at all.
    ///
    /// A verb that is not on the list, or an argument that did not survive
    /// validation — [`alo_capability::CallError`]. The verb name and the
    /// refusal came from outside, so both go through [`Line`], and nothing else
    /// about the attempt is kept.
    #[must_use]
    pub fn turned_away(verb: &str, why: &str, agent: &Grantee, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::TurnedAway {
                agent: Line::of(agent.as_str()),
                verb: Line::of(verb),
                why: Line::of(why),
            },
        )
    }

    /// A question was answered on this machine (ADR 0008).
    ///
    /// Who asked, and when. What was asked is not passed in and there is no
    /// field for it.
    ///
    /// **There is no source to give**, because the only source this constructor
    /// describes is this machine. A question answered anywhere else left the
    /// machine, and what left is [`Entry::left`] — which can only be made from
    /// a departure the indicator showed. That is what stops the record being
    /// able to say an answer came from a provider while saying nothing left.
    #[must_use]
    pub fn answered_here(agent: &Grantee, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::AnsweredHere {
                agent: Line::of(agent.as_str()),
            },
        )
    }

    /// A question from a paired machine, answered on this one.
    ///
    /// What the machine down the corridor writes down when it is the one with
    /// the GPU in it. `machine` is the name this machine paired the other
    /// under — its own person's word for it, which is the only name here
    /// anybody on this machine has reason to trust.
    ///
    /// **No agent, and no question.**
    /// [`Happened::AnsweredForAnotherMachine`] has both reasons, which are
    /// different reasons for the same kind of absence.
    #[must_use]
    pub fn answered_for(machine: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::AnsweredForAnotherMachine {
                origin: Line::of(machine),
            },
        )
    }

    /// A question that was refused before it was put anywhere.
    ///
    /// The other ending of [`Entry::answered_here`]'s event. `why` is the
    /// sentence the person was shown, handed in already rendered so that the
    /// record and the screen cannot become two accounts of one moment — this
    /// does not word anything and has no `Strings` to word it with.
    ///
    /// **What was asked is not passed in and there is no field for it**, as
    /// above. Neither is an endpoint: `why` names the source somebody
    /// configured, which is what they read, and nothing here adds an address
    /// beside it.
    ///
    /// It goes through [`Line`] like every other sentence the record keeps, so
    /// a record stays one line per entry however it was worded.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision, and states what an older
    /// reader does with a tag it has never heard of.
    #[must_use]
    pub fn never_put_anywhere(agent: &Grantee, why: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::NeverPutAnywhere {
                agent: Line::of(agent.as_str()),
                why: Line::of(why),
            },
        )
    }

    /// This machine was asked to read the person's grants again, and did not.
    ///
    /// `why` is the sentence the person was shown, handed in already rendered
    /// so that the record and the screen cannot become two accounts of one
    /// moment — this does not word anything and has no `Strings` to word it
    /// with, exactly as [`Entry::never_put_anywhere`] does not.
    ///
    /// **No agent is passed in and there is no field for one.** Making,
    /// revoking and re-reading a grant are the person's acts; the one way an
    /// agent reaches this is by sending the message on its own door, and that is
    /// refused before anything is read. See [`Happened::GrantsNotReadAgain`].
    ///
    /// **Nothing of a grant is passed in either** — no path, no reach, no
    /// duration, no handle. Nothing was read, so there is nothing read to keep.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn the_grants_were_not_read_again(why: &str, at: SystemTime) -> Self {
        Self::new(at, Happened::GrantsNotReadAgain { why: Line::of(why) })
    }

    /// A pairing with `with` was kept on this machine.
    ///
    /// Written by whatever holds this machine's pairings — the service that
    /// owns the port — at the one moment there is one value to write it
    /// from: the second confirmation arrived and the pairing became a row.
    /// `with` is the other machine's identity as the pairing spells it; no
    /// person's name for it exists yet, and none is invented here. See
    /// [`Happened::Paired`] for why it names no agent.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn paired(with: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::Paired {
                with: Line::of(with),
            },
        )
    }

    /// This machine started on build `to`, having been running build `from`.
    ///
    /// Both are content digests as the base reports them. They go through
    /// [`Line`] like every other string the record keeps; which text is a
    /// digest is decided by `alo-keeping-up`, which this crate does not
    /// depend on. See [`Happened::Updated`] for why it names no agent and when
    /// it is written.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn updated(from: &str, to: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::Updated {
                from: Line::of(from),
                to: Line::of(to),
            },
        )
    }

    /// This machine started on build `to`, the one it ran before, having been
    /// running build `from` — because the person asked it to go back.
    ///
    /// Both are content digests as the base reports them, through [`Line`].
    /// See [`Happened::RolledBack`] for how it is told apart from an update and
    /// why it names no agent.
    ///
    /// Additive; `format` stays `1`.
    #[must_use]
    pub fn rolled_back(from: &str, to: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::RolledBack {
                from: Line::of(from),
                to: Line::of(to),
            },
        )
    }

    /// The person opened `workspace`, which answered at `answers_at` when the
    /// link was looked at.
    ///
    /// Written by the service holding this machine at the moment it hands the
    /// address to the person's session, and before it does, so an address is
    /// never handed anywhere the record does not say. `workspace` is the
    /// identity it was found by and `answers_at` the address measured, as
    /// `address:port`. See [`Happened::WorkspaceOpened`] for why it names no
    /// agent and is not egress.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn a_workspace_was_opened(workspace: &str, answers_at: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::WorkspaceOpened {
                workspace: Line::of(workspace),
                answers_at: Line::of(answers_at),
            },
        )
    }

    /// There was no boundary to run this turn's work inside, so nothing ran.
    ///
    /// `why` is the sentence the person was shown, handed in already rendered
    /// for the reason [`Entry::never_put_anywhere`] gives: the record and the
    /// screen cannot become two accounts of one moment. `machine` is what the
    /// machine said about its own boundary — which pin was gone, which map
    /// was not the one — in the words whoever administers it reads, and it is
    /// kept because that is the fact a review wants and the person's sentence
    /// deliberately does not carry.
    ///
    /// **No call is passed in and there is no field for one.** A file verb
    /// had one and a question had none; [`Happened::NotBounded`] says why the
    /// shape holds neither.
    ///
    /// Additive; `format` stays `1`. `docs/contracts/record-file.md`'s *a new
    /// kind of `happened` is additive* is the decision.
    #[must_use]
    pub fn not_bounded(agent: &Grantee, why: &str, machine: &str, at: SystemTime) -> Self {
        Self::new(
            at,
            Happened::NotBounded {
                agent: Line::of(agent.as_str()),
                why: Line::of(why),
                machine: Line::of(machine),
            },
        )
    }

    /// A properly formed call that was stopped somewhere.
    ///
    /// Private because *where* it was stopped is not a caller's choice to make
    /// freely: each of the three has a constructor above that can only be
    /// reached from the point in the journey it describes.
    fn stopped(
        call: &Call,
        agent: &Grantee,
        how: Stopped,
        strings: &Strings,
        at: SystemTime,
    ) -> Self {
        Self::new(
            at,
            Happened::Stopped {
                agent: Line::of(agent.as_str()),
                what: What::of(call, strings),
                how,
            },
        )
    }

    /// When it happened.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }

    /// What happened.
    #[must_use]
    pub fn happened(&self) -> &Happened {
        &self.happened
    }

    /// Whose authority it was — `None` when nobody's was.
    ///
    /// Every entry an agent caused answers this. The one that does not is the
    /// machine's own errand, and `None` is the answer rather than a gap in it:
    /// see [`crate::happened`].
    #[must_use]
    pub fn agent(&self) -> Option<&Line> {
        self.happened.agent()
    }

    /// What ran or would have run — absent when nothing ever became a call.
    #[must_use]
    pub fn what(&self) -> Option<&What> {
        self.happened.what()
    }

    /// Which machine this was caused from, when it was caused from another one
    /// — `None` for everything caused on this machine.
    ///
    /// One question for both ways a paired machine reaches this one: a verb it
    /// asked for, stamped with [`Entry::from_another_machine`], and a question
    /// it put to this machine's models, which
    /// [`Happened::AnsweredForAnotherMachine`] carries the origin of itself. A
    /// reader asking *what did other machines cause here* should not have to
    /// know that the two arrived through different doors.
    #[must_use]
    pub fn origin(&self) -> Option<&Line> {
        self.origin.as_ref().or(match &self.happened {
            Happened::AnsweredForAnotherMachine { origin } => Some(origin),
            _ => None,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::test_calls::{
        archiving_march, files, granting, granting_both, hour, listing_invoices, mail, noon,
        proposing,
    };
    use crate::testing::in_english;
    use alo_capability::{Approvals, Grants};

    /// What ran, under whose authority, from which approval, against which
    /// grant — ADR 0001 §7's four answers, written down without the record
    /// working any of them out for itself.
    #[test]
    fn an_execution_is_recorded_with_everything_the_adr_asks_for() {
        let grants = granting_both();
        let held: Vec<_> = grants
            .active_at(noon())
            .map(|held| held.id.as_u64())
            .collect();
        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        let approved = approvals.approve(id, noon()).unwrap();
        let authorised = approved.redeem(&grants, noon()).unwrap();

        let entry = Entry::ran(&authorised, &in_english());
        assert_eq!(entry.at(), noon());
        assert!(entry.agent().is_some_and(|agent| agent.is("@files")));
        assert_eq!(entry.happened().from_approval(), Some(id.as_u64()));
        assert_eq!(entry.happened().against(), held);
        assert!(
            entry
                .what()
                .is_some_and(|what| what.touched("/home/anna/Archive"))
        );
    }

    /// A read ran under nobody's approval, and the record says so rather than
    /// leaving the question open.
    #[test]
    fn a_read_is_recorded_with_no_approval_because_it_needed_none() {
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
        let entry = Entry::ran(&authorised, &in_english());
        assert!(entry.happened().ran());
        assert_eq!(entry.happened().from_approval(), None);
        assert_eq!(entry.happened().against().len(), 1);
    }

    /// **The refusal that matters most.** The agent asked for something outside
    /// its grant and was stopped, and the record says what it tried — a record
    /// that only counted refusals could not answer a security review at all.
    #[test]
    fn a_refusal_is_recorded_with_what_was_refused() {
        let refused = Authorised::read(
            &listing_invoices(),
            &files(),
            &granting(&["/home/anna/Taxes"]),
            noon(),
        )
        .unwrap_err();
        let entry = Entry::refused(&refused, &files(), &in_english(), noon());
        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().ran());
        assert!(
            entry
                .what()
                .is_some_and(|what| what.touched("/home/anna/Invoices"))
        );
        let how = entry.happened().stopped();
        assert!(
            how.and_then(Stopped::why)
                .is_some_and(|why| why.as_str().contains("has not been granted")),
            "{how:?}"
        );
    }

    /// A grant revoked between the approval and the execution stops it, and the
    /// record keeps that as the last moment rather than as an argument nobody
    /// had.
    #[test]
    fn a_grant_that_went_away_is_recorded_as_the_last_moment() {
        let mut grants = granting_both();
        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        let approved = approvals.approve(id, noon()).unwrap();
        assert_eq!(grants.revoke_everything_for(&files()), 2);

        let refused = approved.redeem(&grants, noon()).unwrap_err();
        let entry = Entry::refused(&refused, &files(), &in_english(), noon());
        assert!(matches!(
            entry.happened().stopped(),
            Some(Stopped::AtTheMoment(_))
        ));
    }

    /// A change the grants already refuse is never put to a person, and that is
    /// still something that happened to the machine.
    #[test]
    fn a_change_nobody_was_asked_about_is_recorded_as_such() {
        let half = granting(&["/home/anna/Invoices"]);
        let why = Proposal::checked(&archiving_march(), &files(), &half, noon(), hour())
            .unwrap_err()
            .said(&in_english())
            .into_text();
        let entry = Entry::never_asked(&archiving_march(), &files(), &why, &in_english(), noon());
        assert!(matches!(
            entry.happened().stopped(),
            Some(Stopped::BeforeAnybodyWasAsked(_))
        ));
        assert!(entry.happened().was_stopped());
    }

    /// A person saying no is recorded, and nothing is kept about why. "No" is
    /// the whole answer.
    #[test]
    fn a_person_saying_no_is_recorded_without_a_reason() {
        let grants = granting_both();
        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        let declined = approvals.decline(id).unwrap();
        let entry = Entry::declined(&declined, &in_english(), noon());
        assert_eq!(entry.happened().stopped(), Some(&Stopped::ByThePerson));
        assert_eq!(
            entry.happened().stopped().and_then(Stopped::why),
            None,
            "a person is not asked to justify saying no"
        );
        assert!(entry.agent().is_some_and(|agent| agent.is("@files")));
    }

    /// A verb that is not on the list never became a call, so nothing about its
    /// arguments is kept — and the name it asked under cannot rewrite the
    /// record it appears in.
    #[test]
    fn something_that_never_became_a_call_keeps_no_arguments() {
        let entry = Entry::turned_away(
            "delete_everything\u{1b}[2K",
            "there is no verb called delete_everything",
            &files(),
            noon(),
        );
        assert!(entry.what().is_none());
        assert!(entry.happened().was_stopped());
        assert_eq!(entry.happened().stopped(), None);
        let written = serde_json::to_string(&entry).unwrap();
        assert!(!written.contains('\u{1b}'), "{written}");
    }

    /// ADR 0008: that a question was answered here is recorded. **What was
    /// asked is not**, and there is no field it could go in — a record that
    /// kept the questions would be a transcript of everything a person said to
    /// their machine.
    #[test]
    fn a_question_answered_here_is_recorded_and_the_question_is_not() {
        let entry = Entry::answered_here(&mail(), noon());
        assert!(entry.agent().is_some_and(|agent| agent.is("@mail")));
        assert!(entry.what().is_none());

        // An answer given here never left, so there is nothing for law 1's
        // question to find — the zero-egress claim as an absence rather than as
        // a counter that reads zero.
        assert!(!entry.happened().caused_egress());
        assert_eq!(entry.happened().destination(), None);

        // There is nowhere in an entry for a question to be, so an entry about
        // an answer is the moment and the agent, and nothing else.
        let written = serde_json::to_string(&entry).unwrap();
        for question in ["What is in", "the contract", "Northstar"] {
            assert!(!written.contains(question), "{written}");
        }
    }

    /// An entry outlives the session that wrote it, so it has to survive being
    /// written down and read back.
    #[test]
    fn an_entry_survives_being_written_down_and_read_back() {
        let entry = Entry::declined(
            &proposing(&archiving_march(), &granting_both()),
            &in_english(),
            noon() + hour(),
        );
        let written = serde_json::to_string(&entry).unwrap();
        let read = serde_json::from_str::<Entry>(&written).ok();
        assert_eq!(read.as_ref(), Some(&entry), "{written}");
        assert_eq!(read.map(|read| read.at()), Some(noon() + hour()));
    }

    /// A record is evidence, not an instruction: nothing an entry holds is a
    /// grant, an approval or anything that could run.
    #[test]
    fn nothing_read_back_out_of_a_record_can_be_acted_on() {
        let grants = Grants::default();
        let entry = Entry::never_asked(&archiving_march(), &files(), "no", &in_english(), noon());
        let written = serde_json::to_string(&entry).unwrap();
        let read = serde_json::from_str::<Entry>(&written).unwrap();
        // What comes back is words and numbers. There is no method on it that
        // returns a Call, an Approved or an Authorised, so the only way to run
        // the same thing again is to go round the whole journey — which, with
        // no grants, refuses.
        assert!(read.what().is_some_and(|what| what.verb().is("move_file")));
        assert!(!archiving_march().permitted_by(&grants, &files(), noon()));
    }

    /// **A turn the machine would not run without a boundary is written
    /// down**, as a refusal that is nobody's saying no: it counts as stopped,
    /// it was stopped at no point in a call's journey, it names the agent, and
    /// it carries both sentences — the person's, and the machine's own about
    /// which pin was gone.
    #[test]
    fn a_turn_with_no_boundary_is_recorded_as_the_machine_refusing() {
        let entry = Entry::not_bounded(
            &files(),
            "nothing was done: this machine cannot hold an agent inside what you granted it",
            "the boundary is not held on file_open: there is no pin at /sys/fs/bpf/alo/file_open",
            noon(),
        );

        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().ran());
        assert_eq!(entry.happened().stopped(), None);
        assert!(entry.agent().is_some_and(|agent| agent.is("@files")));
        assert_eq!(entry.what(), None);
        assert!(
            entry
                .happened()
                .why_stopped()
                .is_some_and(|why| why.as_str().starts_with("nothing was done"))
        );
        assert!(matches!(
            entry.happened(),
            Happened::NotBounded { machine, .. } if machine.as_str().contains("file_open")
        ));
        assert_eq!(entry.happened().destination(), None);
        assert!(!entry.happened().caused_egress());

        let written = serde_json::to_string(&entry).unwrap();
        assert!(written.contains("\"not-bounded\""), "{written}");
        let read = serde_json::from_str::<Entry>(&written).unwrap();
        assert_eq!(read, entry);
    }

    /// **A verb from a paired machine is recorded with the origin machine
    /// named** (ADR 0003), and with everything a local entry carries: the same
    /// constructor made it, and the stamp added where it came from and nothing
    /// else.
    #[test]
    fn what_a_paired_machine_caused_is_recorded_with_the_machine_named() {
        let grants = granting(&["/home/anna/Invoices"]);
        let authorised = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
        let here = Entry::ran(&authorised, &in_english());
        assert_eq!(here.origin(), None, "a local entry came from somewhere");

        let from_elsewhere =
            Entry::ran(&authorised, &in_english()).from_another_machine("the reception machine");
        assert!(
            from_elsewhere
                .origin()
                .is_some_and(|origin| origin.is("the reception machine"))
        );
        // Everything else is exactly what the local entry says: the stamp
        // changes where it came from and not what happened.
        assert_eq!(from_elsewhere.at(), here.at());
        assert_eq!(from_elsewhere.happened(), here.happened());
        assert_eq!(from_elsewhere.agent(), here.agent());
        assert_eq!(
            from_elsewhere.happened().against(),
            here.happened().against()
        );
    }

    /// **The origin is kept in the file and comes back**, and an entry with
    /// nowhere to have come from carries no field at all — so a record written
    /// before there was such a thing reads back byte for byte as it was.
    #[test]
    fn where_an_entry_came_from_survives_being_written_down_and_is_absent_otherwise() {
        let here = Entry::answered_here(&mail(), noon());
        let written = serde_json::to_string(&here).unwrap();
        assert!(!written.contains("origin"), "{written}");
        assert_eq!(serde_json::from_str::<Entry>(&written).ok(), Some(here));

        let from_elsewhere =
            Entry::answered_here(&mail(), noon()).from_another_machine("the reception machine");
        let written = serde_json::to_string(&from_elsewhere).unwrap();
        assert!(
            written.contains("\"origin\":\"the reception machine\""),
            "{written}"
        );
        let read = serde_json::from_str::<Entry>(&written).unwrap();
        assert_eq!(read, from_elsewhere);
        assert!(
            read.origin()
                .is_some_and(|origin| origin.is("the reception machine"))
        );
    }

    /// **A question a paired machine put to this one's models has an origin
    /// too**, answered by the same question: a reader should not have to know
    /// that a verb and a question arrived through different doors.
    #[test]
    fn a_question_answered_for_another_machine_answers_where_it_came_from() {
        let entry = Entry::answered_for("the reception machine", noon());
        assert!(
            entry
                .origin()
                .is_some_and(|origin| origin.is("the reception machine"))
        );
    }

    /// The name a person gave a machine is data and goes through [`Line`]: one
    /// carrying a control character cannot rewrite the record it appears in.
    #[test]
    fn the_origin_cannot_carry_a_control_character_into_the_record() {
        let entry = Entry::answered_here(&mail(), noon())
            .from_another_machine("the reception machine\u{1b}[2K");
        let written = serde_json::to_string(&entry).unwrap();
        assert!(!written.contains('\u{1b}'), "{written}");
    }

    /// **A workspace opened by the person is recorded with the identity and
    /// the address measured, under nobody's authority, and is not egress**:
    /// the person opened it, and this machine's service dialled nothing.
    #[test]
    fn a_workspace_opened_by_the_person_names_the_identity_and_the_address_and_no_agent() {
        let entry = Entry::a_workspace_was_opened(
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            "192.168.1.20:8443",
            noon(),
        );

        assert_eq!(entry.at(), noon());
        assert_eq!(entry.agent(), None);
        assert_eq!(entry.origin(), None);
        assert_eq!(entry.what(), None);
        assert!(!entry.happened().ran());
        assert!(!entry.happened().was_stopped());
        assert!(!entry.happened().caused_egress());
        assert_eq!(entry.happened().destination(), None);
        assert!(matches!(
            entry.happened(),
            Happened::WorkspaceOpened { workspace, answers_at }
                if workspace.is("0f1e2d3c4b5a69788796a5b4c3d2e1f0")
                    && answers_at.is("192.168.1.20:8443")
        ));

        let written = serde_json::to_string(&entry).unwrap();
        assert!(
            written.contains(
                r#""workspace-opened":{"workspace":"0f1e2d3c4b5a69788796a5b4c3d2e1f0","answers_at":"192.168.1.20:8443"}"#
            ),
            "{written}"
        );
        assert!(!written.contains("agent"), "{written}");
        assert_eq!(serde_json::from_str::<Entry>(&written).unwrap(), entry);
    }

    /// **The machine updating is written with both builds and no agent**, is
    /// not a departure, and reads back as it was written.
    #[test]
    fn an_update_is_recorded_from_one_build_to_another_with_nobody_behind_it() {
        let from = format!("sha256:{}", "aa".repeat(32));
        let to = format!("sha256:{}", "bb".repeat(32));
        let entry = Entry::updated(&from, &to, noon());
        assert_eq!(entry.at(), noon());
        assert_eq!(entry.agent(), None);
        assert_eq!(entry.origin(), None);
        assert_eq!(entry.what(), None);
        assert!(!entry.happened().ran());
        assert!(!entry.happened().was_stopped());
        assert!(!entry.happened().caused_egress());
        assert_eq!(entry.happened().errand(), None);
        assert_eq!(entry.happened().destination(), None);
        assert!(matches!(
            entry.happened(),
            Happened::Updated { from: was, to: now } if was.is(&from) && now.is(&to)
        ));

        let written = serde_json::to_string(&entry).unwrap();
        assert!(
            written.contains(&format!(r#""updated":{{"from":"{from}","to":"{to}"}}"#)),
            "{written}"
        );
        assert!(!written.contains("agent"), "{written}");
        assert_eq!(serde_json::from_str::<Entry>(&written).unwrap(), entry);
    }

    /// **The machine going back is written with both builds and no agent**, is
    /// not an update and not a departure, and reads back as it was written.
    #[test]
    fn going_back_is_recorded_from_one_build_to_the_one_before_with_nobody_behind_it() {
        let from = format!("sha256:{}", "bb".repeat(32));
        let to = format!("sha256:{}", "aa".repeat(32));
        let entry = Entry::rolled_back(&from, &to, noon());
        assert_eq!(entry.at(), noon());
        assert_eq!(entry.agent(), None);
        assert_eq!(entry.origin(), None);
        assert_eq!(entry.what(), None);
        assert!(!entry.happened().ran());
        assert!(!entry.happened().was_stopped());
        assert!(!entry.happened().caused_egress());
        assert_eq!(entry.happened().errand(), None);
        assert_eq!(entry.happened().destination(), None);
        assert!(matches!(
            entry.happened(),
            Happened::RolledBack { from: was, to: now } if was.is(&from) && now.is(&to)
        ));
        assert_ne!(entry, Entry::updated(&from, &to, noon()));

        let written = serde_json::to_string(&entry).unwrap();
        assert!(
            written.contains(&format!(r#""rolled-back":{{"from":"{from}","to":"{to}"}}"#)),
            "{written}"
        );
        assert!(!written.contains("agent"), "{written}");
        assert_eq!(serde_json::from_str::<Entry>(&written).unwrap(), entry);
    }
}
