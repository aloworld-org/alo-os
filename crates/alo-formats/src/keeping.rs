//! `formats.toml`, in the person's own folder: how they want dates and times
//! written, and where they said they are — read and written by this crate and
//! nobody else (ADR 0038).
//!
//! [`alo_kept`] holds the rule it is kept by, the same rule as every other file
//! in that folder: only what the person changed, under a format of this file's
//! own; no file is a person who has changed nothing; a file that is there and
//! wrong is refused whole, in this crate's words ([`FileNotRead`]); and a write
//! is whole or not at all, read back before it counts.
//!
//! # The shape of the file
//!
//! ```toml
//! format = 1
//! timezone = "Europe/Brussels"
//! following-the-network = "yes"
//!
//! [regionally]
//! language = "pt"
//! region = "BE"
//! ```
//!
//! **This crate does not know where the folder is.** It is handed the path of
//! the file, by whoever starts the session.
//!
//! **Nothing here watches the file.** A change made in Settings takes effect at
//! once, because Settings and the compositor are one process; a file edited by
//! hand is read at the next sign-in.
//!
//! # It had a reader and no writer until 2026-10-28
//!
//! The shape was declared on 2026-09-17 and nothing wrote it or read it on a
//! running machine, so the key that is now `following-the-network` was spelled
//! `following_the_network` and the file carried its own `format` field. No
//! machine can hold a file in the old spelling, because nothing ever wrote one;
//! it was changed to the folder's own spelling before the first writer arrived.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};
use serde::{Deserialize, Serialize};

use crate::regionally::{NotARegion, Regionally};
use crate::timezone::{FollowingTheNetwork, Timezone};
use crate::unkept::{FileNotRead, FileNotWritten};

/// What the file is called inside the person's folder.
pub const THE_FILE: &str = "formats.toml";

/// The shape of the file, written as `format = 1` at its top.
pub const FORMAT: i64 = 1;

/// **What a person chose about how things are written, and where they are.**
///
/// Every field absent is a person who has chosen nothing, and the machine
/// follows the rule in `alo_shell::TheirFormats::regionally`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chosen {
    /// The language and region their formats follow, where they said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regionally: Option<Regionally>,
    /// Where they said this machine is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<Timezone>,
    /// Whether the machine may ask the network where it is.
    #[serde(
        default,
        rename = "following-the-network",
        skip_serializing_if = "FollowingTheNetwork::is_no"
    )]
    pub following_the_network: FollowingTheNetwork,
}

impl Chosen {
    /// The region whose way of writing dates and times the person chose, or
    /// [`None`] where they chose none and the language decides.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.regionally.as_ref().and_then(Regionally::region)
    }

    /// **Write dates and times the way this region does** — or, for
    /// [`None`], the way the person's language does.
    ///
    /// The language the formats are in stays what the file already said, and
    /// where it said nothing it is the language the person reads, `reading`. A
    /// person who reads Portuguese and chooses Belgium gets Portuguese words in
    /// Belgium's order, which is the case this crate was written for.
    ///
    /// # Errors
    /// [`NotARegion`] for a region that is not two capital letters, or a
    /// language CLDR cannot be asked about. Nothing is changed then.
    pub fn written_as_in(&mut self, region: Option<&str>, reading: &str) -> Result<(), NotARegion> {
        let Some(region) = region else {
            self.regionally = None;
            return Ok(());
        };
        let language = self
            .regionally
            .as_ref()
            .map_or(reading, Regionally::language);
        self.regionally = Some(Regionally::reading(language)?.in_region(region)?);
        Ok(())
    }
}

/// Every key the file may have at its top, besides `format`.
const KEYS: &[&str] = &["regionally", "timezone", "following-the-network"];

impl Kept for Chosen {
    const FILE: &'static str = THE_FILE;
    const FORMAT: i64 = FORMAT;
    const KEYS: &'static [&'static str] = KEYS;
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

/// **What a person chose**, read from the file at `at`. No file is a person
/// who has chosen nothing, and is not an error.
///
/// # Errors
/// [`FileNotRead`] for a file that is there and did not read. Nothing in it is
/// honoured then.
pub fn read(at: &Path) -> Result<Chosen, FileNotRead> {
    alo_kept::read(at)
}

