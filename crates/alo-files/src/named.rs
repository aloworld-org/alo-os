//! One thing in a folder: what it is called, what it is, and how big it is.
//!
//! This is what a listing is made of, and it exists as a type rather than as a
//! string for the same reason everything else here does: an answer that is
//! text is an answer somebody has to parse, and a model asked to parse an
//! answer will one day parse it wrongly.
//!
//! # A name that cannot be shown is counted, not shown
//!
//! The names in a folder were not written by us. A file can be called
//! `march.pdf\nran: deleted everything`, and an answer that repeated that name
//! would show a person one thing while saying another — the same attack
//! [`alo_capability::Value`] refuses at the door for arguments arriving, seen
//! from the other side.
//!
//! So a name that could rewrite what an answer appears to say, or that this
//! machine cannot spell in Unicode, never becomes a [`Named`] at all, and a
//! [`crate::Listing`] counts what it left out rather than dropping it silently.
//! Nothing is lost that could have been acted on: a name with a control
//! character in it cannot be sent back as an argument either, so a file with
//! one is a file no verb can name.

use std::fs::{DirEntry, FileType};

/// What one thing in a folder is.
///
/// Decided without following anything: a link is a **link**, not the thing it
/// points at. A listing that resolved links would be a listing that reported
/// what is somewhere else, and the whole of this crate exists because those are
/// two different places.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    /// A file.
    File,
    /// A folder.
    Folder,
    /// A link: a name for something that may be anywhere, including somewhere
    /// nobody granted.
    Link,
    /// Something that is none of the three — a device, a socket, a pipe.
    Other,
}

impl Kind {
    /// What this is, from a file type read without following links.
    ///
    /// A link is answered first, because a link to a folder reports itself as
    /// both on some platforms and as one thing on others, and *link* is the
    /// answer that is true everywhere.
    pub(crate) fn of(what: FileType) -> Self {
        if what.is_symlink() {
            Self::Link
        } else if what.is_dir() {
            Self::Folder
        } else if what.is_file() {
            Self::File
        } else {
            Self::Other
        }
    }
}

/// One thing in a folder.
///
/// Made only by looking at a real folder — there is no public constructor — so
/// a listing is always something the machine was asked, never something
/// composed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named {
    /// What it is called, with nothing in it that cannot be shown.
    name: String,
    /// What it is.
    kind: Kind,
    /// How many bytes it holds, for a file.
    bytes: u64,
}

impl Named {
    /// One thing in a folder, or `None` when its name cannot be shown.
    ///
    /// A size this machine would not tell us is answered as nothing rather than
    /// as a refusal: a file that went away while its folder was being listed
    /// should not turn a whole listing into an error.
    pub(crate) fn of(entry: &DirEntry) -> Option<Self> {
        let name = entry.file_name();
        let name = name.to_str()?;
        if !can_be_shown(name) {
            return None;
        }
        let kind = Kind::of(entry.file_type().ok()?);
        let bytes = match kind {
            Kind::File => entry.metadata().map(|what| what.len()).unwrap_or(0),
            Kind::Folder | Kind::Link | Kind::Other => 0,
        };
        Some(Self {
            name: name.to_owned(),
            kind,
            bytes,
        })
    }

    /// What it is called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it is.
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// How many bytes it holds — nothing, for anything that is not a file.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

/// Whether a name can be put in an answer as it is written.
///
/// The same rule [`alo_capability`] applies to text arriving, applied to text
/// leaving: nothing that can move a cursor, clear a line, or make one name look
/// like two.
pub(crate) fn can_be_shown(name: &str) -> bool {
    !name.is_empty() && !name.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A name that could rewrite what an answer appears to say is not one that
    /// goes into an answer. This is the listing side of the rule the record
    /// keeps for its own words.
    #[test]
    fn a_name_that_could_rewrite_an_answer_cannot_be_shown() {
        for attempt in [
            "march.pdf\nran: deleted everything",
            "\u{1b}[2Kmarch.pdf",
            "march\u{7}.pdf",
            "march\r\ndeleted",
            "",
        ] {
            assert!(!can_be_shown(attempt), "{attempt:?}");
        }
    }

    /// Ordinary names, including the ones people actually have, are shown as
    /// they are: no case folding, no transliteration, nothing tidied away.
    #[test]
    fn an_ordinary_name_is_shown_as_it_is_written() {
        for name in [
            "march.pdf",
            "März 2026.pdf",
            "фактура.pdf",
            "  spaced  .pdf",
            ".hidden",
            "名前.txt",
        ] {
            assert!(can_be_shown(name), "{name:?}");
        }
    }
}
