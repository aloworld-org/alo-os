//! The parts of a document stored as a zip, read out one at a time and capped.
//!
//! `alo-opening` reads a zip's list of contents and nothing else, because
//! deciding what a file is must not read what is in it. Saying what a copy
//! could not carry has to read inside, so this is the second reader, and it
//! lives only in the crate that converts (ADR 0039 §5) — the deciding crate's
//! dependency list stays as its own tests hold it.
//!
//! # A compression bomb is a refusal
//!
//! A few kilobytes of a zip can decompress into gigabytes. Every part is read
//! with a ceiling — [`LARGEST_PART`], and no more than [`WIDEST_RATIO`] times
//! its compressed size once past [`SMALL_ENOUGH_NOT_TO_ASK`] — and the whole
//! document with another, [`LARGEST_DOCUMENT`]. A part that would pass either
//! is [`NotRead::TooLarge`], which ends the inventory, which is a refusal to
//! show the copy: a document nobody could check is not converted with an
//! unstated cost.
//!
//! # What is not read
//!
//! A zip larger than the older format's fields can say (`ZIP64`), an encrypted
//! part and a compression method other than *stored* and *deflated*. None is how
//! Office saves the three formats; each is [`NotRead::NotUnderstood`] rather
//! than a guess.

use miniz_oxide::inflate::decompress_to_vec_with_limit;

/// The most one part may decompress to.
pub const LARGEST_PART: usize = 64 * 1024 * 1024;

/// The most all the parts read from one document may decompress to between
/// them.
pub const LARGEST_DOCUMENT: usize = 256 * 1024 * 1024;

/// How many times its compressed size a part may grow to, once it is larger
/// than [`SMALL_ENOUGH_NOT_TO_ASK`]. Text compresses well, and a part of
/// ordinary XML is ten to twenty times smaller in the zip; a hundred leaves room
/// for a repetitive one and none for a bomb.
pub const WIDEST_RATIO: usize = 100;

/// Below this, a part's ratio is not asked about: a tiny part of repeated
/// markup compresses far past any ratio and cannot hurt anything.
pub const SMALL_ENOUGH_NOT_TO_ASK: usize = 1024 * 1024;

/// The most parts one document may list.
pub const MOST_PARTS: usize = 10_000;

/// The record at the end of every zip.
const END: u32 = 0x0605_4b50;
/// One entry of the list of contents.
const ENTRY: u32 = 0x0201_4b50;
/// A part's own header, before its bytes.
const LOCAL: u32 = 0x0403_4b50;
/// How long the end record is without its comment.
const END_LEN: usize = 22;
/// The longest comment a zip can carry after that record.
const LONGEST_COMMENT: usize = 0xffff;
/// How long one entry of the list is before its name.
const ENTRY_LEN: usize = 46;
/// How long a part's own header is before its name.
const LOCAL_LEN: usize = 30;

/// Why a document's parts could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotRead {
    /// It does not hold together as a zip.
    #[error("the document does not hold together as a zip")]
    Damaged,
    /// A part, or all of them, would decompress past a ceiling.
    #[error("the document decompresses past what is read")]
    TooLarge,
    /// It is stored in a way this reader does not read.
    #[error("the document is stored in a way that is not read here")]
    NotUnderstood,
}

/// One part's place in the zip.
#[derive(Debug, Clone)]
struct Part {
    /// Its name.
    name: String,
    /// How it is compressed.
    method: u16,
    /// Whether it is encrypted.
    encrypted: bool,
    /// How many bytes it is in the zip.
    compressed: usize,
    /// How many bytes it says it is once decompressed.
    size: usize,
    /// Where its own header is.
    offset: usize,
}

/// A document stored as a zip, with its list of contents read.
#[derive(Debug)]
pub struct Zipped<'a> {
    /// The whole file.
    bytes: &'a [u8],
    /// Every part it lists.
    parts: Vec<Part>,
    /// How much has been decompressed from it so far.
    read: usize,
}

