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
        Self { at, happened }
    }

    /// A verb ran.
    ///
    /// The moment comes from the authorisation rather than from the caller: it
    /// is the moment the grants were asked, which is the moment the thing was
    /// allowed to happen. A record that stamped its own time would be recording
    /// when it got round to writing.
    #[must_use]
    pub fn ran(authorised: &Authorised) -> Self {
        Self {
            at: authorised.at(),
            happened: Happened::Ran {
                agent: Line::of(authorised.under().as_str()),
                what: What::of(authorised.call()),
                from_approval: authorised.from_approval().map(|from| from.as_u64()),
                against: authorised
                    .against()
                    .iter()
                    .map(|grant| grant.as_u64())
                    .collect(),
            },
        }
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
            at,
        )
    }

    /// A change was never put to a person.
    ///
    /// A read offered for approval, or a change the grants already refused —
    /// [`alo_capability::ProposalError`]. Nobody was interrupted, which is the
    /// intended behaviour and still a thing that happened.
    #[must_use]
    pub fn never_asked(call: &Call, agent: &Grantee, why: &str, at: SystemTime) -> Self {
        Self::stopped(
            call,
            agent,
            Stopped::BeforeAnybodyWasAsked(Line::of(why)),
            at,
        )
    }

    /// A person declined a change.
    #[must_use]
    pub fn declined(proposal: &Proposal, at: SystemTime) -> Self {
        Self::stopped(
            proposal.call(),
            proposal.grantee(),
            Stopped::ByThePerson,
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
        Self {
            at,
            happened: Happened::TurnedAway {
                agent: Line::of(agent.as_str()),
                verb: Line::of(verb),
                why: Line::of(why),
            },
        }
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

    /// A properly formed call that was stopped somewhere.
    ///
    /// Private because *where* it was stopped is not a caller's choice to make
    /// freely: each of the three has a constructor above that can only be
    /// reached from the point in the journey it describes.
    fn stopped(call: &Call, agent: &Grantee, how: Stopped, at: SystemTime) -> Self {
        Self {
            at,
            happened: Happened::Stopped {
                agent: Line::of(agent.as_str()),
                what: What::of(call),
                how,
            },
        }
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

    /// Whose authority it was.
    #[must_use]
    pub fn agent(&self) -> &Line {
        self.happened.agent()
    }

    /// What ran or would have run — absent when nothing ever became a call.
    #[must_use]
    pub fn what(&self) -> Option<&What> {
        self.happened.what()
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

        let entry = Entry::ran(&authorised);
        assert_eq!(entry.at(), noon());
        assert!(entry.agent().is("@files"));
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
        let entry = Entry::ran(&authorised);
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
        let entry = Entry::never_asked(&archiving_march(), &files(), &why, noon());
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
        let entry = Entry::declined(&declined, noon());
        assert_eq!(entry.happened().stopped(), Some(&Stopped::ByThePerson));
        assert_eq!(
            entry.happened().stopped().and_then(Stopped::why),
            None,
            "a person is not asked to justify saying no"
        );
        assert!(entry.agent().is("@files"));
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
        assert!(entry.agent().is("@mail"));
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
        let entry = Entry::never_asked(&archiving_march(), &files(), "no", noon());
        let written = serde_json::to_string(&entry).unwrap();
        let read = serde_json::from_str::<Entry>(&written).unwrap();
        // What comes back is words and numbers. There is no method on it that
        // returns a Call, an Approved or an Authorised, so the only way to run
        // the same thing again is to go round the whole journey — which, with
        // no grants, refuses.
        assert!(read.what().is_some_and(|what| what.verb().is("move_file")));
        assert!(!archiving_march().permitted_by(&grants, &files(), noon()));
    }
}
