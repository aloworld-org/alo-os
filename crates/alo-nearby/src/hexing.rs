//! Bytes as lowercase hexadecimal, and back — the one spelling this crate puts
//! on a wire or in a file.
//!
//! Written here rather than rented because it is twenty lines, and because the
//! one property that matters — a byte string reads back as exactly the bytes
//! it was written from, and anything else is refused rather than guessed — is
//! easier to hold in a file this size than to check in somebody else's.

/// `bytes`, two lowercase hexadecimal characters each.
pub(crate) fn said(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The bytes `said` spells, filled into `into` — or `false` if it does not
/// spell exactly that many, in lowercase, and nothing else.
///
/// Uppercase is refused on purpose: one spelling on the wire means one
/// comparison, and a proof that compared as bytes but not as text would be a
/// proof two records could disagree about.
pub(crate) fn read(said: &str, into: &mut [u8]) -> bool {
    if said.len() != into.len().saturating_mul(2) {
        return false;
    }
    let mut digits = said.bytes().map(|digit| match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    });
    for byte in into.iter_mut() {
        let (Some(Some(high)), Some(Some(low))) = (digits.next(), digits.next()) else {
            return false;
        };
        *byte = (high << 4) | low;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{read, said};

    /// What is written is what is read back, byte for byte.
    #[test]
    fn bytes_written_here_are_read_back_as_they_were() {
        let bytes = [0_u8, 1, 15, 16, 127, 128, 254, 255];
        assert_eq!(said(&bytes), "00010f107f80feff");
        let mut back = [0_u8; 8];
        assert!(read("00010f107f80feff", &mut back));
        assert_eq!(back, bytes);
    }

    /// Anything that is not exactly the bytes asked for, in lowercase, is
    /// refused rather than read as far as it goes.
    #[test]
    fn what_is_not_the_bytes_asked_for_is_refused() {
        let mut into = [0_u8; 2];
        for not_it in ["", "00", "000", "00000", "ABCD", "zz00", "00 1"] {
            assert!(!read(not_it, &mut into), "`{not_it}` was read as two bytes");
        }
        assert!(read("abcd", &mut into));
        assert_eq!(into, [0xab, 0xcd]);
    }
}
