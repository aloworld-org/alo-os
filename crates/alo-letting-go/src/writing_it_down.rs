//! Where the machine writes down which turns can no longer be put back.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s second
//! accepted term asks for this by name, and `alo_record::Entry::let_go` is the
//! entry. This is the file it goes in on a machine, and the one way this crate
//! reaches one.
//!
//! # Its own file, for the broker's reason
//!
//! Not `/var/lib/alo/record.jsonl`. That one is the person's — written by
//! `alo-agentd`, running as them — and it has exactly one writer, because two
//! processes appending to one file interleave. This unit is root's and the
//! machine's, so its entries go in [`THE_RECORD`], in a directory systemd makes
//! for it alone. It is the same shape `alo-brokerd` already has, and for the
//! same reason.
//!
//! **What follows from that**, said plainly rather than left for somebody to
//! discover: a surface putting *what this machine did* in front of a person
//! reads more than one record file, and has had to since the broker gained
//! one. What this crate owes such a surface is an entry it can word, and
//! `alo-recounting` can word this one.
//!
//! # And nothing is written before the snapshot is gone
//!
//! The entry says a person can no longer put something back. It is written
//! **after** the removal and only for what was actually removed, so a line in
//! this file is a fact about the disk rather than an intention about it — which
//! is the one reading a person could not check for themselves.

use alo_keeping::Writing;
use alo_record::Entry;

/// Where the expiry unit's record is on a machine.
pub const THE_RECORD: &str = "/var/lib/alo-letting-go/record.jsonl";

/// And where the forgetting unit's is.
///
/// **A second file, for the same reason there is a first one.** `alo-keeping`
/// says it outright: *two appending to one file would interleave*, so a record
/// file has one writer. These are two units — one a timer starts and one a
/// person's act starts — and systemd will happily run them in the same second,
/// which is two writers whatever either of them intends. A record whose lines
/// can be cut in half is worse than a record in two places, because a reader
/// can open two files and cannot mend one line.
///
/// What it costs is one more file for a surface to read, and that cost was
/// already paid: `crate::writing_it_down` says a surface putting *what this
/// machine did* in front of a person reads more than one record file, and has
/// had to since the broker gained one.
pub const THE_FORGETTING_RECORD: &str = "/var/lib/alo-forgetting/record.jsonl";

/// Something that keeps an entry — a record file on a machine, and a test's
/// stand-in that keeps them in memory.
pub trait WritingItDown {
    /// Keep this entry, or say why not.
    ///
    /// # Errors
    /// A sentence for the journal. Nothing this crate does is undone by a
    /// record that would not take an entry — the snapshot is already gone — so
    /// what it costs is the account, and the unit says so loudly rather than
    /// carrying on quietly.
    fn keep(&mut self, entry: Entry) -> Result<(), String>;
}

/// The machine's own record file, open to be added to.
#[derive(Debug)]
pub struct TheMachinesRecord(Writing);

impl TheMachinesRecord {
    /// The record file this writer adds to.
    #[must_use]
    pub const fn of(writing: Writing) -> Self {
        Self(writing)
    }
}

impl WritingItDown for TheMachinesRecord {
    /// Written and synced before this answers, or refused.
    fn keep(&mut self, entry: Entry) -> Result<(), String> {
        self.0
            .keep(&entry)
            .map_err(|why| format!("{}: {why:?}", self.0.path().display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **It is not the person's record file**, which has one writer and is
    /// `alo-agentd`'s, and it is not the broker's either.
    #[test]
    fn it_is_its_own_file_and_not_the_persons() {
        assert_ne!(THE_RECORD, "/var/lib/alo/record.jsonl");
        assert_ne!(THE_RECORD, "/var/lib/alo-broker/record.jsonl");
        assert!(THE_RECORD.starts_with("/var/lib/"));
        assert!(THE_RECORD.ends_with("/record.jsonl"));
    }

    /// **The two units write two files**, because a record file has one writer
    /// and these are two processes a machine may run in the same second.
    #[test]
    fn the_two_units_do_not_share_one_record_file() {
        assert_ne!(THE_RECORD, THE_FORGETTING_RECORD);
        assert_ne!(THE_FORGETTING_RECORD, "/var/lib/alo/record.jsonl");
        assert_ne!(THE_FORGETTING_RECORD, "/var/lib/alo-broker/record.jsonl");
        assert!(THE_FORGETTING_RECORD.starts_with("/var/lib/"));
        assert!(THE_FORGETTING_RECORD.ends_with("/record.jsonl"));
    }
}
