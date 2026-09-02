//! The checksum a zip archive carries for every file in it.
//!
//! CRC-32 as every zip reader expects it: the reflected polynomial
//! `0xEDB88320`, starting at all ones and finishing inverted. It is here rather
//! than in [`crate::zip`] because it is a different thing to be wrong about —
//! the format says *where* the number goes, and this says *what* the number is.
//!
//! Computed a bit at a time rather than from a lookup table. A table would be
//! four times faster and would need either a `const` block this crate's lint
//! list forbids indexing into, or an initialisation nobody can read at a
//! glance; and the archive is written at disk speed, which this keeps up with.
//! The known answers in the tests are what say it is right.

/// The polynomial, reflected, as every zip reader uses it.
const POLYNOMIAL: u32 = 0xEDB8_8320;

/// A checksum being built, one chunk of a file at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Crc(u32);

impl Crc {
    /// A checksum of nothing yet.
    pub(crate) fn new() -> Self {
        Self(0xFFFF_FFFF)
    }

    /// Take in the next part of the file.
    pub(crate) fn eat(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u32::from(*byte);
            for _ in 0..8 {
                self.0 = if self.0 & 1 == 0 {
                    self.0 >> 1
                } else {
                    (self.0 >> 1) ^ POLYNOMIAL
                };
            }
        }
    }

    /// The number the archive carries.
    pub(crate) fn finish(self) -> u32 {
        !self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answers everybody else's CRC-32 gives. A checksum that agrees with
    /// this crate and with nothing else would make archives only this crate
    /// could open.
    #[test]
    fn the_known_answers_are_the_answers() {
        for (bytes, expected) in [
            ("".as_bytes(), 0x0000_0000_u32),
            ("a".as_bytes(), 0xE8B7_BE43),
            ("abc".as_bytes(), 0x3524_41C2),
            ("123456789".as_bytes(), 0xCBF4_3926),
            (
                "The quick brown fox jumps over the lazy dog".as_bytes(),
                0x414F_A339,
            ),
        ] {
            let mut crc = Crc::new();
            crc.eat(bytes);
            assert_eq!(
                crc.finish(),
                expected,
                "{:?}",
                std::str::from_utf8(bytes).unwrap_or("")
            );
        }
    }

    /// A file arrives in chunks, so the answer must not depend on where the
    /// chunk boundaries fell.
    #[test]
    fn a_file_read_in_pieces_has_the_checksum_of_the_whole() {
        let whole = "123456789".as_bytes();
        let mut in_one = Crc::new();
        in_one.eat(whole);

        let mut in_pieces = Crc::new();
        in_pieces.eat("12".as_bytes());
        in_pieces.eat("".as_bytes());
        in_pieces.eat("3456".as_bytes());
        in_pieces.eat("789".as_bytes());

        assert_eq!(in_one.finish(), in_pieces.finish());
    }

    /// One byte different is a different checksum, which is the whole point of
    /// carrying one.
    #[test]
    fn one_byte_different_is_a_different_answer() {
        let mut one = Crc::new();
        one.eat("an invoice".as_bytes());
        let mut other = Crc::new();
        other.eat("an invoicf".as_bytes());
        assert_ne!(one.finish(), other.finish());
    }
}
