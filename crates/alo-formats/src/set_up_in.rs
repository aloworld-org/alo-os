//! **The language and region a machine was set up in**, on its first-start
//! sheets: `/etc/alo/set-up-in.toml`, written as root before anybody has an
//! account and read by each person's own desktop after they sign in.
//!
//! The owner's decisions of 2026-10-28: the language picked on the first-start
//! sheet is **the person's reading language**, so alo OS speaks and writes
//! dates in it from the first minute; and **the region picked there is the
//! region the machine is in**, which is where a person's dates are written
//! like until they choose another.
//!
//! # Why a machine file and not the person's own
//!
//! The sheets are drawn by `alo-compositor`, which runs as root with every
//! capability dropped — no `CAP_DAC_OVERRIDE` — so it may not so much as open a
//! person's `0700` home (`alo-their-folders`' header measured that wall). And
//! when the sheets are answered there is no person yet: the account is made on
//! the next one. So they write here, where everyone on the machine may read,
//! beside `alo_keyboards::THE_MACHINES_KEYBOARDS`, which the same language
//! choice already writes.
//!
//! **And every person on the machine is given it**, not only the first: a
//! second person who has not said what they read reads the language the
//! machine was set up in, and their dates follow its region. Decided by the
//! owner on 2026-10-28.
//!
//! # The shape of the file
//!
//! ```toml
//! format = 1
//! language = "de"
//! region = "DE"
//! ```
//!
//! No file is a machine nobody has set up, which is not an error.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};
use alo_strings::Language;
use serde::{Deserialize, Serialize};

use crate::region_code::RegionCode;
use crate::unkept::{FileNotRead, FileNotWritten};

/// Where a machine keeps the language and region it was set up in.
pub const THE_MACHINES_SET_UP_IN: &str = "/etc/alo/set-up-in.toml";

/// The shape of the file, written as `format = 1` at its top.
pub const FORMAT: i64 = 1;

/// **What the first-start sheets chose.**
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetUpIn {
    /// The language, as a tag: `de`, `pt`, `ga`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    /// The region the machine is in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    region: Option<RegionCode>,
}

impl SetUpIn {
    /// The language the machine was set up in, or [`None`] where nobody said —
    /// or where the file names something that is not a language, which is a
    /// hand edit this does not guess at.
    #[must_use]
    pub fn language(&self) -> Option<Language> {
        self.language
            .as_deref()
            .and_then(|tag| Language::written(tag).ok())
    }

    /// The region the machine is in, where the sheet was answered.
    #[must_use]
    pub const fn region(&self) -> Option<&RegionCode> {
        self.region.as_ref()
    }
}

impl Kept for SetUpIn {
    const FILE: &'static str = "set-up-in.toml";
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = &["language", "region"];
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

/// **What the machine was set up in**; no file is nothing yet.
///
/// # Errors
/// [`FileNotRead`] for a file that is there and did not read. Nothing in it is
/// honoured then.
pub fn read(at: &Path) -> Result<SetUpIn, FileNotRead> {
    alo_kept::read(at)
}

/// **Keep the language this machine was set up in**, keeping the region the
/// file already names.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was — including a file that did
/// not read, which this does not write over.
pub fn keep_the_language(at: &Path, language: &Language) -> Result<(), FileNotWritten> {
    let mut set_up = read(at).unwrap_or_default();
    set_up.language = Some(language.tag().to_owned());
    alo_kept::keep_what_the_machine_knows(at, &set_up)
}

/// **Keep the region this machine is in**, keeping the language the file
/// already names.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was.
pub fn keep_the_region(at: &Path, region: &RegionCode) -> Result<(), FileNotWritten> {
    let mut set_up = read(at).unwrap_or_default();
    set_up.region = Some(region.clone());
    alo_kept::keep_what_the_machine_knows(at, &set_up)
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
        folder.join("set-up-in.toml")
    }

    /// **A machine nobody set up names nothing**, and that is not an error.
    #[test]
    fn a_machine_nobody_set_up_names_nothing() {
        let set_up = read(&a_file()).unwrap();
        assert_eq!(set_up.language(), None);
        assert_eq!(set_up.region(), None);
    }

    /// **What the sheets kept is what comes back**, each kept without losing
    /// the other.
    #[test]
    fn the_language_and_region_kept_are_read_back() {
        let at = a_file();
        keep_the_language(&at, &Language::written("de").unwrap()).unwrap();
        keep_the_region(&at, &RegionCode::named("AT").unwrap()).unwrap();
        let set_up = read(&at).unwrap();
        assert_eq!(set_up.language(), Some(Language::written("de").unwrap()));
        assert_eq!(set_up.region().map(RegionCode::code), Some("AT"));
        keep_the_language(&at, &Language::written("fr").unwrap()).unwrap();
        assert_eq!(
            read(&at).unwrap().region().map(RegionCode::code),
            Some("AT"),
            "a second language lost the region"
        );
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n\nlanguage = \"fr\"\nregion = \"AT\"\n"
        );
    }

    /// **Everyone on the machine may read it**, because a person's desktop
    /// reads it as the person and the sheet wrote it as root.
    #[cfg(unix)]
    #[test]
    fn everyone_on_the_machine_may_read_it() {
        use std::os::unix::fs::PermissionsExt as _;
        let at = a_file();
        keep_the_language(&at, &Language::written("ga").unwrap()).unwrap();
        let mode = std::fs::metadata(&at).unwrap().permissions().mode();
        assert_eq!(mode & 0o004, 0o004, "{mode:o}");
    }

    /// **A file that is not this shape is refused whole, and not written
    /// over**; a tag that is not a language is not guessed at.
    #[test]
    fn a_file_that_is_wrong_gives_nobody_anything() {
        let at = a_file();
        std::fs::write(&at, "format = 1\nlanguage = \"de\"\nwallpaper = 3\n").unwrap();
        assert_eq!(read(&at).unwrap_err().key(), Some("wallpaper"));
        assert!(keep_the_region(&at, &RegionCode::named("DE").unwrap()).is_err());
        assert!(
            std::fs::read_to_string(&at)
                .unwrap()
                .contains("wallpaper")
        );
        std::fs::write(&at, "format = 1\nlanguage = \"not a language\"\n").unwrap();
        assert_eq!(read(&at).unwrap().language(), None);
    }
}
