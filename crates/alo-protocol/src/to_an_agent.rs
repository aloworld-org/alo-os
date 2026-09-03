//! What the daemon says back to an agent, during a turn that is under way.
//!
//! Four answers to three requests: what a read found, the number and sentence a
//! change is waiting under, what a model said, and — for any of the three — the
//! refusal, in the language the person reads.
//!
//! # An agent is never told what its person has in front of them
//!
//! There is no shape here for *what is waiting*, and that is the division
//! [`crate::told`] argues. An agent knows what it proposed; what it must not be
//! handed is the person's own list, and the way to make that true is to have
//! nowhere to put it.
//!
//! # Every refusal in the workspace crosses as one sentence
//!
//! A call that never formed, the grants at the moment of execution, a full
//! disk, a question nothing answered, a turn that stopped because the record
//! could not be written: all of them are `alo_turn::NotDone::said` or
//! `alo_turn::NoAnswer::said`, and all of them arrive here as a
//! [`crate::Wording`]. This crate words none of it — item 9e's rule at the last
//! boundary before a person reads the sentence — and what it adds is the one
//! thing text alone would have lost, which is whether anybody translated it.
//!
//! What is **not** carried is which refusal it was. A client that could branch
//! on the kind of refusal is a client that would, and an agent choosing what to
//! try next from *the grants said no* rather than from the sentence is an agent
//! working around the capability model. What it needs is what a person needs:
//! the sentence.

use alo_files::Answer;
use alo_strings::{Said, Strings};

use crate::done::Done;
use crate::frame;
use crate::refusing::NotUnderstood;
use crate::standing::Standing;
use crate::told::Told;
use crate::wording::Wording;

/// One thing the daemon told an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToAnAgent {
    /// What a read found.
    Did(Done),
    /// A change is waiting for the person under this number.
    Proposed(Standing),
    /// A model answered.
    Answered {
        /// What the model said, in its own words. Never put inside a sentence
        /// of ours: `alo-asking` refuses to compose one out of it, and so does
        /// this.
        text: String,
        /// Where it came from, said in the language the person reads.
        came_from: Wording,
        /// Which model was asked, as it was named when the question was put.
        model: String,
    },
    /// It did not happen, and this is what to say about it.
    Refused(Wording),
}

impl ToAnAgent {
    /// What a read found.
    #[must_use]
    pub fn did(answer: &Answer) -> Self {
        Self::Did(Done::of(answer))
    }

    /// A change is waiting under this number, with the sentence it waits on.
    ///
    /// Takes the change rather than the number, so an answer cannot be composed
    /// without the sentence the person will be asked — [`crate::standing`] is
    /// where that is argued.
    #[must_use]
    pub fn proposed(
        waiting: &alo_capability::Waiting,
        strings: &Strings,
        now: std::time::SystemTime,
    ) -> Self {
        Self::Proposed(Standing::of(waiting, strings, now))
    }

    /// A model answered.
    ///
    /// The provenance is an argument and not an option: `docs/features.md`
    /// promises **where the answer came from is said where the answer appears**,
    /// and this is the last place that promise can be lost. A daemon holding an
    /// `alo_asking::Answer` renders `came_from` in order to make one of these at
    /// all.
    ///
    /// This crate does not depend on `alo-asking`, deliberately: that crate
    /// carries an HTTP client and a TLS stack, and item 4 refused the same
    /// dependency for `alo-record` for the same reason — the crate reading the
    /// untrusted side of a socket should be small enough to audit.
    #[must_use]
    pub fn answered(text: &str, came_from: &Said, model: &str) -> Self {
        Self::Answered {
            text: text.to_owned(),
            came_from: Wording::of(came_from),
            model: model.to_owned(),
        }
    }

    /// It did not happen, in the words of whoever refused it.
    #[must_use]
    pub fn refused(said: &Said) -> Self {
        Self::Refused(Wording::of(said))
    }

    /// Read one line as something the daemon told an agent.
    ///
    /// # Errors
    /// [`NotUnderstood`] — the envelope's five refusals, and
    /// [`NotUnderstood::NotAnAnswerForAnAgent`] for a well-formed answer that
    /// is only ever given to a person's screen.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match frame::reply(line)? {
            Told::Did(done) => Ok(Self::Did(done)),
            Told::Proposed(standing) => Ok(Self::Proposed(standing)),
            Told::Answered {
                text,
                came_from,
                model,
            } => Ok(Self::Answered {
                text,
                came_from,
                model,
            }),
            Told::Refused(wording) => Ok(Self::Refused(wording)),
            Told::Waiting { .. } | Told::Declined {} => Err(NotUnderstood::NotAnAnswerForAnAgent),
        }
    }

    /// This answer as the line that carries it.
    ///
    /// # Errors
    /// A `serde_json::Error`. See [`crate::frame`] for why it is handed back
    /// rather than swallowed, and for the bound a client holds this to when it
    /// reads one.
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

    /// What to say to the person, when nothing was done.
    #[must_use]
    pub fn refusal(&self) -> Option<&Wording> {
        match self {
            Self::Refused(wording) => Some(wording),
            _ => None,
        }
    }

    /// Whether this change is now waiting for a person.
    ///
    /// What an agent stops for: a turn where the next thing is somebody
    /// reading a sentence is not a turn to go on asking in.
    #[must_use]
    pub fn waits_for_a_person(&self) -> bool {
        matches!(self, Self::Proposed(_))
    }
}

