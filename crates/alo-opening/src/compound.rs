//! What a file in the older Office storage format is, from the names at its
//! top level and nothing else.
//!
//! Word, Excel and PowerPoint before 2007 all stored a document as a small file
//! system inside one file, and so do a few things that are not documents at
//! all — an installer, a saved mail message. What tells them apart is **the
//! names of the streams at the top**: `WordDocument`, `Workbook`, `PowerPoint
//! Document`. A current Office document locked with a password is stored this
//! way too, as a stream called `EncryptedPackage`.
//!
//! So what is read is the storage format's own header, the sectors of its
//! directory — reached through its allocation table, which the header says
//! where to find — and the names in it. No stream's contents are read.
//!
//! # A directory that does not hold together
//!
//! [`Stored::Damaged`]: a header with a version or sector size the format does
//! not have, a sector that points outside the file, a chain that loops, or a
//! tree of names that points at entries that are not there. Each is bounded by
//! the length of the file, so no file can make this read without end.

use std::io::{self, Read, Seek};

use crate::appears::Macros;
use crate::kind::Kind;
use crate::reading::{Reading, u16_at, u32_at};

/// What a file in the older Office storage format is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stored {
    /// One of the three older Office kinds.
    A(Kind, Macros),
    /// A current Office document locked with a password.
    PasswordProtected,
    /// Stored this way, and not a document this crate knows.
    Unrecognised,
    /// A storage file that does not hold together.
    Damaged,
}

/// The eight bytes every file in this format begins with.
pub(crate) const SIGNATURE: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];

/// The end of a chain of sectors.
const END_OF_CHAIN: u32 = 0xffff_fffe;
/// A sector nobody uses.
const FREE: u32 = 0xffff_ffff;
/// An entry that points at no other.
const NO_ENTRY: u32 = 0xffff_ffff;
/// How long one directory entry is.
const ENTRY_LEN: usize = 128;
/// How many allocation-table sectors the header itself lists.
const LISTED_IN_THE_HEADER: usize = 109;

/// The header, read.
struct Header {
    /// How long one sector is.
    sector: u64,
    /// Where the directory begins.
    first_directory: u32,
    /// Every sector holding the allocation table, in order.
    table: Vec<u32>,
}

/// One entry of the directory.
struct Entry {
    /// Its name.
    name: String,
    /// Whether it is used at all.
    used: bool,
    /// The entry to its left in the tree its siblings form.
    left: u32,
    /// The entry to its right.
    right: u32,
    /// The first of the entries inside it.
    child: u32,
}

/// Decide what a file beginning with this format's signature is.
///
/// `head` is the first bytes of the file, already read.
///
/// # Errors
/// Whatever the file said.
pub(crate) fn look<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    head: &[u8],
) -> io::Result<Stored> {
    let Some(header) = the_header(reading, head)? else {
        return Ok(Stored::Damaged);
    };
    let Some(entries) = the_directory(reading, &header)? else {
        return Ok(Stored::Damaged);
    };
    let Some(top) = the_top_level(&entries) else {
        return Ok(Stored::Damaged);
    };

    let has = |wanted: &str| top.iter().any(|name| name.eq_ignore_ascii_case(wanted));
    if has("EncryptedPackage") {
        return Ok(Stored::PasswordProtected);
    }
    let word = has("WordDocument");
    let excel = has("Workbook") || has("Book");
    let powerpoint = has("PowerPoint Document");
    let macros = if has("Macros") || has("_VBA_PROJECT_CUR") {
        Macros::Inside
    } else {
        Macros::NoneSeen
    };
    Ok(match (word, excel, powerpoint) {
        (true, false, false) => Stored::A(Kind::OlderWordDocument, macros),
        (false, true, false) => Stored::A(Kind::OlderExcelWorkbook, macros),
        (false, false, true) => Stored::A(Kind::OlderPowerPointPresentation, macros),
        _ => Stored::Unrecognised,
    })
}

