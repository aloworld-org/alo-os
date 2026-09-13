//! What a file is, read from its own bytes and never from its name.
//!
//! An extension is a claim the name makes, and a person searching for *the
//! PDF* wants the files that are PDFs — including the scan somebody saved as
//! `.jpg` and excluding the shell script somebody named `.pdf` as a joke. So
//! a [`Kind`] is decided from the first [`SNIFFED`] bytes of the file: a known
//! signature first, then whether what is there reads as text, and *bytes of
//! no known kind* otherwise. The name is never consulted.
//!
//! # Text is decided, not assumed
//!
//! A file is [`Kind::Text`] when what was read is UTF-8 and holds no control
//! character beyond the five a text file legitimately has — tab, newline,
//! carriage return, form feed and escape. A file in UTF-16, which begins with
//! a byte-order mark and then alternates bytes with zeros, is not text by this
//! rule and its words are not read; that is a limitation and the report says
//! so, rather than a guess that would index half the letters of every word.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// How many bytes of a file are read to decide what it is.
///
/// Every signature below is inside the first sixteen, and eight kibibytes of
/// text is enough to know whether it is text without reading a novel to
/// decide the question.
pub const SNIFFED: usize = 8 * 1024;

/// What a file is, from its own bytes.
///
/// Spelled in the index file exactly as the variant is named, in lower case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Kind {
    /// Text, whose words are what [`crate::Contents::Read`] holds.
    Text,
    /// A PDF document.
    Pdf,
    /// A PNG image.
    Png,
    /// A JPEG image.
    Jpeg,
    /// A GIF image.
    Gif,
    /// A zip archive, which is also what an office document is on the disk.
    Zip,
    /// A program the machine could run.
    Program,
    /// A file with nothing in it.
    Empty,
    /// A file the machine would not let this reader open, so its bytes were
    /// never seen.
    Unread,
    /// A folder.
    Folder,
    /// A link, which is an entry of its own and is never followed.
    Link,
    /// Something that is none of the above — a device, a socket, a pipe.
    Other,
    /// Bytes of no kind this crate knows — and, on the way in from a file, a
    /// kind this version does not know, which a later one may have added.
    /// Last in the list because that is where `serde` wants the catch-all.
    #[serde(other)]
    Bytes,
}

/// The signatures this crate knows, each with the kind it names.
const SIGNATURES: [(&[u8], Kind); 7] = [
    (b"%PDF-", Kind::Pdf),
    (b"\x89PNG\r\n\x1a\n", Kind::Png),
    (b"\xFF\xD8\xFF", Kind::Jpeg),
    (b"GIF87a", Kind::Gif),
    (b"GIF89a", Kind::Gif),
    (b"PK\x03\x04", Kind::Zip),
    (b"\x7FELF", Kind::Program),
];

impl Kind {
    /// Every kind, in the order a window might list them.
    pub const EVERY: [Self; 13] = [
        Self::Text,
        Self::Pdf,
        Self::Png,
        Self::Jpeg,
        Self::Gif,
        Self::Zip,
        Self::Program,
        Self::Bytes,
        Self::Empty,
        Self::Unread,
        Self::Folder,
        Self::Link,
        Self::Other,
    ];

    /// What a file is, from its first bytes — at most [`SNIFFED`] of them,
    /// and fewer only when the file itself is shorter.
    #[must_use]
    pub fn of_bytes(first: &[u8]) -> Self {
        if first.is_empty() {
            return Self::Empty;
        }
        for (signature, kind) in SIGNATURES {
            if first.starts_with(signature) {
                return kind;
            }
        }
        if is_text(first) {
            Self::Text
        } else {
            Self::Bytes
        }
    }

    /// What a thing that is not a file is — or [`None`] for a file, whose
    /// kind is a question for its bytes.
    #[must_use]
    pub fn of_thing(what: alo_files::Kind) -> Option<Self> {
        match what {
            alo_files::Kind::File => None,
            alo_files::Kind::Folder => Some(Self::Folder),
            alo_files::Kind::Link => Some(Self::Link),
            alo_files::Kind::Other => Some(Self::Other),
        }
    }

    /// Whether a file of this kind has words to index.
    #[must_use]
    pub const fn has_words(self) -> bool {
        matches!(self, Self::Text)
    }

    /// Whether this is the kind of a file, rather than of a folder, a link
    /// or something else the walk found.
    #[must_use]
    pub const fn is_a_file(self) -> bool {
        !matches!(self, Self::Folder | Self::Link | Self::Other)
    }

    /// The word this kind is named by.
    #[must_use]
    pub fn word(self) -> &'static Word {
        match self {
            Self::Text => &words::KIND_TEXT,
            Self::Pdf => &words::KIND_PDF,
            Self::Png => &words::KIND_PNG,
            Self::Jpeg => &words::KIND_JPEG,
            Self::Gif => &words::KIND_GIF,
            Self::Zip => &words::KIND_ZIP,
            Self::Program => &words::KIND_PROGRAM,
            Self::Bytes => &words::KIND_BYTES,
            Self::Empty => &words::KIND_EMPTY,
            Self::Unread => &words::KIND_UNREAD,
            Self::Folder => &words::KIND_FOLDER,
            Self::Link => &words::KIND_LINK,
            Self::Other => &words::KIND_OTHER,
        }
    }

    /// This kind, named in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Whether these bytes read as text.
