//! The only way this crate reads a file: a bounded piece at a known place.
//!
//! **Handed an open file, never a path.** Which file a person or an agent may
//! reach is a grant, and a grant is `alo-granted`'s; a crate that took a path
//! would be a second door to the disk that nobody granted. So whoever holds the
//! grant opens the file, and this crate is handed something it can read and
//! seek — which is also what lets every rule be tested against bytes in memory.
//!
//! **Every read says how much, and where.** There is no *read the rest*: each
//! rule in `crate::looking` asks for a piece at an offset with a most it will
//! take, which is what makes *nothing reads more of a file than deciding
//! requires* something a test can count rather than a hope.

use std::io::{self, Read, Seek, SeekFrom};

/// A file being looked at, and how long it is.
pub(crate) struct Reading<'f, F> {
    /// The file, as whoever opened it handed it over.
    file: &'f mut F,
    /// How long it was when looking began.
    len: u64,
}

impl<'f, F: Read + Seek> Reading<'f, F> {
    /// Begin looking at a file.
    ///
    /// # Errors
    /// Whatever the file said when asked how long it is.
    pub(crate) fn of(file: &'f mut F) -> io::Result<Self> {
        let len = file.seek(SeekFrom::End(0))?;
        Ok(Self { file, len })
    }

    /// How long the file is.
    pub(crate) const fn len(&self) -> u64 {
        self.len
    }

    /// Up to `most` bytes starting at `offset`, and fewer only where the file
    /// ends first.
    ///
    /// # Errors
    /// Whatever the file said — including a file that ends sooner than it did
    /// when looking began, which is a file changed while it was being looked at
    /// and is not a thing this crate decides about.
    pub(crate) fn at(&mut self, offset: u64, most: usize) -> io::Result<Vec<u8>> {
        let there = self.len.saturating_sub(offset);
        let taking = usize::try_from(there).map_or(most, |there| there.min(most));
        let mut piece = vec![0; taking];
        if taking > 0 {
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.read_exact(&mut piece)?;
        }
        Ok(piece)
    }
}

/// A little-endian `u16` at `at` in `bytes`, or [`None`] where `bytes` is too
/// short to hold one.
pub(crate) fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    let piece: [u8; 2] = bytes.get(at..end)?.try_into().ok()?;
    Some(u16::from_le_bytes(piece))
}

/// A little-endian `u32` at `at` in `bytes`, or [`None`] where `bytes` is too
/// short to hold one.
pub(crate) fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    let piece: [u8; 4] = bytes.get(at..end)?.try_into().ok()?;
    Some(u32::from_le_bytes(piece))
}

/// A little-endian `u64` at `at` in `bytes`, or [`None`] where `bytes` is too
/// short to hold one.
pub(crate) fn u64_at(bytes: &[u8], at: usize) -> Option<u64> {
    let end = at.checked_add(8)?;
    let piece: [u8; 8] = bytes.get(at..end)?.try_into().ok()?;
    Some(u64::from_le_bytes(piece))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A piece is what was asked for, and shorter only where the file ends.
    #[test]
    fn a_piece_is_what_was_asked_for_and_no_more() {
        let mut file = Cursor::new(b"0123456789".to_vec());
        let mut reading = Reading::of(&mut file).unwrap();
        assert_eq!(reading.len(), 10);
        assert_eq!(reading.at(2, 3).unwrap(), b"234");
        assert_eq!(reading.at(8, 10).unwrap(), b"89");
        assert!(reading.at(10, 4).unwrap().is_empty());
        assert!(reading.at(400, 4).unwrap().is_empty());
    }

    /// Numbers are read where they are, and a place too short to hold one is
    /// nothing rather than a panic.
    #[test]
    fn a_number_too_close_to_the_end_is_nothing() {
        let bytes = [1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(u16_at(&bytes, 0), Some(1));
        assert_eq!(u32_at(&bytes, 2), Some(2));
        assert_eq!(u64_at(&bytes, 6), Some(3));
        assert_eq!(u16_at(&bytes, 13), None);
        assert_eq!(u32_at(&bytes, 12), None);
        assert_eq!(u64_at(&bytes, usize::MAX), None);
    }
}
