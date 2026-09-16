//! Handing a message to the application, and what can come back.
//!
//! [`Delivers`] is the one thing that touches the machine. On Linux it is the
//! person's session bus (`crate::session_bus`); a test hands it a bus of its
//! own. What comes back is one of four facts, and they are not the same fact:
//!
//! - **the application answered** — it did what was asked;
//! - **it is not there**, **it does not offer that**, or **it refused** —
//!   nothing was done, and a person is told which;
//! - **it did not answer** — the message reached it and nobody knows what
//!   happened. That is said as it is, and it is recorded as having run, because
//!   something was sent under the approval and a record that left it out would
//!   be a record of less than happened.

use alo_strings::{Filling, Said, Strings};

use crate::message::Message;
use crate::words;

/// Why a message was not answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotDelivered {
    /// The application is not on the bus and could not be started.
    NotThere,
    /// It has no such object, interface or method.
    DoesNotOffer,
    /// It answered with an error.
    Refused,
    /// No answer came back in time.
    DidNotAnswer,
}

impl NotDelivered {
    /// Whether anything may have happened in the application.
    #[must_use]
    pub fn may_have_happened(self) -> bool {
        matches!(self, Self::DidNotAnswer)
    }

    /// What a person reads.
    #[must_use]
    pub fn said(self, application: &str, strings: &Strings) -> Said {
        let word = match self {
            Self::NotThere => words::NOT_THERE,
            Self::DoesNotOffer => words::DOES_NOT_OFFER,
            Self::Refused => words::REFUSED_IT,
            Self::DidNotAnswer => words::DID_NOT_ANSWER,
        };
        strings.say(
            &word.key(),
            &Filling::of(words::APPLICATION, application.to_owned()),
        )
    }
}

/// Something a message can be handed to.
pub trait Delivers {
    /// Send this message and wait for the application's answer.
    ///
    /// # Errors
    /// [`NotDelivered`], saying which of the four it was.
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered>;
}
