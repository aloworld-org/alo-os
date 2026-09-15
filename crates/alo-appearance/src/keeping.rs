//! Where what a person changed about how their machine looks is kept:
//! `appearance.toml`, in their own folder, read and written by this crate and
//! nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference
//! — under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and the machine looks the way the release
//! ships it. A write is whole or not at all, and read back before it counts.
//!
//! **This crate does not know where the folder is.** It is handed the path,
//! by whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about wallpaper.
//!
//! **Nothing here watches the file.** A change made in Settings is drawn at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::appearance::Appearance;
use crate::changes::Changes;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "appearance.toml";

/// The shape of `appearance.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: the dock changing its file says nothing about
/// this one.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &[
    "background",
    "displays",
    "lock",
    "following",
    "text",
    "accent",
];

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

/// What the person changed, read from the file at `at`.
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
/// the text would not have read back as these changes. The file is as it was.
pub fn keep(at: &Path, changes: &Changes) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, changes)
}

/// What a session draws when a person signs in, from the file at `at`.
///
/// The release's appearance with the person's changes over it — or, when the
/// file did not read, the release's appearance alone and the refusal beside it,
/// for Settings to say in that section. Never the half of a file that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Appearance, Option<FileNotRead>) {
    match read(at) {
        Ok(changes) => (Appearance::shipped().with(changes), None),
        Err(refused) => (Appearance::shipped(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::accent::Accent;
    use crate::background::Background;
    use crate::display::DisplayId;
    use crate::lock::Lock;
    use crate::picture::{Fitting, Picture};
    use crate::rotating::{Every, Rotating};
    use crate::scheme::{Following, Schedule};
    use crate::text::TextScale;
    use crate::time::TimeOfDay;
    use crate::token::Token;

    /// A whole path on whichever machine runs the test.
    fn whole(path: &str) -> std::path::PathBuf {
        if cfg!(windows) {
            std::path::PathBuf::from(format!(r"C:\{path}"))
        } else {
            std::path::PathBuf::from(format!("/{path}"))
        }
    }

    /// Every setting changed, so every key is written.
    fn everything_changed() -> Changes {
        let mut changes = Changes::untouched();
        changes.set_background(Background::from(
            Rotating::folder(whole("home/ada/Pictures"), Every::minutes(10).unwrap())
                .unwrap()
                .fitted(Fitting::Fit),
        ));
        changes.set_background_on(
            DisplayId::named("HDMI-1").unwrap(),
            Background::from(Token::Navy.colour()),
        );
        changes.set_background_on(
            DisplayId::named("eDP-1").unwrap(),
            Background::from(Picture::file(whole("home/ada/harbour.jpg")).unwrap()),
        );
        changes.set_lock(Lock::Its(Background::from(
            Picture::shipped("alo").unwrap(),
        )));
        changes.follow(Following::from(
            Schedule::checked(
                TimeOfDay::checked(18, 0).unwrap(),
                TimeOfDay::checked(7, 30).unwrap(),
            )
            .unwrap(),
        ));
        changes.set_text(TextScale::percent(150).unwrap());
        changes.set_accent(Accent::Moss);
        changes
    }

    /// **The list of keys is every key a change writes**, so no field of
    /// [`Changes`] can be added without the file learning it — the text of a
    /// fully changed appearance reads back as itself, which it could not if a
    /// key were missing from the list.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        let text = alo_kept::text_of(&everything_changed()).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let written: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .collect();
        assert_eq!(written.len(), KEYS.len(), "{text}");
        for key in KEYS {
            assert!(written.contains(key), "{key} is not written: {text}");
        }
    }

    /// **An untouched appearance is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }
}
