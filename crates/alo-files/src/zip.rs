//! An archive, in the one format alo OS makes: a zip, with everything stored.
//!
//! **Why zip, and why stored.** An archive is made so that a person can send
//! it, keep it, or open it somewhere else, and the only archive every desktop
//! opens without being told how is a zip. Nothing is compressed, because
//! compression is a second thing to be wrong about inside a security boundary
//! and it buys a folder of PDFs and photographs almost nothing; a folder of
//! text will be compressed by whatever it is sent through. The format's own
//! word for that is "stored", and every reader understands it.
//!
//! **What is deliberately not here.** No ZIP64, no encryption, no comments, no
//! extra fields, no data descriptors. The bounds in [`crate::archiving`] keep
//! an archive inside what those absences allow, and a bound refused in words is
//! better than a format written half-way.
//!
//! # Every size is written twice, and this file writes them both
//!
//! A stored zip records each file's size and checksum *before* its bytes and
//! again in the directory at the end. Neither is known until the file has been
//! read, and reading it twice would let it change between the two readings and
//! produce an archive whose header disagrees with its own contents. So each
//! file is copied once and its header is corrected afterwards by seeking back
//! to it, which is what an archive being a seekable file on a disk is for.
//!
//! # Dates
//!
//! Zip keeps the DOS date of 1980, which is a real limit rather than one of
//! ours: a file older than 1980 or newer than 2107 cannot be written in it, and
//! is clamped to the nearest end. The alternative is refusing to archive
//! somebody's document because of the year in its timestamp, which nobody would
//! thank us for. Two-second steps are the same kind of limit, and the same
//! answer.
//!
//! **The moment written is UTC**, and the format has nowhere to say so — a DOS
//! timestamp is conventionally the local time of whoever wrote the archive, and
//! carries no offset. Writing local time would mean asking this machine what
//! its offset is, which `std` cannot do and which every crate that can does
//! either through a dependency whose lookup is unsound in a threaded process or
//! through code this repository forbids. So it is UTC, consistently, and
//! `docs/quirks.md` records that a reader will show a file's time shifted by
//! the offset of wherever the archive was made.

use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::crc::Crc;
use crate::failed::Failed;

/// How much of a file is read at once on its way into an archive.
const AT_A_TIME: usize = 64 * 1024;

/// The signature every local file header starts with.
const LOCAL: u32 = 0x0403_4b50;
/// The signature every entry in the directory at the end starts with.
const IN_DIRECTORY: u32 = 0x0201_4b50;
/// The signature the record at the very end starts with.
const END: u32 = 0x0605_4b50;
/// The version of the format needed to read what this writes: 2.0, which is
/// what stored entries have needed since 1993.
const NEEDS: u16 = 20;
/// The flag that says the names in this archive are UTF-8.
const NAMES_ARE_UTF8: u16 = 0x0800;
/// The compression method: none, which the format calls stored.
const STORED: u16 = 0;
/// The attribute bit that says an entry is a folder.
const IS_A_FOLDER: u32 = 0x10;
/// Where a local header keeps the checksum and the two sizes, counted from the
/// start of the header — the twelve bytes this file goes back to correct.
const CORRECTION_AT: u64 = 14;
/// How many bytes of a local header come before the name.
const LOCAL_HEADER: usize = 30;
/// How many bytes of an entry in the directory at the end come before the name.
const DIRECTORY_HEADER: usize = 46;

/// One thing that has gone into the archive, remembered for the directory that
/// is written at the end.
#[derive(Debug, Clone)]
struct Went {
    /// Its name inside the archive, with `/` between the parts.
    name: String,
    /// Its checksum.
    crc: u32,
    /// How many bytes of it were written.
    bytes: u32,
    /// Where its local header is, counted from the start of the archive.
    at: u32,
    /// Whether it is a folder.
    folder: bool,
    /// When it was last written, in the format's own way of saying so.
    when: DosMoment,
}

/// An archive being written.
#[derive(Debug)]
pub(crate) struct Archive {
    /// Where the bytes go.
    out: File,
    /// Where the archive is, for the words in a failure.
    at: PathBuf,
    /// What has gone in.
    went: Vec<Went>,
    /// How many bytes have been written.
    written: u64,
    /// The most bytes this archive may hold.
    most: u64,
}

