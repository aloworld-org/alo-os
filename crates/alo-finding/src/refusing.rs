//! The seven ways there is no index at all.
//!
//! Every one of these is a refusal of the whole question rather than a gap in
//! the answer. A file whose words could not be read is a [`crate::Contents`]
//! on that file's entry, and a folder the walk could not enter is a line in
//! [`crate::Covered`]; this is for when there is no index to put an entry in.

use std::path::PathBuf;

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Why a folder could not be indexed, or an index could not be kept or read.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum NotIndexed {
    /// The folder was named from somewhere rather than from the root of the
    /// machine, so the index would be of a different folder each time the
    /// asking process was started somewhere else.
    #[error("{} does not say where it is from the root of the machine", at.display())]
    NotAbsolute {
        /// The folder, as it was named.
        at: PathBuf,
    },

    /// The folder itself could not be walked: it is not there, it is a file,
    /// or the machine would not read it. A folder found *inside* it that
    /// cannot be read is a line in the index saying so, because the rest of
    /// the index is still true.
    #[error("{} could not be indexed", at.display())]
    NotWalked {
        /// The folder, as it was named.
        at: PathBuf,
        /// What the file half said about it, in its own words.
        why: alo_files::Failed,
    },

    /// The index file read is an index of another folder: copied from
    /// somewhere, or two folders whose paths hash to one name.
    #[error("the index at hand is of {}, not of {}", indexed.display(), asked.display())]
    NotTheSame {
        /// The folder asked about.
        asked: PathBuf,
        /// The folder the index at hand is of.
        indexed: PathBuf,
    },

    /// The index could not be written where it is kept.
    #[error("the index could not be written to {}: {why}", at.display())]
    NotKept {
        /// Where it was to be written.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },

    /// The index file could not be opened or read.
    #[error("the index at {} could not be read: {why}", at.display())]
    NotOpened {
        /// Where it was looked for.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },

    /// The file is there and is not an index this machine can read: a torn
    /// write, another program's file, or a format from a later version.
    #[error("{} is not an index this machine can read: {why}", at.display())]
    NotAnIndex {
        /// The file.
        at: PathBuf,
        /// What was wrong with it, as a sentence.
        why: String,
    },

    /// The session has no home directory, so there is nowhere the person's
    /// index could be kept.
    #[error("there is no home directory to keep an index in")]
    NowhereToKeepIt,
}

impl NotIndexed {
    /// The word this refusal is said with.
    #[must_use]
    pub fn word(&self) -> &'static Word {
        match self {
            Self::NotAbsolute { .. } => &words::NOT_ABSOLUTE,
            Self::NotWalked { .. } => &words::NOT_WALKED,
            Self::NotTheSame { .. } => &words::NOT_THE_SAME,
            Self::NotKept { .. } => &words::NOT_KEPT,
            Self::NotOpened { .. } => &words::NOT_OPENED,
            Self::NotAnIndex { .. } => &words::NOT_AN_INDEX,
            Self::NowhereToKeepIt => &words::NOWHERE_TO_KEEP_IT,
        }
    }

    /// This refusal, in the language the person reads.
    ///
    /// For [`Self::NotWalked`] the sentence carries the file half's own,
    /// which `alo-files` words; a vocabulary that holds only this crate's
    /// list shows that half as a key, marked as such, rather than as English
    /// nobody offered to translate.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotAbsolute { at } => Filling::of("at", at.display().to_string()),
            Self::NotWalked { at, why } => {
                Filling::of("at", at.display().to_string()).and_said("why", &why.said(strings))
            }
            Self::NotTheSame { asked, indexed } => {
                Filling::of("asked", asked.display().to_string())
                    .and("indexed", indexed.display().to_string())
            }
            Self::NotKept { at, why }
            | Self::NotOpened { at, why }
            | Self::NotAnIndex { at, why } => {
                Filling::of("at", at.display().to_string()).and("why", why.clone())
            }
            Self::NowhereToKeepIt => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every refusal for these tests, one of each.
    pub(crate) fn every_refusal() -> Vec<NotIndexed> {
        vec![
            NotIndexed::NotAbsolute {
                at: PathBuf::from("Documents"),
            },
            NotIndexed::NotWalked {
                at: PathBuf::from("/home/ada/Documents"),
                why: alo_files::Failed::Gone {
                    path: "/home/ada/Documents".to_owned(),
                },
            },
            NotIndexed::NotTheSame {
                asked: PathBuf::from("/home/ada/Pictures"),
                indexed: PathBuf::from("/home/ada/Documents"),
            },
            NotIndexed::NotKept {
                at: PathBuf::from("/home/ada/.local/share/alo/finding/x.index"),
                why: "read-only filesystem".to_owned(),
            },
            NotIndexed::NotOpened {
                at: PathBuf::from("/home/ada/.local/share/alo/finding/x.index"),
                why: "no such file".to_owned(),
            },
            NotIndexed::NotAnIndex {
                at: PathBuf::from("/home/ada/.local/share/alo/finding/x.index"),
                why: "the first line is not an index's".to_owned(),
            },
            NotIndexed::NowhereToKeepIt,
        ]
    }

    /// Every refusal has a sentence, none of them is a key on somebody's
    /// screen, and the ones with a path in them name it.
    #[test]
    fn every_refusal_is_said_in_a_sentence_a_person_reads() {
        let mut vocabulary = crate::finding_words().unwrap();
        alo_files::words::declare_into(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(!said.text().starts_with("finding."), "{said}");
            assert!(!said.text().contains("files."), "{said}");
            if !matches!(refusal, NotIndexed::NowhereToKeepIt) {
                assert!(
                    said.text().contains("/home/ada") || said.text().contains("Documents"),
                    "{said}"
                );
            }
        }
    }
}
