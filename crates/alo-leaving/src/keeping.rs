//! Where what a person chose about leaving is kept, and where what was open is
//! written down: `leaving.toml`, in their own folder, read and written by this
//! crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives the file to the crate that declares its shape, and [`alo_kept`] holds
//! the rule it is kept by. What is written is [`Changes`] — only the difference —
//! under a `format` number of this file's own. No file is a person who has
//! changed nothing. A file that is there and wrong is refused whole, in this
//! crate's words ([`FileNotRead`]), and nothing in it is reopened. A write is
//! whole or not at all, and read back before it counts.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! # The list is written at one moment, and that moment is a log-out
//!
//! [`at_sign_out`] takes a [`crate::MayEnd`]. There is no other door in this
//! crate that writes a list, so what was open is written **once, as the session
//! ends**, and never while a person is working. A machine that kept the file up
//! to date as windows opened and closed would be a background reader of what
//! somebody is doing, which `CLAUDE.md` calls a bug in this product rather than
//! a feature.
//!
//! The cost is honest and is worth stating: a session that ends without a
//! log-out — a power cut, a crash — leaves nothing to reopen. The alternative is
//! a file that follows a person around all day, and it is not close.
//!
//! # And it is written only for somebody who asked
//!
//! [`at_sign_out`] reads the person's own choice at the moment of the write. If
//! *open my applications again* is off, no list is written and any list that was
//! there is taken out — so a machine nobody configured holds no record of what
//! anybody had open, and turning the setting off is enough to remove one.
//!
//! **Its section of
//! [the contract](../../../docs/contracts/person-settings.md)** is what this
//! file looks like to everybody outside this repository — its keys, its values,
//! what a missing file means and what a file that will not read is told — and
//! `tests/the_contract_describes_this_file.rs` holds the two together.
//!
//! **This crate does not know where the folder is.** It is handed the path, by
//! whoever starts the session, so that there is one answer to *where is a
//! person's folder* and it is not in a crate about logging out.
//!
//! **Nothing here watches the file.** A change made in Settings is drawn at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::changes::{Changes, Setting, Settings};
use crate::ending::MayEnd;
use crate::open::WasOpen;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "leaving.toml";

/// The shape of `leaving.toml`, written as `format = 1` at its top.
///
/// Its own, and nobody else's: when leaving gains a setting at a later release,
/// this number is this file's to move, and appearance's file does not notice.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Changes`] writes, and a test holds the two together.
const KEYS: &[&str] = &["reopen", "was-open"];

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

/// What the person changed about leaving, and what was open when they last
/// left, read from the file at `at`.
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
pub fn keep(at: &Path, changes: &Changes) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, changes)
}

/// Put this section back as alo OS ships it: the file at `at` replaced by the
/// format line alone, **whatever is there now — including a file that did not
/// read.**
///
/// The one door that writes over such a file, for the person's deliberate act in
/// Settings. It takes no changes, so nothing but the release's settings can reach
/// a broken file through it; afterwards the file reads as [`Changes::untouched`],
/// which is nothing reopened and no list kept.
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Changes>(at)
}

/// The person is logging out: keep what was open, if they asked for it to be.
///
/// The one door in this crate that writes a list, and it can only be called by
/// somebody holding a [`MayEnd`] — which is to say by a log-out that asked every
/// application to close first (`crate::logging_out`). The choice is read from
/// the file as it is at this moment, so a person who turned the setting off five
/// minutes ago is not written a list, and the list they had is taken out.
///
/// A person who has changed nothing and asked for nothing is left with **no
/// file**, because an untouched machine has none and a log-out is not a change.
///
/// # Errors
///
/// [`FileNotWritten`], including [`FileNotWritten::did_not_read`] when the
/// person's file is there and does not read — their file is kept as it is, and
/// nothing is reopened at the next sign-in.
pub fn at_sign_out(at: &Path, may_end: &MayEnd, was_open: &WasOpen) -> Result<(), FileNotWritten> {
    // The value is not read: a `MayEnd` is a fact about the walk that happened,
    // and its only job here is that one cannot be made without it.
    let _ended = may_end;
    let mut changes = read(at).unwrap_or_else(|_| Changes::untouched());
    if Settings::shipped().with(&changes).reopen {
        changes.set_what_was_open(was_open.clone());
    } else {
        let _had_a_list = changes.forget(Setting::WhatWasOpen);
    }
    if changes.is_untouched() && !at.exists() {
        return Ok(());
    }
    keep(at, &changes)
}