impl Archive {
    /// Begin an archive at this path.
    ///
    /// Created **only if nothing is there** — one syscall that refuses an
    /// existing file, an existing folder and an existing link alike. Opening
    /// for writing and truncating would follow a link and empty whatever it
    /// pointed at, which is the escape this crate exists to stop arriving by
    /// the back door.
    ///
    /// # Errors
    /// [`Failed::AlreadyThere`] if anything is at that path, or
    /// [`Failed::TheMachineSaidNo`].
    pub(crate) fn beginning(at: &Path, most: u64) -> Result<Self, Failed> {
        let out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(at)
            .map_err(|why| {
                if why.kind() == ErrorKind::AlreadyExists {
                    Failed::AlreadyThere {
                        path: at.display().to_string(),
                    }
                } else {
                    Failed::machine(at, "written", &why)
                }
            })?;
        Ok(Self {
            out,
            at: at.to_owned(),
            went: Vec::new(),
            written: 0,
            most,
        })
    }

    /// Put a folder in, so that an empty one survives being archived.
    ///
    /// # Errors
    /// [`Failed`] if the archive would outgrow what this format holds, or the
    /// machine would not write.
    pub(crate) fn folder(&mut self, name: &str, when: SystemTime) -> Result<(), Failed> {
        let name = format!("{}/", inside(name));
        let at = self.where_we_are()?;
        let when = DosMoment::of(when);
        let header = self.header(LOCAL, &name, 0, 0, when)?;
        self.put(&header)?;
        self.went.push(Went {
            name,
            crc: 0,
            bytes: 0,
            at,
            folder: true,
            when,
        });
        Ok(())
    }

    /// Put a file in, copying it once and correcting its header afterwards.
    ///
    /// # Errors
    /// [`Failed`] if the file went away, the archive would outgrow what this
    /// format holds, or the machine would not read or write.
    pub(crate) fn file(&mut self, name: &str, from: &Path, when: SystemTime) -> Result<(), Failed> {
        let name = inside(name);
        let at = self.where_we_are()?;
        let when = DosMoment::of(when);
        let header = self.header(LOCAL, &name, 0, 0, when)?;
        self.put(&header)?;

        let mut reading = File::open(from).map_err(|why| Failed::machine(from, "read", &why))?;
        let mut buffer = [0_u8; AT_A_TIME];
        let mut crc = Crc::new();
        let mut bytes = 0_u64;
        loop {
            let read = reading
                .read(&mut buffer)
                .map_err(|why| Failed::machine(from, "read", &why))?;
            if read == 0 {
                break;
            }
            let chunk = buffer.get(..read).ok_or_else(|| Failed::TheMachineSaidNo {
                path: from.display().to_string(),
                doing: "read".to_owned(),
                why: "the machine said it read more than it was asked for".to_owned(),
            })?;
            crc.eat(chunk);
            bytes += read as u64;
            self.put(chunk)?;
        }
        let crc = crc.finish();
        let bytes = self.four_bytes(bytes)?;

        // The header was written before the file was read, so the checksum and
        // the two sizes in it are still zeroes. This is the correction, and it
        // is why an archive is written to a file rather than to a pipe.
        let mut fixed = Vec::with_capacity(12);
        push_u32(&mut fixed, crc);
        push_u32(&mut fixed, bytes);
        push_u32(&mut fixed, bytes);
        self.out
            .seek(SeekFrom::Start(u64::from(at) + CORRECTION_AT))
            .and_then(|_| self.out.write_all(&fixed))
            .and_then(|()| self.out.seek(SeekFrom::End(0)))
            .map_err(|why| Failed::machine(&self.at, "written", &why))?;

        self.went.push(Went {
            name,
            crc,
            bytes,
            at,
            folder: false,
            when,
        });
        Ok(())
    }

