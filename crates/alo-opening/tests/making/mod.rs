//! The files these tests look at, built byte by byte to their formats'
//! specifications, and a disk to put them on.
//!
//! **Built, not borrowed.** A document downloaded from somewhere would be a
//! file whose licence and whose contents nobody here checked, and a test that
//! depended on one would pass or fail on what some other program happened to
//! write that year. Each builder writes the structure the rule under test
//! reads — a zip's entries and its list of contents, the older Office format's
//! header, allocation table and directory — exactly as the format says, and
//! nothing a rule does not read is faked: every entry's contents are really
//! there, at the offsets the list says.
//!
//! Shared by every test file through `mod making;`, so that one builder is
//! wrong everywhere or right everywhere.

#![allow(
    dead_code,
    reason = "each test file uses a different part of the builders"
)]
#![expect(
    clippy::expect_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// A folder of this test's own, on the disk the tests are running on.
pub fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-opening-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).expect("a folder of our own");
    folder
}

/// A little-endian `u16`, onto the end of `bytes`.
fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// A little-endian `u32`, onto the end of `bytes`.
fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// A little-endian `u64`, onto the end of `bytes`.
fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

/// A length that a fixture knows fits.
fn fits_u16(value: usize) -> u16 {
    u16::try_from(value).expect("a fixture's length fits")
}

/// A length that a fixture knows fits.
fn fits_u32(value: usize) -> u32 {
    u32::try_from(value).expect("a fixture's length fits")
}

/// The CRC-32 a zip records for each entry.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

/// How a zip is to be finished.
#[derive(Default, Clone)]
pub struct Finish {
    /// A comment after the end record.
    pub comment: Vec<u8>,
    /// Whether to write the larger end record a zip over four gigabytes needs.
    pub as_zip64: bool,
}

/// A zip with these entries, each stored as it is, in this order.
pub fn a_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    a_zip_finished(entries, &Finish::default())
}

/// A zip with these entries, finished this way.
pub fn a_zip_finished(entries: &[(&str, &[u8])], finish: &Finish) -> Vec<u8> {
    let mut zip = Vec::new();
    let mut directory = Vec::new();
    for (name, contents) in entries {
        let offset = fits_u32(zip.len());
        let crc = crc32(contents);
        put_u32(&mut zip, 0x0403_4b50);
        put_u16(&mut zip, 20);
        put_u16(&mut zip, 0);
        put_u16(&mut zip, 0);
        put_u16(&mut zip, 0);
        put_u16(&mut zip, 0x21);
        put_u32(&mut zip, crc);
        put_u32(&mut zip, fits_u32(contents.len()));
        put_u32(&mut zip, fits_u32(contents.len()));
        put_u16(&mut zip, fits_u16(name.len()));
        put_u16(&mut zip, 0);
        zip.extend_from_slice(name.as_bytes());
        zip.extend_from_slice(contents);

        put_u32(&mut directory, 0x0201_4b50);
        put_u16(&mut directory, 20);
        put_u16(&mut directory, 20);
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0x21);
        put_u32(&mut directory, crc);
        put_u32(&mut directory, fits_u32(contents.len()));
        put_u32(&mut directory, fits_u32(contents.len()));
        put_u16(&mut directory, fits_u16(name.len()));
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0);
        put_u16(&mut directory, 0);
        put_u32(&mut directory, 0);
        put_u32(&mut directory, offset);
        directory.extend_from_slice(name.as_bytes());
    }
    let directory_at = zip.len();
    let directory_len = directory.len();
    zip.extend_from_slice(&directory);

    if finish.as_zip64 {
        let record_at = zip.len();
        put_u32(&mut zip, 0x0606_4b50);
        put_u64(&mut zip, 44);
        put_u16(&mut zip, 45);
        put_u16(&mut zip, 45);
        put_u32(&mut zip, 0);
        put_u32(&mut zip, 0);
        put_u64(&mut zip, entries.len() as u64);
        put_u64(&mut zip, entries.len() as u64);
        put_u64(&mut zip, directory_len as u64);
        put_u64(&mut zip, directory_at as u64);

        put_u32(&mut zip, 0x0706_4b50);
        put_u32(&mut zip, 0);
        put_u64(&mut zip, record_at as u64);
        put_u32(&mut zip, 1);
    }

    put_u32(&mut zip, 0x0605_4b50);
    put_u16(&mut zip, 0);
    put_u16(&mut zip, 0);
    if finish.as_zip64 {
        put_u16(&mut zip, 0xffff);
        put_u16(&mut zip, 0xffff);
        put_u32(&mut zip, u32::MAX);
        put_u32(&mut zip, u32::MAX);
    } else {
        put_u16(&mut zip, fits_u16(entries.len()));
        put_u16(&mut zip, fits_u16(entries.len()));
        put_u32(&mut zip, fits_u32(directory_len));
        put_u32(&mut zip, fits_u32(directory_at));
    }
    put_u16(&mut zip, fits_u16(finish.comment.len()));
    zip.extend_from_slice(&finish.comment);
    zip
}

