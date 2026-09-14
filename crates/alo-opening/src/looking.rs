//! What a file's bytes say it is, rule by rule, in the order the rules run.
//!
//! 1. **Nothing in it** is [`Appears::Nothing`], and nothing is read.
//! 2. **The first [`THE_HEAD`] bytes** are read once, and every signature is
//!    matched against them — each at offset zero, never *somewhere near the
//!    start*, because a rule that searched would find `%PDF-` in a letter about
//!    PDFs and call the letter a damaged document.
//! 3. **A signature that needs more** asks for exactly that: the last kilobyte
//!    of a PDF, a zip's list of contents (`crate::zip`), the older Office
//!    format's directory (`crate::compound`), or the four bytes a program's
//!    header points at.
//! 4. **No signature** leaves text, decided from the whole file
//!    (`crate::text`), and bytes that are not text are
//!    [`Appears::Unrecognised`].
//!
//! # A PDF is damaged when it has no end
//!
//! A PDF finishes with `%%EOF`, and the rule every widely used reader applies
//! is that it must be somewhere in the last 1024 bytes — trailing bytes after it
//! are tolerated, a file cut off before it is not. That is the rule here, and a
//! PDF without it is [`Container::Pdf`] damaged.
//!
//! # Images are recognised and not checked
//!
//! A JPEG with bytes after its end, or a PNG with a chunk a viewer skips, opens
//! in every viewer there is. No damage rule is written for an image, because a
//! rule strict enough to be definite would call good photographs damaged.

use std::io::{self, Read, Seek};

use crate::appears::{Appears, Container, Macros};
use crate::compound::{self, Stored};
use crate::kind::Kind;
use crate::reading::{Reading, u32_at};
use crate::text::{self, Texted};
use crate::zip::{self, Zipped};

/// How many bytes are read from the start of every file that is not empty.
///
/// Enough for the older Office format's whole header, which is the longest
/// thing any rule reads from the start; every other signature is in the first
/// dozen, and an OpenDocument's declared type in the first hundred.
pub const THE_HEAD: usize = 512;

/// How far from the end of a PDF its `%%EOF` may be.
pub(crate) const THE_PDF_TAIL: usize = 1024;

/// What a file's bytes say, and whether a document among them carries macros.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Looked {
    /// What it is.
    pub(crate) appears: Appears,
    /// Whether macros were seen in it.
    pub(crate) macros: Macros,
}

impl Looked {
    /// Something with no macros where they would be.
    const fn plainly(appears: Appears) -> Self {
        Self {
            appears,
            macros: Macros::NoneSeen,
        }
    }
}

/// Look at a file.
///
/// # Errors
/// Whatever the file said.
pub(crate) fn look<F: Read + Seek>(reading: &mut Reading<'_, F>) -> io::Result<Looked> {
    if reading.len() == 0 {
        return Ok(Looked::plainly(Appears::Nothing));
    }
    let head = reading.at(0, THE_HEAD)?;

    if head.starts_with(b"%PDF-") {
        let back = reading.len().min(THE_PDF_TAIL as u64);
        let tail = reading.at(reading.len() - back, THE_PDF_TAIL)?;
        let ends = tail.windows(5).any(|five| five == b"%%EOF");
        return Ok(Looked::plainly(if ends {
            Appears::A(Kind::Pdf)
        } else {
            Appears::Damaged(Container::Pdf)
        }));
    }

    for (signature, kind) in [
        (&b"\x89PNG\r\n\x1a\n"[..], Kind::PngImage),
        (&b"\xff\xd8\xff"[..], Kind::JpegImage),
        (&b"GIF87a"[..], Kind::GifImage),
        (&b"GIF89a"[..], Kind::GifImage),
        (&b"{\\rtf"[..], Kind::RichText),
    ] {
        if head.starts_with(signature) {
            return Ok(Looked::plainly(Appears::A(kind)));
        }
    }
    if head.starts_with(b"RIFF") && head.get(8..12) == Some(&b"WEBP"[..]) {
        return Ok(Looked::plainly(Appears::A(Kind::WebpImage)));
    }

    if head.starts_with(b"\x7fELF") || is_a_windows_program(reading, &head)? {
        return Ok(Looked::plainly(Appears::AProgram));
    }

    if head.starts_with(b"PK\x03\x04") || head.starts_with(b"PK\x05\x06") {
        return Ok(match zip::look(reading, &head)? {
            Zipped::A(kind, macros) => Looked {
                appears: Appears::A(kind),
                macros,
            },
            Zipped::Unrecognised => Looked::plainly(Appears::Unrecognised),
            Zipped::Damaged => Looked::plainly(Appears::Damaged(Container::Compressed)),
        });
    }

    if head.starts_with(&compound::SIGNATURE) {
        return Ok(match compound::look(reading, &head)? {
            Stored::A(kind, macros) => Looked {
                appears: Appears::A(kind),
                macros,
            },
            Stored::PasswordProtected => Looked::plainly(Appears::APasswordProtectedDocument),
            Stored::Unrecognised => Looked::plainly(Appears::Unrecognised),
            Stored::Damaged => Looked::plainly(Appears::Damaged(Container::OlderOffice)),
        });
    }

    Ok(Looked::plainly(match text::look(reading)? {
        Texted::Text => Appears::A(Kind::Text),
        Texted::OlderCharacterSet => Appears::A(Kind::TextInAnOlderCharacterSet),
        Texted::NotText => Appears::Unrecognised,
    }))
}

/// Whether a file beginning `MZ` is a Windows program: its header points, at
/// offset `0x3c`, to the four bytes `PE\0\0` inside the file.
///
/// `MZ` alone is two letters a text file can begin with, so it is not enough.
fn is_a_windows_program<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    head: &[u8],
) -> io::Result<bool> {
    if !head.starts_with(b"MZ") {
        return Ok(false);
    }
    let Some(points_at) = u32_at(head, 0x3c) else {
        return Ok(false);
    };
    Ok(reading.at(u64::from(points_at), 4)? == b"PE\0\0")
}
