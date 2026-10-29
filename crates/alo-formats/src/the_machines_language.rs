//! **The language a machine was set up in**, on its first-start language
//! sheet: `/etc/alo/language.toml`, written as root before anybody has an
//! account and read by the person's own desktop after they sign in.
//!
//! The owner's decision of 2026-10-28: *the language picked on the first-start
//! language sheet is saved as the person's reading language, not only their
//! keyboard, so alo OS speaks and writes dates in it from the first minute.*
//!
//! # Why a machine file and not the person's own
//!
//! The sheet is drawn by `alo-compositor`, which runs as root with every
//! capability dropped — no `CAP_DAC_OVERRIDE` — so it may not so much as open a
//! person's `0700` home (`alo-their-folders`' header measured that wall). And
//! when the sheet is answered there is no person yet: the account is made on
//! the next sheet. So the sheet writes here, where everyone on the machine may
//! read, beside `alo_keyboards::THE_MACHINES_KEYBOARDS` which the same choice
//! already writes; and the person's own desktop, running as them, gives it to
//! them through `settings.toml`'s own door the first time they sign in having
//! said nothing about what they read. That half is `alo_shell`'s
//! `the_language_set_up_in`.
//!
//! # The shape of the file
//!
//! ```toml
//! format = 1
//! language = "de"
//! ```
//!
//! No file is a machine nobody has set up, which is not an error: a person on
//! it reads what they say they read, or the machine's source language.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};
use alo_strings::Language;
use serde::{Deserialize, Serialize};

use crate::unkept::{FileNotRead, FileNotWritten};

/// Where a machine keeps the language it was set up in.
pub const THE_MACHINES_LANGUAGE: &str = "/etc/alo/language.toml";

/// The shape of the file, written as `format = 1` at its top.
pub const FORMAT: i64 = 1;

/// **What the first-start sheet chose**, as the file holds it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetUpIn {
    /// The language, as a tag: `de`, `pt`, `ga`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

impl Kept for SetUpIn {
    const FILE: &'static str = "language.toml";
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = &["language"];
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

/// **Keep the language this machine was set up in**, where every person on it
/// can read it.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was.
pub fn keep(at: &Path, language: &Language) -> Result<(), FileNotWritten> {
    alo_kept::keep_what_the_machine_knows(
        at,
        &SetUpIn {
            language: Some(language.tag().to_owned()),
        },
    )
}

/// **The language this machine was set up in**, or [`None`] where nobody has
/// set it up — or where the file names something that is not a language,
/// which is a hand edit this does not guess at.
///
/// # Errors
/// [`FileNotRead`] for a file that is there and did not read. Nothing in it is
/// honoured then.
pub fn read(at: &Path) -> Result<Option<Language>, FileNotRead> {
    let set_up: SetUpIn = alo_kept::read(at)?;
    Ok(set_up.language.and_then(|tag| Language::written(&tag).ok()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_file() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let folder = std::env::temp_dir().join(format!(
            "alo-formats-machine-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&folder).unwrap();
        folder.join("language.toml")
    }

    /// **A machine nobody set up names no language**, and that is not an error.
    #[test]
    fn a_machine_nobody_set_up_names_none() {
        assert_eq!(read(&a_file()).unwrap(), None);
    }

    /// **What the sheet kept is what comes back**, and the file is the one
    /// line it needs.
    #[test]
    fn the_language_kept_is_the_language_read() {
        let at = a_file();
        keep(&at, &Language::written("de").unwrap()).unwrap();
        assert_eq!(read(&at).unwrap(), Some(Language::written("de").unwrap()));
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n\nlanguage = \"de\"\n"
        );
    }

    /// **Everyone on the machine may read it**, because the person's desktop
    /// reads it as the person and the sheet wrote it as root.
    #[cfg(unix)]
    #[test]
    fn everyone_on_the_machine_may_read_it() {
        use std::os::unix::fs::PermissionsExt as _;
        let at = a_file();
        keep(&at, &Language::written("ga").unwrap()).unwrap();
        let mode = std::fs::metadata(&at).unwrap().permissions().mode();
        assert_eq!(mode & 0o004, 0o004, "{mode:o}");
    }

    /// **A file that is not this shape is refused whole**, and a tag that is
    /// not a language is not guessed at.
    #[test]
    fn a_file_that_is_wrong_gives_nobody_a_language() {
        let at = a_file();
        std::fs::write(&at, "format = 1\nlanguage = \"de\"\nregion = 3\n").unwrap();
        assert_eq!(read(&at).unwrap_err().key(), Some("region"));
        std::fs::write(&at, "format = 1\nlanguage = \"not a language\"\n").unwrap();
        assert_eq!(read(&at).unwrap(), None);
    }
}
