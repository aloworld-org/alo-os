//! Where what a person changed about notifications is kept: `notifying.toml`,
//! in their own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and notifications behave as the release
//! ships them. A write is whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **Its section of
//! [the contract](../../../docs/contracts/person-settings.md)** is what this
//! file looks like to everybody outside this repository — its keys, its values,
//! what a missing file means and what a file that will not read is told — and
//! `tests/the_contract_describes_this_file.rs` holds the two together.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about notifications.
//!
//! # Two keys, and no notification is one of them
//!
//! `do-not-disturb` and `quiet-hours`. A title, a body, a sender and a list of
//! what somebody missed are **not** in this file and are not in any file: they
//! live in [`crate::Missed`], in the session's own memory, and the machine
//! forgets them when the person signs out. `tests/what_a_person_missed_is_kept_here_and_never_synced.rs`
//! holds that, and [`crate::missed`] says why it is the right answer rather
//! than the easy one.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::{Changes, Settings};
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "notifying.toml";

/// The shape of `notifying.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when notifications gain a setting at a later
/// release, this number is this file's to move, and appearance's file does not
/// notice.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &["do-not-disturb", "quiet-hours"];

impl Kept for Changes {
    const FILE: &'static str = THE_FILE;
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = KEYS;
    type NotRead = FileNotRead;
    type NotWritten = FileNotWritten;

    fn untouched() -> Self {
        Self::untouched()
    }

    fn not_read(at: &Path, why: Unread) -> Self::NotRead {
        FileNotRead::new(at, why)
    }

    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten {
        FileNotWritten::new(at, why)
    }
}

/// What the person changed about notifications, read from the file at `at`.
///
/// # Errors
///
/// [`FileNotRead`] for a file that is there and did not read, naming the file
/// and — when that was what was wrong — the key. A file that is not there is
/// not an error: it is [`Changes::untouched`].
pub fn read(at: &Path) -> Result<Changes, FileNotRead> {
    alo_kept::read(at)
}

/// These changes, kept as the whole of the file at `at`.
///
/// # Errors
///
/// [`FileNotWritten`] when the file was not replaced — by the disk, or because
/// the text would not have read back as these changes, or because the file is
/// there and does not read ([`FileNotWritten::did_not_read`]). The file is as
/// it was.
pub fn keep(at: &Path, changes: &Changes) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, changes)
}

/// Put this section back as alo OS ships it: the file at `at` replaced by the
/// format line alone, **whatever is there now — including a file that did not
/// read.**
///
/// The one door that writes over such a file, for the person's deliberate act
/// in Settings. It takes no changes, so nothing but the release's settings can
/// reach a broken file through it; afterwards the file reads as
/// [`Changes::untouched`].
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

/// The settings a session runs by when a person signs in, from the file at
/// `at`.
///
/// The release's settings with the person's changes over it — or, when the file
/// did not read, the release's settings alone and the refusal beside it, for
/// Settings to say in that section. Never the half of a file that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Settings, Option<FileNotRead>) {
    match read(at) {
        Ok(changes) => (Settings::shipped().with(&changes), None),
        Err(refused) => (Settings::shipped(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::quiet_hours::QuietHours;
    use crate::testing::{a_folder_of_our_own, at};

    /// **The list of keys is every key a change writes**: both settings
    /// changed are written as `do-not-disturb` and `quiet-hours` and nothing
    /// else, and read back as themselves.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        let mut changes = Changes::untouched();
        changes.set_do_not_disturb(true);
        changes.set_quiet_hours(QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap());
        let text = alo_kept::text_of(&changes).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut written: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .collect();
        written.sort_unstable();
        let mut expected: Vec<&str> = KEYS.to_vec();
        expected.sort_unstable();
        assert_eq!(written, expected, "{text}");
    }

    /// **Nothing changed is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }

    /// **Nothing a sender wrote can be written into this file.** Not a title,
    /// not a body, not a sender, not a list of what somebody missed: the two
    /// keys are the whole shape, and a file naming anything else is refused
    /// whole with the key said back.
    #[test]
    fn nothing_a_sender_wrote_can_be_written_into_this_file() {
        let folder = a_folder_of_our_own("no-notification-in-the-file");
        let file = folder.join(THE_FILE);
        for key in ["missed", "title", "body", "sender", "waiting"] {
            std::fs::write(&file, format!("format = 1\n{key} = \"Anna Pärt\"\n")).unwrap();
            let (settings, refused) = at_sign_in(&file);
            assert_eq!(settings, Settings::shipped(), "{key}");
            assert_eq!(refused.unwrap().key(), Some(key));
        }
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **A person's choice survives being kept and read back at the next
    /// sign-in, and a file that is there and wrong is refused whole** — the
    /// machine behaves as it ships and the refusal names the key.
    #[test]
    fn a_choice_is_kept_and_a_wrong_file_is_refused_whole() {
        let folder = a_folder_of_our_own("keeping");
        let file = folder.join(THE_FILE);

        assert_eq!(at_sign_in(&file), (Settings::shipped(), None));

        let night = QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap();
        let mut changes = Changes::untouched();
        changes.set_quiet_hours(night);
        keep(&file, &changes).unwrap();
        let (settings, refused) = at_sign_in(&file);
        assert_eq!(refused, None);
        assert_eq!(settings.quiet_hours, Some(night));
        assert!(!settings.do_not_disturb);

        std::fs::write(&file, "format = 1\ndo-not-disturb = true\nloud = true\n").unwrap();
        let (settings, refused) = at_sign_in(&file);
        assert_eq!(
            settings,
            Settings::shipped(),
            "nothing in the file honoured"
        );
        assert_eq!(refused.unwrap().key(), Some("loud"));
        assert!(keep(&file, &changes).unwrap_err().did_not_read().is_some());

        put_back_as_shipped(&file).unwrap();
        assert_eq!(read(&file).unwrap(), Changes::untouched());
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **Hours that begin and end at once are refused where the file is
    /// read**, in this crate's words rather than in the words of whichever
    /// crate spells a time of day.
    #[test]
    fn hours_that_begin_and_end_at_once_are_refused_when_the_file_is_read() {
        let folder = a_folder_of_our_own("a-stretch-that-is-not-one");
        let file = folder.join(THE_FILE);
        std::fs::write(
            &file,
            "format = 1\n[quiet-hours]\nbegin = { hour = 23, minute = 0 }\nend = { hour = 23, \
             minute = 0 }\n",
        )
        .unwrap();
        let (settings, refused) = at_sign_in(&file);
        assert_eq!(settings, Settings::shipped());
        let refused = refused.unwrap();
        assert_eq!(refused.word(), crate::words::KEPT_NOT_UNDERSTOOD);
        assert_eq!(refused.at(), file.as_path());
        let _ = std::fs::remove_dir_all(&folder);
    }
}
