//! Where the broker writes down what it answered, and what it hands a verb to.
//!
//! Two seams, one file, because they are the two things the door is handed
//! rather than things it decides — and the reason they are seams at all is the
//! same: **the door carries no verb out and chooses no record file.**
//!
//! - [`Recording`] is where an entry goes. `alo_record::Record` is one, in
//!   memory; the process that runs the broker on a machine,
//!   `crates/alo-brokerd`, writes to the machine's own record file.
//! - [`Carrying`] is what carries a verb out. `crates/alo-brokerd` carries the
//!   network's verbs (task 3 of
//!   `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`); printers are task
//!   2 and updates and storage task 4, and each adds its verbs there and
//!   nothing else.

use alo_record::{Entry, Record};

use crate::verbs::SystemVerb;

/// Somewhere entries are written down.
pub trait Recording {
    /// Write this entry down, or say why it could not be.
    ///
    /// # Errors
    /// [`NotKept`]. The broker treats one as the end of carrying anything out:
    /// a privileged component that has stopped keeping evidence is one that
    /// stops.
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept>;
}

/// An entry could not be written down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotKept(pub String);

impl std::fmt::Display for NotKept {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the broker could not write down what it answered, so it carries nothing out: {}",
            self.0
        )
    }
}

impl std::error::Error for NotKept {}

impl Recording for Record {
    /// A record in memory keeps everything, and cannot refuse.
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
        Record::keep(self, entry);
        Ok(())
    }
}

/// What carries a verb out, once the door has decided it is exactly one a
/// person approved.
pub trait Carrying {
    /// Carry this verb out, once.
    ///
    /// `approval` is the turn's number for the approval it was proven under, for
    /// whatever the verb writes down of its own.
    ///
    /// # Errors
    /// [`NotCarried`] when the machine could not do it. The broker records it and
    /// answers `refused not-carried`; it never tries again, because one approval
    /// is one execution whether or not the execution worked.
    fn carry(&mut self, verb: SystemVerb, approval: u64) -> Result<(), NotCarried>;
}

/// The machine could not carry a verb out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotCarried(pub String);

impl std::fmt::Display for NotCarried {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the machine could not carry the verb out: {}", self.0)
    }
}

impl std::error::Error for NotCarried {}