impl From<ToAnAgent> for Told {
    fn from(told: ToAnAgent) -> Self {
        match told {
            ToAnAgent::Did(done) => Self::Did(done),
            ToAnAgent::Proposed(standing) => Self::Proposed(standing),
            ToAnAgent::Answered {
                text,
                came_from,
                model,
            } => Self::Answered {
                text,
                came_from,
                model,
            },
            ToAnAgent::Refused(wording) => Self::Refused(wording),
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
    use crate::testing::{a_change_waiting, in_english, the_change, the_moment};
    use crate::words;
    use crate::{ToAPerson, wording::CameFrom};
    use alo_strings::Filling;

    /// One sentence this crate really declares.
    fn a_sentence() -> Said {
        in_english().say(&words::NOT_READABLE.key(), &Filling::nothing())
    }

    /// What a model said, if that is what this was.
    ///
    /// A function rather than a `match` with an `else { panic!(…) }`, which is
    /// `alo-asking`'s note about the same shape: the lints this workspace
    /// deploys are the ones a test reaches for first.
    fn answered(told: ToAnAgent) -> Option<(String, Wording, String)> {
        match told {
            ToAnAgent::Answered {
                text,
                came_from,
                model,
            } => Some((text, came_from, model)),
            _ => None,
        }
    }

    /// The four, written and read back.
    #[test]
    fn the_four_an_agent_is_told_read_back() {
        let (approvals, strings) = a_change_waiting();
        for told in [
            ToAnAgent::did(&Answer::Read("March, 4180.00".to_owned())),
            ToAnAgent::proposed(the_change(&approvals), &strings, the_moment()),
            ToAnAgent::answered("Three are unpaid.", &a_sentence(), "mistral-small-latest"),
            ToAnAgent::refused(&a_sentence()),
        ] {
            let written = told.written().unwrap();
            assert_eq!(ToAnAgent::read(&written).unwrap(), told, "{written}");
        }
    }

    /// **An agent is never told what its person is being asked.** The one
    /// answer that would carry the person's own list has no shape on this door,
    /// so a daemon that got the wiring wrong is refused by the type rather than
    /// by a check somebody has to remember.
    #[test]
    fn an_agent_is_not_told_what_the_person_has_in_front_of_them() {
        let (approvals, strings) = a_change_waiting();
        let waiting =
            ToAPerson::waiting(approvals.waiting_at(the_moment()), &strings, the_moment())
                .written()
                .unwrap();
        assert_eq!(
            ToAnAgent::read(&waiting),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );

        let declined = ToAPerson::Declined.written().unwrap();
        assert_eq!(
            ToAnAgent::read(&declined),
            Err(NotUnderstood::NotAnAnswerForAnAgent)
        );
    }

    /// **An answer carries where it came from**, and there is no constructor
    /// that leaves it out — which is what makes the v0.01 promise survive the
    /// last boundary before a person reads it.
    #[test]
    fn an_answer_from_a_model_carries_where_it_came_from() {
        let told = ToAnAgent::answered("Three are unpaid.", &a_sentence(), "mistral-small-latest");
        let written = told.written().unwrap();
        assert!(written.contains("came_from"), "{written}");
        let (text, came_from, model) = answered(ToAnAgent::read(&written).unwrap()).unwrap();
        assert_eq!(text, "Three are unpaid.");
        assert_eq!(came_from.came_from(), CameFrom::TheSource);
        assert_eq!(model, "mistral-small-latest");
    }

    /// A refusal crosses as a sentence and as nothing else: there is no kind, no
    /// code and no name of the crate that made it, because a client that could
    /// branch on which refusal it was is a client that would.
    #[test]
    fn a_refusal_crosses_as_a_sentence_and_carries_no_kind() {
        let told = ToAnAgent::refused(&a_sentence());
        let written = told.written().unwrap();
        assert!(told.refusal().unwrap().text().len() > 10);
        assert!(told.done().is_none());
        assert!(!told.waits_for_a_person());
        assert!(!written.contains("capability"), "{written}");
    }

    /// A change that is waiting says so, which is what an agent stops on.
    #[test]
    fn a_change_that_is_waiting_says_so() {
        let (approvals, strings) = a_change_waiting();
        let told = ToAnAgent::proposed(the_change(&approvals), &strings, the_moment());
        assert!(told.waits_for_a_person());
        assert!(told.done().is_none());
        assert!(told.refusal().is_none());
    }
}
