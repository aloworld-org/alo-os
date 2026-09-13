//! What became of a change an asking machine proposed, asked for on the
//! wire.
//!
//! A change put to a paired machine comes back as the number it waits under
//! and nothing more, because the person there answers it on their own
//! surface in their own time. This is the additive path the asking machine
//! asks afterwards, so that a person is not left guessing whether a file
//! moved on a machine down the corridor: [`AskedAbout`] is the question —
//! one number — and [`Outcome`] is the answer, [`alo_turn::Became`] spelt
//! for the wire.
//!
//! # It is judged like a verb, and it is not one
//!
//! The question travels the road a verb does — proven at the door before
//! anything else, inside the turn the asking machine has open here, and the
//! answer leaves under a departure — because what it answers is this
//! machine's business and leaves this machine. What it does *not* do is ask
//! any grant or run anything: it is a read of the turn's own memory, and
//! [`alo_turn::Arriving::became`] is the one thing it calls.
//!
//! # A number is its turn's
//!
//! The number a change waits under is given by the turn that put it, and
//! means nothing outside that turn. A remote turn lasts as long as the
//! daemon says and lapses; a question about a number after the turn it
//! belonged to has ended is answered *nothing waiting*, which is true — no
//! change of that number waits — and is said as such rather than guessed
//! at. What the person there decided is in that machine's record either
//! way.

use alo_nearby::NotNearby;
use alo_turn::Became;
use serde::{Deserialize, Serialize};

/// The question: which change, by the number it waits under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskedAbout {
    /// The number the change was told to wait under.
    number: u64,
}

/// The answer: what became of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Outcome {
    /// Still waiting for the person there.
    Waiting,
    /// The person there approved it; whether the machine then could is in
    /// that machine's record.
    Approved,
    /// The person there declined it.
    Declined,
    /// Nobody answered it in the time it stood.
    Lapsed,
    /// No change of that number waits there: never put, or its turn over.
    NothingWaiting,
}

impl AskedAbout {
    /// Asking after the change waiting under `number`.
    #[must_use]
    pub const fn the_change(number: u64) -> Self {
        Self { number }
    }

    /// The number.
    #[must_use]
    pub const fn number(self) -> u64 {
        self.number
    }

    /// As it goes on the wire; these bytes are what the proof is over.
    #[must_use]
    pub fn said(self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    /// Off the wire, refusing anything that is not exactly this.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAMessage`] when the bytes do not read as one number
    /// and nothing else.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        serde_json::from_str(said)
            .map_err(|why| NotNearby::NotAMessage(format!("not a question about a change: {why}")))
    }
}

impl From<Became> for Outcome {
    fn from(became: Became) -> Self {
        match became {
            Became::Waiting => Self::Waiting,
            Became::Approved => Self::Approved,
            Became::Declined => Self::Declined,
            Became::Lapsed => Self::Lapsed,
            // `Became` is a closed list on a crate this one names; a new arm
            // there is not a new outcome on this wire until it is spelt here.
            Became::NothingWaiting | _ => Self::NothingWaiting,
        }
    }
}

#[cfg(test)]
mod tests {
    use alo_turn::Became;

    use super::{AskedAbout, Outcome};

    /// A question reads back as it was said, and it is the one field.
    #[test]
    fn a_question_about_a_change_is_one_number_and_nothing_else() {
        let asked = AskedAbout::the_change(7);
        assert_eq!(asked.said(), r#"{"number":7}"#);
        assert_eq!(AskedAbout::read(&asked.said()), Ok(asked));
        for not_one in ["", "{}", "7", r#"{"number":7,"verb":"rename_file"}"#] {
            assert!(AskedAbout::read(not_one).is_err(), "{not_one}");
        }
    }

    /// Every outcome crosses as one word and comes back as itself.
    #[test]
    fn every_outcome_crosses_as_one_word_and_comes_back_as_itself() {
        for (became, outcome) in [
            (Became::Waiting, Outcome::Waiting),
            (Became::Approved, Outcome::Approved),
            (Became::Declined, Outcome::Declined),
            (Became::Lapsed, Outcome::Lapsed),
            (Became::NothingWaiting, Outcome::NothingWaiting),
        ] {
            assert_eq!(Outcome::from(became), outcome);
            let said = serde_json::to_string(&outcome).unwrap_or_default();
            assert_eq!(serde_json::from_str::<Outcome>(&said).ok(), Some(outcome));
        }
    }
}
