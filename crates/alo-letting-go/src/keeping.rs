//! Where how far back a person asked their machine to keep what an agent
//! changed is kept: `undo.toml`, in their own folder, read and written by this
//! crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and the machine keeps what the release
//! ships. A write is whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **Its section of
//! [the contract](../../../docs/contracts/person-settings.md)** is what this
//! file looks like to everybody outside this repository, and
//! `tests/the_contract_describes_this_file.rs` holds the two together.
//!
//! **This crate does not know where the folder is.** For a person's own session
//! `alo_choosing::where_the_folder_is` works it out; for the privileged unit,
//! which has no session and must not guess at one, the folder is read from
//! [`crate::the_folder::Theirs`] — the path the person's own session wrote
//! down beside what it kept.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::{Changes, Settings};
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "undo.toml";

/// The shape of `undo.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when this gains a setting at a later release,
/// this number is this file's to move, and nothing else in the person's folder
/// notices.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &["window"];

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

/// What the person changed about how far back an undo reaches, read from the
/// file at `at`.
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
/// there and does not read ([`FileNotWritten::did_not_read`]). The file is as
/// it was.
pub fn keep(at: &Path, changes: &Changes) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, changes)
}

/// Put this back as alo OS ships it: the file at `at` replaced by the format
/// line alone, **whatever is there now — including a file that did not read.**
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

/// How far back this machine keeps what an agent changed for the person whose
/// folder holds the file at `at`.
///
/// The release's window with the person's change over it — or, when the file
/// did not read, the release's window alone and the refusal beside it, for
/// Settings to say in that section. Never the half of a file that read.
///
/// **This is the one road the unit takes.** It never invents a window of its
/// own and never reads a folder it was not pointed at: a refusal here is the
/// shipped window, which keeps rather than removes, so a file somebody typed
/// wrong can only ever cost a person snapshots they would have lost anyway —
/// never ones they would have kept.
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
    use alo_keeping_up::HowFarBack;

    /// A directory of this test's own, on a real disk.
    fn a_folder(what: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-letting-go-keeping-{what}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    /// **The list of keys is every key a change writes**, and nothing changed
    /// is a format line and nothing else.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        let changes = Changes::with_window(HowFarBack::of(30, 200).unwrap());
        let text = alo_kept::text_of(&changes).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let written: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .collect();
        assert_eq!(written, KEYS, "{text}");
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }

    /// **A person's window survives being kept and read back**, and a machine
    /// with no file keeps what alo OS ships.
    #[test]
    fn a_window_is_kept_and_read_back_and_no_file_is_what_alo_os_ships() {
        let at = a_folder("kept").join(THE_FILE);
        assert_eq!(at_sign_in(&at), (Settings::shipped(), None));

        let window = HowFarBack::of(30, 200).unwrap();
        keep(&at, &Changes::with_window(window)).unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(refused, None);
        assert_eq!(settings.window, window);
    }

    /// **A file that is there and wrong is refused whole and the machine keeps
    /// what alo OS ships** — including the one edit that matters most, a person
    /// trying to switch expiry off with a key nobody declared.
    #[test]
    fn a_wrong_file_is_refused_whole_and_the_shipped_window_is_used() {
        let at = a_folder("wrong").join(THE_FILE);
        std::fs::write(&at, "format = 1\nnever-expire = true\n").unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(settings, Settings::shipped());
        let refused = refused.unwrap();
        assert_eq!(refused.key(), Some("never-expire"));
    }

    /// **A window of nothing is refused**, so undo cannot be switched off by
    /// arithmetic in a settings file.
    #[test]
    fn a_window_of_nothing_in_the_file_is_refused() {
        let at = a_folder("nothing").join(THE_FILE);
        std::fs::write(&at, "format = 1\n[window]\ndays = 0\nturns = 50\n").unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(settings, Settings::shipped());
        assert!(refused.is_some());
    }

    /// **A file that did not read is not written over**, and
    /// [`put_back_as_shipped`] is the one door that replaces it.
    #[test]
    fn a_file_that_did_not_read_is_kept_until_it_is_put_back() {
        let at = a_folder("kept-wrong").join(THE_FILE);
        std::fs::write(&at, "format = 1\nnever-expire = true\n").unwrap();

        let refused =
            keep(&at, &Changes::with_window(HowFarBack::of(30, 200).unwrap())).unwrap_err();
        assert!(refused.did_not_read().is_some());
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\nnever-expire = true\n"
        );

        put_back_as_shipped(&at).unwrap();
        assert_eq!(at_sign_in(&at), (Settings::shipped(), None));
    }
}
