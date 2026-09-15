//! Where what a person changed about their shortcuts is kept: `shortcuts.toml`,
//! in their own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the actions a
//! person moved or cleared — under a `format` number of this file's own. No file
//! is a person who has changed nothing. A file that is there and wrong is
//! refused whole, in this crate's words ([`FileNotRead`]), and every shortcut is
//! the one the release ships. A write is whole or not at all, and read back
//! before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! # The shape of the file
//!
//! [`Changes`] is a list, and a TOML file is a table of keys, so the list sits
//! under one key, `changed`, one table to an action:
//!
//! ```toml
//! format = 1
//!
//! [[changed]]
//! action = "TheAgent"
//! chord = { modifiers = ["Ctrl", "Alt"], key = "Space" }
//!
//! [[changed]]
//! action = "Launcher"
//! ```
//!
//! **An action with no `chord` is an action the person wants no shortcut for**,
//! which is a decision and is kept as one. That is why a table in the list with
//! a key it does not have is refused rather than ignored: a hand-edited `chrod`
//! would otherwise be read as a shortcut cleared.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about keys.
//!
//! **Nothing here watches the file.** A change made in Settings takes effect at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};
use serde::{Deserialize, Serialize};

use crate::changes::Changes;
use crate::shortcuts::Shortcuts;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "shortcuts.toml";

/// The shape of `shortcuts.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: appearance or the dock changing their files says
/// nothing about this one.
pub const FORMAT: i64 = 1;

/// The one key the file has besides `format`.
const CHANGED: &str = "changed";

/// [`Changes`] as the whole of the file: the list, under its key.
///
/// Private, because nobody but this module has any use for a list wearing a
/// key — what goes in and comes out is [`Changes`].
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
struct InTheFile {
    /// What the person moved or cleared, oldest first. Absent when nothing was.
    #[serde(default, skip_serializing_if = "Changes::is_empty")]
    changed: Changes,
}

impl Kept for InTheFile {
    const FILE: &'static str = THE_FILE;
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = &[CHANGED];
    type NotRead = FileNotRead;
    type NotWritten = FileNotWritten;

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(at: &Path, why: Unread) -> Self::NotRead {
        FileNotRead::new(at, why)
    }

    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten {
        FileNotWritten::new(at, why)
    }
}

/// What the person changed about their shortcuts, read from the file at `at`.
///
/// # Errors
///
/// [`FileNotRead`] for a file that is there and did not read, naming the file
/// and — when that was what was wrong — the key. A file that is not there is not
/// an error: it is [`Changes::none`].
pub fn read(at: &Path) -> Result<Changes, FileNotRead> {
    alo_kept::read::<InTheFile>(at).map(|file| file.changed)
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
    alo_kept::keep(
        at,
        &InTheFile {
            changed: changes.clone(),
        },
    )
}

/// Put this section back as alo OS ships it: the file at `at` replaced by the
/// format line alone, **whatever is there now — including a file that did not
/// read.**
///
/// The one door that writes over such a file, for the person's deliberate act
/// in Settings. It takes no changes, so nothing but the release's shortcuts can
/// reach a broken file through it; afterwards the file reads as
/// [`Changes::none`].
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<InTheFile>(at)
}

/// The shortcuts a session answers to when a person signs in, from the file at
/// `at`.
///
/// The release's shortcuts with the person's changes over them — or, when the
/// file did not read, the release's shortcuts alone and the refusal beside
/// them, for Settings to say in that section. Never the half of a file that
/// read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Shortcuts, Option<FileNotRead>) {
    match read(at) {
        Ok(changes) => (Shortcuts::shipped().with(changes), None),
        Err(refused) => (Shortcuts::shipped(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::chord::Chord;
    use crate::key::Key;
    use crate::modifier::{Modifier, Modifiers};

    /// **An untouched set of shortcuts is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&InTheFile::default()).unwrap(),
            "format = 1\n"
        );
    }

    /// **A moved shortcut and a cleared one are both written under the one
    /// key**, and the cleared one has no chord.
    #[test]
    fn a_moved_and_a_cleared_shortcut_are_written_under_the_one_key() {
        let mut changes = Changes::none();
        changes.set(
            Action::TheAgent,
            Some(
                Chord::checked(
                    Modifiers::just(Modifier::Ctrl).and(Modifier::Alt),
                    Key::Space,
                )
                .unwrap(),
            ),
        );
        changes.set(Action::Launcher, None);
        let text = alo_kept::text_of(&InTheFile { changed: changes }).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut keys: Vec<&str> = table.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, [CHANGED, alo_kept::THE_FORMAT_KEY], "{text}");
        let chords: Vec<bool> = table
            .get(CHANGED)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row.get("chord").is_some())
            .collect();
        assert_eq!(chords, [true, false], "{text}");
    }
}
