//! Shortening a record, which is the only thing in alo OS that removes
//! evidence.
//!
//! `alo-record` has no `remove`, no `edit` and no `forget`. This is the one
//! place an entry can stop existing, and everything about how it is written is
//! about making that hard to do quietly:
//!
//! - **It takes a rule and a moment, and nothing else.** There is no way to say
//!   *remove this entry*, or *remove everything about that agent*. What goes is
//!   decided by [`Keeping`] and by when it is run, so what a person set is what
//!   happens.
//! - **It is a method on the writer.** A record cannot be shortened by anything
//!   that is not already holding it open to be written to, and a `&mut` means
//!   nothing is appended while it happens. The file is *replaced*, so anything
//!   holding an older handle would go on writing into a file that is no longer
//!   the record — [`crate::writing`] says so.
//! - **It leaves a mark that cannot age out.** What is removed is entries;
//!   where the record now starts is written into the first line, which pruning
//!   never walks. Two rounds of shortening leave a record that still says it
//!   has been shortened. [`crate::head`] has the reasoning.
//! - **It refuses a record it cannot read all of.** A line that was written
//!   whole and is not whole now is disk trouble or somebody's hand, and
//!   rewriting the file would tidy it away. [`crate::damage`] has that one.
//!
//! # Nothing is removed until the replacement is on the disk
//!
//! The shortened record is written beside the old one, flushed and synced, and
//! only then renamed over it. A machine that loses power partway leaves a
//! record that is exactly as long as it was, plus a file beside it that nothing
//! reads and the next shortening writes over.
//!
//! # A record that loses nothing is not rewritten at all
//!
//! A rule that keeps everything, and a rule under which nothing is old enough
//! yet, both answer without touching the file. The safest way to write a record
//! is not to.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

use alo_record::Entry;

use crate::failing::NotKept;
use crate::head::Head;
use crate::keeping::Keeping;
use crate::reading::Reading;
use crate::writing::{Writing, a_line};

/// What a shortening did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pruned {
    /// How many entries were removed.
    removed: usize,
    /// How many are still there.
    kept: usize,
    /// The moment the record now starts at, where anything has ever been
    /// removed from it.
    since: Option<SystemTime>,
}

impl Pruned {
    /// How many entries were removed.
    #[must_use]
    pub fn removed(self) -> usize {
        self.removed
    }

    /// How many are still there.
    #[must_use]
    pub fn kept(self) -> usize {
        self.kept
    }

    /// The moment the record now starts at, where anything has ever been
    /// removed from it — including by an earlier shortening.
    #[must_use]
    pub fn since(self) -> Option<SystemTime> {
        self.since
    }

    /// Whether this one removed anything.
    #[must_use]
    pub fn anything_removed(self) -> bool {
        self.removed > 0
    }
}

impl Writing {
    /// Remove what this rule no longer keeps, as of this moment.
    ///
    /// Nothing reads the clock: `now` is passed in, so what a rule removes is
    /// arithmetic that can be tested rather than waited for, and the daemon and
    /// a settings panel cannot disagree about the moment.
    ///
    /// # Errors
    ///
    /// [`NotKept::Damaged`] if part of the record cannot be read, which is a
    /// refusal to do something destructive rather than a failure;
    /// [`NotKept::NotShortened`] if the machine would not write the replacement
    /// — and in both, **nothing has been removed**. [`NotKept::NotARecord`] and
    /// the rest of the reading refusals, if the record went away underneath.
    pub fn prune(&mut self, keeping: Keeping, now: SystemTime) -> Result<Pruned, NotKept> {
        let reading = Reading::at(self.path())?;
        if reading.damage().must_be_looked_at() {
            return Err(NotKept::Damaged {
                path: self.path().display().to_string(),
            });
        }

        let unchanged = Pruned {
            removed: 0,
            kept: reading.record().len(),
            since: reading.head().since(),
        };
        let Some(oldest) = keeping.oldest_kept(now) else {
            return Ok(unchanged);
        };
        let kept: Vec<&Entry> = reading
            .record()
            .everything()
            .filter(|entry| keeping.keeps(entry.at(), now))
            .collect();
        let removed = reading.record().len() - kept.len();
        if removed == 0 && !reading.damage().last_line_is_unfinished() {
            return Ok(unchanged);
        }

        let head = reading.head().shortened_to(oldest, keeping);
        let shortened = self.written_beside(&head, &kept)?;
        fs::rename(&shortened, self.path()).map_err(|why| {
            // The replacement is what did not arrive, so the record is as it
            // was. Leaving the half-written one behind would be a file nothing
            // reads and something might.
            let could_not = NotKept::shortening(self.path(), &why);
            drop(fs::remove_file(&shortened));
            could_not
        })?;

        let file = OpenOptions::new()
            .append(true)
            .open(self.path())
            .map_err(|why| NotKept::opening(self.path(), &why))?;
        self.writing_to(file);
        self.now_starts_at(head.clone());

        Ok(Pruned {
            removed,
            kept: kept.len(),
            since: head.since(),
        })
    }

