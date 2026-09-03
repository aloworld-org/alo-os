//! Everything the daemon can say back, in one closed list.
//!
//! Six answers, and there is no seventh. It is [`crate::asked`]'s shape from
//! the other direction and for the same reason: the list is one thing, the
//! doors are two, and neither door can produce the other's.
//!
//! # Why the answers divide as well as the requests
//!
//! The division of the requests is ADR 0001 §5 — the side that proposes a
//! change must not be the side that approves it. Nothing an agent *sends* can
//! reach the person's list, and item 21a made that structural.
//!
//! What that leaves open is the daemon. `alo_turn::Turning::waiting_at` is a
//! method on the turn, so a daemon holding an agent's connection can read
//! everything the person is being asked and could write it onto that
//! connection. One public answer type is one where that compiles. Two make it
//! impossible: [`crate::ToAnAgent`] has no shape for *what is waiting*, so an
//! agent cannot be told what its person has in front of them even by a daemon
//! that got the wiring wrong.
//!
//! The same holds in the other direction, and it matters less but costs
//! nothing: a person's shell is never handed a model's answer, because a
//! question is the agent's and the answer comes back where the question was
//! asked.
//!
//! # What is deliberately not here
//!
//! **No moment.** Nothing on the wire names one, in either direction, and
//! [`crate::standing`] has the client's own reason as well as the machine's.
//!
//! **Nothing that could be run.** An answer names what happened, and the
//! shapes it names it in are `alo_files::Answer`'s six with their paths — no
//! command, no handle to something the client may then invoke, and nothing that
//! turns back into a call. It is `alo-record`'s *a record is evidence, not an
//! instruction*, met at the socket.
//!
//! **No arguments of a call that never validated.** A refusal crosses as the
//! sentence whoever made it worded, and no answer here quotes back what a
//! client sent.

use serde::{Deserialize, Serialize};

use crate::done::Done;
use crate::standing::Standing;
use crate::wording::Wording;

/// Everything the daemon can put on the wire.
///
/// `pub(crate)`: see this file's header. A public version of this type would be
/// a value holding *what is waiting* that the agent's door could hand back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Told {
    /// What a read found, or what an approved change did.
    Did(Done),
    /// A change is waiting under this number, with the sentence it waits on.
    Proposed(Standing),
    /// A model answered.
    Answered {
        /// What the model said, in its own words.
        text: String,
        /// Where the answer came from, in the language the person reads.
        came_from: Wording,
        /// Which model was asked, as it was named when the question was put.
        model: String,
    },
    /// Everything the person has been asked and has not answered.
    Waiting {
        /// In the order they were proposed, which is the order they are read
        /// in.
        changes: Vec<Standing>,
    },
    /// The person said no, and it is written down.
    ///
    /// Nothing is carried about why, because nothing was asked: *no* is the
    /// whole answer, and a protocol with a field for a reason would be a
    /// protocol that asked for one.
    Declined {},
    /// It did not happen, and this is what the person is told.
    Refused(Wording),
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
    use alo_files::Answer;
    use alo_strings::Filling;

    /// One of each, as they are written on the wire.
    fn every_answer() -> Vec<Told> {
        let (approvals, strings) = a_change_waiting();
        let standing = Standing::of(the_change(&approvals), &strings, the_moment());
        vec![
            Told::Did(Done::of(&Answer::Read("March, 4180.00".to_owned()))),
            Told::Proposed(standing.clone()),
            Told::Answered {
                text: "Three of them are unpaid.".to_owned(),
                came_from: Wording::of(
                    &in_english().say(&words::NOT_READABLE.key(), &Filling::nothing()),
                ),
                model: "mistral-small-latest".to_owned(),
            },
            Told::Waiting {
                changes: vec![standing],
            },
            Told::Declined {},
            Told::Refused(Wording::of(
                &in_english().say(&words::NOT_READABLE.key(), &Filling::nothing()),
            )),
        ]
    }

    /// **The six read back as what was written**, so a shell and a daemon built
    /// from this crate cannot disagree about what happened.
    #[test]
    fn the_six_read_back_as_what_was_written() {
        for told in every_answer() {
            let written = serde_json::to_string(&told).unwrap();
            let back: Told = serde_json::from_str(&written).unwrap();
            assert_eq!(back, told, "{written}");
        }
    }

    /// **There is no seventh.** An answer that is not one of the six has
    /// nowhere to land, which is what stops a daemon from being extended by
    /// whatever a client is willing to parse.
    #[test]
    fn an_answer_that_is_not_one_of_the_six_is_not_an_answer() {
        for message in [
            r#"{"ran":{"command":"rm -rf /"}}"#,
            r#"{"granted":{"path":"/"}}"#,
            r#"{"approved":{"number":7}}"#,
            r#"{"context":{"document":"/home/anna/a.pdf"}}"#,
            r#"{"declined":{"why":"no"}}"#,
        ] {
            assert!(serde_json::from_str::<Told>(message).is_err(), "{message}");
        }
    }

    /// **An answer from a model cannot be written without saying where it came
    /// from.** `docs/features.md` promises it beside the answer at v0.01, and
    /// on the wire that is a field with no default and no way to leave it out.
    #[test]
    fn an_answer_cannot_cross_without_where_it_came_from() {
        for message in [
            r#"{"answered":{"text":"three","model":"mistral-small-latest"}}"#,
            r#"{"answered":{"text":"three","came_from":null,"model":"m"}}"#,
            r#"{"answered":{"text":"three"}}"#,
        ] {
            assert!(serde_json::from_str::<Told>(message).is_err(), "{message}");
        }
    }

    /// A field nobody declared is refused rather than ignored, on the way back
    /// as on the way in.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        for message in [
            r#"{"waiting":{"changes":[],"agent":"@files"}}"#,
            r#"{"answered":{"text":"a","came_from":{"text":"b","came_from":"translation"},"model":"m","cost":3}}"#,
        ] {
            assert!(serde_json::from_str::<Told>(message).is_err(), "{message}");
        }
    }
}
