//! Bytes as the door writes them: lowercase hexadecimal, one spelling only.
//!
//! Its own file because two things use it — an identity and a proof — and a
//! second copy of a parser in a privileged component is a second place for the
//! one bug that matters to hide.

/// The characters, in order.
const DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Read exactly `into.len()` bytes out of `written`, or read nothing.
///
/// `true` only when `written` is exactly twice as long as `into` and every
/// character is a lowercase hexadecimal digit.
pub fn read_into(written: &str, into: &mut [u8]) -> bool {
    let written = written.as_bytes();
    if written.len() != into.len().saturating_mul(2) {
        return false;
    }
    let (pairs, _) = written.as_chunks::<2>();
    for (byte, [high, low]) in into.iter_mut().zip(pairs) {
        let (Some(high), Some(low)) = (digit(*high), digit(*low)) else {
            return false;
        };
        *byte = (high << 4) | low;
    }
    true
}

/// The value of one lowercase hexadecimal digit.
fn digit(character: u8) -> Option<u8> {
    match character {
        b'0'..=b'9' => Some(character - b'0'),
        b'a'..=b'f' => Some(character - b'a' + 10),
        _ => None,
    }
}

/// These bytes, written.
pub fn written(bytes: &[u8]) -> String {
    bytes
        .iter()
        .flat_map(|byte| {
            [
                DIGITS.get(usize::from(byte >> 4)),
                DIGITS.get(usize::from(byte & 0x0f)),
            ]
        })
        .flatten()
        .map(|digit| char::from(*digit))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every byte there is survives being written and read back.
    #[test]
    fn every_byte_reads_back_as_it_was_written() {
        let every: Vec<u8> = (0..=255).collect();
        let mut back = vec![0; every.len()];
        assert!(read_into(&written(&every), &mut back));
        assert_eq!(back, every);
    }

    /// Anything but lowercase digits of exactly the right length reads as
    /// nothing.
    #[test]
    fn anything_else_reads_as_nothing() {
        let mut into = [0; 2];
        for written in ["abc", "abcdef", "ABCD", "gg00", "+1ff", "ab d", "ÿÿ"] {
            assert!(!read_into(written, &mut into), "{written}");
        }
    }
}
