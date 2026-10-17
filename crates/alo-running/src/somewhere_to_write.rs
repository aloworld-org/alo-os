//! Somewhere a run can be written down, which a run cannot happen without.
//!
//! ADR 0064 keeps the record on at **every** one of its three levels. That is a
//! sentence until something makes it unskippable, and this is what makes it
//! unskippable: [`crate::WhereCommandsRun`] takes one of these and there is no
//! constructor that does not, so a caller cannot reach [`crate::ran`] holding
//! nothing to write to.
//!
//! It is `alo_turn::Kept`'s argument, arriving in another crate for the same
//! reason it was made there:
//!
//! > *That promise is only structural if a turn cannot be made without
//! > somewhere to write.*
//!
//! # Why this is not `alo_turn::Kept`
//!
//! Because the trait is in `alo-turn`, and `alo-turn` is the agent-turn crate
//! with seventeen dependencies — models, nearby, egress, applications, the whole
//! agent world. Reaching through it for one trait would pull all of that into
//! the sealed box and re-couple exactly what
//! [ADR 0117](../../../docs/decisions/0117-the-thing-that-starts-a-program-is-not-the-thing-that-bounds-a-turn.md)
//! separated: the crate serving a closed verb list, and the crate that exists
//! because a person chose to let the agent run code.
//!
//! So the same shape is declared here over the same two implementations. **The
//! duplication is deliberate and has a condition for ending**: `Kept` names no
//! turn and belongs in `alo-keeping`, which is the crate about where the record
//! is written and which both of these already depend on. If it moves there,
//! this file should become a re-export and these two impls should go.
//!
//! # The two implementations, and why both are real
//!
//! `alo_turn::kept` argues this and it holds here unchanged: [`alo_record::Record`]
//! keeps entries a process has made and forgets them when it stops;
//! [`alo_keeping::Writing`] puts them on a disk one line at a time so they
//! outlive the machine being turned off. A daemon uses the second. A test, and
//! anything asking *what would this run have written*, uses the first.
//!
//! One takes an entry by value and cannot fail; the other takes it by reference
//! and can. Unifying those two signatures is the whole of what this trait does,
//! and is why a caller cannot simply be handed either one.

use alo_keeping::NotKept;
use alo_record::Entry;

/// Somewhere one thing that happened can be written down.
///
/// The entry is taken by value, because whoever wrote it has no further use for
/// one they have handed over.
pub trait SomewhereToWrite {
    /// Write one thing that happened down.
    ///
    /// # Errors
    /// [`NotKept`] when the machine could not — almost always the disk. An
    /// implementation that answers `Ok` when it did not write is an
    /// implementation that turns off the guarantee this trait exists for.
    fn write_down(&mut self, entry: Entry) -> Result<(), NotKept>;
}

/// The record a process holds, which forgets when it stops.
impl SomewhereToWrite for alo_record::Record {
    /// Cannot fail, because keeping an entry in memory cannot.
    fn write_down(&mut self, entry: Entry) -> Result<(), NotKept> {
        self.keep(entry);
        Ok(())
    }
}

/// The record on the disk, which outlives the machine being turned off.
impl SomewhereToWrite for alo_keeping::Writing {
    fn write_down(&mut self, entry: Entry) -> Result<(), NotKept> {
        self.keep(&entry)
    }
}
