//! The broker's record, on the machine's disk.
//!
//! `alo_keeping::Writing` is the one writer of a record file: an entry is on the
//! disk before `keep` answers, and a file whose first line does not say it is a
//! record is refused rather than added to. The broker's door writes every answer
//! through [`alo_broker::Recording`] before it gives it, so this is the whole of
//! what makes that promise true on a machine rather than in memory.
//!
//! # Its own file
//!
//! Not the file `alo-agentd` writes. That one is the person's (`0700`, theirs,
//! under `/var/lib/alo`) and has exactly one writer, because two processes
//! appending to one file interleave; the broker is root and the machine's, so
//! its entries go in [`THE_RECORD`], in a directory systemd makes for it alone.
//! A `brokered` entry carries the turn's approval number, which is what puts the
//! two records side by side.

use alo_broker::{NotKept, Recording};
use alo_keeping::Writing;
use alo_record::Entry;

/// Where the broker's record is on a machine.
pub const THE_RECORD: &str = "/var/lib/alo-broker/record.jsonl";

/// The broker's record file, open to be added to.
#[derive(Debug)]
pub struct MachinesRecord(Writing);

impl MachinesRecord {
    /// The record file this writer adds to.
    #[must_use]
    pub const fn of(writing: Writing) -> Self {
        Self(writing)
    }
}

impl Recording for MachinesRecord {
    /// Written and synced before this answers, or refused — and a refusal stops
    /// the broker carrying anything out again.
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
        self.0
            .keep(&entry)
            .map_err(|why| NotKept(format!("{}: {why:?}", self.0.path().display())))
    }
}
