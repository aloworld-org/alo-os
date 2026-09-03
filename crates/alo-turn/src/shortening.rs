//! Shortening the record a machine holds, which is the one thing in alo OS
//! that removes evidence.
//!
//! [`crate::Kept`] is where a turn writes down what happened, and it is
//! deliberately the smaller of the two: a turn can reach it, and everything a
//! turn does goes through it. This is the other half of what a machine's record
//! is, and **nothing a turn offers reaches it** — there is no verb behind it, no
//! request that arrives at it, and no door in [`crate::Turning`] that leads
//! here. Its one caller is the service's own clock (`alo-agentd`, queue item
//! 20), and what it removes is decided by a rule and a moment exactly as
//! `alo_keeping::Writing::prune` describes.
//!
//! # Why a trait on top of `Kept` rather than a second method on it
//!
//! Because of what the two mean. `Kept` is *somewhere one thing that happened
//! can be written down*, and an implementation of it is a thing that keeps
//! evidence. Removal is not a second way of keeping, it is the opposite one, and
//! a `Kept` that could remove would make every implementer of the smaller trait
//! — including the one in `alo-record`, whose whole promise is that nothing
//! takes an entry out — a type with a way to take entries out.
//!
//! So the supertrait is the direction that is true: **everything that can be
//! shortened can be written to, and not the other way round.** A [`Machine`]
//! holds the larger one, because a machine has a record; a turn asks it for the
//! smaller one, because a turn writes.
//!
//! [`Machine`]: crate::Machine
//!
//! # A record that is not on a disk says so, rather than answering nothing
//!
//! [`alo_record::Record`] cannot be shortened — there is no `remove` in that
//! crate and this file does not add one. It answers [`Shortened::NotOnADisk`],
//! which is a fact about that record rather than a failure, and the difference
//! matters to whoever is waiting: a service that read *nothing was removed*
//! would go on waking up on an interval for the rest of the day to remove
//! nothing again.

use std::time::SystemTime;

use alo_keeping::{Keeping, NotKept, Pruned, Writing};
use alo_record::Record;

use crate::kept::Kept;

/// A machine's record: written to one entry at a time, and shortened under a
/// rule.
pub trait Shortening: Kept {
    /// Remove what this rule no longer keeps, as of this moment.
    ///
    /// Nothing here reads the clock, as everywhere else in this workspace: what
    /// a rule removes is arithmetic that can be tested rather than waited for.
    ///
    /// # Errors
    ///
    /// [`NotKept`] when the machine would not, or when the record cannot be
    /// read all of — and in every one of them **nothing has been removed**,
    /// which is why a caller that meets one can go on serving. `alo-keeping`'s
    /// `Writing::prune` is where each of them is decided.
    fn shorten(&mut self, keeping: Keeping, now: SystemTime) -> Result<Shortened, NotKept>;
}

/// What a shortening did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortened {
    /// There is no file here, so there is nothing a rule can remove.
    ///
    /// Not an error and not *nothing was old enough yet*: it is the answer of a
    /// record that only exists while a process is running, and a caller reading
    /// it knows there is no reason to ask again.
    NotOnADisk,
    /// It ran, against the record on the disk.
    Ran(Pruned),
}

impl Shortened {
    /// How many entries were removed, which is none where there was nothing to
    /// remove them from.
    #[must_use]
    pub fn removed(self) -> usize {
        match self {
            Self::NotOnADisk => 0,
            Self::Ran(pruned) => pruned.removed(),
        }
    }

    /// Whether there is a record on a disk here at all.
    #[must_use]
    pub fn on_a_disk(self) -> bool {
        matches!(self, Self::Ran(_))
    }
}

/// The record a process holds, which has nothing to shorten.
///
/// `alo-record` has no `remove`, no `edit` and no `forget`, and this
/// implementation adds none of them: it answers that there is no file here. A
/// machine keeping evidence only until it is turned off is a machine whose
/// evidence is not being kept at all, and shortening is about a file.
impl Shortening for Record {
    fn shorten(&mut self, _keeping: Keeping, _now: SystemTime) -> Result<Shortened, NotKept> {
        Ok(Shortened::NotOnADisk)
    }
}

