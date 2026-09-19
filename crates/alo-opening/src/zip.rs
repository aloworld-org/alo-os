//! What a file stored as a zip is, from its list of contents and nothing else.
//!
//! A Word document, a spreadsheet, a presentation, their OpenDocument
//! equivalents and a plain zip archive all begin with the same four bytes. What
//! tells them apart is **which parts they hold**, and a zip says that in one
//! place: the central directory at its end, a list of names with no document in
//! it. So that list is what is read — the record at the very end that says
//! where it is, then the list itself, a piece at a time — and not one byte of
//! any part's contents, with one exception.
//!
//! **The exception is an OpenDocument's `mimetype`.** The standard requires it
//! to be the first entry, stored uncompressed, so its few bytes sit inside the
//! first [`crate::looking::THE_HEAD`] already read and say exactly which of the
//! three documents it is. Reading it costs nothing further.
//!
//! # What a zip that does not hold together is
//!
//! [`Zipped::Damaged`]: a file that begins like a zip and has no record at its
//! end, a record pointing outside the file, or a list that does not add up to
//! what the record says it holds. The commonest way to get one is a download
//! that stopped, and *damaged* is what sends a person to ask for it again.

use std::io::{self, Read, Seek};

use crate::appears::Macros;
use crate::kind::Kind;
use crate::reading::{Reading, u16_at, u32_at, u64_at};

/// What a file stored as a zip is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Zipped {
    /// One of the kinds stored this way.
    A(Kind, Macros),
    /// A zip that holds a document of a kind this crate does not know.
    Unrecognised,
    /// A zip that does not hold together.
    Damaged,
}

/// The record at the end of every zip.
const END: u32 = 0x0605_4b50;
/// The same record for a zip too large for the first one's fields.
const END_64: u32 = 0x0606_4b50;
/// Where that larger record is.
const END_64_LOCATOR: u32 = 0x0706_4b50;
/// One entry of the list of contents.
const ENTRY: u32 = 0x0201_4b50;
/// The first entry's own header, at the very start of the file.
const LOCAL: u32 = 0x0403_4b50;

/// How long the end record is without its comment.
const END_LEN: usize = 22;
/// The longest comment a zip can carry after that record.
const LONGEST_COMMENT: usize = 0xffff;
/// How long one entry of the list is before its name.
const ENTRY_LEN: usize = 46;

/// Where the list of contents is and how many entries it holds.
struct Directory {
    /// Where it starts.
    offset: u64,
    /// How long it is.
    size: u64,
    /// How many entries it says it holds.
    entries: u64,
}

/// What the names in a list of contents say.
#[derive(Default)]
struct Named {
    /// `[Content_Types].xml`, which every current Office document has.
    content_types: bool,
    /// A part under `word/`.
    word: bool,
    /// A part under `xl/`.
    xl: bool,
    /// A part under `ppt/`.
    ppt: bool,
    /// A `vbaProject.bin`, which is where a current Office document keeps
    /// macros.
    vba: bool,
    /// A macro module under `Basic/` or a script under `Scripts/`, which is
    /// where an OpenDocument keeps them.
    open_document_macros: bool,
    /// `Index/Document.iwa`, which every iWork document carries — a Pages
    /// document, a Keynote presentation and a Numbers spreadsheet alike.
    iwork_document: bool,
    /// A part only a Keynote presentation has: its slides, its masters or its
    /// theme.
    keynote: bool,
    /// A part only a Numbers spreadsheet has: its tables.
    numbers: bool,
}

