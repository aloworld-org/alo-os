//! Everything that happened on this machine, in the order it happened.
//!
//! The list, and only the list. What can be *asked* of it lives in
//! [`crate::explain`], because keeping entries and answering questions about
//! them are two reasons for a file to change and one of them is going to change
//! far more often than the other.
//!
//! **Nothing takes an entry out.** There is no `remove`, no `edit` and no
//! `forget` — not even the housekeeping [`alo_capability::Grants`] and
//! [`alo_capability::Approvals`] have, and the difference is deliberate. An
//! expired grant permits nothing whether or not it has been swept up, so
//! sweeping it changes no answer; a record somebody can shorten answers a
//! different question afterwards than it did before, and that is not evidence.
//! How long a record is kept, and by what, is a decision for whatever writes it
//! to a disk, made once and in the open, rather than a method anything holding
//! this type can reach for.

use serde::{Deserialize, Serialize};

use crate::entry::Entry;
use crate::explain::Asking;

/// Everything that happened, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// In the order it happened, which is the order somebody reads it in.
    #[serde(default)]
    kept: Vec<Entry>,
}

impl Record {
    /// Keep one entry.
    ///
    /// Cannot refuse. A record that could decline to keep something would be a
    /// record with a way to be silent about exactly the entry that mattered.
    pub fn keep(&mut self, entry: Entry) {
        self.kept.push(entry);
    }

    /// Everything, oldest first.
    pub fn everything(&self) -> impl Iterator<Item = &Entry> {
        self.kept.iter()
    }

    /// Answer a question (see [`Asking`]).
    ///
    /// This is what ADR 0001 §7 means by *"explain what it did" is a query, not
    /// a log to grep*: the questions a person or a security review actually
    /// asks are asked of the record in its own terms, rather than by matching
    /// text against lines that were formatted for somebody else.
    pub fn answering<'a>(&'a self, asking: &'a Asking) -> impl Iterator<Item = &'a Entry> {
        self.kept.iter().filter(move |entry| asking.matches(entry))
    }

    /// How many entries there are.
    #[must_use]
    pub fn len(&self) -> usize {
        self.kept.len()
    }

    /// Whether nothing has happened at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.kept.is_empty()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::test_calls::{
        archiving_march, files, granting, granting_both, hour, listing_invoices, mail, noon,
        proposing,
    };
    use crate::testing::in_english;
    use alo_capability::{Approvals, Authorised};

    /// One of each thing that can happen, in the order it happened.
    fn a_working_afternoon() -> Record {
        let mut record = Record::default();
        let grants = granting_both();

        let read = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
        record.keep(Entry::ran(&read, &in_english()));

        let mut approvals = Approvals::default();
        let id = approvals.propose(proposing(&archiving_march(), &grants));
        let approved = approvals.approve(id, noon()).unwrap();
        let running = approved.redeem(&grants, noon()).unwrap();
        record.keep(Entry::ran(&running, &in_english()));

        let refused = Authorised::read(
            &listing_invoices(),
            &files(),
            &granting(&["/home/anna/Taxes"]),
            noon() + hour(),
        )
        .unwrap_err();
        record.keep(Entry::refused(
            &refused,
            &files(),
            &in_english(),
            noon() + hour(),
        ));

        record.keep(Entry::answered_here(&mail(), noon() + hour()));
        record
    }

    /// Entries are kept in the order they happened, which is the order somebody
    /// reads them in when they ask what the agent did.
    #[test]
    fn what_happened_is_kept_in_the_order_it_happened() {
        let record = a_working_afternoon();
        assert_eq!(record.len(), 4);
        assert!(!record.is_empty());
        let moments: Vec<_> = record.everything().map(Entry::at).collect();
        assert_eq!(moments, [noon(), noon(), noon() + hour(), noon() + hour()]);
        assert!(Record::default().is_empty());
    }

    /// **A record keeping only successes cannot answer what a security review
    /// asks.** Every execution and every refusal is here, and the refusals are
    /// findable as refusals rather than by reading each entry.
    #[test]
    fn every_execution_and_every_refusal_leaves_a_record() {
        let record = a_working_afternoon();
        assert_eq!(
            record.everything().filter(|e| e.happened().ran()).count(),
            2
        );
        assert_eq!(
            record
                .everything()
                .filter(|e| e.happened().was_stopped())
                .count(),
            1
        );
    }

    /// Nothing takes an entry out. There is no method for it, so a record
    /// cannot be quietly shortened by whatever is holding one.
    #[test]
    fn nothing_takes_an_entry_out_of_the_record() {
        let mut record = a_working_afternoon();
        let before = record.len();
        record.keep(Entry::turned_away(
            "delete_everything",
            "there is no verb called delete_everything",
            &files(),
            noon() + hour(),
        ));
        assert_eq!(record.len(), before + 1);
        // Every entry that was there before is still there, unchanged.
        let kept: Vec<_> = record.everything().take(before).cloned().collect();
        assert_eq!(
            kept,
            a_working_afternoon()
                .everything()
                .cloned()
                .collect::<Vec<_>>()
        );
    }

    /// A record outlives the session that wrote it, or it can only answer
    /// questions asked before the machine was next turned off.
    #[test]
    fn a_record_survives_being_written_down_and_read_back() {
        let record = a_working_afternoon();
        let written = serde_json::to_string(&record).unwrap();
        let read = serde_json::from_str::<Record>(&written).unwrap();
        assert_eq!(read, record);
        assert_eq!(read.len(), 4);
    }
}