/// **What a session uses at sign-in**: the file, or — when it did not read —
/// nothing chosen and the refusal beside it, never the half that read.
#[must_use]
pub fn at_sign_in(at: &Path) -> (Chosen, Option<FileNotRead>) {
    match read(at) {
        Ok(chosen) => (chosen, None),
        Err(refused) => (Chosen::default(), Some(refused)),
    }
}

/// **Keep what a person chose**, as the whole of the file at `at`.
///
/// # Errors
/// [`FileNotWritten`], and the file is as it was. A file that does not read is
/// not written over: [`put_back_as_shipped`] is the one door that replaces it.
pub fn keep(at: &Path, chosen: &Chosen) -> Result<(), FileNotWritten> {
    alo_kept::keep(at, chosen)
}

/// Put the file back as alo OS ships it — nothing chosen — whatever is there
/// now, including a file that did not read.
///
/// # Errors
/// [`FileNotWritten`] when the disk would not take it. The file is as it was.
pub fn put_back_as_shipped(at: &Path) -> Result<(), FileNotWritten> {
    alo_kept::put_back_as_shipped::<Chosen>(at)
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
            "alo-formats-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&folder).unwrap();
        folder.join(THE_FILE)
    }

    /// **An untouched machine has chosen nothing, and follows no network.**
    #[test]
    fn a_machine_nobody_has_touched_has_chosen_nothing() {
        let at = a_file();
        let read_back = read(&at).unwrap();
        assert_eq!(read_back, Chosen::default());
        assert_eq!(
            read_back.following_the_network,
            FollowingTheNetwork::No,
            "a machine nobody asked would have looked up where it is"
        );
    }

    /// **What is kept is what comes back**, region and zone apart.
    #[test]
    fn what_a_person_chose_is_what_is_read_back() {
        let at = a_file();
        let chosen = Chosen {
            regionally: Some(Regionally::reading("pt").unwrap().in_region("BE").unwrap()),
            timezone: Some(Timezone::named("Europe/Brussels").unwrap()),
            following_the_network: FollowingTheNetwork::Yes,
        };
        keep(&at, &chosen).unwrap();
        assert_eq!(read(&at).unwrap(), chosen);
    }

    /// **A file that is not formats is refused whole**, and a session given it
    /// uses nothing in it and says why.
    #[test]
    fn a_file_that_does_not_read_is_refused_whole_and_says_so() {
        let at = a_file();
        std::fs::write(
            &at,
            "format = 1\ntimezone = \"Europe/Brussels\"\nsize = 3\n",
        )
        .unwrap();
        let (chosen, refused) = at_sign_in(&at);
        assert_eq!(chosen, Chosen::default(), "half the file was honoured");
        assert_eq!(refused.unwrap().key(), Some("size"));
    }

    /// **A change is not written over a file that did not read**, and putting
    /// it back as shipped is the one door that replaces it.
    #[test]
    fn a_file_that_did_not_read_is_kept_until_it_is_put_back() {
        let at = a_file();
        std::fs::write(&at, "this is not toml at all").unwrap();
        let refused = keep(&at, &Chosen::default()).unwrap_err();
        assert!(refused.did_not_read().is_some());
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "this is not toml at all"
        );
        put_back_as_shipped(&at).unwrap();
        assert_eq!(read(&at).unwrap(), Chosen::default());
    }

    /// **Choosing a region keeps the language the formats were in**, and
    /// choosing none gives the decision back to the language.
    #[test]
    fn choosing_a_region_keeps_the_language_and_choosing_none_gives_it_back() {
        let mut chosen = Chosen::default();
        chosen.written_as_in(Some("BE"), "pt").unwrap();
        assert_eq!(chosen.region(), Some("BE"));
        assert_eq!(
            chosen.regionally.as_ref().map(Regionally::language),
            Some("pt")
        );
        chosen.written_as_in(Some("FR"), "de").unwrap();
        assert_eq!(
            chosen.regionally.as_ref().map(Regionally::language),
            Some("pt"),
            "a second region moved the language"
        );
        chosen.written_as_in(None, "de").unwrap();
        assert_eq!(chosen, Chosen::default());
    }

    /// **A region that is not one is refused and nothing changes.**
    #[test]
    fn a_region_that_is_not_one_changes_nothing() {
        let mut chosen = Chosen::default();
        chosen.written_as_in(Some("BE"), "pt").unwrap();
        let before = chosen.clone();
        assert!(chosen.written_as_in(Some("belgium"), "pt").is_err());
        assert_eq!(chosen, before);
    }
}
