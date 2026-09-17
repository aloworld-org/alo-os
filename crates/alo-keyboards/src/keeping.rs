//! Where a person's keyboards are kept: `keyboards.toml`, in their own folder,
//! read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by: only the difference, under a format of this file's
//! own; no file is a person who has changed nothing; a file that is there and
//! wrong is refused whole, in this crate's words ([`crate::FileNotRead`]); and a
//! write is whole or not at all, read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! # The shape of the file
//!
//! ```toml
//! format = 1
//! layouts = ["de", "gr"]
//! compose = "Menu"
//! input-methods = ["Japanese"]
//! ```
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about keyboards.
//!
//! **Nothing here watches the file.** A change made in Settings takes effect at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::Changes;
use crate::keyboards::Keyboards;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "keyboards.toml";

/// The shape of `keyboards.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: shortcuts or the dock changing their files says
/// nothing about this one.
pub const FORMAT: i64 = 1;

impl Kept for Changes {
    const FILE: &'static str = THE_FILE;
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = &["layouts", "compose", "input-methods"];
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

/// This person's keyboards: what the release ships, with their own file over it.
///
/// # Errors
/// [`FileNotRead`] when the file is there and did not read. Nothing in it is
/// honoured then, and the keyboards are the ones the release ships.
pub fn read(at: &Path) -> Result<Keyboards, FileNotRead> {
    Ok(Keyboards::shipped().with(alo_kept::read::<Changes>(at)?))
}

/// Keep what this person changed about their keyboards.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was.
pub fn keep(at: &Path, keyboards: &Keyboards) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, &keyboards.changes())
}

/// Put this person's keyboards back as the release ships them, replacing a file
/// that did not read.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::compose_key::ComposeKey;
    use crate::layout::Layout;
    use crate::testing::{a_language, rented};

    /// The file this crate keeps is named once, here.
    #[test]
    fn the_file_is_this_crates_own() {
        assert_eq!(<Changes as Kept>::FILE, "keyboards.toml");
        assert_eq!(<Changes as Kept>::FORMAT, 1);
    }

    /// **Every key the shape has is a key the file may hold**, so a key that is
    /// declared and left off the list would be refused as a hand edit.
    #[test]
    fn every_key_the_shape_writes_is_a_key_the_file_may_hold() {
        let mut keyboards = Keyboards::offered_with(&a_language("de"));
        keyboards
            .add(Layout::named("gr").unwrap(), &rented())
            .unwrap();
        keyboards.composes_with(ComposeKey::Menu);
        keyboards.type_language(&a_language("ja")).unwrap();
        let written = alo_kept::text_of(&keyboards.changes()).unwrap();
        for line in written.lines() {
            if let Some((key, _)) = line.split_once(" = ") {
                assert!(
                    key == "format" || <Changes as Kept>::KEYS.contains(&key),
                    "{key} is written and is not a key of this file"
                );
            }
        }
        assert!(written.contains("layouts"), "{written}");
        assert!(written.contains("input-methods"), "{written}");
    }
}
