//! What the daemon says back to the person's shell.
//!
//! Ten answers to nine requests: what a change did once they approved it,
//! that a change they declined is written down, everything still waiting for
//! them, how much is granted after the machine read its list again, the four
//! about pairing — the proposal waiting with its code, what became of a
//! confirmation, what became of a revocation, and everything paired and
//! waiting — the machine now chosen to answer their questions, and, for any of
//! the nine, the refusal in the language they read.
//!
//! The four about pairing are on this side and no other, for the reason
//! `waiting` is: a pairing is the person's list of which machines may ask this
//! one (ADR 0003), and an agent told what is paired — or handed a code to
//! confirm — would be an agent reading the person's own list.
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
use crate::pairing::{AfterConfirming, AfterRevoking, Paired, WaitingToPair};
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
    /// What is granted was read again, and this is how many grants that came to.
    ///
    /// The answer to `granted` on this door. A count and no list — what is
    /// granted is drawn by the surface that lists it, out of the person's own
    /// file, and a list here would be a second copy of it crossing a socket for
    /// nobody to check against.
    Granted {
        /// How many grants are in force after the reading.
        holding: u64,
    },
    /// It did not happen, and this is what to say about it.
    Refused(Wording),
    /// The pairing they proposed is waiting, with the code to show them.
    Pairing(WaitingToPair),
    /// What became of the confirmation they gave.
    Confirmed {
        /// Waiting for the other person, paired, or paired until a restart.
        became: AfterConfirming,
    },
    /// What became of the revocation they made.
    Revoked {
        /// Revoked, or revoked until a restart.
        became: AfterRevoking,
    },
    /// Everything this machine is paired with, and every proposal waiting.
    Pairings {
        /// In the order they were made.
        paired: Vec<Paired>,
        /// In the order they began waiting.
        waiting: Vec<WaitingToPair>,
    },
    /// The machine they chose now answers their questions, by the identity
    /// written down in their settings.
    ChosenToAnswer {
        /// The machine chosen, by its identity.
        machine: String,
    },
}

impl ToAPerson {
    /// The pairing they proposed, waiting with the code known.
    #[must_use]
    pub const fn pairing(waiting: WaitingToPair) -> Self {
        Self::Pairing(waiting)
    }

    /// What became of their confirmation.
    #[must_use]
    pub const fn confirmed(became: AfterConfirming) -> Self {
        Self::Confirmed { became }
    }

    /// What became of their revocation.
    #[must_use]
    pub const fn revoked(became: AfterRevoking) -> Self {
        Self::Revoked { became }
    }

    /// Everything paired and everything waiting.
    #[must_use]
    pub const fn pairings(paired: Vec<Paired>, waiting: Vec<WaitingToPair>) -> Self {
        Self::Pairings { paired, waiting }
    }

    /// The machine they chose to answer their questions, by its identity.
    #[must_use]
    pub fn chosen_to_answer(machine: &str) -> Self {
        Self::ChosenToAnswer {
            machine: machine.to_owned(),
        }
    }

    /// The machine now answering their questions, when that is what they were
    /// told.
    #[must_use]
    pub fn machine_chosen_to_answer(&self) -> Option<&str> {
        match self {
            Self::ChosenToAnswer { machine } => Some(machine),
            _ => None,
        }
    }

    /// The proposal waiting, when that is what they were told.
    ///
    /// The one answered to `pair`; what `pairings` lists is
    /// [`ToAPerson::waiting_to_pair`].
    #[must_use]
    pub const fn proposed_pairing(&self) -> Option<&WaitingToPair> {
        match self {
            Self::Pairing(waiting) => Some(waiting),
            _ => None,
        }
    }

    /// What became of their confirmation, when that is what they were told.
    #[must_use]
    pub const fn became_of_confirming(&self) -> Option<AfterConfirming> {
        match self {
            Self::Confirmed { became } => Some(*became),
            _ => None,
        }
    }

    /// What became of their revocation, when that is what they were told.
    #[must_use]
    pub const fn became_of_revoking(&self) -> Option<AfterRevoking> {
        match self {
            Self::Revoked { became } => Some(*became),
            _ => None,
        }
    }

    /// What is paired, when that is what they asked.
    #[must_use]
    pub fn paired(&self) -> Option<&[Paired]> {
        match self {
            Self::Pairings { paired, .. } => Some(paired),
            _ => None,
        }
    }

