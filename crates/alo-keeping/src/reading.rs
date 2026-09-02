//! Reading a record back off a disk.
//!
//! What comes back is three things, and a caller gets all three: where the
//! record starts ([`crate::Head`]), what happened ([`Record`]), and what could
//! not be read ([`crate::Damage`]). None of them is optional, because each of
//! them is a way a short record can be short.
//!
//! # A file that is not there is not an empty record
//!
//! [`Reading::at`] refuses a missing file rather than answering with a record
//! of nothing. A machine that has done nothing and a machine whose record has
//! been deleted look identical from here, and the difference matters more than
//! any other difference this crate deals with — so the one thing that must not
//! happen is for a reader to answer *nothing happened* and be believed.
//!
//! A first boot creates the file through [`crate::Writing::opening`], which is
//! a deliberate act by the daemon rather than a side effect of somebody asking
//! a question.
//!
//! # Why the whole file at once
//!
//! A record is read to answer a question about all of it — `alo-record`'s
//! `Asking` walks every entry — and `Record` holds every entry anyway, so
//! streaming would buy a smaller peak for a moment and then allocate the same
//! thing. Reading it whole is also what makes the last line answerable: a line
//! that stops partway has no newline after it, and that is only visible to
//! something that can see the end of the file.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use alo_record::{Entry, Record};

use crate::damage::Damage;
use crate::failing::NotKept;
use crate::head::{Head, THE_FORMAT};

/// A record as it was read back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// Where the record starts.
    head: Head,
    /// What happened, oldest first.
    record: Record,
    /// What could not be read.
    damage: Damage,
}

impl Reading {
    /// Read the record at this path.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotThere`] if there is no record there, which is not the same
    /// as an empty one; [`NotKept::NotARecord`] if what is there does not begin
    /// the way a record begins; [`NotKept::FromANewerAlo`] if it is in a shape
    /// this version does not know; [`NotKept::NotRead`] if the machine would
    /// not read it.
    ///
    /// A line that cannot be read is **not** an error: it comes back as
    /// [`Reading::damage`], with everything that could be read, because a
    /// reader that refused the whole record over one bad line would make one
    /// bad line a way to hide the other ten thousand.
    pub fn at(path: &Path) -> Result<Self, NotKept> {
        let text = fs::read_to_string(path).map_err(|why| NotKept::reading(path, &why))?;
        Self::of(path, &text)
    }

    /// The same, from what the file held.
    ///
    /// Split out so that every shape a damaged record can take is testable
    /// without a disk — and so that the disk tests are about the disk.
    fn of(path: &Path, text: &str) -> Result<Self, NotKept> {
        let ends_with_a_newline = text.ends_with('\n');
        let mut lines: Vec<&str> = text.split('\n').collect();
        if ends_with_a_newline {
            // What follows the final newline is not a line; it is the end.
            lines.pop();
        }

        let mut lines = lines.into_iter();
        let head: Head = lines
            .next()
            .and_then(|first| serde_json::from_str(first).ok())
            .ok_or_else(|| NotKept::not_a_record(path))?;
        if head.format() > THE_FORMAT {
            return Err(NotKept::FromANewerAlo {
                path: path.display().to_string(),
                format: head.format(),
            });
        }
        if head.format() < THE_FORMAT {
            // Nothing this version can read wrote it, and it is not from a
            // newer alo OS either.
            return Err(NotKept::not_a_record(path));
        }

        let mut record = Record::default();
        let mut damage = Damage::none();
        let mut lines = lines.peekable();
        // The head is line one, so the first entry is line two.
        let mut at_line = 1_u64;
        while let Some(line) = lines.next() {
            at_line += 1;
            match serde_json::from_str::<Entry>(line) {
                Ok(entry) => record.keep(entry),
                Err(_) if lines.peek().is_none() && !ends_with_a_newline => {
                    // The last line, with nothing after it: a write the machine
                    // interrupted. Ordinary, and not the same as a line that
                    // was written whole and is not whole now.
                    damage.ends_partway();
                }
                Err(_) => damage.unreadable_at(at_line),
            }
        }

        Ok(Self {
            head,
            record,
            damage,
        })
    }

    /// Where the record starts, and whether anything has been removed.
    #[must_use]
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// What happened, oldest first.
    #[must_use]
    pub fn record(&self) -> &Record {
        &self.record
    }

    /// What could not be read.
    #[must_use]
    pub fn damage(&self) -> &Damage {
        &self.damage
    }

    /// Where a record starts, without reading the rest of it.
    ///
    /// What [`crate::Writing::opening`] asks before it appends anything: the
    /// first line says which shape the file is in, and appending to a shape
    /// this version does not know would leave a record neither version can
    /// read. Reading the whole file to answer that would be reading a year of
    /// evidence to write one line.
    pub(crate) fn head_of(path: &Path) -> Result<Head, NotKept> {
        let file = fs::File::open(path).map_err(|why| NotKept::reading(path, &why))?;
        let mut first = String::new();
        BufReader::new(file)
            .read_line(&mut first)
            .map_err(|why| NotKept::reading(path, &why))?;
        let head: Head = serde_json::from_str(first.trim_end_matches('\n'))
            .map_err(|_| NotKept::not_a_record(path))?;
        if head.format() > THE_FORMAT {
            return Err(NotKept::FromANewerAlo {
                path: path.display().to_string(),
                format: head.format(),
            });
        }
        if head.format() < THE_FORMAT {
            return Err(NotKept::not_a_record(path));
        }
        Ok(head)
    }

    /// What happened, taken out to be asked questions of.
    ///
    /// Takes `self`, so a caller that wants only the entries has to give up the
    /// head and the damage on purpose rather than by forgetting they were
    /// there.
    #[must_use]
    pub fn into_record(self) -> Record {
        self.record
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_line, an_afternoon, nowhere};