///
/// `first` is the first [`SNIFFED`] bytes of the file, or the whole file when
/// it is shorter. A multi-byte character cut in half at the end of a full
/// [`SNIFFED`] bytes is not a fault — the rest of it is in the file — but the
/// same cut at the end of a shorter file is the file ending mid-character,
/// which text does not do.
fn is_text(first: &[u8]) -> bool {
    let text = match std::str::from_utf8(first) {
        Ok(text) => text,
        Err(why) => {
            if why.error_len().is_some() || first.len() < SNIFFED {
                return false;
            }
            let (whole, _) = first.split_at(why.valid_up_to());
            match std::str::from_utf8(whole) {
                Ok(text) => text,
                Err(_) => return false,
            }
        }
    };
    text.chars()
        .all(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r' | '\u{0c}' | '\u{1b}'))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The kind is the bytes, whatever the name says.** Every signature is
    /// recognised, text is text, and bytes nobody recognises are bytes.
    #[test]
    fn every_kind_is_read_from_the_bytes() {
        assert_eq!(Kind::of_bytes(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3"), Kind::Pdf);
        assert_eq!(Kind::of_bytes(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"), Kind::Png);
        assert_eq!(Kind::of_bytes(b"\xFF\xD8\xFF\xE0\0\x10JFIF"), Kind::Jpeg);
        assert_eq!(Kind::of_bytes(b"GIF89a\x01\0"), Kind::Gif);
        assert_eq!(Kind::of_bytes(b"GIF87a\x01\0"), Kind::Gif);
        assert_eq!(Kind::of_bytes(b"PK\x03\x04\x14\0"), Kind::Zip);
        assert_eq!(Kind::of_bytes(b"\x7FELF\x02\x01"), Kind::Program);
        assert_eq!(Kind::of_bytes(b"Dear Anna,\r\n\tthe contract"), Kind::Text);
        assert_eq!(
            Kind::of_bytes("za\u{17c}\u{f3}\u{142}\u{107} g\u{119}\u{15b}l\u{105}\n".as_bytes()),
            Kind::Text
        );
        assert_eq!(Kind::of_bytes(b"\x1b[32mgreen\x1b[0m\n"), Kind::Text);
        assert_eq!(Kind::of_bytes(b""), Kind::Empty);
        assert_eq!(Kind::of_bytes(b"\0\0\0\x18ftypmp42"), Kind::Bytes);
        assert_eq!(Kind::of_bytes(b"hello\0world"), Kind::Bytes);
        // UTF-16 is not text by this rule, and the module says so.
        assert_eq!(Kind::of_bytes(b"\xFF\xFEh\0i\0"), Kind::Bytes);
    }

    /// A character cut by the end of what was read is text when the file
    /// goes on, and not when the file ends there.
    #[test]
    fn a_character_cut_at_the_end_is_text_only_when_the_file_goes_on() {
        let mut full = vec![b'a'; SNIFFED - 1];
        full.push(0xC3);
        assert_eq!(Kind::of_bytes(&full), Kind::Text);
        assert_eq!(Kind::of_bytes(b"caf\xC3"), Kind::Bytes);
    }

    /// A folder, a link and a device have a kind without being opened; a
    /// file does not.
    #[test]
    fn what_is_not_a_file_has_a_kind_without_being_read() {
        assert_eq!(Kind::of_thing(alo_files::Kind::File), None);
        assert_eq!(Kind::of_thing(alo_files::Kind::Folder), Some(Kind::Folder));
        assert_eq!(Kind::of_thing(alo_files::Kind::Link), Some(Kind::Link));
        assert_eq!(Kind::of_thing(alo_files::Kind::Other), Some(Kind::Other));
        for kind in Kind::EVERY {
            assert_eq!(
                kind.is_a_file(),
                !matches!(kind, Kind::Folder | Kind::Link | Kind::Other),
                "{kind:?}"
            );
        }
    }

    /// Only text has words, every kind has a name, and the spelling in the
    /// file is the variant in lower case.
    #[test]
    fn every_kind_is_named_and_spelled_in_lower_case() {
        for kind in Kind::EVERY {
            assert_eq!(kind.has_words(), kind == Kind::Text);
            let spelled = serde_json::to_string(&kind).unwrap();
            assert_eq!(
                spelled,
                format!("\"{}\"", format!("{kind:?}").to_lowercase())
            );
            assert_eq!(serde_json::from_str::<Kind>(&spelled).unwrap(), kind);
            assert!(kind.word().named().starts_with("finding.kind."));
        }
        assert_eq!(
            serde_json::from_str::<Kind>("\"hologram\"").unwrap(),
            Kind::Bytes,
            "a kind a later version added is bytes to this one"
        );
    }
}