/// The header, or [`None`] when it is not one this format has.
fn the_header<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    head: &[u8],
) -> io::Result<Option<Header>> {
    let (Some(major), Some(order), Some(shift)) =
        (u16_at(head, 0x1a), u16_at(head, 0x1c), u16_at(head, 0x1e))
    else {
        return Ok(None);
    };
    if order != 0xfffe || !matches!((major, shift), (3, 9) | (4, 12)) {
        return Ok(None);
    }
    let sector = 1_u64 << shift;
    let (Some(fat_sectors), Some(first_directory), Some(first_difat), Some(difat_sectors)) = (
        u32_at(head, 0x2c),
        u32_at(head, 0x30),
        u32_at(head, 0x44),
        u32_at(head, 0x48),
    ) else {
        return Ok(None);
    };
    let sectors_in_the_file = reading.len() / sector;
    if u64::from(fat_sectors) > sectors_in_the_file
        || u64::from(difat_sectors) > sectors_in_the_file
    {
        return Ok(None);
    }

    let mut table = Vec::new();
    for listed in 0..LISTED_IN_THE_HEADER {
        let Some(at) = u32_at(head, 0x4c + listed * 4) else {
            return Ok(None);
        };
        if at != FREE {
            table.push(at);
        }
    }

    // Past the first 109, the list of allocation-table sectors continues in
    // sectors of its own, each ending with where the next one is.
    let per_sector = usize::try_from(sector / 4).unwrap_or(0);
    let mut next = first_difat;
    for _ in 0..difat_sectors {
        if next == END_OF_CHAIN || next == FREE {
            break;
        }
        let Some(piece) = a_sector(reading, sector, next)? else {
            return Ok(None);
        };
        for listed in 0..per_sector.saturating_sub(1) {
            match u32_at(&piece, listed * 4) {
                Some(FREE) => {}
                Some(at) => table.push(at),
                None => return Ok(None),
            }
        }
        let Some(following) = u32_at(&piece, per_sector.saturating_sub(1) * 4) else {
            return Ok(None);
        };
        next = following;
    }
    if table.len() < fat_sectors as usize {
        return Ok(None);
    }
    table.truncate(fat_sectors as usize);

    Ok(Some(Header {
        sector,
        first_directory,
        table,
    }))
}

/// One whole sector, or [`None`] when the file does not hold all of it.
fn a_sector<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    sector: u64,
    number: u32,
) -> io::Result<Option<Vec<u8>>> {
    let Some(offset) = (u64::from(number) + 1).checked_mul(sector) else {
        return Ok(None);
    };
    let wanted = usize::try_from(sector).unwrap_or(usize::MAX);
    let piece = reading.at(offset, wanted)?;
    Ok((piece.len() == wanted).then_some(piece))
}

/// The sector after this one in its chain, from the allocation table.
fn the_next<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    header: &Header,
    number: u32,
) -> io::Result<Option<u32>> {
    let per_sector = header.sector / 4;
    let Ok(which) = usize::try_from(u64::from(number) / per_sector) else {
        return Ok(None);
    };
    let Some(&table_sector) = header.table.get(which) else {
        return Ok(None);
    };
    let Some(offset) = (u64::from(table_sector) + 1)
        .checked_mul(header.sector)
        .and_then(|start| start.checked_add((u64::from(number) % per_sector) * 4))
    else {
        return Ok(None);
    };
    let piece = reading.at(offset, 4)?;
    Ok(u32_at(&piece, 0))
}

/// Every entry of the directory, or [`None`] when its chain does not hold
/// together.
fn the_directory<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    header: &Header,
) -> io::Result<Option<Vec<Entry>>> {
    let most_sectors = reading.len() / header.sector + 1;
    let mut entries = Vec::new();
    let mut next = header.first_directory;
    let mut walked = 0_u64;
    while next != END_OF_CHAIN {
        walked += 1;
        if walked > most_sectors {
            return Ok(None);
        }
        let Some(piece) = a_sector(reading, header.sector, next)? else {
            return Ok(None);
        };
        let (whole, _) = piece.as_chunks::<ENTRY_LEN>();
        for raw in whole {
            entries.push(an_entry(raw));
        }
        let Some(following) = the_next(reading, header, next)? else {
            return Ok(None);
        };
        next = following;
    }
    Ok(Some(entries))
}

/// One directory entry, read.
fn an_entry(raw: &[u8]) -> Entry {
    let name_bytes = usize::from(u16_at(raw, 0x40).unwrap_or(0)).min(64);
    let units: Vec<u16> = (0..name_bytes.saturating_sub(2) / 2)
        .filter_map(|unit| u16_at(raw, unit * 2))
        .collect();
    let kind = raw.get(0x42).copied().unwrap_or(0);
    Entry {
        name: String::from_utf16_lossy(&units),
        used: kind != 0,
        left: u32_at(raw, 0x44).unwrap_or(NO_ENTRY),
        right: u32_at(raw, 0x48).unwrap_or(NO_ENTRY),
        child: u32_at(raw, 0x4c).unwrap_or(NO_ENTRY),
    }
}

/// The names directly inside the root, or [`None`] when the tree points at
/// entries that are not there or at one twice.
fn the_top_level(entries: &[Entry]) -> Option<Vec<&str>> {
    let root = entries.first().filter(|root| root.used)?;
    let mut names = Vec::new();
    let mut visited = vec![false; entries.len()];
    let mut waiting = vec![root.child];
    while let Some(at) = waiting.pop() {
        if at == NO_ENTRY {
            continue;
        }
        let index = usize::try_from(at).ok()?;
        let entry = entries.get(index)?;
        let seen = visited.get_mut(index)?;
        if *seen || index == 0 || !entry.used {
            return None;
        }
        *seen = true;
        names.push(entry.name.as_str());
        waiting.push(entry.left);
        waiting.push(entry.right);
    }
    Some(names)
}