    /// A record written the way this crate writes one reads back whole: the
    /// beginning, every entry in the order it happened, and nothing wrong.
    #[test]
    fn a_record_reads_back_in_the_order_it_happened() {
        let afternoon = an_afternoon();
        let mut text = "{\"format\":1}\n".to_owned();
        for entry in &afternoon {
            text.push_str(&a_line(entry));
        }

        let reading = Reading::of(nowhere(), &text).unwrap();
        assert!(reading.head().is_whole());
        assert!(reading.damage().nothing_wrong());
        assert_eq!(reading.record().len(), afternoon.len());
        let moments: Vec<_> = reading.record().everything().map(Entry::at).collect();
        let written: Vec<_> = afternoon.iter().map(Entry::at).collect();
        assert_eq!(moments, written);
    }

    /// A record with nothing in it yet is a record, not a mistake: a machine
    /// signed into and not yet asked to do anything has a beginning and no
    /// entries.
    #[test]
    fn a_record_with_nothing_in_it_yet_is_still_a_record() {
        let reading = Reading::of(nowhere(), "{\"format\":1}\n").unwrap();
        assert!(reading.record().is_empty());
        assert!(reading.damage().nothing_wrong());
        assert!(reading.head().is_whole());
    }

    /// **A line that cannot be read is reported, not stepped over.** Everything
    /// around it still comes back, because refusing the whole record over one
    /// line would make one line a way to hide all of them.
    #[test]
    fn a_line_that_cannot_be_read_comes_back_as_damage_and_the_rest_comes_back() {
        let afternoon = an_afternoon();
        let mut text = "{\"format\":1}\n".to_owned();
        text.push_str(&a_line(afternoon.first().unwrap()));
        text.push_str("{\"at\":\"halfway through a sentence\n");
        for entry in afternoon.iter().skip(1) {
            text.push_str(&a_line(entry));
        }

        let reading = Reading::of(nowhere(), &text).unwrap();
        assert_eq!(reading.record().len(), afternoon.len());
        assert_eq!(reading.damage().unreadable(), [3]);
        assert!(reading.damage().must_be_looked_at());
        assert!(!reading.damage().last_line_is_unfinished());
    }

    /// **A write the machine interrupted is not damage to look at.** The last
    /// line, with no newline after it, is one entry that was being written when
    /// the power went — everything before it is intact and says so.
    #[test]
    fn a_last_line_that_stops_partway_is_an_interrupted_write() {
        let afternoon = an_afternoon();
        let mut text = "{\"format\":1}\n".to_owned();
        for entry in &afternoon {
            text.push_str(&a_line(entry));
        }
        text.push_str("{\"at\":{\"secs_since");

        let reading = Reading::of(nowhere(), &text).unwrap();
        assert_eq!(reading.record().len(), afternoon.len());
        assert!(reading.damage().last_line_is_unfinished());
        assert!(
            !reading.damage().must_be_looked_at(),
            "an interrupted write does not hold a record back from being shortened"
        );
        assert!(!reading.damage().nothing_wrong());
    }

    /// A whole entry that happens to have no newline after it is an entry, not
    /// a torn one — what makes a line unfinished is that it cannot be read, and
    /// this one can.
    #[test]
    fn a_whole_last_line_with_no_newline_after_it_is_an_entry() {
        let afternoon = an_afternoon();
        let last = a_line(afternoon.first().unwrap());
        let text = format!("{{\"format\":1}}\n{}", last.trim_end());
        let reading = Reading::of(nowhere(), &text).unwrap();
        assert_eq!(reading.record().len(), 1);
        assert!(reading.damage().nothing_wrong());
    }

    /// **Nothing that is not a record is read as one.** A file whose first line
    /// is an entry has lost its beginning, and a beginning is what says whether
    /// anything was removed — so it is refused rather than read as a record
    /// nobody has ever shortened.
    #[test]
    fn nothing_that_does_not_begin_like_a_record_is_read_as_one() {
        let entry = a_line(an_afternoon().first().unwrap());
        for not_a_record in ["", "\n", "notes about the invoice\n", &entry] {
            let refused = Reading::of(nowhere(), not_a_record).unwrap_err();
            assert!(
                matches!(refused, NotKept::NotARecord { .. }),
                "{not_a_record:?} was read as a record"
            );
        }
    }

    /// **A record from a newer alo OS is refused, and the shape it is in is
    /// carried back.** Reading it as though it were this shape would be reading
    /// a file by guessing.
    #[test]
    fn a_record_from_a_newer_alo_os_is_refused() {
        let refused = Reading::of(nowhere(), "{\"format\":2}\n").unwrap_err();
        assert!(matches!(refused, NotKept::FromANewerAlo { format: 2, .. }));

        // And one claiming a shape that never existed is simply not a record.
        let never = Reading::of(nowhere(), "{\"format\":0}\n").unwrap_err();
        assert!(matches!(never, NotKept::NotARecord { .. }));
    }

    /// A record that has been shortened says so as soon as it is read, before
    /// anybody counts what is in it.
    #[test]
    fn a_shortened_record_says_where_it_starts_as_soon_as_it_is_read() {
        let head = "{\"format\":1,\"since\":{\"secs_since_epoch\":1760000000,\
                    \"nanos_since_epoch\":0},\"under\":{\"for-days\":30}}\n";
        let reading = Reading::of(nowhere(), head).unwrap();
        assert!(!reading.head().is_whole());
        assert!(reading.head().since().is_some());
        assert_eq!(
            reading.head().under(),
            crate::Keeping::for_days(30).ok(),
            "the rule that removed it is kept beside the moment"
        );
    }
}
