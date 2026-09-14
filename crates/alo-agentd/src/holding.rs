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

use alo_corridor::Doorway;
use alo_keeping::NotKept;
use alo_strings::Said;
use alo_turn::{Arriving, Machine, Turning};

use crate::network::TheNetwork;
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
    /// No local turn is under way, so the network's door has the machine —
    /// holding it for a remote turn, or holding it for the next verb.
    ///
    /// This is what the rounds between local turns hold since the daemon
    /// bound the port. What a knock writes goes through the remote turn if
    /// one is open and through the machine otherwise; a change a paired
    /// machine proposed is approved, declined and listed through the remote
    /// turn (`crate::reaching`), against the pairings behind the one lock;
    /// and a question from a paired machine is answered by what the person
    /// here chose, looked for once per question.
    TheNetwork {
        /// The network's door onto the machine, holding the proofs seen.
        doorway: &'a mut Doorway<'m, 't>,
        /// The pairings and the proposals, behind the one lock.
        network: &'a TheNetwork,
        /// What a question from a paired machine is put to.
        questions: &'a mut Questions,
    },
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
            Self::Nobody(_) | Self::TheNetwork { .. } => None,
        }
    }

    /// The remote turn under way and the pairings it is judged against, when
    /// the network's door holds the machine for one.
    ///
    /// What `crate::reaching` answers the three requests about a turn with
    /// when the turn is a paired machine's: a change it proposed waits for
    /// the person here, and the pairing is asked again at the moment of
    /// approval. `None` while no remote turn is open — including while the
    /// network's door holds a free machine — so that a number nothing is
    /// waiting under is refused in the ordinary words.
    pub fn remote(&mut self) -> Option<(&mut Arriving<'m, 't>, &'a TheNetwork)> {
        match self {
            Self::TheNetwork {
                doorway, network, ..
            } => doorway.turn().map(|arriving| (arriving, *network)),
            Self::ATurn { .. } | Self::Nobody(_) => None,
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
            Self::Nobody(_) | Self::TheNetwork { .. } => None,
        }
    }

    /// What a question is put to, whether a turn or the network's door holds
    /// the machine — which is where the person's settings are found.
    ///
    /// `None` only while nobody holds the machine, which is a test's state:
    /// a running service always holds it one way or the other.
    #[must_use]
    pub fn questions(&self) -> Option<&Questions> {
        match self {
            Self::ATurn { questions, .. } | Self::TheNetwork { questions, .. } => Some(questions),
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
            Self::TheNetwork { doorway, .. } => {
                if let Some(arriving) = doorway.turn() {
                    arriving.the_grants_were_not_read_again(why, now)
                } else if let Some(machine) = doorway.machine() {
                    machine.the_grants_were_not_read_again(why, now)
                } else {
                    Err(the_machine_was_lost())
                }
            }
        }
    }

    /// Write down that a pairing with another machine was kept.
    ///
    /// The person's own door confirms a pairing whether or not a turn — local
    /// or remote — holds the machine, and the record is behind whatever does.
    /// The same entry either way, naming no agent: two people made it.
    ///
    /// # Errors
    ///
    /// [`NotKept`] when the record could not be written; the service stops on
    /// it, for the reason `crate::hearing` stops when the same entry cannot be
    /// written for a confirmation that arrived on the wire.
    pub fn a_pairing_was_kept(&mut self, with: &str, now: SystemTime) -> Result<(), NotKept> {
        match self {
            Self::ATurn { turning, .. } => turning.a_pairing_was_kept(with, now),
            Self::Nobody(machine) => machine.a_pairing_was_kept(with, now),
            Self::TheNetwork { doorway, .. } => {
                if let Some(arriving) = doorway.turn() {
                    arriving.a_pairing_was_kept(with, now)
                } else if let Some(machine) = doorway.machine() {
                    machine.a_pairing_was_kept(with, now)
                } else {
                    Err(the_machine_was_lost())
                }
            }
        }
    }
}

/// The record's answer for a machine that is no longer there to write into.
///
/// The machine was lost when a remote turn could not begin, which has already
/// ended the service; nothing was added and the truthful answer is that
/// nothing could be.
fn the_machine_was_lost() -> NotKept {
    NotKept::NotAddedTo {
        path: String::new(),
        why: "the machine was lost when a remote turn could not begin".to_owned(),
    }
}