impl<'a> Zipped<'a> {
    /// Read a zip's list of contents.
    ///
    /// # Errors
    /// [`NotRead`].
    pub fn of(bytes: &'a [u8]) -> Result<Self, NotRead> {
        let end = the_end(bytes).ok_or(NotRead::Damaged)?;
        let listed = usize::from(u16_at(bytes, end + 10).ok_or(NotRead::Damaged)?);
        let length = u32_at(bytes, end + 12).ok_or(NotRead::Damaged)?;
        let start = u32_at(bytes, end + 16).ok_or(NotRead::Damaged)?;
        if length == u32::MAX || start == u32::MAX || listed == usize::from(u16::MAX) {
            return Err(NotRead::NotUnderstood);
        }
        if listed > MOST_PARTS {
            return Err(NotRead::TooLarge);
        }
        let mut at = as_usize(start)?;
        let mut parts = Vec::with_capacity(listed);
        for _ in 0..listed {
            if u32_at(bytes, at) != Some(ENTRY) {
                return Err(NotRead::Damaged);
            }
            let flags = u16_at(bytes, at + 8).ok_or(NotRead::Damaged)?;
            let method = u16_at(bytes, at + 10).ok_or(NotRead::Damaged)?;
            let compressed = u32_at(bytes, at + 20).ok_or(NotRead::Damaged)?;
            let size = u32_at(bytes, at + 24).ok_or(NotRead::Damaged)?;
            let name_len = usize::from(u16_at(bytes, at + 28).ok_or(NotRead::Damaged)?);
            let extra_len = usize::from(u16_at(bytes, at + 30).ok_or(NotRead::Damaged)?);
            let comment_len = usize::from(u16_at(bytes, at + 32).ok_or(NotRead::Damaged)?);
            let offset = u32_at(bytes, at + 42).ok_or(NotRead::Damaged)?;
            if [compressed, size, offset].contains(&u32::MAX) {
                return Err(NotRead::NotUnderstood);
            }
            let name = bytes
                .get(at + ENTRY_LEN..at + ENTRY_LEN + name_len)
                .ok_or(NotRead::Damaged)?;
            parts.push(Part {
                name: String::from_utf8_lossy(name).into_owned(),
                method,
                encrypted: flags & 1 == 1,
                compressed: as_usize(compressed)?,
                size: as_usize(size)?,
                offset: as_usize(offset)?,
            });
            at += ENTRY_LEN + name_len + extra_len + comment_len;
        }
        Ok(Self {
            bytes,
            parts,
            read: 0,
        })
    }

    /// Every part's name, in the order the zip lists them.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.parts.iter().map(|part| part.name.as_str())
    }

    /// Whether a part of this name is listed.
    #[must_use]
    pub fn holds(&self, name: &str) -> bool {
        self.names().any(|listed| listed == name)
    }

    /// One part's bytes, decompressed; [`None`] when no part has the name.
    ///
    /// # Errors
    /// [`NotRead`], including [`NotRead::TooLarge`] for a part past a ceiling —
    /// found while decompressing, never trusted from the size the zip claims.
    pub fn read(&mut self, name: &str) -> Result<Option<Vec<u8>>, NotRead> {
        let Some(part) = self.parts.iter().find(|part| part.name == name).cloned() else {
            return Ok(None);
        };
        if part.encrypted {
            return Err(NotRead::NotUnderstood);
        }
        if u32_at(self.bytes, part.offset) != Some(LOCAL) {
            return Err(NotRead::Damaged);
        }
        let name_len = usize::from(u16_at(self.bytes, part.offset + 26).ok_or(NotRead::Damaged)?);
        let extra_len = usize::from(u16_at(self.bytes, part.offset + 28).ok_or(NotRead::Damaged)?);
        let from = part.offset + LOCAL_LEN + name_len + extra_len;
        let stored = self
            .bytes
            .get(from..from + part.compressed)
            .ok_or(NotRead::Damaged)?;

        let ceiling = self.ceiling(&part);
        let bytes = match part.method {
            0 if stored.len() <= ceiling => stored.to_vec(),
            0 => return Err(NotRead::TooLarge),
            8 => decompress_to_vec_with_limit(stored, ceiling).map_err(|failed| {
                if failed.status == miniz_oxide::inflate::TINFLStatus::HasMoreOutput {
                    NotRead::TooLarge
                } else {
                    NotRead::Damaged
                }
            })?,
            _ => return Err(NotRead::NotUnderstood),
        };
        if bytes.len() != part.size {
            return Err(NotRead::Damaged);
        }
        self.read += bytes.len();
        Ok(Some(bytes))
    }

    /// The most this part may decompress to, given what has been read already.
    fn ceiling(&self, part: &Part) -> usize {
        let by_ratio = part
            .compressed
            .saturating_mul(WIDEST_RATIO)
            .max(SMALL_ENOUGH_NOT_TO_ASK);
        LARGEST_PART
            .min(by_ratio)
            .min(LARGEST_DOCUMENT.saturating_sub(self.read))
    }
}