    /// What is waiting to be paired, when that is what they asked.
    #[must_use]
    pub fn waiting_to_pair(&self) -> Option<&[WaitingToPair]> {
        match self {
            Self::Pairings { waiting, .. } => Some(waiting),
            _ => None,
        }
    }

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

    /// Everything waiting for them that a paired machine proposed, in the
    /// order it was proposed, each change saying which machine.
    ///
    /// [`ToAPerson::waiting`] with every change stamped
    /// [`Standing::from_a_machine`]: what `alo_turn::Arriving::waiting_at`
    /// hands out is one machine's changes, and the person answering them is
    /// owed the name (ADR 0003: a change waits for the **receiving** machine's
    /// person, and what they approve is the sentence).
    #[must_use]
    pub fn waiting_from_a_machine<'a>(
        changes: impl Iterator<Item = &'a Waiting>,
        machine: &str,
        strings: &Strings,
        now: SystemTime,
    ) -> Self {
        Self::Waiting {
            changes: changes
                .map(|waiting| Standing::of(waiting, strings, now).from_a_machine(machine))
                .collect(),
        }
    }

    /// What is granted, read again, and how many grants that came to.
    #[must_use]
    pub const fn granted(holding: u64) -> Self {
        Self::Granted { holding }
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
            Told::Granted { holding } => Ok(Self::Granted { holding }),
            Told::Refused(wording) => Ok(Self::Refused(wording)),
            Told::Pairing(waiting) => Ok(Self::Pairing(waiting)),
            Told::Confirmed { became } => Ok(Self::Confirmed { became }),
            Told::Revoked { became } => Ok(Self::Revoked { became }),
            Told::Pairings { paired, waiting } => Ok(Self::Pairings { paired, waiting }),
            Told::ChosenToAnswer { machine } => Ok(Self::ChosenToAnswer { machine }),
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

    /// How many grants are in force, when the machine has just read them again.
    #[must_use]
    pub const fn holding(&self) -> Option<u64> {
        match self {
            Self::Granted { holding } => Some(*holding),
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
            ToAPerson::Granted { holding } => Self::Granted { holding },
            ToAPerson::Refused(wording) => Self::Refused(wording),
            ToAPerson::Pairing(waiting) => Self::Pairing(waiting),
            ToAPerson::Confirmed { became } => Self::Confirmed { became },
            ToAPerson::Revoked { became } => Self::Revoked { became },
            ToAPerson::Pairings { paired, waiting } => Self::Pairings { paired, waiting },
            ToAPerson::ChosenToAnswer { machine } => Self::ChosenToAnswer { machine },
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

    /// The five, written and read back.
    #[test]
    fn the_five_a_person_is_told_read_back() {
        let (approvals, strings) = a_change_waiting();
        for told in [
            ToAPerson::did(&Answer::Renamed(
                "/home/anna/Invoices/march-final.pdf".into(),
            )),
            ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment()),
            ToAPerson::Declined,
            ToAPerson::granted(2),
            ToAPerson::refused(&a_sentence()),
        ] {
            let written = told.written().unwrap();
            assert_eq!(ToAPerson::read(&written).unwrap(), told, "{written}");
        }
    }

    /// **The machine chosen to answer comes back to the person, by its
    /// identity, and never to an agent.**
    #[test]
    fn the_machine_chosen_to_answer_is_told_to_the_person_and_not_to_an_agent() {
        let told = ToAPerson::chosen_to_answer("0f1e2d3c4b5a69788796a5b4c3d2e1f0");
        assert_eq!(
            told.machine_chosen_to_answer(),
            Some("0f1e2d3c4b5a69788796a5b4c3d2e1f0")
        );
        let written = told.written().unwrap();
        assert!(
            written
                .contains(r#""chosen-to-answer":{"machine":"0f1e2d3c4b5a69788796a5b4c3d2e1f0"}"#),
            "{written}"
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );
        assert!(ToAPerson::Declined.machine_chosen_to_answer().is_none());
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

    /// **What a paired machine proposed says which machine on every change**,
    /// and reads back as written.
    #[test]
    fn what_a_paired_machine_proposed_names_the_machine_on_every_change() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAPerson::waiting_from_a_machine(
            approvals.waiting_at(the_moment()),
            "the reception machine",
            &strings,
            the_moment(),
        );
        let changes = told.changes().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(
            changes.first().unwrap().from(),
            Some("the reception machine")
        );
        let written = told.written().unwrap();
        assert!(
            written.contains(r#""from":"the reception machine""#),
            "{written}"
        );
        assert_eq!(ToAPerson::read(&written).unwrap(), told);
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
