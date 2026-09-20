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
//! # A film is recognised by its wrapping, and that is all it is
//!
//! The media rules say which container a file is in — Matroska, the one MP4
//! names, Ogg, RIFF — and **nothing about what is inside it**. Whether this
//! machine can play what is in there is `alo-playing`'s question, asked of the
//! tracks rather than of the first four bytes (ADR 0051). What this crate
//! decides is what every other file it reads decides: what the file *is*, so
//! that a person who cannot open it is told what they have.
//!
//! **The wrapping is not the same as the header, though.** `ftyp` at offset
//! four begins a whole family of formats, of which the container MP4 names is
//! one; which of them a file is is written in the brands after it, and
//! `crate::iso_media` is the rule that reads them. Until 2026-09-19 this file
//! took the header for the container and reported a photograph from a telephone
//! as a film.
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
use crate::iso_media::{self, Iso};
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

/// What every Matroska and WebM file begins with: EBML's own signature.
const EBML: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];

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
    // RIFF holds several formats and the four bytes at eight say which. The
    // three people are sent are a picture, a sound recording and an older film.
    if head.starts_with(b"RIFF") {
        return Ok(Looked::plainly(match head.get(8..12) {
            Some(b"WEBP") => Appears::A(Kind::WebpImage),
            Some(b"WAVE") => Appears::A(Kind::WaveAudio),
            Some(b"AVI ") => Appears::A(Kind::AviVideo),
            _ => Appears::Unrecognised,
        }));
    }

    // Matroska and WebM are one container with two names, and the name is
    // written in the header this signature starts. It is read by looking for
    // the name inside that header rather than by walking EBML's elements: the
    // search is bounded to the head of a file that already begins with the
    // signature, and the two names are the only ones that decide anything.
    // Anything else in that container is a Matroska file, which is what it is.
    if head.starts_with(&EBML) {
        return Ok(Looked::plainly(Appears::A(if says(&head, b"webm") {
            Kind::WebmVideo
        } else {
            Kind::MatroskaVideo
        })));
    }

    // The container MP4 names says `ftyp` at four — and so do several other
    // formats, which is why the brands after it decide rather than the header
    // (`crate::iso_media`).
    //
    // A photograph from a telephone is saved in this same container and is
    // asked about first, because *a photograph reported as a film* is the
    // worse answer of the two: somebody looking for a picture is told to find
    // a video player. What is left is a film, sound with no picture, or — when
    // the brands name none of the family this machine reads — nothing claimed
    // at all, which is the refusal `iso_media` exists for.
    if iso_media::is_the_header(&head) {
        let brand = head.get(8..12).unwrap_or_default();
        if is_a_photo(brand) {
            return Ok(Looked::plainly(Appears::A(Kind::HeicPhoto)));
        }
        return Ok(Looked::plainly(match iso_media::look(&head) {
            Iso::AFilm => Appears::A(Kind::Mp4Video),
            Iso::ASoundRecording => Appears::A(Kind::Mp4Audio),
            Iso::NotOneOfOurs => Appears::Unrecognised,
        }));
    }

    if head.starts_with(b"OggS") {
        return Ok(Looked::plainly(Appears::A(Kind::OggMedia)));
    }
    if head.starts_with(b"fLaC") {
        return Ok(Looked::plainly(Appears::A(Kind::FlacAudio)));
    }
    if head.starts_with(b"ID3") || is_an_mpeg_audio_frame(&head) {
        return Ok(Looked::plainly(Appears::A(Kind::Mp3Audio)));
    }

    // A drawing carries its version where every other format carries a
    // signature: the first six characters are the version marker and there is
    // nothing after them to confirm it, so the rule is the list of markers and
    // the list is closed (`is_a_drawing`).
    if is_a_drawing(&head) {
        return Ok(Looked::plainly(Appears::A(Kind::AutocadDrawing)));
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

/// Whether this brand says the file is a photograph rather than a film.
///
/// **HEIF and MP4 are the same container**, and the brand after `ftyp` is the
/// only thing that tells them apart. Until this rule existed a photograph from a
/// telephone came out of the branch below as *a film* — which is worse than not
/// recognising it, because the machine was confidently wrong about the commonest
/// file anybody is sent.
///
/// The brands are the ones the HEIF standard registers for still pictures:
/// `heic` and `heix` for one picture coded with HEVC, `heim` and `heis` for the
/// multi-image forms, `mif1` and `msf1` for the generic image and image-sequence
/// brands a file written by something other than a telephone carries, and `avif`
/// and `avis` for the same containers holding AV1 instead. `hevc` and `hevx` are
/// **not** here: those are the brands of an HEVC *sequence*, which is a film.
fn is_a_photo(brand: &[u8]) -> bool {
    matches!(
        brand,
        b"heic" | b"heix" | b"heim" | b"heis" | b"mif1" | b"msf1" | b"avif" | b"avis"
    )
}

/// Whether these bytes begin a drawing, read from the six characters every one
/// of them starts with.
///
/// **Those six are the whole of the evidence.** There is no signature after
/// them and no length to check, so a loose rule here is dangerous in the exact
/// way [ADR 0057](../../../docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
/// refuses: *anything beginning `AC`* would claim files nobody here has ever
/// seen, and *this is a drawing* said of something that is not one is a
/// confident wrong answer rather than an honest *not recognised*. So the
/// markers are written out one by one, and anything else falls through to the
/// rules below.
///
/// **Admitted**, each a released version of the format, oldest first:
///
/// | Marker | Written by |
/// |---|---|
/// | `AC1012` | Release 13 |
/// | `AC1014` | Release 14 |
/// | `AC1015` | 2000, 2000i and 2002 |
/// | `AC1018` | 2004, 2005 and 2006 |
/// | `AC1021` | 2007, 2008 and 2009 |
/// | `AC1024` | 2010, 2011 and 2012 |
/// | `AC1027` | 2013 to 2017 |
/// | `AC1032` | 2018 onward |
///
/// **`AC1032` is the one this repository has a real file of** —
/// `tests/files/drawing.dwg`, with its provenance and its digest beside it, and
/// `tests/a_drawing_from_a_cad_program.rs` measured against it. The other seven
/// are the released versions the format's published history names, and they are
/// admitted because a drawing saved ten years ago is the same file to the person
/// who was sent it today. Anything outside this list — a marker from before
/// Release 13, a version nobody released, or a file of text that happens to
/// begin with those two letters — is **not** claimed to be a drawing.
fn is_a_drawing(head: &[u8]) -> bool {
    matches!(
        head.get(..6),
        Some(
            b"AC1012"
                | b"AC1014"
                | b"AC1015"
                | b"AC1018"
                | b"AC1021"
                | b"AC1024"
                | b"AC1027"
                | b"AC1032"
        )
    )
}

/// Whether this name is written in the head of the file.
///
/// Used for the one thing a Matroska file's header says that decides anything:
/// whether the file calls itself WebM.
fn says(head: &[u8], name: &[u8]) -> bool {
    head.windows(name.len()).any(|seen| seen == name)
}

/// Whether these bytes begin a valid MPEG audio frame.
///
/// An MP3 with tags on it begins `ID3` and needs none of this. One without them
/// begins with a frame, and the eleven bits every frame starts with are not
/// enough to go on by themselves: each of the fields after them has a value
/// that means *reserved*, and a header carrying one of those is not a frame.
/// All four are checked, which is what a player does before it decodes
/// anything — and what keeps a file of bytes that happen to start `0xFF` from
/// being called a sound recording.
fn is_an_mpeg_audio_frame(head: &[u8]) -> bool {
    let (Some(first), Some(second), Some(third)) = (head.first(), head.get(1), head.get(2)) else {
        return false;
    };
    *first == 0xFF
        && second & 0b1110_0000 == 0b1110_0000
        && (second >> 3) & 0b11 != 0b01
        && (second >> 1) & 0b11 != 0b00
        && third >> 4 != 0b1111
        && (third >> 2) & 0b11 != 0b11
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
