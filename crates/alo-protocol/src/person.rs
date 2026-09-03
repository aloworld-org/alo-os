//! What a person's shell sends, on behalf of the person in front of it.
//!
//! Three requests. Two of them are the same act — answering a change that was
//! put to them in one sentence — and ADR 0001 §5 says a person approves a
//! sentence rather than a session, so there is nothing here that approves more
//! than one thing, nothing that approves everything from an agent, and nothing
//! that stands until it is revoked.
//!
//! The third is the one that makes the other two usable: what is waiting.
//!
//! # A number is not a handle
//!
//! Both of these carry a `u64`, and it is deliberately not an
//! `alo_capability::ProposalId`. That type has no public constructor, so one
//! cannot be made off the wire — a number that arrived has to be **found**
//! among the changes actually waiting, and a number naming nothing is
//! `alo_capability::AnswerError::NothingWaiting` rather than something this
//! crate invented. [`FromAPerson::number`] hands the number over and the turn
//! does the finding.
//!
//! It is `alo_capability::GrantId`'s rule met one crate out: an answer to a
//! stale list must fail rather than land somewhere it was not aimed.
//!
//! # `waiting` is a read of the turn, and it is the person's
//!
//! This file used to say there was no `waiting` here *yet*, on the ground that
//! what a shell draws is something the daemon answers rather than something a
//! person asks for. Half of that was right, and the half that was wrong is the
//! half that matters: a daemon answers what it was asked, and a shell that
//! never asked would be drawing whatever it happened to have been told — which
//! is nothing at all if it was started, restarted or attached after the change
//! was proposed.
//!
//! So it is a request, and it is on this door rather than the agent's, because
//! what is waiting is what the **person** has been asked. An agent reaching for
//! it is refused in the same words as an agent trying to approve something,
//! because it is the same fact about the same list.
//! [`crate::ToAPerson::waiting`] is what comes back, and it carries the
//! sentence with every number: a number on its own would be an approval of
//! something nobody read.
//!
//! It carries **nothing** on the way in: no agent, no number, no moment. What
//! is waiting is what this turn has put to this person, and a field would be a
//! way to ask about somebody else's.

use crate::asked::Asked;
use crate::frame;
use crate::refusing::NotUnderstood;

/// One thing a person's shell sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromAPerson {
    /// They approved the change waiting under this number. Worth exactly one
    /// execution, which is `alo_capability::Approved::redeem`'s and not this
    /// crate's.
    Approve {
        /// The number they answered.
        number: u64,
    },
    /// They said no. Nothing is carried about why, because nothing was asked.
    Decline {
        /// The number they answered.
        number: u64,
    },
    /// What is waiting for them to answer.
    ///
    /// A read of the turn rather than an answer to it — see this file's header
    /// — and the one request on either door that carries nothing at all.
    Waiting,
}

impl FromAPerson {
    /// Read one line as something a person answered.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's four refusals, and
    /// [`NotUnderstood::NotForAPerson`] for a well-formed request that only an
    /// agent makes during a turn.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::message(line)? {
            Asked::Approve { number } => Ok(Self::Approve { number }),
            Asked::Decline { number } => Ok(Self::Decline { number }),
            Asked::Waiting {} => Ok(Self::Waiting),
            Asked::Read { .. } | Asked::Propose { .. } | Asked::Ask { .. } => {
                Err(NotUnderstood::NotForAPerson)
            }
        }
    }

    /// This answer as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`, which an answer cannot cause. See
    /// [`crate::frame`] for why it is handed back rather than swallowed.
    pub fn written(&self) -> Result<String, serde_json::Error> {
        frame::line((*self).into())
    }

    /// The number they answered, for the two that answer one.
    ///
    /// A number and not a handle: see this file's header. Nothing for
    /// [`FromAPerson::Waiting`], which answers no change and asks about all of
    /// them.
    #[must_use]
    pub fn number(&self) -> Option<u64> {
        match self {
            Self::Approve { number } | Self::Decline { number } => Some(*number),
            Self::Waiting => None,
        }
    }

    /// Whether they said yes.
    #[must_use]
    pub fn is_yes(&self) -> bool {
        matches!(self, Self::Approve { .. })
    }

    /// Whether this asks about the turn rather than answering something in it.
    ///
    /// A convenience for a daemon choosing what to do next: what is waiting is
    /// read off the turn and changes nothing, so it is the one request on this
    /// door that spends no approval.
    #[must_use]
    pub fn is_a_question_about_the_turn(&self) -> bool {
        matches!(self, Self::Waiting)
    }
}

