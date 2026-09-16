//! What happened to the picture, once it had happened.
//!
//! Three endings, and they are [`crate::WhereItGoes`]' three answered: a file,
//! the clipboard, or both. There is no fourth and no *partly* — a picture that
//! was to go to a file and to the clipboard and reached only one of them is a
//! [`crate::NotTaken`], not a [`Taken`] with a field to check.
//!
//! # It says where, and never what
//!
//! A [`Taken`] carries the path a file was written at and nothing else. Not the
//! picture, not its size, not what was captured, not who asked. A value that
//! carried the bytes would be a copy of somebody's screen living for as long as
//! whoever held it, in a type whose whole purpose is to be shown in a message
//! and then dropped.
//!
//! # The message names the file and not the folder
//!
//! *Saved as 2026-09-16-120000.png* rather than the whole path: the person
//! chose the folder, so the folder is the thing they already know, and a
//! message that reads out somebody's home directory in a notification is a
//! message that says their account name out loud on a shared screen.

use std::path::{Path, PathBuf};

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What happened to a picture of the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken {
    /// It was written into the folder the person chose.
    Saved {
        /// Where the file is.
        at: PathBuf,
    },
    /// It was put on the clipboard, and nothing was written.
    Copied,
    /// Both, because somebody asked for both.
    SavedAndCopied {
        /// Where the file is.
        at: PathBuf,
    },
}

impl Taken {
    /// Where the file is, if one was written.
    #[must_use]
    pub fn at(&self) -> Option<&Path> {
        match self {
            Self::Saved { at } | Self::SavedAndCopied { at } => Some(at),
            Self::Copied => None,
        }
    }

    /// Whether the picture is on the clipboard.
    #[must_use]
    pub const fn is_on_the_clipboard(&self) -> bool {
        matches!(self, Self::Copied | Self::SavedAndCopied { .. })
    }

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::Saved { .. } => words::SAVED,
            Self::Copied => words::COPIED,
            Self::SavedAndCopied { .. } => words::SAVED_AND_COPIED,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Names the file rather than the whole path, for the reason this module's
    /// documentation gives.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let named = self
            .at()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
        strings.say(&self.word().key(), &Filling::of(words::NAME, named))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Where the pictures in these tests were written.
    fn at() -> PathBuf {
        PathBuf::from("/home/anna/Pictures/2026-09-16-120000.png")
    }

    /// **Each ending says where the picture went**, and only the two that wrote
    /// a file have a path.
    #[test]
    fn each_ending_says_where_the_picture_went() {
        assert_eq!(Taken::Saved { at: at() }.at(), Some(at().as_path()));
        assert!(!Taken::Saved { at: at() }.is_on_the_clipboard());

        assert_eq!(Taken::Copied.at(), None);
        assert!(Taken::Copied.is_on_the_clipboard());

        assert_eq!(
            Taken::SavedAndCopied { at: at() }.at(),
            Some(at().as_path())
        );
        assert!(Taken::SavedAndCopied { at: at() }.is_on_the_clipboard());
    }

    /// **The message names the file and not the folder.** A notification that
    /// read out somebody's home directory would say their account name out loud
    /// on a shared screen.
    #[test]
    fn the_message_names_the_file_and_not_the_folder() {
        let strings = in_english();
        let said = Taken::Saved { at: at() }.said(&strings);
        assert!(said.text().contains("2026-09-16-120000.png"), "{said}");
        assert!(!said.text().contains("/home/anna"), "{said}");
        assert!(!said.text().contains("Pictures"), "{said}");
    }

    /// **Each of the three reads as itself**, and no two read the same: a
    /// person told *copied* about a file they will go looking for has been told
    /// the wrong thing.
    #[test]
    fn every_ending_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let every = [
            Taken::Saved { at: at() },
            Taken::Copied,
            Taken::SavedAndCopied { at: at() },
        ];
        let mut seen: Vec<String> = Vec::new();
        for ending in &every {
            let said = ending.said(&strings);
            assert!(!said.is_a_bug(), "{ending:?} is not declared");
            assert!(!seen.contains(&said.text().to_owned()), "two say {said}");
            seen.push(said.text().to_owned());
        }
        assert!(seen.first().is_some_and(|said| said.contains("saved")));
        assert!(seen.get(1).is_some_and(|said| said.contains("copied")));
    }

    /// **The message arrives in the language the person reads**, and says so.
    #[test]
    fn a_message_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::COPIED,
            "Ein Bild Ihres Bildschirms wurde kopiert und kann eingefügt werden",
        )]);
        let said = Taken::Copied.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Ein Bild"));

        // The one nobody translated is still English, and says it is.
        let untranslated = Taken::Saved { at: at() }.said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
