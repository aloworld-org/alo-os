//! The one thing a turn needs from the compositor, as a trait.
//!
//! [ADR 0108](../../../docs/decisions/0108-the-compositor-carries-an-application-verb-out-and-needs-no-token-to.md):
//! the capability model is here and the windows are in the compositor, and
//! between them is one socket. `docs/contracts/application-door.md` is that
//! socket's published surface and `alo-acting` is its vocabulary.
//!
//! # Why a trait and not a socket
//!
//! This crate is portable and holds no path. The socket is
//! `/run/alo/<uid>/shell.sock` on a Linux machine with a compositor running,
//! and `alo-agentd` is the thing that knows both of those — which is the same
//! division `Machine::installing_with` already makes for the rented tool, and
//! `alo_shell::an_application_is_started`'s `AsksAnApplicationToStart` makes
//! from the other side.
//!
//! It also makes the executor testable without a compositor, which is the
//! whole of what a unit test can hold here.
//!
//! # A door that is not there is not an error in the machine
//!
//! A person may be signed in with no compositor running — a machine at the
//! sign-in screen, a daemon started before the shell, a shell that crashed. The
//! verbs are still declared and a turn may still reach this, so *there is no
//! door* has to be an answer rather than a panic or a hang.
//!
//! It answers [`NotAtTheDoor::NoCompositor`], which the executor turns into the
//! person's own *this machine could not reach {application}* — the same
//! sentence as a compositor that answered `could-not-ask`, because from the
//! person's side they are the same fact: nothing was changed.

use alo_acting::{Answer, Request};

/// Whatever can put one request to the compositor and read one answer.
///
/// One line in, one line out, and **nothing about a session**: ADR 0108 is
/// explicit that one line is one act, because nothing about this door is an
/// authorisation in the first place — the authority was the turn's and it was
/// spent before the line was written.
pub trait TheDoorTheCompositorServes {
    /// Put this to the compositor and wait for its one-word answer.
    ///
    /// # Errors
    /// [`NotAtTheDoor`] when there was nothing to ask or the asking failed.
    /// **Never a refusal of the capability model** — *may this run* was decided
    /// before this was called, and a door that could answer it would be a
    /// second place deciding it.
    fn asking(&mut self, request: &Request) -> Result<Answer, NotAtTheDoor>;
}

/// Why the compositor was not asked.
///
/// Every one of these means **nothing was changed**, which is what the person
/// is told. They are told apart here rather than in the sentence because only
/// one of them is worth somebody reading a log about.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAtTheDoor {
    /// No compositor is serving a door on this machine.
    ///
    /// Ordinary rather than exceptional: a machine at the sign-in screen is in
    /// exactly this state.
    #[error("no compositor is serving an application door on this machine")]
    NoCompositor,
    /// The door was there and the asking did not finish.
    ///
    /// The socket went away mid-conversation, the compositor stopped, or it
    /// answered something this machine's vocabulary does not know — which
    /// `alo_acting::Answer::read` deliberately never reads as success.
    #[error("the compositor was asked and did not answer: {why}")]
    NotAnswered {
        /// What went wrong, for a log and never for a person.
        why: String,
    },
}