/// Where the end record is, searching back through the longest comment.
fn the_end(bytes: &[u8]) -> Option<usize> {
    let last = bytes.len().checked_sub(END_LEN)?;
    let first = last.saturating_sub(LONGEST_COMMENT);
    (first..=last)
        .rev()
        .find(|at| u32_at(bytes, *at) == Some(END))
}

/// A little-endian `u16`.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let two: [u8; 2] = bytes.get(at..at + 2)?.try_into().ok()?;
    Some(u16::from_le_bytes(two))
}

/// A little-endian `u32`.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let four: [u8; 4] = bytes.get(at..at + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(four))
}

/// A size from the zip as a size on this machine.
fn as_usize(value: u32) -> Result<usize, NotRead> {
    usize::try_from(value).map_err(|_| NotRead::NotUnderstood)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_zip, the_document};

    /// **A real document's parts are listed and read.**
    #[test]
    fn a_real_documents_parts_are_read() {
        let bytes = the_document("sample.docx");
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert!(zipped.holds("word/document.xml"));
        let document = zipped.read("word/document.xml").unwrap().unwrap();
        assert!(document.starts_with(b"<?xml"));
        assert_eq!(zipped.read("word/nothing.xml").unwrap(), None);
    }

    /// **A compression bomb is refused while it decompresses**, whatever size
    /// the zip claims for it.
    #[test]
    fn a_compression_bomb_is_refused() {
        let zeros = vec![0_u8; LARGEST_PART + 1];
        let bytes = a_zip(&[("word/document.xml", &zeros)]);
        assert!(bytes.len() < 1024 * 1024);
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert_eq!(zipped.read("word/document.xml"), Err(NotRead::TooLarge));

        // Past the ratio, well under the ceiling.
        let ratio = vec![b'a'; 8 * 1024 * 1024];
        let bytes = a_zip(&[("xl/styles.xml", &ratio)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert_eq!(zipped.read("xl/styles.xml"), Err(NotRead::TooLarge));
    }

    /// A part small enough is read however well it compresses.
    #[test]
    fn a_small_part_is_read_however_well_it_compresses() {
        let small = vec![b'a'; 512 * 1024];
        let bytes = a_zip(&[("a.xml", &small)]);
        let mut zipped = Zipped::of(&bytes).unwrap();
        assert_eq!(
            zipped.read("a.xml").unwrap().map(|read| read.len()),
            Some(small.len())
        );
    }

    /// **A zip that was cut short, or that lies about a part, is damaged.**
    #[test]
    fn a_zip_cut_short_or_lying_is_damaged() {
        let bytes = the_document("sample.xlsx");
        let cut = bytes.get(..bytes.len() / 2).unwrap();
        assert_eq!(Zipped::of(cut).err(), Some(NotRead::Damaged));
        assert_eq!(Zipped::of(b"PK\x03\x04").err(), Some(NotRead::Damaged));

        let mut lying = a_zip(&[("a.xml", b"<a/>")]);
        // The central directory's claimed size for the one part, made wrong.
        let entry = lying
            .windows(4)
            .position(|window| window == ENTRY.to_le_bytes())
            .unwrap();
        if let Some(size) = lying.get_mut(entry + 24) {
            *size = 99;
        }
        let mut zipped = Zipped::of(&lying).unwrap();
        assert_eq!(zipped.read("a.xml"), Err(NotRead::Damaged));
    }
}