    /// The shortened record, written beside the one it will replace and on the
    /// disk before anything is renamed.
    fn written_beside(&self, head: &Head, kept: &[&Entry]) -> Result<PathBuf, NotKept> {
        let beside = beside(self.path());
        let mut lines = head
            .line()
            .map_err(|why| NotKept::shortening_because(self.path(), &why.to_string()))?;
        for entry in kept {
            lines.push_str(
                &a_line(entry)
                    .map_err(|why| NotKept::shortening_because(self.path(), &why.to_string()))?,
            );
        }

        // Whatever a shortening interrupted last time left here is not a
        // record and nothing reads it, so it is written over rather than
        // treated as somebody's file.
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&beside)
            .map_err(|why| NotKept::shortening(self.path(), &why))?;
        file.write_all(lines.as_bytes())
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|why| NotKept::shortening(self.path(), &why))?;
        drop(file);
        Ok(beside)
    }
}

/// Where the shortened record is written while it is being written.
///
/// Beside the record and named after it, so it is in the same folder — a
/// replacement written somewhere else could not be renamed over the record on
/// most machines, because a rename is not a copy across filesystems.
fn beside(path: &std::path::Path) -> PathBuf {
    let mut beside = path.as_os_str().to_owned();
    beside.push(".shortening");
    PathBuf::from(beside)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Damage;
    use crate::testing::{a_folder_of_our_own, day, noon, turned_away_at};

    /// A record with a fortnight of afternoons in it, one entry a day, oldest
    /// first.
    fn a_fortnight(path: &std::path::Path) -> Writing {
        let mut writing = Writing::opening(path).unwrap();
        for days_ago in (0..14_u32).rev() {
            writing
                .keep(&turned_away_at(noon() - day() * days_ago))
                .unwrap();
        }
        writing
    }

    /// **A machine that keeps everything loses nothing**, and the file is not
    /// rewritten at all: the safest way to write a record is not to.
    #[test]
    fn a_record_kept_for_good_is_never_touched() {
        let folder = a_folder_of_our_own("forever");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        let before = fs::read_to_string(&path).unwrap();

        let pruned = writing.prune(Keeping::Forever, noon()).unwrap();
        assert!(!pruned.anything_removed());
        assert_eq!(pruned.kept(), 14);
        assert_eq!(pruned.since(), None);
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(writing.head().is_whole());
    }

    /// A rule under which nothing is old enough yet removes nothing, and the
    /// record still says it is whole — a record that claimed to have been
    /// shortened when it had not would make the claim worthless.
    #[test]
    fn a_rule_nothing_is_old_enough_for_yet_leaves_the_record_whole() {
        let folder = a_folder_of_our_own("young");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        let before = fs::read_to_string(&path).unwrap();

        let pruned = writing
            .prune(Keeping::for_days(90).unwrap(), noon())
            .unwrap();
        assert!(!pruned.anything_removed());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(Reading::at(&path).unwrap().head().is_whole());
    }

    /// **What a rule removes is what is older than it, and nothing else.** The
    /// entries that stay are the ones a person would expect to find, and the
    /// record says where it now begins.
    #[test]
    fn a_rule_removes_what_is_older_than_it_and_says_where_the_record_starts() {
        let folder = a_folder_of_our_own("shortening");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);

        let pruned = writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();
        assert_eq!(pruned.removed(), 6, "the seven days before last week");
        assert_eq!(pruned.kept(), 8);
        assert_eq!(pruned.since(), Some(noon() - day() * 7));

        let reading = Reading::at(&path).unwrap();
        assert_eq!(reading.record().len(), 8);
        assert!(!reading.head().is_whole());
        assert_eq!(reading.head().under(), Keeping::for_days(7).ok());
        assert!(reading.damage().nothing_wrong());
        for entry in reading.record().everything() {
            assert!(entry.at() >= noon() - day() * 7);
        }
    }

    /// **The record goes on being written to afterwards**, into the file that
    /// replaced it rather than into the one that was renamed away.
    #[test]
    fn a_shortened_record_is_added_to_afterwards() {
        let folder = a_folder_of_our_own("afterwards");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();

        writing.keep(&turned_away_at(noon())).unwrap();
        let reading = Reading::at(&path).unwrap();
        assert_eq!(reading.record().len(), 9);
        assert!(
            !reading.head().is_whole(),
            "and it still says it was shortened"
        );
    }

    /// **Shortening twice does not make a record look untouched.** The mark is
    /// in the first line, which nothing prunes, so it survives every later
    /// round — which an entry saying the same thing would not have.
    #[test]
    fn a_record_shortened_twice_still_says_it_was_shortened() {
        let folder = a_folder_of_our_own("twice");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);

        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();
        let again = writing
            .prune(Keeping::for_days(3).unwrap(), noon() + day())
            .unwrap();
        assert!(again.anything_removed());

        let reading = Reading::at(&path).unwrap();
        assert!(!reading.head().is_whole());
        assert_eq!(reading.head().since(), Some(noon() + day() - day() * 3));
        for entry in reading.record().everything() {
            assert!(entry.at() >= noon() + day() - day() * 3);
        }
    }

    /// **A record with a line nobody can read is not shortened.** Rewriting the
    /// file would tidy the one thing somebody needs to look at out of it, so
    /// this is a refusal rather than a repair — and nothing is removed.
    #[test]
    fn a_record_with_an_unreadable_line_in_it_is_not_shortened() {
        let folder = a_folder_of_our_own("damaged");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        drop(writing);

        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("{\"at\":{\"secs_since_epoch\":0\n");
        fs::write(&path, &text).unwrap();

        writing = Writing::opening(&path).unwrap();
        let refused = writing
            .prune(Keeping::for_days(1).unwrap(), noon())
            .unwrap_err();
        assert!(matches!(refused, NotKept::Damaged { .. }), "{refused:?}");
        assert!(refused.record_is_still_whole());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            text,
            "nothing was removed"
        );
    }

    /// **A write the machine interrupted is tidied up by a shortening**, which
    /// is the one difference between the two kinds of damage: nothing complete
    /// was ever there to lose.
    #[test]
    fn a_shortening_drops_a_line_that_was_never_finished() {
        let folder = a_folder_of_our_own("unfinished");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        drop(writing);

        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("{\"at\":{\"secs_since");
        fs::write(&path, &text).unwrap();
        assert!(
            Reading::at(&path)
                .unwrap()
                .damage()
                .last_line_is_unfinished()
        );

        writing = Writing::opening(&path).unwrap();
        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();
        assert_eq!(Reading::at(&path).unwrap().damage(), &Damage::none());
    }

    /// Nothing is left beside the record when a shortening finishes, so the
    /// next one starts from a folder holding one record and nothing that looks
    /// like one.
    #[test]
    fn nothing_is_left_beside_the_record_afterwards() {
        let folder = a_folder_of_our_own("beside");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();

        let left: Vec<String> = fs::read_dir(&folder)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(left, ["record.jsonl"]);
    }

    /// The head a shortening writes is the head that is read back, so what a
    /// person is shown after a prune is what the file says.
    #[test]
    fn what_the_writer_says_afterwards_is_what_the_file_says() {
        let folder = a_folder_of_our_own("agreeing");
        let path = folder.join("record.jsonl");
        let mut writing = a_fortnight(&path);
        writing
            .prune(Keeping::for_days(7).unwrap(), noon())
            .unwrap();

        let read_back: Head = Reading::at(&path).unwrap().head().clone();
        assert_eq!(writing.head(), &read_back);
    }
}