/// The parts a Word document holds, with a body of this many bytes.
pub fn a_word_document(body: usize) -> Vec<u8> {
    let document = vec![b'w'; body];
    a_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("_rels/.rels", b"<Relationships/>"),
        ("word/document.xml", &document),
        ("docProps/core.xml", b"<coreProperties/>"),
    ])
}

/// The parts an OpenDocument holds, declaring this type.
pub fn an_open_document(declared: &str, more: &[(&str, &[u8])]) -> Vec<u8> {
    let mut entries: Vec<(&str, &[u8])> = vec![
        ("mimetype", declared.as_bytes()),
        ("content.xml", b"<office:document-content/>"),
        ("META-INF/manifest.xml", b"<manifest:manifest/>"),
    ];
    entries.extend_from_slice(more);
    a_zip(&entries)
}

/// The end of a chain of sectors.
const END_OF_CHAIN: u32 = 0xffff_fffe;
/// A sector that holds the allocation table.
const TABLE_SECTOR: u32 = 0xffff_fffd;
/// A sector nobody uses.
const FREE: u32 = 0xffff_ffff;

/// How an older Office storage file is to be written.
#[derive(Clone, Default)]
pub struct Stored {
    /// The names at the top level, each a stream (`false`) or a storage
    /// (`true`).
    pub top: Vec<(String, bool)>,
    /// When set, the allocation table's entry for the first directory sector
    /// points back at that sector, so the chain never ends.
    pub looping: bool,
    /// When set, the root's child points at an entry past the end.
    pub pointing_nowhere: bool,
    /// When set, the file stops after its header.
    pub cut_after_the_header: bool,
}

impl Stored {
    /// A storage file with these names at its top level, all streams.
    pub fn with(names: &[&str]) -> Self {
        Self {
            top: names
                .iter()
                .map(|name| ((*name).to_owned(), false))
                .collect(),
            ..Self::default()
        }
    }
}