    /// Write the directory at the end, and say how big the archive is.
    ///
    /// # Errors
    /// [`Failed`] if the archive would outgrow what this format holds, or the
    /// machine would not write.
    pub(crate) fn finish(mut self) -> Result<u64, Failed> {
        let directory_at = self.where_we_are()?;
        let went = std::mem::take(&mut self.went);
        for one in &went {
            let mut entry = self.header(IN_DIRECTORY, &one.name, one.crc, one.bytes, one.when)?;
            push_u16(&mut entry, 0); // no comment
            push_u16(&mut entry, 0); // the first and only disk
            push_u16(&mut entry, 0); // nothing to say about the contents
            push_u32(&mut entry, if one.folder { IS_A_FOLDER } else { 0 });
            push_u32(&mut entry, one.at);
            entry.extend_from_slice(one.name.as_bytes());
            self.put(&entry)?;
        }
        let directory_bytes = self.four_bytes(self.written - u64::from(directory_at))?;

        let mut end = Vec::with_capacity(22);
        let how_many = how_many(went.len(), &self.at)?;
        push_u32(&mut end, END);
        push_u16(&mut end, 0); // this disk
        push_u16(&mut end, 0); // the disk the directory starts on
        push_u16(&mut end, how_many);
        push_u16(&mut end, how_many);
        push_u32(&mut end, directory_bytes);
        push_u32(&mut end, directory_at);
        push_u16(&mut end, 0); // no comment
        self.put(&end)?;

        self.out
            .flush()
            .map_err(|why| Failed::machine(&self.at, "written", &why))?;
        Ok(self.written)
    }

    /// The part of a local header and of a directory entry that is the same in
    /// both, up to and including the name of a local one.
    ///
    /// The two records differ in what comes after — a directory entry carries
    /// where the file's header is and what the file was on the machine that
    /// wrote it — and are otherwise the same fields in the same order, which is
    /// why one function writes both rather than two that have to be kept in
    /// step.
    fn header(
        &self,
        signature: u32,
        name: &str,
        crc: u32,
        bytes: u32,
        when: DosMoment,
    ) -> Result<Vec<u8>, Failed> {
        let long = how_many(name.len(), &self.at)?;
        let fixed = if signature == IN_DIRECTORY {
            DIRECTORY_HEADER
        } else {
            LOCAL_HEADER
        };
        let mut header = Vec::with_capacity(fixed + name.len());
        push_u32(&mut header, signature);
        if signature == IN_DIRECTORY {
            push_u16(&mut header, NEEDS); // written by
        }
        push_u16(&mut header, NEEDS);
        push_u16(&mut header, NAMES_ARE_UTF8);
        push_u16(&mut header, STORED);
        push_u16(&mut header, when.time);
        push_u16(&mut header, when.date);
        push_u32(&mut header, crc);
        push_u32(&mut header, bytes);
        push_u32(&mut header, bytes);
        push_u16(&mut header, long);
        push_u16(&mut header, 0); // nothing extra
        if signature == LOCAL {
            header.extend_from_slice(name.as_bytes());
        }
        Ok(header)
    }

    /// Where in the archive the next thing will start.
    ///
    /// Four bytes, because that is what the format keeps an offset in. The
    /// bounds in [`crate::archiving`] are what stop this being reachable, and
    /// it is a refusal rather than a truncation because an archive with a wrong
    /// offset in it is an archive that opens and lies.
    fn where_we_are(&self) -> Result<u32, Failed> {
        self.four_bytes(self.written)
    }

    /// A number as the format keeps it, in four bytes.
    fn four_bytes(&self, number: u64) -> Result<u32, Failed> {
        u32::try_from(number).map_err(|_| Failed::TooMuch {
            folder: self.at.display().to_string(),
            most: self.most,
        })
    }

    /// Write bytes, refusing before writing any that would take the archive
    /// past what it may hold.
    fn put(&mut self, bytes: &[u8]) -> Result<(), Failed> {
        let after = self.written.saturating_add(bytes.len() as u64);
        if after > self.most {
            return Err(Failed::TooMuch {
                folder: self.at.display().to_string(),
                most: self.most,
            });
        }
        self.out
            .write_all(bytes)
            .map_err(|why| Failed::machine(&self.at, "written", &why))?;
        self.written = after;
        Ok(())
    }
}

/// A name as it is written inside an archive: parts separated by `/`, whatever
/// this machine separates them with.
fn inside(name: &str) -> String {
    name.replace('\\', "/")
}

/// A count the format keeps in two bytes.
fn how_many(number: usize, at: &Path) -> Result<u16, Failed> {
    u16::try_from(number).map_err(|_| Failed::TooMany {
        folder: at.display().to_string(),
        most: usize::from(u16::MAX),
    })
}

