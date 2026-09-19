//! What a person is told when their own `leaving.toml` did not read, or a change
//! to it was not written.
//!
//! [`alo_kept`] decides *when* a file is refused and hands the reason here
//! without words, because what a person reads about their own settings is this
//! crate's to say. Every sentence names the file, and the one about a key nobody
//! declared names the key — *your settings could not be read* without either is
//! a sentence nobody can act on.
//!
//! **A file that did not read reopens nothing.** That is the clause worth
//! reading twice: where every other kept file falls back to what the release
//! ships, this one falls back to what the release ships *and* to an empty list,
//! so a file that cannot be understood never becomes a half-understood list of
//! applications opening themselves at somebody's next sign-in.
//!
//! There is no `Display`, for the reason the rest of this crate has none: the
//! only road to words is `said`, in the language the person reads.

use std::path::{Path, PathBuf};

use alo_kept::{Unread, Unwritten};
use alo_strings::{Filling, Said, Strings, Word};

use crate::words;

/// `leaving.toml` is there and did not read, so nothing in it is honoured:
/// nothing is reopened and this machine leaves as the release ships it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNotRead {
    /// The file that did not read.
    at: PathBuf,
    /// What was wrong with it.
    why: Unread,
}

impl FileNotRead {
    /// The file at `at` did not read, for this reason.
    pub(crate) fn new(at: &Path, why: Unread) -> Self {
        Self {
            at: at.to_owned(),
            why,
        }
    }

    /// The file that did not read.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// What was wrong with it, for whoever is deciding what to offer next.
    #[must_use]
    pub fn why(&self) -> &Unread {
        &self.why
    }

    /// The key at the top of the file that is not one of this file's, when that
    /// is what was wrong.
    #[must_use]
    pub fn key(&self) -> Option<&str> {
        match &self.why {
            Unread::UnknownKey { key } => Some(key),
            _ => None,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match &self.why {
            Unread::NotWhereItBelongs | Unread::Disk(_) => words::KEPT_NOT_READ,
            Unread::NotToml { line: Some(_) } => words::KEPT_NOT_UNDERSTOOD_AT,
            Unread::NotText
            | Unread::NotToml { line: None }
            | Unread::NoFormat
            | Unread::NotItsShape { .. } => words::KEPT_NOT_UNDERSTOOD,
            Unread::AnotherFormat { .. } => words::KEPT_ANOTHER_FORMAT,
            Unread::UnknownKey { .. } => words::KEPT_UNKNOWN_KEY,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let mut filling = Filling::of("path", self.at.display().to_string());
        match &self.why {
            Unread::NotToml { line: Some(line) } => filling = filling.and("line", line.to_string()),
            Unread::UnknownKey { key } => filling = filling.and("key", key.clone()),
            _ => {}
        }
        strings.say(&self.word().key(), &filling)
    }
}

/// A change to `leaving.toml` was not written, so the file — and what happens at
/// the next sign-in — is as it was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNotWritten {
    /// The file that was not replaced.
    at: PathBuf,
    /// Why not.
    why: Unwritten,
}

impl FileNotWritten {
    /// The file at `at` was not replaced, for this reason.
    pub(crate) fn new(at: &Path, why: Unwritten) -> Self {
        Self {
            at: at.to_owned(),
            why,
        }
    }

    /// The file that was not replaced.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// Why not, for whoever is fixing alo OS.
    #[must_use]
    pub fn why(&self) -> &Unwritten {
        &self.why
    }

    /// The file that was not written over because it did not read, and why it
    /// did not — asked at the moment of the write, so it is the file as it was
    /// then. [`None`] for every other refusal.
    #[must_use]
    pub fn did_not_read(&self) -> Option<FileNotRead> {
        match &self.why {
            Unwritten::OverAFileThatDidNotRead(why) => {
                Some(FileNotRead::new(&self.at, why.clone()))
            }
            _ => None,
        }
    }

