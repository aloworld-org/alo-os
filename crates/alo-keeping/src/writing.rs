//! Putting what happened on a disk, one entry at a time.
//!
//! A record that exists only in memory answers questions until the machine is
//! next turned off, and ADR 0001 §7 asks a question that outlives a session:
//! *what did the agent do, under whose authority, from which approval, against
//! which grant.* This is where that becomes true.
//!
//! # One entry, one line
//!
//! The file is a beginning ([`crate::Head`]) and then one line per entry, and
//! it is only ever appended to. Three things follow, and they are the reason
//! for the format:
//!
//! - **Writing an entry does not read the file.** A record whose every entry
//!   rewrote the whole thing would spend the day rewriting a year, and would
//!   put every entry ever written at risk of the one write that fails.
//! - **A write interrupted partway costs one entry, not the file.** What is on
//!   the disk before the last newline is whole, and a reader can say so
//!   ([`crate::Damage`]).
//! - **Somebody else's tools can read it.** A record is a public surface: at v1
//!   it is exported to a security team's own console, and a line of JSON is
//!   something they already have a reader for.
//!
//! # Each line reaches the disk before the next one starts
//!
//! [`Writing::keep`] flushes and syncs before it answers. That is slower than
//! letting the operating system decide when to write, and it is the right way
//! round: an entry sitting in a page cache when the machine loses power is an
//! entry that never happened, and the entry a machine loses to a crash is
//! disproportionately likely to be the one an incident is about.
//!
//! It also buys the guarantee above. Without a sync per entry, a machine losing
//! power could leave several whole lines missing behind a complete one, and
//! *the last line may be unfinished* would stop being true.
//!
//! # One writer
//!
//! The daemon holds one [`Writing`] for the machine's record. Two appending to
//! one file would interleave — and pruning, which is [`crate::pruning`], is a
//! method on this type for the same reason: it replaces the file, so anything
//! holding an older handle would go on writing into a file that is no longer
//! the record.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use alo_record::Entry;

use crate::failing::NotKept;
use crate::head::Head;
use crate::reading::Reading;

/// The machine's record, open to be added to.
#[derive(Debug)]
pub struct Writing {
    /// Where it is.
    path: PathBuf,
    /// The open file, positioned at the end.
    file: File,
    /// What its first line says.
    head: Head,
}

impl Writing {
    /// Open the record at this path, making one if there is none.
    ///
    /// A record that is already there is **checked before it is added to**: its
    /// first line has to say it is a record, and to say it is in a shape this
    /// version of alo OS knows.
    ///
    /// The folder it lives in is not created. Where the record goes is the
    /// daemon's decision, and a library that made directories on the way to a
    /// path it was handed would make a typo into a second record.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotARecord`] or [`NotKept::FromANewerAlo`] if what is there
    /// cannot be added to, and [`NotKept::NotOpened`] if the machine would not
    /// open or create it. In every one of them, nothing has been written.
    pub fn opening(path: &Path) -> Result<Self, NotKept> {
        match OpenOptions::new().append(true).open(path) {
            Ok(file) => Ok(Self {
                path: path.to_path_buf(),
                file,
                head: Reading::head_of(path)?,
            }),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Self::starting(path),
            Err(why) => Err(NotKept::opening(path, &why)),
        }
    }

    /// A record where there was none.
    ///
    /// Created with `create_new`, so a record that appeared between the failed
    /// open above and this line is never written over.
    fn starting(path: &Path) -> Result<Self, NotKept> {
        let head = Head::new();
        let line = head.line().map_err(|why| NotKept::NotOpened {
            path: path.display().to_string(),
            why: why.to_string(),
        })?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|why| NotKept::opening(path, &why))?;
        file.write_all(line.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|why| NotKept::opening(path, &why))?;
        Ok(Self {
            path: path.to_path_buf(),
            file,
            head,
        })
    }

    /// Add one thing that happened.
    ///
    /// The entry is written and on the disk before this answers.
    ///
    /// # Errors
    ///
    /// [`NotKept::NotAddedTo`], which is the one failure in this crate that
    /// means the record is no longer everything that happened — see
    /// [`NotKept::record_is_still_whole`]. A daemon that meets it has to say
    /// so rather than carry on.
    pub fn keep(&mut self, entry: &Entry) -> Result<(), NotKept> {
        let line = a_line(entry).map_err(|why| NotKept::adding(&self.path, &why.to_string()))?;
        self.file
            .write_all(line.as_bytes())
            .and_then(|()| self.file.flush())
            .and_then(|()| self.file.sync_data())
            .map_err(|why| NotKept::adding(&self.path, &why.to_string()))
    }

    /// Where the record is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Where the record starts, and whether anything has been removed from it.
    #[must_use]
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// The head, to be moved by a shortening.
    pub(crate) fn now_starts_at(&mut self, head: Head) {
        self.head = head;
    }

    /// The open file, given back after the record underneath it was replaced.
    pub(crate) fn writing_to(&mut self, file: File) {
        self.file = file;
    }
}

