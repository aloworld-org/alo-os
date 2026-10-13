//! The one thing this crate needs from a machine: a line in, a line out.
//!
//! A trait rather than a socket, for `alo_files::Resolving`'s reason one level
//! along: the loop's decisions are what this crate is, and a loop that opened
//! its own socket could only be tested on a machine with a daemon running on
//! it. Behind this trait the tests put a door that answers from a list, and
//! `alo-agent`'s binary puts the real `alo-agentd` socket.
//!
//! # It is the agent's door and not the person's
//!
//! `docs/contracts/daemon-protocol.md` has two doors and says why a message
//! cannot cross: *if one door took both, the side that proposed a change could
//! approve it*, and ADR 0001 §5 would be true of the capability model and false
//! of the socket in front of it. This trait is typed to the agent's half —
//! [`alo_protocol::FromAnAgent`] in and [`alo_protocol::ToAnAgent`] out — so
//! there is no method here through which an approval could be sent, whatever
//! anybody later wants to add.

use alo_protocol::{FromAnAgent, ToAnAgent};

/// A machine an agent can ask things of.
pub trait TheDoor {
    /// Put one request and wait for the one answer.
    ///
    /// # Errors
    /// [`NotAnswered`] when the machine could not be reached or did not answer
    /// something this protocol understands. It is **not** how a refusal
    /// arrives: a refusal is an answer, and comes back as
    /// [`ToAnAgent::Refused`].
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered>;
}

/// The machine could not be asked, or did not answer the protocol.
///
/// English and a `Display`: it is not a sentence a person reads, because a
/// person reads what the *turn* said. This is read by whoever is looking at why
/// the agent could not talk to its own machine.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnswered {
    /// The door could not be reached at all.
    #[error("the machine could not be asked: {why}")]
    NotReached {
        /// What the machine said about it.
        why: String,
    },
    /// Something came back and it was not an answer to an agent.
    #[error("what came back is not an answer this protocol carries: {why}")]
    NotTheProtocol {
        /// What was wrong with it.
        why: String,
    },
}