    /// The string this crate declares for this refusal: the disk, a file that
    /// did not read and was kept, or alo OS.
    #[must_use]
    pub const fn word(&self) -> Word {
        match &self.why {
            Unwritten::Disk(_) => words::KEPT_NOT_WRITTEN,
            Unwritten::OverAFileThatDidNotRead(_) => words::KEPT_NOT_REPLACED,
            Unwritten::NotWhereItBelongs
            | Unwritten::NotExpressible { .. }
            | Unwritten::NotOnlyTheDifference
            | Unwritten::ReadBackAsSomethingElse
            | Unwritten::ReadBackRefused(_) => words::KEPT_NOT_EXPRESSIBLE,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of("path", self.at.display().to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// Where the tests pretend the file is. Nothing is opened.
    fn the_file() -> PathBuf {
        PathBuf::from("/home/ada/.config/alo/leaving.toml")
    }

    /// **Every reason a file did not read says a sentence that is declared**,
    /// with every gap filled and the file named.
    #[test]
    fn every_reason_a_file_did_not_read_is_said_with_the_file_named() {
        let strings = in_english();
        for why in [
            Unread::NotWhereItBelongs,
            Unread::Disk(std::io::ErrorKind::PermissionDenied),
            Unread::NotText,
            Unread::NotToml { line: None },
            Unread::NotToml { line: Some(3) },
            Unread::NoFormat,
            Unread::AnotherFormat { found: 2 },
            Unread::UnknownKey {
                key: "what-i-was-writing".to_owned(),
            },
            Unread::NotItsShape {
                said: "unknown field `title`".to_owned(),
            },
        ] {
            let refused = FileNotRead::new(&the_file(), why.clone());
            let said = refused.said(&strings);
            assert!(words::EVERY_WORD.contains(&refused.word()), "{why:?}");
            assert!(said.unfilled().is_empty(), "{why:?}: {said}");
            assert!(
                said.text().contains(&the_file().display().to_string()),
                "{why:?}: {said}"
            );
        }
    }

    /// **A key nobody declared is named**, and the line a file stopped at is
    /// given, because both are what a person needs to find in their editor.
    #[test]
    fn the_key_and_the_line_are_named() {
        let strings = in_english();
        let unknown = FileNotRead::new(
            &the_file(),
            Unread::UnknownKey {
                key: "what-i-was-writing".to_owned(),
            },
        );
        assert_eq!(unknown.key(), Some("what-i-was-writing"));
        assert!(
            unknown
                .said(&strings)
                .text()
                .contains("says what-i-was-writing")
        );

        let at_line = FileNotRead::new(&the_file(), Unread::NotToml { line: Some(7) });
        assert_eq!(at_line.key(), None);
        assert!(at_line.said(&strings).text().contains("line 7"));
    }

    /// **Every reason a change was not written is said**, and the disk is told
    /// apart from a fault in alo OS.
    #[test]
    fn every_reason_a_change_was_not_written_is_said() {
        let strings = in_english();
        for (why, word) in [
            (
                Unwritten::Disk(std::io::ErrorKind::StorageFull),
                words::KEPT_NOT_WRITTEN,
            ),
            (Unwritten::NotWhereItBelongs, words::KEPT_NOT_EXPRESSIBLE),
            (
                Unwritten::NotExpressible {
                    why: "no".to_owned(),
                },
                words::KEPT_NOT_EXPRESSIBLE,
            ),
            (Unwritten::NotOnlyTheDifference, words::KEPT_NOT_EXPRESSIBLE),
            (
                Unwritten::ReadBackAsSomethingElse,
                words::KEPT_NOT_EXPRESSIBLE,
            ),
            (
                Unwritten::ReadBackRefused(Unread::NoFormat),
                words::KEPT_NOT_EXPRESSIBLE,
            ),
            (
                Unwritten::OverAFileThatDidNotRead(Unread::NoFormat),
                words::KEPT_NOT_REPLACED,
            ),
        ] {
            let refused = FileNotWritten::new(&the_file(), why.clone());
            assert_eq!(refused.word(), word, "{why:?}");
            let said = refused.said(&strings);
            assert!(said.unfilled().is_empty(), "{why:?}: {said}");
            assert_eq!(
                refused.did_not_read().is_some(),
                word == words::KEPT_NOT_REPLACED,
                "{why:?}"
            );
        }
    }

    /// **A file kept because it did not read says why it did not**, with the
    /// same file and the same reason a reading of it would have given.
    #[test]
    fn a_file_kept_because_it_did_not_read_says_why() {
        let why = Unread::UnknownKey {
            key: "title".to_owned(),
        };
        let refused =
            FileNotWritten::new(&the_file(), Unwritten::OverAFileThatDidNotRead(why.clone()));
        assert_eq!(
            refused.did_not_read(),
            Some(FileNotRead::new(&the_file(), why))
        );
    }
}