/// The record on a disk, which is what a daemon holds.
///
/// Straight to `alo_keeping::Writing::prune`, which is the only thing in alo OS
/// that removes evidence and is careful about every part of it: the replacement
/// is on the disk before anything is renamed, the mark is in a first line no
/// later shortening walks, and a record with a line nobody can read is refused
/// rather than rewritten.
impl Shortening for Writing {
    fn shorten(&mut self, keeping: Keeping, now: SystemTime) -> Result<Shortened, NotKept> {
        Writing::prune(self, keeping, now).map(Shortened::Ran)
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
    use alo_record::Entry;
    use std::time::Duration;

    /// A day, for a record with a fortnight of them in it.
    fn day() -> Duration {
        Duration::from_secs(24 * 60 * 60)
    }

    /// **A record that only exists while the process does says so**, which is
    /// what lets whoever asked stop asking. It is not an error and not a
    /// shortening that removed nothing.
    #[test]
    fn a_record_in_memory_says_there_is_nothing_on_a_disk_to_shorten() {
        let mut record = Record::default();
        Kept::keep(&mut record, Entry::answered_here(&files(), noon())).unwrap();

        let shortening: &mut dyn Shortening = &mut record;
        let shortened = shortening
            .shorten(Keeping::for_days(1).unwrap(), noon() + day() * 30)
            .unwrap();

        assert_eq!(shortened, Shortened::NotOnADisk);
        assert!(!shortened.on_a_disk());
        assert_eq!(shortened.removed(), 0);
        assert_eq!(record.len(), 1, "an entry was taken out of a record");
    }

    /// **And the one on a disk is shortened under the rule it is given**, which
    /// is the implementation a daemon uses and the reason this trait exists.
    #[test]
    fn a_record_on_a_disk_loses_what_the_rule_no_longer_keeps() {
        let path = a_folder_of_our_own("shortened").join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        for days_ago in (0..14_u32).rev() {
            Kept::keep(
                &mut writing,
                Entry::answered_here(&files(), noon() - day() * days_ago),
            )
            .unwrap();
        }

        let shortened = {
            let shortening: &mut dyn Shortening = &mut writing;
            shortening
                .shorten(Keeping::for_days(7).unwrap(), noon())
                .unwrap()
        };

        assert!(shortened.on_a_disk());
        assert_eq!(shortened.removed(), 6, "the days before last week");
        assert_eq!(
            alo_keeping::Reading::at(&path).unwrap().record().len(),
            8,
            "the record on the disk is not what the shortening said it was"
        );
    }

    /// **A machine that keeps everything is not rewritten at all**, and the
    /// answer still says there is a record on a disk here — which is the
    /// difference between *nothing was old enough* and *there is nothing to
    /// shorten*.
    #[test]
    fn a_record_kept_for_good_is_answered_about_without_losing_anything() {
        let path = a_folder_of_our_own("kept-for-good").join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        Kept::keep(&mut writing, Entry::answered_here(&files(), noon())).unwrap();

        let shortened = {
            let shortening: &mut dyn Shortening = &mut writing;
            shortening
                .shorten(Keeping::Forever, noon() + day() * 3650)
                .unwrap()
        };

        assert!(shortened.on_a_disk());
        assert_eq!(shortened.removed(), 0);
        assert_eq!(alo_keeping::Reading::at(&path).unwrap().record().len(), 1);
    }

    /// **A record with a line nobody can read is refused, and nothing is
    /// removed.** The refusal travels as `alo-keeping`'s own, so a service that
    /// meets it can ask whether the record is still whole — and it is.
    #[test]
    fn a_damaged_record_is_refused_rather_than_rewritten() {
        let path = a_folder_of_our_own("damaged-record").join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        Kept::keep(
            &mut writing,
            Entry::answered_here(&files(), noon() - day() * 30),
        )
        .unwrap();
        drop(writing);

        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("{\"at\":{\"secs_since_epoch\":0\n");
        std::fs::write(&path, &text).unwrap();

        let mut writing = Writing::opening(&path).unwrap();
        let refused = {
            let shortening: &mut dyn Shortening = &mut writing;
            shortening
                .shorten(Keeping::for_days(1).unwrap(), noon())
                .unwrap_err()
        };

        assert!(refused.record_is_still_whole());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}
