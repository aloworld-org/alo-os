//! Where a turn writes down what happened.
//!
//! Two things in this workspace can receive an [`Entry`], and they are not
//! interchangeable in the way an interface usually means. [`Record`] holds the
//! entries a process has made and forgets them when it stops; [`Writing`] puts
//! them on a disk, one line at a time, so that they outlive the machine being
//! turned off. A daemon uses the second. A test, and anything asking *what
//! would this turn have written*, uses the first.
//!
//! # Why this is a trait rather than a field of one type
//!
//! Because of what [`crate::Turning`] promises: a turn hands nothing back that
//! it has not written down first. That promise is only structural if a turn
//! cannot be made without somewhere to write — so [`crate::Machine`] takes one
//! of these and there is no constructor that does not.
//!
//! A turn that took a [`Writing`] directly would make the promise true and the
//! crate untestable without a disk; one that took a [`Record`] would make every
//! test pass and every real machine lose its evidence at shutdown. The trait is
//! the smallest thing that is honest about both, and it is the shape
//! `alo_files::Resolving` already has for the same reason: one interface, one
//! implementation that is real, and the crate's decisions testable without the
//! machine underneath them.
//!
//! # Failing to write is not a refusal
//!
//! [`NotKept`] is `alo-keeping`'s, and that crate is careful about the
//! difference: nothing here is the capability model saying no. A record that
//! could not be written is a machine problem, and a turn that reported it as a
//! refusal would tell a security review that the grants stopped something they
//! did not. [`crate::NotDone`] keeps the two apart for the same reason.

use alo_keeping::{NotKept, Writing};
use alo_record::{Entry, Record};

/// Somewhere one thing that happened can be written down.
///
/// The entry is taken by value, because a turn has no further use for one it
/// has handed over: an entry is a fact about a moment that has passed, and the
/// thing that keeps it is the thing that can answer questions about it
/// afterwards.
pub trait Kept {
    /// Write one thing that happened down.
    ///
    /// # Errors
    /// [`NotKept`] when the machine could not — almost always the disk. A turn
    /// that meets one of these does nothing else, so an implementation that
    /// answers `Ok` when it did not write is an implementation that turns off
    /// the guarantee this trait exists for.
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept>;
}

/// The record a process holds, which forgets everything when it stops.
///
/// Answers `Ok` always, because there is nothing here that can fail: an entry
/// goes onto a list in memory. That is the honest signature for it and not a
/// weakening of the trait — a machine that keeps evidence only until it is
/// turned off is a machine whose evidence is in `alo-keeping`'s hands, and this
/// implementation is for asking what a turn would write rather than for keeping
/// it.
impl Kept for Record {
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
        Record::keep(self, entry);
        Ok(())
    }
}

/// The record on a disk, which is what a daemon writes to.
///
/// One line of JSON appended per entry, in the format
/// `docs/contracts/record-file.md` states. Nothing here shortens a record; that
/// is `alo_keeping::Writing::prune`, reached through [`crate::Shortening`],
/// which is the larger trait a machine holds and the one a turn is never handed.
impl Kept for Writing {
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
        Writing::keep(self, &entry)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, files, noon};
    use alo_record::{Asking, Only};

    /// The in-memory record keeps what it is given and says so.
    #[test]
    fn a_record_in_memory_keeps_what_it_is_handed() {
        let mut record = Record::default();
        let kept: &mut dyn Kept = &mut record;
        kept.keep(Entry::answered_here(&files(), noon())).unwrap();

        assert_eq!(record.len(), 1);
        assert_eq!(
            record
                .answering(&Asking::anything().by(files().as_str()))
                .count(),
            1,
            "the entry is not findable under the agent that made it"
        );
    }

    /// **And the one on a disk keeps it where it survives the machine being
    /// turned off**, which is the implementation a daemon uses and the reason
    /// the trait exists at all.
    #[test]
    fn a_record_on_a_disk_keeps_what_it_is_handed() {
        let folder = a_folder_of_our_own("kept");
        let path = folder.join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        {
            let kept: &mut dyn Kept = &mut writing;
            kept.keep(Entry::answered_here(&files(), noon())).unwrap();
        }

        let read = alo_keeping::Reading::at(&path).unwrap();
        assert_eq!(read.record().len(), 1);
        assert_eq!(
            read.record()
                .answering(&Asking::anything().only(Only::ByAnAgent))
                .count(),
            1
        );
    }
}
