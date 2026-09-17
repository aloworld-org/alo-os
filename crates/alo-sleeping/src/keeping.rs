//! Where what a person changed about sleep is kept: `sleeping.toml`, in their
//! own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and this machine sleeps as the release ships it.
//! A write is whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about sleep.
//!
//! **Nothing here watches the file.** A change made in Settings is drawn at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::Changes;
use crate::changes::Settings;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "sleeping.toml";

/// The shape of `sleeping.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when sleep gains a setting at a later
/// release, this number is this file's to move, and appearance's file
/// does not notice.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &["keep-awake", "lid"];

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

/// What the person changed about sleep, read from the file at `at`.
///
/// # Errors
///
/// [`FileNotRead`] for a file that is there and did not read, naming the file
/// and — when that was what was wrong — the key. A file that is not there is not
/// an error: it is [`Changes::untouched`].
pub fn read(at: &Path) -> Result<Changes, FileNotRead> {
    alo_kept::read(at)
}

/// These changes, kept as the whole of the file at `at`.
///
/// # Errors
///
/// [`FileNotWritten`] when the file was not replaced — by the disk, or because
/// the text would not have read back as these changes, or because the file is
/// there and does not read ([`FileNotWritten::did_not_read`]). The file is as it
/// was.
///
/// **A file that does not read is not written over.** It is asked as it is at
/// this moment, not as it was at sign-in, and a person's hand edit with one
/// mistake in it is kept for them to mend; [`put_back_as_shipped`] is the one
/// way to replace it.
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

/// The settings a session runs by when a person signs in, from the file at `at`.
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
    use crate::lid::Lid;

    /// **The list of keys is every key a change writes**: both settings
    /// changed are written as `keep-awake` and `lid` and nothing else, and read
    /// back as themselves.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        for lid in [Lid::Sleeps, Lid::StaysAwakeWithADisplay] {
            let mut changes = Changes::untouched();
            changes.set_lid(lid);
            changes.set_keep_awake(true);
            let text = alo_kept::text_of(&changes).unwrap();
            let table: toml::Table = toml::from_str(&text).unwrap();
            let written: Vec<&str> = table
                .keys()
                .map(String::as_str)
                .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
                .collect();
            assert_eq!(written, KEYS, "{text}");
        }
    }

    /// **Nothing changed is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }

    /// **A person's choice survives being kept and read back at the next
    /// sign-in, and a file that is there and wrong is refused whole** — the
    /// machine sleeps as it ships and the refusal names the key.
    #[test]
    fn a_choice_is_kept_and_a_wrong_file_is_refused_whole() {
        let folder = std::env::temp_dir().join(format!(
            "alo-sleeping-keeping-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let at = folder.join(THE_FILE);

        assert_eq!(at_sign_in(&at), (Settings::shipped(), None));

        let mut changes = Changes::untouched();
        changes.set_lid(Lid::StaysAwakeWithADisplay);
        keep(&at, &changes).unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(refused, None);
        assert_eq!(settings.lid, Lid::StaysAwakeWithADisplay);
        assert!(!settings.keep_awake);

        std::fs::write(
            &at,
            "format = 1\nlid = \"stays-awake-with-a-display\"\nnever-sleep = true\n",
        )
        .unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(
            settings,
            Settings::shipped(),
            "nothing in the file honoured"
        );
        assert_eq!(refused.unwrap().key(), Some("never-sleep"));
        assert!(keep(&at, &changes).unwrap_err().did_not_read().is_some());

        put_back_as_shipped(&at).unwrap();
        assert_eq!(read(&at).unwrap(), Changes::untouched());
        let _ = std::fs::remove_dir_all(&folder);
    }
}
