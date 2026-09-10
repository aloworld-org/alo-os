//! What is holding this machine while one line from the person is answered.
//!
//! `alo_turn::Turning` borrows the machine mutably and there is one machine, so
//! while a turn is under way there is nothing else that can reach the record —
//! and between turns there is no turn to reach it through. Every question a
//! person's shell asks has an answer in both states (`crate::answering`), and
//! until this file existed that was expressible as an `Option<&mut Turning>`,
//! because nothing on the person's door touched anything outside the turn.
//!
//! The knock does. `crate::rereading` writes down a refusal, and a refusal has
//! to be written down whether or not an agent happens to be connected — so what
//! the person's door is handed is the machine **or** the turn holding it, and
//! never neither. That is this type, and it is a type rather than two arguments
//! for `crate::serving::Underway`'s reason: two `Option`s of which exactly one is
//! ever `Some` is a state nothing can produce and every reader has to rule out.

use std::time::SystemTime;

use alo_keeping::NotKept;
use alo_strings::Said;
use alo_turn::{Machine, Turning};

use crate::questions::Questions;

/// The machine, or the turn that is holding it.
///
/// Where a question in this turn goes travels with the turn rather than beside
/// it, and that is not tidiness: `crate::questions` is looked for once a turn,
/// so the two begin together and are absent together. A round given one and not
/// the other would be a state nothing can produce.
#[derive(Debug)]
pub enum Holding<'a, 'm, 't> {
    /// A turn is under way and has the machine.
    ATurn {
        /// The turn every request in this round is carried out against.
        turning: &'a mut Turning<'m, 't>,
        /// What a question in this turn is put to, found once and held.
        questions: &'a mut Questions,
    },
    /// Nobody is, so the machine itself is here.
    Nobody(&'a mut Machine<'m>),
}

impl<'a, 'm, 't> Holding<'a, 'm, 't> {
    /// The turn, when there is one.
    ///
    /// What `crate::answering` asks for the three requests that are about the
    /// turn — approving, declining, and what is waiting — each of which already
    /// has an answer for a machine on which nothing is happening.
    pub fn turning(&mut self) -> Option<&mut Turning<'m, 't>> {
        match self {
            Self::ATurn { turning, .. } => Some(turning),
            Self::Nobody(_) => None,
        }
    }

    /// The turn and where a question in it goes, when there is one.
    ///
    /// What `crate::doing` needs, and it is one call rather than two so that a
    /// caller cannot hold the turn and reach for the questions of a different
    /// one.
    pub fn underway(&mut self) -> Option<(&mut Turning<'m, 't>, &mut Questions)> {
        match self {
            Self::ATurn { turning, questions } => Some((turning, questions)),
            Self::Nobody(_) => None,
        }
    }

    /// Write down that the person's grants were not read again.
    ///
    /// The same entry either way, and neither road names an agent: this is
    /// about the person's own list rather than about anything an agent did, and
    /// the turn's grantee would be the wrong name even where it is to hand.
    ///
    /// # Errors
    ///
    /// [`NotKept`] when the record could not be written. A turn that meets it is
    /// closed by it, and either way the service has stopped keeping evidence and
    /// stops.
    pub fn the_grants_were_not_read_again(
        &mut self,
        why: &Said,
        now: SystemTime,
    ) -> Result<(), NotKept> {
        match self {
            Self::ATurn { turning, .. } => turning.the_grants_were_not_read_again(why, now),
            Self::Nobody(machine) => machine.the_grants_were_not_read_again(why, now),
        }
    }
}
