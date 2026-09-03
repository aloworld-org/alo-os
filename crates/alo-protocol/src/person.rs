//! What a person answers, through the shell in front of them.
//!
//! Two requests, and both of them are the same act: answering a change that was
//! put to them in one sentence. ADR 0001 §5 says a person approves a sentence
//! rather than a session, so there is nothing here that approves more than one
//! thing, nothing that approves everything from an agent, and nothing that
//! stands until it is revoked.
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
//! # There is no `waiting` here yet
//!
//! What a shell **draws** — the changes waiting and the sentence for each — is
//! something the daemon answers rather than something a person asks for, and
//! what comes back over this socket is the other half of item 21. This list is
//! what a person *sends*, and a person sends an answer.

use crate::asked::Asked;
use crate::frame;
use crate::refusing::NotUnderstood;

/// One answer a person gave to a change that was put to them.
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

    /// The number they answered.
    ///
    /// A number and not a handle: see this file's header.
    #[must_use]
    pub fn number(&self) -> u64 {
        match self {
            Self::Approve { number } | Self::Decline { number } => *number,
        }
    }

    /// Whether they said yes.
    #[must_use]
    pub fn is_yes(&self) -> bool {
        matches!(self, Self::Approve { .. })
    }
}

impl From<FromAPerson> for Asked {
    fn from(answered: FromAPerson) -> Self {
        match answered {
            FromAPerson::Approve { number } => Self::Approve { number },
            FromAPerson::Decline { number } => Self::Decline { number },
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

    /// The two, off the wire.
    #[test]
    fn the_two_a_person_may_answer_read_back() {
        let yes = FromAPerson::read(r#"{"format":1,"asks":{"approve":{"number":7}}}"#).unwrap();
        assert_eq!(yes, FromAPerson::Approve { number: 7 });
        assert!(yes.is_yes());
        assert_eq!(yes.number(), 7);

        let no = FromAPerson::read(r#"{"format":1,"asks":{"decline":{"number":7}}}"#).unwrap();
        assert!(!no.is_yes());
        assert_eq!(no.number(), 7);
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
        assert_eq!(nothing_waiting.number(), 9999);
    }

    /// A shell and a daemon built from this crate cannot disagree about the
    /// format.
    #[test]
    fn what_a_person_writes_this_crate_reads_back() {
        for answered in [
            FromAPerson::Approve { number: 1 },
            FromAPerson::Decline { number: 2 },
        ] {
            let written = answered.written().unwrap();
            assert_eq!(FromAPerson::read(&written).unwrap(), answered);
        }
    }
}
