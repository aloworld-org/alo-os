//! The changes waiting for an answer, and the answering of them.
//!
//! Two audiences again, as with [`crate::Grants`], and they owe each other
//! nothing. **The person** reads what is waiting and answers one of them —
//! [`Approvals::waiting_at`], [`Approvals::approve`], [`Approvals::decline`].
//! **The daemon** proposes changes and executes what comes back.
//!
//! One number, one answer. A proposal leaves this list the moment it is
//! answered, so answering it again finds nothing — that is *one approval, one
//! execution* at the list's end, and [`crate::Approved::redeem`] is the same
//! rule at the executor's. Numbers are never reused, so an answer arriving from
//! a panel that was showing yesterday's list cannot land on today's question.
//!
//! `Serialize`, so a pending question can be shown or written down. Not
//! `Deserialize`, and [`crate::Call`] is what makes that so: a question read
//! back off a disk would be one nobody watched being asked, about a machine
//! that has restarted since. Unanswered proposals do not survive a restart, and
//! that is the intended behaviour rather than a limitation.

use std::time::SystemTime;

use serde::Serialize;

use crate::approval::Approved;
use crate::grant::Grantee;
use crate::proposal::Proposal;

/// The number a person answers a proposal by.
///
/// Unique for the life of the list and never reused, for the same reason
/// [`crate::GrantId`] is: an answer to a stale list must fail rather than land
/// somewhere it was not aimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct ProposalId(u64);

impl ProposalId {
    /// The number behind the handle, for showing and recording.
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// A proposal as the list holds it: the question, and the number it is answered
/// by.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Waiting {
    /// What to pass to [`Approvals::approve`] or [`Approvals::decline`].
    pub id: ProposalId,
    /// The question itself.
    pub proposal: Proposal,
}

/// Why a proposal could not be answered.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AnswerError {
    /// A number that is not waiting for an answer.
    #[error(
        "nothing is waiting to be approved under number {number} — it has been answered already, or it was never asked"
    )]
    NothingWaiting {
        /// The number that was answered.
        number: u64,
    },
    /// A question that stood too long.
    #[error("\"{sentence}\" was proposed too long ago to answer — ask again if it is still wanted")]
    Lapsed {
        /// What the question was, since the person is being told to ask again.
        sentence: String,
    },
}

/// Every change waiting for one person to answer it.
#[derive(Debug, Default, Serialize)]
pub struct Approvals {
    /// The next number to hand out. Never goes backwards.
    next: u64,
    /// In the order they were proposed, which is the order they are read in.
    waiting: Vec<Waiting>,
}

impl Approvals {
    /// Put a change to the person, and return the number it is answered by.
    ///
    /// Everything that has to be true of a proposal was checked when it was
    /// built ([`Proposal::checked`]), so this cannot refuse: a question that
    /// should not be asked never becomes a [`Proposal`] at all.
    pub fn propose(&mut self, proposal: Proposal) -> ProposalId {
        let id = ProposalId(self.next);
        self.next = self.next.saturating_add(1);
        self.waiting.push(Waiting { id, proposal });
        id
    }

    /// Approve one proposal, and take it off the list in the same act.
    ///
    /// The grants are not consulted here. They are asked at the moment of
    /// execution instead, in [`Approved::redeem`], so that a grant revoked
    /// between the answer and the action still stops it.
    ///
    /// A lapsed question is taken off the list as it is refused: it was not
    /// answerable, and leaving it there invites the same click again.
    ///
    /// # Errors
    /// [`AnswerError`], saying whether it was already answered or stood too
    /// long.
    pub fn approve(&mut self, id: ProposalId, now: SystemTime) -> Result<Approved, AnswerError> {
        let waiting = self.take(id)?;
        if !waiting.proposal.is_waiting_at(now) {
            return Err(AnswerError::Lapsed {
                sentence: waiting.proposal.sentence().to_owned(),
            });
        }
        Ok(Approved::of(waiting.id, waiting.proposal, now))
    }

