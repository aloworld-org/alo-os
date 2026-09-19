//! Where a person's choices of what opens what are kept: `what-opens-what.toml`,
//! in their own folder, read and written by this crate and nobody else.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
//! gives each of a person's settings to the crate that declares its shape, and
//! [`alo_kept`] holds the rule it is kept by. What is written is [`Chosen`] —
//! only what the person chose — under a `format` number of this file's own. No
//! file is a person who has chosen nothing, and every kind opens in what the
//! applications declare. A file that is there and wrong is refused whole, in
//! this crate's words ([`FileNotRead`]), and nothing in it is honoured: a
//! half-read file would be the machine choosing the other half.
//!
//! **A file that did not read is not written over by the next change** — a hand
//! edit with one mistake in it stays for the person to mend — and
//! [`put_back_as_shipped`] is the one door that replaces it.
//!
//! **This crate does not know where the folder is.** It is handed the path by
//! whoever starts the session (`alo_choosing::the_persons_folder`, whose refusal
//! is the sentence a session with no home directory shows before a change).
//!
//! **Nothing here watches the file**, and nothing that answers an application's
//! request writes it: see [`crate::chosen`] on who changes a choice.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::chosen::Chosen;
use crate::unkept::{FileNotRead, FileNotWritten};

/// The file's name inside the person's folder.
pub const THE_FILE: &str = "what-opens-what.toml";

/// The shape of `what-opens-what.toml`, written as `format = 1` at its top.
pub const FORMAT: i64 = 1;

/// Every key the file may have besides `format` — which is every field a
/// [`Chosen`] writes, and a test holds the two together.
const KEYS: &[&str] = &["kinds"];

impl Kept for Chosen {
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

/// What the person chose, read from the file at `at`.
///
/// # Errors
///
/// [`FileNotRead`] for a file that is there and did not read, naming the file
/// and — when that was what was wrong — the key. A file that is not there is not
/// an error: it is [`Chosen::untouched`].
pub fn read(at: &Path) -> Result<Chosen, FileNotRead> {
    alo_kept::read(at)
}

/// These choices, kept as the whole of the file at `at`.
///
/// # Errors
///
/// [`FileNotWritten`] when the file was not replaced — by the disk, because the
/// text would not have read back as these choices, or because the file is there
/// and does not read ([`FileNotWritten::did_not_read`]). The file is as it was.
pub fn keep(at: &Path, chosen: &Chosen) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, chosen)
}

/// Put this section back as alo OS ships it: the file at `at` replaced by the
/// format line alone, **whatever is there now — including a file that did not
/// read.**
///
/// The one door that writes over such a file, for the person's deliberate act
/// in Settings. Afterwards every kind opens in what the applications declare.
///
/// # Errors
///
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Chosen>(at)
}

/// What a session answers *what opens this* with after a person signs in, from
/// the file at `at`.
///
/// The person's choices — or, when the file did not read, no choices and the
/// refusal beside it, for Settings to say in that section. Never the half of a
/// file that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Chosen, Option<FileNotRead>) {
    match read(at) {
        Ok(chosen) => (chosen, None),
        Err(refused) => (Chosen::untouched(), Some(refused)),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::application::Application;
    use alo_opening::Kind;

    /// **The list of keys is every key a choice writes.**
    #[test]
    fn every_key_a_choice_writes_is_on_the_list() {
        let mut chosen = Chosen::untouched();
        chosen.choose(
            Kind::Pdf,
            &Application::identified("org.gnome.Papers").unwrap(),
        );
        let text = alo_kept::text_of(&chosen).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let written: Vec<&str> = table
            .keys()
            .map(String::as_str)
            .filter(|key| *key != alo_kept::THE_FORMAT_KEY)
            .collect();
        assert_eq!(written, KEYS, "{text}");
    }

    /// **Nothing chosen is a format line and nothing else.**
    #[test]
    fn nothing_chosen_is_a_format_line() {
        assert_eq!(
            alo_kept::text_of(&Chosen::untouched()).unwrap(),
            "format = 1\n"
        );
    }
}
