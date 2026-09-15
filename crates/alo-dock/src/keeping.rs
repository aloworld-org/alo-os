//! Where what a person changed about their dock is kept: `dock.toml`, in their
//! own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and the dock is where the release puts it.
//! A write is whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about a dock.
//!
//! **Nothing here watches the file.** A change made in Settings is drawn at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::Changes;
use crate::dock::Dock;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "dock.toml";

/// The shape of `dock.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when the dock gains its size and one dock per
/// display at v0.5, this number is the dock's to move, and appearance's file
/// does not notice.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &["edge"];

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

/// What the person changed about their dock, read from the file at `at`.
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
/// in Settings. It takes no changes, so nothing but the release's dock can
/// reach a broken file through it; afterwards the file reads as
/// [`Changes::untouched`].
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

/// The dock a session draws when a person signs in, from the file at `at`.
///
/// The release's dock with the person's changes over it — or, when the file
/// did not read, the release's dock alone and the refusal beside it, for
/// Settings to say in that section. Never the half of a file that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Dock, Option<FileNotRead>) {
    match read(at) {
        Ok(changes) => (Dock::shipped().with(changes), None),
        Err(refused) => (Dock::shipped(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::edge::Edge;

    /// **The list of keys is every key a change writes**: a dock moved to every
    /// edge is written as `edge` and nothing else, and reads back as itself.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        for edge in [Edge::Bottom, Edge::Left, Edge::Right, Edge::Top] {
            let mut changes = Changes::untouched();
            changes.set_edge(edge);
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

    /// **An untouched dock is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }
}