impl From<FromAPerson> for Asked {
    fn from(answered: FromAPerson) -> Self {
        match answered {
            FromAPerson::Approve { number } => Self::Approve { number },
            FromAPerson::Decline { number } => Self::Decline { number },
            FromAPerson::Waiting => Self::Waiting {},
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The three, off the wire.
    #[test]
    fn the_three_a_persons_shell_may_send_read_back() {
        let yes = FromAPerson::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#).unwrap();
        assert_eq!(yes, FromAPerson::Approve { number: 7 });
        assert!(yes.is_yes());
        assert_eq!(yes.number(), Some(7));
        assert!(!yes.is_a_question_about_the_turn());

        let no = FromAPerson::read(r#"{"format":1,"asks":{"decline":{"number":7}}}"#).unwrap();
        assert!(!no.is_yes());
        assert_eq!(no.number(), Some(7));

        let waiting = FromAPerson::read(r#"{"format":1,"asks":{"waiting":{}}}"#).unwrap();
        assert_eq!(waiting, FromAPerson::Waiting);
        assert!(waiting.is_a_question_about_the_turn());
        assert_eq!(waiting.number(), None);
        assert!(!waiting.is_yes());
    }

    /// **A person's side is not a way in for a verb.** The division goes both
    /// ways: a shell is where a person answers, and a request that runs
    /// something arriving on it is refused rather than carried out on their
    /// behalf.
    #[test]
    fn a_request_an_agent_makes_is_not_something_a_person_sends() {
        for message in [
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
            r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[]}}}"#,
            r#"{"format":1,"asks":{"ask":{"question":"how many?"}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotForAPerson),
                "{message}"
            );
        }
    }

    /// **Nothing approves more than one thing.** An approval is of one
    /// sentence, so there is no shape on the wire for *approve these*, *approve
    /// everything* or *approve whatever this agent asks next*.
    #[test]
    fn nothing_a_person_sends_approves_more_than_one_change() {
        for message in [
            r#"{"format":1,"asks":{"approve":{"numbers":[7,8]}}}"#,
            r#"{"format":1,"asks":{"approve":{"number":7,"and":8}}}"#,
            r#"{"format":1,"asks":{"approve-all":{"agent":"@files"}}}"#,
            r#"{"format":1,"asks":{"approve":{"agent":"@files"}}}"#,
        ] {
            assert_eq!(
                FromAPerson::read(message),
                Err(NotUnderstood::NotReadable),
                "{message}"
            );
        }
    }

    /// A number off the wire is a number, and finding what it names is the
    /// turn's — so there is no way here to make a handle to something that was
    /// never waiting.
    #[test]
    fn a_number_is_carried_and_not_turned_into_a_handle() {
        let nothing_waiting =
            FromAPerson::read(r#"{"format":1,"asks":{"approve":{"number":9999}}}"#).unwrap();
        assert_eq!(nothing_waiting.number(), Some(9999));
    }

    /// A shell and a daemon built from this crate cannot disagree about the
    /// format.
    #[test]
    fn what_a_person_writes_this_crate_reads_back() {
        for answered in [
            FromAPerson::Approve { number: 1 },
            FromAPerson::Decline { number: 2 },
            FromAPerson::Waiting,
        ] {
            let written = answered.written().unwrap();
            assert_eq!(FromAPerson::read(&written).unwrap(), answered);
        }
    }
}