/// One entry as one line, with its newline.
///
/// # Errors
///
/// A `serde_json::Error`, which an [`Entry`] cannot cause — it is words,
/// numbers and moments. It is a `Result` rather than an unwrap for the reason
/// `alo-files`' `Failed::Missing` is one: this runs inside the daemon on every
/// execution, and a library that panics there takes the daemon with it.
pub(crate) fn a_line(entry: &Entry) -> Result<String, serde_json::Error> {
    let mut line = serde_json::to_string(entry)?;
    line.push('\n');
    Ok(line)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, an_afternoon, noon, turned_away};
    use crate::{Damage, Keeping, THE_FORMAT};
    use std::fs;

    /// **One entry is one line, whatever an agent put in it.** A verb name
    /// carrying a newline would otherwise end an entry early and start a line
    /// nobody wrote — the record's own version of an agent writing into a log.
    #[test]
    fn nothing_an_agent_can_say_writes_a_second_line() {
        let entry = turned_away("delete\neverything\r\u{1b}[2K", "there is no such verb");
        let line = a_line(&entry).unwrap();
        assert_eq!(line.matches('\n').count(), 1, "{line}");
        assert!(line.ends_with('\n'));
        assert!(!line.contains('\r'), "{line}");
    }

    /// A record where there was none begins with a line saying what it is, and
    /// says nothing has been removed from it.
    #[test]
    fn a_machine_with_no_record_gets_one_that_says_it_is_whole() {
        let folder = a_folder_of_our_own("starting");
        let path = folder.join("record.jsonl");
        let writing = Writing::opening(&path).unwrap();
        assert_eq!(writing.path(), path);
        assert_eq!(writing.head().format(), THE_FORMAT);
        assert!(writing.head().is_whole());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{\"format\":1}\n");
    }

    /// What was written is read back, in the order it happened — which is the
    /// whole of what this file has to get right.
    #[test]
    fn what_is_written_is_read_back_in_the_order_it_happened() {
        let folder = a_folder_of_our_own("keeping");
        let path = folder.join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        let afternoon = an_afternoon();
        for entry in &afternoon {
            writing.keep(entry).unwrap();
        }
        drop(writing);

        let reading = Reading::at(&path).unwrap();
        assert_eq!(reading.damage(), &Damage::none());
        assert_eq!(
            reading.record().everything().cloned().collect::<Vec<_>>(),
            afternoon
        );
    }

    /// **A record survives the machine being turned off**, which is the
    /// difference between evidence and a session. Opening an existing record
    /// adds to it rather than replacing it.
    #[test]
    fn a_record_is_added_to_rather_than_replaced_when_the_machine_comes_back() {
        let folder = a_folder_of_our_own("again");
        let path = folder.join("record.jsonl");

        let mut before = Writing::opening(&path).unwrap();
        before.keep(&turned_away("first", "no such verb")).unwrap();
        drop(before);

        let mut after = Writing::opening(&path).unwrap();
        assert!(after.head().is_whole());
        after.keep(&turned_away("second", "no such verb")).unwrap();
        drop(after);

        assert_eq!(Reading::at(&path).unwrap().record().len(), 2);
    }

    /// **A file that is not a record is refused, and nothing is written to
    /// it.** A daemon pointed at somebody's notes must not append to them, and
    /// must not read them as a record either.
    #[test]
    fn a_file_that_is_not_a_record_is_refused_and_left_alone() {
        let folder = a_folder_of_our_own("notes");
        let path = folder.join("notes.txt");
        fs::write(&path, "the invoice from Northstar\n").unwrap();

        let refused = Writing::opening(&path).unwrap_err();
        assert!(matches!(refused, NotKept::NotARecord { .. }), "{refused:?}");
        assert!(refused.record_is_still_whole());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "the invoice from Northstar\n",
            "nothing was written to it"
        );
    }

    /// **A record from a newer alo OS is refused rather than appended to.**
    /// Adding a line in a shape this version does not know would leave a file
    /// neither version can read.
    #[test]
    fn a_record_from_a_newer_alo_os_is_refused_and_left_alone() {
        let folder = a_folder_of_our_own("newer");
        let path = folder.join("record.jsonl");
        let from_the_future = "{\"format\":9,\"kept\":\"however a later alo OS says it\"}\n";
        fs::write(&path, from_the_future).unwrap();

        let refused = Writing::opening(&path).unwrap_err();
        assert!(matches!(refused, NotKept::FromANewerAlo { format: 9, .. }));
        assert_eq!(fs::read_to_string(&path).unwrap(), from_the_future);
    }

    /// A record in a folder that is not there is a refusal, not a folder this
    /// crate makes: where the record goes is the daemon's decision, and a typo
    /// must not become a second record nobody reads.
    #[test]
    fn a_record_in_a_folder_that_is_not_there_is_refused() {
        let folder = a_folder_of_our_own("missing-folder");
        let path = folder.join("nowhere").join("record.jsonl");
        let refused = Writing::opening(&path).unwrap_err();
        assert!(matches!(refused, NotKept::NotOpened { .. }), "{refused:?}");
        assert!(!path.exists());
    }

    /// The rule is not held by the writer. A record is written the same way
    /// whatever a machine is set to keep, and the rule is asked for only when
    /// something is about to be removed — which is where somebody can see it.
    #[test]
    fn nothing_about_how_long_a_record_is_kept_is_needed_to_write_one() {
        let folder = a_folder_of_our_own("no-rule");
        let path = folder.join("record.jsonl");
        let mut writing = Writing::opening(&path).unwrap();
        writing
            .keep(&turned_away("list_folder", "no grant"))
            .unwrap();
        assert_eq!(writing.head().under(), None);
        assert!(Keeping::default().keeps(noon(), noon()));
    }
}