/// Put a number in, smallest byte first, as the format keeps it.
fn push_u16(into: &mut Vec<u8>, number: u16) {
    into.extend_from_slice(&number.to_le_bytes());
}

/// Put a number in, smallest byte first, as the format keeps it.
fn push_u32(into: &mut Vec<u8>, number: u32) {
    into.extend_from_slice(&number.to_le_bytes());
}

/// A moment as a zip keeps it: two sixteen-bit numbers, counted from 1980.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DosMoment {
    /// The hour, minute and two-second interval.
    time: u16,
    /// The year since 1980, the month and the day.
    date: u16,
}

impl DosMoment {
    /// The first moment this format can express: midnight, 1 January 1980.
    const START: Self = Self {
        time: 0,
        date: (1 << 5) | 1,
    };

    /// The last moment it can express: a second before 2108.
    const LAST: Self = Self {
        time: (23 << 11) | (59 << 5) | 29,
        date: (127 << 9) | (12 << 5) | 31,
    };

    /// When this was, in the format's own way of saying so, clamped to the ends
    /// it can express.
    pub(crate) fn of(when: SystemTime) -> Self {
        match when.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(since) => Self::at(since),
            Err(_) => Self::START,
        }
    }

    /// When this was, counted from the start of 1970.
    fn at(since: Duration) -> Self {
        /// How many seconds a day is.
        const A_DAY: u64 = 24 * 60 * 60;

        let seconds = since.as_secs();
        let (days, rest) = (seconds / A_DAY, seconds % A_DAY);
        let (year, month, day) = civil(days);
        if year < 1980 {
            return Self::START;
        }
        if year > 2107 {
            return Self::LAST;
        }
        let (hour, minute, second) = (rest / 3600, (rest % 3600) / 60, rest % 60);
        Self {
            time: ((hour as u16) << 11) | ((minute as u16) << 5) | (second as u16 / 2),
            date: (((year - 1980) as u16) << 9) | ((month as u16) << 5) | (day as u16),
        }
    }
}