/// Decide what a file beginning with a zip's signature is.
///
/// `head` is the first bytes of the file, already read.
///
/// # Errors
/// Whatever the file said.
pub(crate) fn look<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    head: &[u8],
) -> io::Result<Zipped> {
    let Some(directory) = the_directory(reading)? else {
        return Ok(Zipped::Damaged);
    };
    let Some(named) = the_names(reading, &directory)? else {
        return Ok(Zipped::Damaged);
    };

    if let Some(declared) = open_document_type(head) {
        let macros = if named.open_document_macros {
            Macros::Inside
        } else {
            Macros::NoneSeen
        };
        return Ok(match declared {
            b"application/vnd.oasis.opendocument.text" => Zipped::A(Kind::OpenDocumentText, macros),
            b"application/vnd.oasis.opendocument.spreadsheet" => {
                Zipped::A(Kind::OpenDocumentSpreadsheet, macros)
            }
            b"application/vnd.oasis.opendocument.presentation" => {
                Zipped::A(Kind::OpenDocumentPresentation, macros)
            }
            _ => Zipped::Unrecognised,
        });
    }

    if named.content_types {
        let macros = if named.vba {
            Macros::Inside
        } else {
            Macros::NoneSeen
        };
        return Ok(match (named.word, named.xl, named.ppt) {
            (true, false, false) => Zipped::A(Kind::WordDocument, macros),
            (false, true, false) => Zipped::A(Kind::ExcelWorkbook, macros),
            (false, false, true) => Zipped::A(Kind::PowerPointPresentation, macros),
            _ => Zipped::Unrecognised,
        });
    }

    // An iWork document, of which this crate names one: a Pages document.
    //
    // **The shape is `crate::iso_media`'s, and for its reason.** The three
    // iWork applications write the same container — `Index/*.iwa` beside
    // `Metadata/` and the previews — so the index alone says *one of these
    // three* and not which. What is named here is a document carrying that
    // index and **neither of the parts the other two have**, and anything else
    // is claimed to be nothing rather than guessed at.
    //
    // Turning it round — *an iWork document is a Pages document unless it
    // looks like the others* — reads the same until somebody is sent a form
    // nobody here listed, and then says *this is a Pages document* about a
    // presentation. That is the mistake this crate made about a photograph
    // until 2026-09-19, and it is worse than saying nothing because a person
    // acts on it.
    if named.iwork_document {
        return Ok(if named.keynote || named.numbers {
            Zipped::Unrecognised
        } else {
            Zipped::A(Kind::PagesDocument, Macros::NoneSeen)
        });
    }

    Ok(Zipped::A(Kind::ZipArchive, Macros::NoneSeen))
}

/// The type an OpenDocument declares in its first entry, or [`None`] when the
/// first entry is not one.
fn open_document_type(head: &[u8]) -> Option<&[u8]> {
    if u32_at(head, 0)? != LOCAL {
        return None;
    }
    let method = u16_at(head, 8)?;
    let stored = u32_at(head, 18)?;
    let name_len = usize::from(u16_at(head, 26)?);
    let extra_len = usize::from(u16_at(head, 28)?);
    let name = head.get(30..30_usize.checked_add(name_len)?)?;
    if name != b"mimetype" || method != 0 {
        return None;
    }
    let start = 30_usize.checked_add(name_len)?.checked_add(extra_len)?;
    let end = start.checked_add(usize::try_from(stored).ok()?)?;
    head.get(start..end)
}

/// Where the list of contents is, or [`None`] when the file has no record
/// saying so that holds together.
fn the_directory<F: Read + Seek>(reading: &mut Reading<'_, F>) -> io::Result<Option<Directory>> {
    let len = reading.len();
    if len < END_LEN as u64 {
        return Ok(None);
    }

    // Almost every zip has no comment, so the record is the last 22 bytes. Only
    // when it is not is the longest a comment could be read back through.
    let mut at = len - END_LEN as u64;
    let mut record = reading.at(at, END_LEN)?;
    if !(u32_at(&record, 0) == Some(END) && u16_at(&record, 20) == Some(0)) {
        let back = len.min((END_LEN + LONGEST_COMMENT) as u64);
        let tail = reading.at(len - back, END_LEN + LONGEST_COMMENT)?;
        let Some(found) = (0..=tail.len().saturating_sub(END_LEN))
            .rev()
            .find(|&start| {
                u32_at(&tail, start) == Some(END)
                    && u16_at(&tail, start + 20).map(usize::from)
                        == Some(tail.len() - start - END_LEN)
            })
        else {
            return Ok(None);
        };
        at = len - back + found as u64;
        record = tail
            .get(found..found + END_LEN)
            .unwrap_or_default()
            .to_vec();
    }

    let (Some(this_disk), Some(directory_disk)) = (u16_at(&record, 4), u16_at(&record, 6)) else {
        return Ok(None);
    };
    if this_disk != 0 || directory_disk != 0 {
        // One piece of an archive split across several files: this file alone
        // is not a whole zip, and nothing it lists can be checked.
        return Ok(None);
    }
    let (Some(entries), Some(size), Some(offset)) = (
        u16_at(&record, 10),
        u32_at(&record, 12),
        u32_at(&record, 16),
    ) else {
        return Ok(None);
    };

    let (directory, ends_before) = if entries == 0xffff || size == u32::MAX || offset == u32::MAX {
        let Some(found) = the_larger_record(reading, at)? else {
            return Ok(None);
        };
        found
    } else {
        (
            Directory {
                offset: u64::from(offset),
                size: u64::from(size),
                entries: u64::from(entries),
            },
            at,
        )
    };
    match directory.offset.checked_add(directory.size) {
        Some(end) if end <= ends_before => Ok(Some(directory)),
        _ => Ok(None),
    }
}

