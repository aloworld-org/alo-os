//! What the daemon says back to the person's shell.
//!
//! Four answers to three requests: what a change did once they approved it,
//! that a change they declined is written down, everything still waiting for
//! them, and — for any of the three — the refusal in the language they read.
//!
//! # A person is never handed a model's answer here
//!
//! It costs nothing to leave out and it keeps one thing straight: a question
//! belongs to the turn that asked it, and the answer comes back on the
//! connection the question went out on. A shell that could be handed one would
//! be a second place an answer can appear, and *where the answer came from is
//! said where the answer appears* would then be a promise made in two places.
//!
//! # What is waiting is a read, and it is the person's
//!
//! [`ToAPerson::waiting`] is the answer to the one request item 21b added:
//! `waiting`, on the person's door. It is what a shell draws — the number and
//! the sentence for each change — and it is on this side because ADR 0001 §5
//! puts the answering there. An agent asking for it is
//! `NotUnderstood::NotForAnAgent`, in the same words as an agent trying to
//! approve something.

use std::time::SystemTime;

use alo_capability::Waiting;
use alo_files::Answer;
use alo_strings::{Said, Strings};

use crate::done::Done;
use crate::frame;
use crate::refusing::NotUnderstood;
use crate::standing::Standing;
use crate::told::Told;
use crate::wording::Wording;

/// One thing the daemon told a person's shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToAPerson {
    /// What the change they approved did.
    Did(Done),
    /// Everything they have been asked and have not answered.
    Waiting {
        /// In the order they were proposed, which is the order they are read
        /// in.
        changes: Vec<Standing>,
    },
    /// The change they declined is written down, and nothing ran.
    Declined,
    /// It did not happen, and this is what to say about it.
    Refused(Wording),
}

impl ToAPerson {
    /// What the change they approved did.
    #[must_use]
    pub fn did(answer: &Answer) -> Self {
        Self::Did(Done::of(answer))
    }

    /// Everything waiting for them, in the order it was proposed.
    ///
    /// Takes what `alo_capability::Approvals::waiting_at` and
    /// `alo_turn::Turning::waiting_at` hand out, so the list a shell draws is
    /// the list the turn holds rather than one assembled beside it — and every
    /// change in it carries its own sentence.
    #[must_use]
    pub fn waiting<'a>(
        changes: impl Iterator<Item = &'a Waiting>,
        strings: &Strings,
        now: SystemTime,
    ) -> Self {
        Self::Waiting {
            changes: changes
                .map(|waiting| Standing::of(waiting, strings, now))
                .collect(),
        }
    }

    /// It did not happen, in the words of whoever refused it.
    #[must_use]
    pub fn refused(said: &Said) -> Self {
        Self::Refused(Wording::of(said))
    }

    /// Read one line as something the daemon told a person's shell.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's five refusals, and
    /// [`NotUnderstood::NotAnAnswerForAPerson`] for a well-formed answer that
    /// is only ever given to an agent.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::reply(line)? {
            Told::Did(done) => Ok(Self::Did(done)),
            Told::Waiting { changes } => Ok(Self::Waiting { changes }),
            Told::Declined {} => Ok(Self::Declined),
            Told::Refused(wording) => Ok(Self::Refused(wording)),
            Told::Proposed(_) | Told::Answered { .. } => Err(NotUnderstood::NotAnAnswerForAPerson),
        }
    }

    /// This answer as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`. See `frame.rs` for why it is handed back rather
    /// than swallowed, and for the bound a client holds this to when it reads
    /// one.
    pub fn written(&self) -> Result<String, serde_json::Error> {
        frame::spoken(self.clone().into())
    }

    /// What the machine did, when it did something.
    #[must_use]
    pub fn done(&self) -> Option<&Done> {
        match self {
            Self::Did(done) => Some(done),
            _ => None,
        }
    }

    /// What is waiting for them, when that is what they asked.
    #[must_use]
    pub fn changes(&self) -> Option<&[Standing]> {
        match self {
            Self::Waiting { changes } => Some(changes),
            _ => None,
        }
    }

    /// What to say to the person, when nothing was done.
    #[must_use]
    pub fn refusal(&self) -> Option<&Wording> {
        match self {
            Self::Refused(wording) => Some(wording),
            _ => None,
        }
    }
}