/// The settings a session runs by when a person signs in, from the file at `at`.
///
/// The release's settings with the person's changes over it — or, when the file
/// did not read, the release's settings alone and the refusal beside it, for
/// Settings to say in that section. Never the half of a file that read.
///
/// What is **reopened** is `crate::restoring::at_sign_in`, which asks the same
/// file the same way and applies the one extra rule this file's list has.
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
    use crate::split::Split;
    use crate::testing::{a_desk, a_folder_of_our_own, the_editor, the_laptop};

    /// A machine where every application closed, so a list may be kept.
    fn everything_closed() -> MayEnd {
        MayEnd::everything_closed()
    }

    /// **The list of keys is every key a change writes**: both are written as
    /// `reopen` and `was-open` and nothing else.
    #[test]
    fn every_key_a_change_writes_is_on_the_list() {
        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        changes.set_what_was_open(a_desk());
        let text = alo_kept::text_of(&changes).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let written: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .collect();
        assert_eq!(written, KEYS, "{text}");
    }

    /// **Nothing changed is a format line and nothing else.**
    #[test]
    fn nothing_changed_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Changes::untouched()).unwrap(),
            "format = 1\n"
        );
    }

    /// **A person who asked for nothing is left with no file at a log-out**, so
    /// a machine nobody configured holds no record of what was open.
    #[test]
    fn a_person_who_asked_for_nothing_has_no_file_after_a_log_out() {
        let at = a_folder_of_our_own("asked-for-nothing").join(THE_FILE);
        at_sign_out(&at, &everything_closed(), &a_desk()).unwrap();
        assert!(!at.exists(), "a log-out wrote a file nobody asked for");
        assert_eq!(at_sign_in(&at), (Settings::shipped(), None));
    }

    /// **A person who asked for it gets the list, and it survives to the next
    /// sign-in**, as the applications and places it was written from.
    #[test]
    fn a_person_who_asked_for_it_gets_the_list_back() {
        let at = a_folder_of_our_own("asked-for-it").join(THE_FILE);
        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        keep(&at, &changes).unwrap();

        at_sign_out(&at, &everything_closed(), &a_desk()).unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(refused, None);
        assert!(settings.reopen);
        assert_eq!(read(&at).unwrap().what_was_open(), a_desk());
    }

    /// **Turning it off takes the list with it at the next log-out**, and the
    /// file says the choice and nothing else.
    #[test]
    fn turning_it_off_takes_the_list_with_it() {
        let at = a_folder_of_our_own("turned-off").join(THE_FILE);
        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        keep(&at, &changes).unwrap();
        at_sign_out(&at, &everything_closed(), &a_desk()).unwrap();
        assert!(!read(&at).unwrap().what_was_open().is_nothing());

        let mut off = Changes::untouched();
        off.set_reopen(false);
        keep(&at, &off).unwrap();
        at_sign_out(&at, &everything_closed(), &a_desk()).unwrap();
        assert!(read(&at).unwrap().what_was_open().is_nothing());
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n\nreopen = false\n"
        );
    }

    /// **A file that is there and wrong is refused whole**, the machine leaves
    /// as it ships, and the refusal names the key — and a change is not written
    /// over it.
    #[test]
    fn a_wrong_file_is_refused_whole_and_not_written_over() {
        let at = a_folder_of_our_own("wrong-file").join(THE_FILE);
        std::fs::write(
            &at,
            "format = 1\nreopen = true\nwhat-i-was-writing = \"march.odt\"\n",
        )
        .unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(
            settings,
            Settings::shipped(),
            "nothing in the file honoured"
        );
        assert_eq!(refused.unwrap().key(), Some("what-i-was-writing"));

        let before = std::fs::read(&at).unwrap();
        let refused = at_sign_out(&at, &everything_closed(), &a_desk()).unwrap_err();
        assert!(refused.did_not_read().is_some());
        assert_eq!(std::fs::read(&at).unwrap(), before, "the file was kept");

        put_back_as_shipped(&at).unwrap();
        assert_eq!(read(&at).unwrap(), Changes::untouched());
    }

    /// **A window with a title in it refuses the whole file.** There is no key
    /// for one, and a file that has one is a file alo OS does not read rather
    /// than a file it reads past.
    #[test]
    fn a_window_with_a_title_in_it_refuses_the_whole_file() {
        let at = a_folder_of_our_own("a-title").join(THE_FILE);
        std::fs::write(
            &at,
            "format = 1\nreopen = true\n\n[[was-open]]\napplication = \"org.example.Editor\"\non = \
             \"eDP-1 Built-in display\"\nsplit = \"left-half\"\ntitle = \"Invoices — March\"\n",
        )
        .unwrap();
        let (settings, refused) = at_sign_in(&at);
        assert_eq!(settings, Settings::shipped());
        assert!(refused.is_some(), "a title was read rather than refused");
    }

    /// **What the file holds is the identifier and the place**, and a person can
    /// read it: one table per window, three keys each.
    #[test]
    fn what_the_file_holds_is_an_identifier_and_a_place() {
        let at = a_folder_of_our_own("what-it-holds").join(THE_FILE);
        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        changes.set_what_was_open(
            WasOpen::nothing().and(
                crate::open::Open::of(
                    the_editor(),
                    the_laptop(),
                    Split::Of(alo_dividing::Place::LeftHalf),
                )
                .unwrap(),
            ),
        );
        keep(&at, &changes).unwrap();
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n\nreopen = true\n\n[[was-open]]\napplication = \"org.example.Editor\"\non \
             = \"eDP-1 Built-in display\"\nsplit = \"left-half\"\n"
        );
    }
}