/// The record a zip too large for the first record's fields keeps before it,
/// and where that record starts.
fn the_larger_record<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    end_at: u64,
) -> io::Result<Option<(Directory, u64)>> {
    let Some(locator_at) = end_at.checked_sub(20) else {
        return Ok(None);
    };
    let locator = reading.at(locator_at, 20)?;
    if u32_at(&locator, 0) != Some(END_64_LOCATOR) {
        return Ok(None);
    }
    let Some(record_at) = u64_at(&locator, 8) else {
        return Ok(None);
    };
    if record_at >= locator_at {
        return Ok(None);
    }
    let record = reading.at(record_at, 56)?;
    if u32_at(&record, 0) != Some(END_64) {
        return Ok(None);
    }
    let (Some(entries), Some(size), Some(offset)) = (
        u64_at(&record, 32),
        u64_at(&record, 40),
        u64_at(&record, 48),
    ) else {
        return Ok(None);
    };
    Ok(Some((
        Directory {
            offset,
            size,
            entries,
        },
        record_at,
    )))
}

/// What the names in the list say, or [`None`] when the list does not add up
/// to what the record says it holds.
fn the_names<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    directory: &Directory,
) -> io::Result<Option<Named>> {
    let mut named = Named::default();
    let mut held: Vec<u8> = Vec::new();
    let mut read_up_to = directory.offset;
    let end = directory.offset.saturating_add(directory.size);
    let mut seen = 0_u64;

    while seen < directory.entries {
        // An entry's fixed part, then its name, extra field and comment.
        if !fill(reading, &mut held, &mut read_up_to, end, ENTRY_LEN)? {
            return Ok(None);
        }
        if u32_at(&held, 0) != Some(ENTRY) {
            return Ok(None);
        }
        let (Some(name_len), Some(extra_len), Some(comment_len)) =
            (u16_at(&held, 28), u16_at(&held, 30), u16_at(&held, 32))
        else {
            return Ok(None);
        };
        let whole =
            ENTRY_LEN + usize::from(name_len) + usize::from(extra_len) + usize::from(comment_len);
        if !fill(reading, &mut held, &mut read_up_to, end, whole)? {
            return Ok(None);
        }
        let name = held
            .get(ENTRY_LEN..ENTRY_LEN + usize::from(name_len))
            .unwrap_or_default();
        let size = u32_at(&held, 24).unwrap_or(0);
        notice(&mut named, name, size);
        held.drain(..whole);
        seen += 1;
    }

    // Every byte the record said the list was has been accounted for.
    if !held.is_empty() || read_up_to != end {
        return Ok(None);
    }
    Ok(Some(named))
}

/// Make sure `held` has at least `wanted` bytes of the list in it, reading the
/// next piece of the list if not. `false` when the list ends first.
fn fill<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    held: &mut Vec<u8>,
    read_up_to: &mut u64,
    end: u64,
    wanted: usize,
) -> io::Result<bool> {
    while held.len() < wanted {
        let left = end.saturating_sub(*read_up_to);
        if left == 0 {
            return Ok(false);
        }
        let most = usize::try_from(left)
            .map_or(crate::text::A_PIECE, |left| left.min(crate::text::A_PIECE));
        let piece = reading.at(*read_up_to, most)?;
        if piece.is_empty() {
            return Ok(false);
        }
        *read_up_to += piece.len() as u64;
        held.extend_from_slice(&piece);
    }
    Ok(true)
}

/// Note what one name in the list says.
fn notice(named: &mut Named, name: &[u8], size: u32) {
    let lower = name.to_ascii_lowercase();
    if lower == b"[content_types].xml" {
        named.content_types = true;
    }
    named.word |= lower.starts_with(b"word/");
    named.xl |= lower.starts_with(b"xl/");
    named.ppt |= lower.starts_with(b"ppt/");
    named.vba |= lower.ends_with(b"vbaproject.bin");

    // An OpenDocument always lists its macro libraries, even when it has none;
    // what means a macro is a module beside those two lists, or a script.
    let is_a_listing = lower.ends_with(b"/script-lc.xml") || lower.ends_with(b"/script-lb.xml");
    let is_a_folder = lower.ends_with(b"/");
    if size > 0
        && !is_a_folder
        && ((name.starts_with(b"Basic/") && !is_a_listing) || name.starts_with(b"Scripts/"))
    {
        named.open_document_macros = true;
    }

    // What an iWork document carries, and what tells the three of them apart.
    // Measured against a real Pages document saved by Pages 15.3.1
    // (`tests/files/README.md`): `Index/Document.iwa` beside a stylesheet, the
    // metadata, the view state and the previews.
    named.iwork_document |= lower == b"index/document.iwa";
    named.keynote |= lower.starts_with(b"index/slide")
        || lower.starts_with(b"index/masterslide")
        || lower.starts_with(b"index/theme");
    named.numbers |= lower.starts_with(b"index/tables/");
}