impl From<ToAPerson> for Told {
    fn from(told: ToAPerson) -> Self {
        match told {
            ToAPerson::Did(done) => Self::Did(done),
            ToAPerson::Waiting { changes } => Self::Waiting { changes },
            ToAPerson::Declined => Self::Declined {},
            ToAPerson::Refused(wording) => Self::Refused(wording),
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
    use crate::ToAnAgent;
    use crate::testing::{a_change_waiting, an_hour_in, in_english, the_change, the_moment};
    use crate::words;
    use alo_strings::Filling;

    /// One sentence this crate really declares.
    fn a_sentence() -> Said {
        in_english().say(&words::NOT_READABLE.key(), &Filling::nothing())
    }

    /// The four, written and read back.
    #[test]
    fn the_four_a_person_is_told_read_back() {
        let (approvals, strings) = a_change_waiting();
        for told in [
            ToAPerson::did(&Answer::Renamed(
                "/home/anna/Invoices/march-final.pdf".into(),
            )),
            ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment()),
            ToAPerson::Declined,
            ToAPerson::refused(&a_sentence()),
        ] {
            let written = told.written().unwrap();
            assert_eq!(ToAPerson::read(&written).unwrap(), told, "{written}");
        }
    }

    /// **What a shell draws is the number and the sentence**, one for each
    /// change the person has not answered — which is the request item 21b owed
    /// the person's side, answered in the shape it is drawn in.
    #[test]
    fn what_is_waiting_carries_a_number_and_a_sentence_for_each_change() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment());
        let changes = told.changes().unwrap();
        assert_eq!(changes.len(), 1);
        let only = changes.first().unwrap();
        assert_eq!(only.number(), the_change(&approvals).id.as_u64());
        assert!(only.sentence().text().contains("march-final.pdf"));
        assert_eq!(only.lapses_in(), Some(300));
    }

    /// A list with nothing on it is an answer and not an absence: a shell that
    /// could not tell *nothing is waiting* from *the daemon said nothing* would
    /// draw the last list it saw.
    #[test]
    fn nothing_waiting_is_an_answer_of_its_own() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting(approvals.waiting_at(an_hour_in()), &strings, an_hour_in());
        assert_eq!(told.changes().unwrap().len(), 0);
        let written = told.written().unwrap();
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
    }

    /// **A person's screen is not where a model's answer appears**, and neither
    /// is it where a change that is only proposed lands: both are the agent's
    /// side of one turn.
    #[test]
    fn what_is_only_ever_said_to_an_agent_is_refused_here() {
        let (approvals, strings) = a_change_waiting();
        let proposed = ToAnAgent::proposed(the_change(&approvals), &strings, the_moment())
            .written()
            .unwrap();
        assert_eq!(
            ToAPerson::read(&proposed),
            Err(NotUnderstood::NotAnAnswerForAPerson)
        );

        let answered = ToAnAgent::answered("three", &a_sentence(), "mistral-small-latest")
            .written()
            .unwrap();
        assert_eq!(
            ToAPerson::read(&answered),
            Err(NotUnderstood::NotAnAnswerForAPerson)
        );
    }

    /// **Declining carries nothing about why**, because nothing was asked. The
    /// wire has no field for a reason, so nothing can start collecting one.
    #[test]
    fn a_declined_change_says_nothing_about_why() {
        let written = ToAPerson::Declined.written().unwrap();
        assert_eq!(written, r#"{"format":1,"tells":{"declined":{}}}"#);
        assert_eq!(ToAPerson::read(&written).unwrap(), ToAPerson::Declined);
    }

    /// What the change did comes back to the person who approved it, which is
    /// the other half of *a read answers inside the turn*: a change answers to
    /// whoever said yes.
    #[test]
    fn what_an_approved_change_did_comes_back_to_the_person() {
        let told = ToAPerson::did(&Answer::Renamed(
            "/home/anna/Invoices/march-final.pdf".into(),
        ));
        assert_eq!(
            told.done().unwrap(),
            &Done::Renamed {
                now_at: Some("/home/anna/Invoices/march-final.pdf".to_owned())
            }
        );
        assert!(told.changes().is_none());
        assert!(told.refusal().is_none());
    }
}