    /// Decline one proposal, and give it back so the record can keep what was
    /// declined.
    ///
    /// `None` when there is nothing under that number. Declining takes no
    /// moment: a person may say no to a question that has lapsed, and the
    /// answer is the same.
    pub fn decline(&mut self, id: ProposalId) -> Option<Proposal> {
        self.take(id).ok().map(|waiting| waiting.proposal)
    }

    /// Take a proposal off the list, by number.
    fn take(&mut self, id: ProposalId) -> Result<Waiting, AnswerError> {
        let position = self
            .waiting
            .iter()
            .position(|waiting| waiting.id == id)
            .ok_or(AnswerError::NothingWaiting {
                number: id.as_u64(),
            })?;
        Ok(self.waiting.remove(position))
    }

    /// What is still waiting to be answered, in the order it was asked.
    pub fn waiting_at(&self, now: SystemTime) -> impl Iterator<Item = &Waiting> {
        self.waiting
            .iter()
            .filter(move |waiting| waiting.proposal.is_waiting_at(now))
    }

    /// What one agent is waiting on.
    pub fn waiting_for<'a>(
        &'a self,
        grantee: &'a Grantee,
        now: SystemTime,
    ) -> impl Iterator<Item = &'a Waiting> {
        self.waiting_at(now)
            .filter(move |waiting| waiting.proposal.grantee() == grantee)
    }

    /// One waiting proposal, by number, whether or not it has lapsed.
    #[must_use]
    pub fn of(&self, id: ProposalId) -> Option<&Waiting> {
        self.waiting.iter().find(|waiting| waiting.id == id)
    }

    /// Drop the questions nobody answered in time, and say how many went.
    ///
    /// Housekeeping only: a lapsed question cannot be approved whether or not
    /// this has been called.
    pub fn forget_lapsed(&mut self, now: SystemTime) -> usize {
        let before = self.waiting.len();
        self.waiting
            .retain(|waiting| waiting.proposal.is_waiting_at(now));
        before.saturating_sub(self.waiting.len())
    }

    /// How many proposals are on the list, lapsed ones included.
    #[must_use]
    pub fn len(&self) -> usize {
        self.waiting.len()
    }

    /// Whether nothing is waiting at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::call::Call;
    use crate::grants::Grants;
    use crate::test_calls::{archiving_april, archiving_march, files, granting_both, hour, noon};

    fn proposing(call: &Call, grants: &Grants) -> Proposal {
        Proposal::checked(call, &files(), grants, noon(), hour()).unwrap()
    }

    fn one_waiting() -> (Approvals, ProposalId) {
        let grants = granting_both();
        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        (approvals, id)
    }

    /// **One approval, one execution.** The list has nothing left to answer the
    /// second time, so an approval cannot be replayed into a second action.
    #[test]
    fn an_approval_cannot_be_replayed() {
        let (mut approvals, id) = one_waiting();
        assert!(approvals.approve(id, noon()).is_ok());
        let err = approvals.approve(id, noon()).unwrap_err();
        assert_eq!(err, AnswerError::NothingWaiting { number: 0 });
        assert!(err.to_string().contains("answered already"), "{err}");
        assert!(approvals.is_empty());
    }

    /// Approving nothing runs nothing. With an empty list there is no number
    /// that answers, and no [`Approved`] can be had at all.
    #[test]
    fn approving_nothing_runs_nothing() {
        let (mut approvals, id) = one_waiting();
        assert!(approvals.decline(id).is_some());
        assert!(approvals.is_empty());
        assert_eq!(approvals.waiting_at(noon()).count(), 0);
        assert!(matches!(
            approvals.approve(id, noon()),
            Err(AnswerError::NothingWaiting { .. })
        ));
    }

    /// An approval answers the proposal it names and no other. The one that
    /// was not answered is still waiting, unchanged.
    #[test]
    fn an_approval_answers_one_proposal_and_leaves_the_others() {
        let grants = granting_both();
        let mut approvals = Approvals::default();
        let march = approvals.propose(proposing(&archiving_march(), &grants));
        let april = approvals.propose(proposing(&archiving_april(), &grants));

        let approved = approvals.approve(march, noon()).unwrap();
        assert!(approved.sentence().contains("march.pdf"));
        assert!(!approved.sentence().contains("april.pdf"));

        let left: Vec<_> = approvals.waiting_at(noon()).collect();
        assert_eq!(left.len(), 1);
        assert_eq!(left.first().unwrap().id, april);
        assert!(
            approvals
                .of(april)
                .unwrap()
                .proposal
                .sentence()
                .contains("april.pdf")
        );
    }

    /// A question that stood too long cannot be answered, and the refusal says
    /// what it was so the person can ask for it again.
    #[test]
    fn a_question_that_lapsed_cannot_be_answered() {
        let (mut approvals, id) = one_waiting();
        assert_eq!(approvals.waiting_at(noon() + hour()).count(), 0);
        let err = approvals.approve(id, noon() + hour()).unwrap_err();
        assert_eq!(
            err,
            AnswerError::Lapsed {
                sentence: "move /home/anna/Invoices/march.pdf into /home/anna/Archive".to_owned()
            }
        );
        assert!(err.to_string().contains("ask again"), "{err}");
        // It is gone rather than left there for somebody to click again.
        assert!(approvals.is_empty());
    }

    /// Lapsed questions can be swept, and sweeping them changes nothing about
    /// what could be approved.
    #[test]
    fn lapsed_questions_are_swept_without_changing_any_answer() {
        let (mut approvals, id) = one_waiting();
        assert_eq!(approvals.forget_lapsed(noon()), 0);
        assert_eq!(approvals.len(), 1);
        assert_eq!(approvals.forget_lapsed(noon() + hour()), 1);
        assert!(approvals.is_empty());
        assert!(matches!(
            approvals.approve(id, noon()),
            Err(AnswerError::NothingWaiting { .. })
        ));
    }

    /// A number is never reused, so an answer aimed at a question that has gone
    /// cannot land on one asked since.
    #[test]
    fn a_number_is_never_reused() {
        let grants = granting_both();
        let (mut approvals, first) = one_waiting();
        assert!(approvals.approve(first, noon()).is_ok());
        let second = approvals.propose(proposing(&archiving_april(), &grants));
        assert_ne!(first, second);
        assert!(matches!(
            approvals.approve(first, noon()),
            Err(AnswerError::NothingWaiting { .. })
        ));
        assert!(approvals.approve(second, noon()).is_ok());
    }

    /// Declining gives back what was declined, because a refusal is recorded
    /// with what it refused.
    #[test]
    fn declining_gives_back_what_was_declined() {
        let (mut approvals, id) = one_waiting();
        let declined = approvals.decline(id).unwrap();
        assert_eq!(declined.call(), &archiving_march());
        assert_eq!(declined.grantee(), &files());
        assert!(approvals.is_empty());
        assert!(approvals.decline(id).is_none());
    }

    /// The list a person reads: what is waiting, in the order it was asked, and
    /// which agent is waiting on it.
    #[test]
    fn the_list_says_what_is_waiting_and_who_is_waiting_on_it() {
        let grants = granting_both();
        let mut approvals = Approvals::default();
        approvals.propose(proposing(&archiving_march(), &grants));
        approvals.propose(proposing(&archiving_april(), &grants));

        let waiting: Vec<_> = approvals.waiting_at(noon()).collect();
        assert_eq!(waiting.len(), 2);
        assert!(
            waiting
                .first()
                .unwrap()
                .proposal
                .sentence()
                .contains("march.pdf")
        );
        assert_eq!(approvals.waiting_for(&files(), noon()).count(), 2);
        assert_eq!(
            approvals
                .waiting_for(&Grantee::named("@mail"), noon())
                .count(),
            0
        );
    }

    /// What is waiting can be shown and written down, sentence and all.
    #[test]
    fn what_is_waiting_can_be_written_down() {
        let (approvals, _) = one_waiting();
        let written = serde_json::to_string(&approvals).unwrap();
        assert!(written.contains("move_file"), "{written}");
        assert!(
            written.contains("move /home/anna/Invoices/march.pdf into"),
            "{written}"
        );
    }
}