/// An older Office storage file, version 3, with 512-byte sectors: the header,
/// one allocation-table sector, and as many directory sectors as the names
/// need — four entries to a sector, so more than three names makes the
/// directory a chain the reader has to follow through the table.
pub fn an_older_office_file(stored: &Stored) -> Vec<u8> {
    const SECTOR: usize = 512;
    let entries = stored.top.len() + 1;
    let directory_sectors = entries.div_ceil(4);

    let mut file = Vec::new();
    file.extend_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    file.extend_from_slice(&[0; 16]);
    put_u16(&mut file, 0x3e);
    put_u16(&mut file, 3);
    put_u16(&mut file, 0xfffe);
    put_u16(&mut file, 9);
    put_u16(&mut file, 6);
    file.extend_from_slice(&[0; 6]);
    put_u32(&mut file, 0);
    put_u32(&mut file, 1);
    put_u32(&mut file, 1);
    put_u32(&mut file, 0);
    put_u32(&mut file, 4096);
    put_u32(&mut file, END_OF_CHAIN);
    put_u32(&mut file, 0);
    put_u32(&mut file, END_OF_CHAIN);
    put_u32(&mut file, 0);
    put_u32(&mut file, 0);
    for _ in 1..109 {
        put_u32(&mut file, FREE);
    }
    assert_eq!(file.len(), SECTOR);
    if stored.cut_after_the_header {
        return file;
    }

    // Sector 0: the allocation table.
    let mut table = Vec::new();
    put_u32(&mut table, TABLE_SECTOR);
    for sector in 1..=directory_sectors {
        let next = if stored.looping && sector == 1 {
            1
        } else if sector == directory_sectors {
            END_OF_CHAIN
        } else {
            fits_u32(sector + 1)
        };
        put_u32(&mut table, next);
    }
    while table.len() < SECTOR {
        put_u32(&mut table, FREE);
    }
    file.extend_from_slice(&table);

    // The directory: the root, then each name as the right sibling of the one
    // before it.
    let mut directory = Vec::new();
    let root_child = if stored.pointing_nowhere {
        fits_u32(entries + 40)
    } else if stored.top.is_empty() {
        FREE
    } else {
        1
    };
    an_entry(&mut directory, "Root Entry", 5, FREE, root_child);
    for (index, (name, is_storage)) in stored.top.iter().enumerate() {
        let right = if index + 2 < entries {
            fits_u32(index + 2)
        } else {
            FREE
        };
        an_entry(
            &mut directory,
            name,
            if *is_storage { 1 } else { 2 },
            right,
            FREE,
        );
    }
    while directory.len() < directory_sectors * SECTOR {
        an_entry(&mut directory, "", 0, FREE, FREE);
    }
    file.extend_from_slice(&directory);
    file
}

/// One directory entry.
fn an_entry(directory: &mut Vec<u8>, name: &str, kind: u8, right: u32, child: u32) {
    let start = directory.len();
    let units: Vec<u16> = name.encode_utf16().collect();
    for unit in &units {
        put_u16(directory, *unit);
    }
    directory.resize(start + 64, 0);
    put_u16(
        directory,
        if name.is_empty() {
            0
        } else {
            fits_u16((units.len() + 1) * 2)
        },
    );
    directory.push(kind);
    directory.push(1);
    put_u32(directory, FREE);
    put_u32(directory, right);
    put_u32(directory, child);
    directory.extend_from_slice(&[0; 16]);
    put_u32(directory, 0);
    directory.extend_from_slice(&[0; 16]);
    put_u32(directory, END_OF_CHAIN);
    put_u64(directory, 0);
    assert_eq!(directory.len(), start + 128);
}

/// A file in memory that counts every byte read from it and records where.
pub struct Counted {
    /// The file.
    inner: Cursor<Vec<u8>>,
    /// Every read, as where it began and how many bytes it returned.
    pub reads: Vec<(u64, usize)>,
}

impl Counted {
    /// A counted file holding these bytes.
    pub fn holding(bytes: Vec<u8>) -> Self {
        Self {
            inner: Cursor::new(bytes),
            reads: Vec::new(),
        }
    }

    /// How many bytes were read altogether.
    pub fn read(&self) -> usize {
        self.reads.iter().map(|(_, how_many)| how_many).sum()
    }

    /// Whether any byte in this range was read.
    pub fn touched(&self, from: u64, to: u64) -> bool {
        self.reads
            .iter()
            .any(|(at, how_many)| *at < to && at + *how_many as u64 > from)
    }
}

impl Read for Counted {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let at = self.inner.position();
        let how_many = self.inner.read(buffer)?;
        self.reads.push((at, how_many));
        Ok(how_many)
    }
}

impl Seek for Counted {
    fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(to)
    }
}