/// The year, month and day a count of days since 1 January 1970 lands on.
///
/// Howard Hinnant's `civil_from_days`, which is the calendar arithmetic every
/// date library uses and which is written out here rather than depended upon:
/// it is ten lines, it has no state, and the alternative is a dependency inside
/// a crate whose value is being small enough to read.
fn civil(days: u64) -> (u64, u64, u64) {
    let days = days + 719_468;
    let era = days / 146_097;
    let of_era = days - era * 146_097; // 0..=146_096
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // 0..=365
    let shifted = (5 * of_year + 2) / 153; // 0..=11, counted from March
    let day = of_year - (153 * shifted + 2) / 5 + 1;
    let month = if shifted < 10 {
        shifted + 3
    } else {
        shifted - 9
    };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[expect(
    clippy::indexing_slicing,
    reason = "a header read back by the offsets the format states is the test; a wrong offset should fail here"
)]
mod tests {
    use super::*;
    use crate::testing::a_folder_of_our_own;

    /// The calendar, against dates everybody knows. Getting this wrong would
    /// put every file in every archive on the wrong day, which nobody would
    /// notice until they went looking for one.
    #[test]
    fn the_calendar_lands_on_the_days_everybody_knows() {
        for (seconds, expected) in [
            (0_u64, (1970, 1, 1)),
            (315_532_800, (1980, 1, 1)),
            (946_684_800, (2000, 1, 1)),
            (951_782_400, (2000, 2, 29)),
            (1_000_000_000, (2001, 9, 9)),
            (1_600_000_000, (2020, 9, 13)),
        ] {
            assert_eq!(civil(seconds / (24 * 60 * 60)), expected, "{seconds}");
        }
    }

    /// A moment goes in as the year, month, day and time a zip reader will show
    /// for it.
    #[test]
    fn a_moment_is_kept_the_way_a_zip_reader_reads_it() {
        // 13 September 2020, 12:26:40 UTC.
        let moment = DosMoment::at(Duration::from_secs(1_600_000_000));
        assert_eq!(moment.date >> 9, 2020 - 1980);
        assert_eq!((moment.date >> 5) & 0xF, 9);
        assert_eq!(moment.date & 0x1F, 13);
        assert_eq!(moment.time >> 11, 12);
        assert_eq!((moment.time >> 5) & 0x3F, 26);
        assert_eq!((moment.time & 0x1F) * 2, 40);
    }

    /// A file older than the format can express is clamped rather than refused.
    /// Refusing to archive somebody's document because of the year in its
    /// timestamp would be a rule nobody would thank us for.
    #[test]
    fn a_moment_the_format_cannot_hold_is_clamped_to_the_end_it_is_past() {
        assert_eq!(DosMoment::at(Duration::from_secs(0)), DosMoment::START);
        assert_eq!(
            DosMoment::of(SystemTime::UNIX_EPOCH - Duration::from_secs(60)),
            DosMoment::START
        );
        assert_eq!(
            DosMoment::at(Duration::from_secs(5_000_000_000)),
            DosMoment::LAST
        );
        // And the first moment it can hold is not clamped away from itself.
        assert_eq!(
            DosMoment::at(Duration::from_secs(315_532_800)),
            DosMoment::START
        );
    }

    /// A name is spelled with `/` inside an archive whatever this machine
    /// spells it with, because that is what a reader on another machine
    /// expects.
    #[test]
    fn a_name_inside_an_archive_is_spelled_the_way_the_format_spells_it() {
        assert_eq!(inside("2026\\March\\march.pdf"), "2026/March/march.pdf");
        assert_eq!(inside("2026/March/march.pdf"), "2026/March/march.pdf");
    }

    /// A local header is the thirty bytes the format states, with the name
    /// after them — read back by offset, so that a field written in the wrong
    /// order is a failing test rather than an archive nothing opens.
    #[test]
    fn a_local_header_is_the_bytes_the_format_asks_for() {
        let folder = a_folder_of_our_own("header");
        let archive = Archive::beginning(&folder.join("invoices.zip"), 1024).unwrap();
        let header = archive
            .header(LOCAL, "march.pdf", 0x3524_41C2, 10, DosMoment::START)
            .unwrap();

        assert_eq!(header.len(), LOCAL_HEADER + "march.pdf".len());
        assert_eq!(u32::from_le_bytes(header[0..4].try_into().unwrap()), LOCAL);
        assert_eq!(u16::from_le_bytes(header[4..6].try_into().unwrap()), NEEDS);
        assert_eq!(
            u16::from_le_bytes(header[6..8].try_into().unwrap()),
            NAMES_ARE_UTF8
        );
        assert_eq!(
            u16::from_le_bytes(header[8..10].try_into().unwrap()),
            STORED
        );
        assert_eq!(
            u32::from_le_bytes(header[14..18].try_into().unwrap()),
            0x3524_41C2,
            "the checksum is not where the correction goes back to"
        );
        assert_eq!(u32::from_le_bytes(header[18..22].try_into().unwrap()), 10);
        assert_eq!(u32::from_le_bytes(header[22..26].try_into().unwrap()), 10);
        assert_eq!(u16::from_le_bytes(header[26..28].try_into().unwrap()), 9);
        assert_eq!(&header[LOCAL_HEADER..], "march.pdf".as_bytes());

        drop(archive);
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// An archive that would outgrow what it may hold is refused while it is
    /// being written, rather than finished as something that opens and lies.
    #[test]
    fn an_archive_that_would_outgrow_what_it_holds_is_refused() {
        let folder = a_folder_of_our_own("toomuch");
        let big = folder.join("big.txt");
        std::fs::write(&big, vec![b'x'; 4096]).unwrap();

        let mut archive = Archive::beginning(&folder.join("invoices.zip"), 512).unwrap();
        let too_much = archive
            .file("big.txt", &big, SystemTime::UNIX_EPOCH)
            .unwrap_err();
        assert!(matches!(too_much, Failed::TooMuch { .. }), "{too_much:?}");

        let _ = std::fs::remove_dir_all(&folder);
    }

    /// Nothing is written where something already is — not over a file, and not
    /// through a link to somewhere nobody granted.
    #[test]
    fn an_archive_is_never_written_where_something_already_is() {
        let folder = a_folder_of_our_own("exists");
        let taken = folder.join("invoices.zip");
        std::fs::write(&taken, b"somebody else's").unwrap();

        let refused = Archive::beginning(&taken, 1024).unwrap_err();
        assert!(
            matches!(refused, Failed::AlreadyThere { .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read(&taken).unwrap(), b"somebody else's");

        let _ = std::fs::remove_dir_all(&folder);
    }
}
