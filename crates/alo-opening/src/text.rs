//! Whether a file with no signature is text, decided from all of it.
//!
//! Every other kind begins with something that says what it is. Text does not,
//! so the only definite answer is the whole file: *the first few kilobytes were
//! text* is a guess about the rest, and a file that turns to binary after its
//! first page is exactly the file a viewer shows as garbage. This is therefore
//! the one rule that reads to the end — in pieces of [`A_PIECE`], holding only
//! what the rules below need between them, and stopping at the first byte that
//! settles it is not text.
//!
//! # The rules
//!
//! - **UTF-16** only when the file says so with a byte-order mark, is a whole
//!   number of code units, pairs every surrogate, and holds no control
//!   character beyond the five text legitimately has — tab, newline, carriage
//!   return, form feed and escape (the same five `alo-finding` allows).
//! - **UTF-8**, with or without a byte-order mark, under the same five.
//! - **An older character set** when the file is not UTF-8 and every byte is
//!   printable in one: none of the control bytes below space except those five,
//!   and no delete. Which character set it is the file does not say, and this
//!   crate does not guess — [`Kind::TextInAnOlderCharacterSet`] is the finding.
//!
//! [`Kind::TextInAnOlderCharacterSet`]: crate::Kind::TextInAnOlderCharacterSet

use std::io::{self, Read, Seek};

use crate::reading::Reading;

/// How much is read at a time.
pub(crate) const A_PIECE: usize = 64 * 1024;

/// What the whole file turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Texted {
    /// Unicode text.
    Text,
    /// Text in a character set older than Unicode.
    OlderCharacterSet,
    /// Not text.
    NotText,
}

/// Whether a character is one text legitimately holds.
fn is_texts(character: char) -> bool {
    !character.is_control() || matches!(character, '\t' | '\n' | '\r' | '\u{0c}' | '\u{1b}')
}

/// Whether a byte is printable in some character set older than Unicode.
const fn is_older_texts(byte: u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\r' | 0x0c | 0x1b | 0x20..=0x7e | 0x80..=0xff)
}

/// Decide whether a file, which is not empty, is text.
///
/// # Errors
/// Whatever the file said.
pub(crate) fn look<F: Read + Seek>(reading: &mut Reading<'_, F>) -> io::Result<Texted> {
    let first = reading.at(0, 2)?;
    match first.as_slice() {
        [0xff, 0xfe] => utf16(reading, u16::from_le_bytes),
        [0xfe, 0xff] => utf16(reading, u16::from_be_bytes),
        _ => utf8_or_older(reading),
    }
}

/// The UTF-16 rule, in the byte order the mark named.
fn utf16<F: Read + Seek>(
    reading: &mut Reading<'_, F>,
    unit: fn([u8; 2]) -> u16,
) -> io::Result<Texted> {
    if !reading.len().is_multiple_of(2) {
        return Ok(Texted::NotText);
    }
    let mut offset = 2_u64;
    let mut waiting_high: Option<u16> = None;
    while offset < reading.len() {
        let piece = reading.at(offset, A_PIECE)?;
        offset = offset.saturating_add(piece.len() as u64);
        let (pairs, _) = piece.as_chunks::<2>();
        for pair in pairs {
            let code = unit(*pair);
            let units: Vec<u16> = match waiting_high.take() {
                Some(high) => vec![high, code],
                None if (0xd800..0xdc00).contains(&code) => {
                    waiting_high = Some(code);
                    continue;
                }
                None => vec![code],
            };
            for decoded in char::decode_utf16(units) {
                match decoded {
                    Ok(character) if is_texts(character) => {}
                    _ => return Ok(Texted::NotText),
                }
            }
        }
        if piece.is_empty() {
            break;
        }
    }
    if waiting_high.is_some() {
        return Ok(Texted::NotText);
    }
    Ok(Texted::Text)
}

/// The UTF-8 rule and the older-character-set rule, in one pass.
fn utf8_or_older<F: Read + Seek>(reading: &mut Reading<'_, F>) -> io::Result<Texted> {
    let mut offset = 0_u64;
    let mut still_utf8 = true;
    let mut still_older = true;
    let mut carried: Vec<u8> = Vec::new();
    while offset < reading.len() && (still_utf8 || still_older) {
        let piece = reading.at(offset, A_PIECE)?;
        if piece.is_empty() {
            break;
        }
        offset = offset.saturating_add(piece.len() as u64);

        if still_older && !piece.iter().copied().all(is_older_texts) {
            still_older = false;
        }
        if still_utf8 {
            carried.extend_from_slice(&piece);
            let (whole, rest) = match std::str::from_utf8(&carried) {
                Ok(text) => (text.chars().all(is_texts), Vec::new()),
                Err(why) if why.error_len().is_none() => {
                    let (valid, rest) = carried.split_at(why.valid_up_to());
                    let text = std::str::from_utf8(valid).unwrap_or_default();
                    (text.chars().all(is_texts), rest.to_vec())
                }
                Err(_) => (false, Vec::new()),
            };
            still_utf8 = whole;
            carried = rest;
        }
    }
    if !carried.is_empty() {
        // The file ended part way through a character.
        still_utf8 = false;
    }
    Ok(if still_utf8 {
        Texted::Text
    } else if still_older {
        Texted::OlderCharacterSet
    } else {
        Texted::NotText
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// What these bytes are, as a whole file.
    fn of(bytes: &[u8]) -> Texted {
        let mut file = Cursor::new(bytes.to_vec());
        look(&mut Reading::of(&mut file).unwrap()).unwrap()
    }

    /// UTF-8 is text, with a mark or without one, in any language.
    #[test]
    fn utf8_is_text_in_any_language() {
        assert_eq!(of(b"Dear Anna,\r\n\tthe contract"), Texted::Text);
        assert_eq!(
            of("za\u{17c}\u{f3}\u{142}\u{107} g\u{119}\u{15b}l\u{105}\n".as_bytes()),
            Texted::Text
        );
        assert_eq!(of(b"\xef\xbb\xbfname;amount\n"), Texted::Text);
    }

    /// **Windows-1252 from an older spreadsheet is text in an older character
    /// set**, and not bytes nobody recognises: it is the commonest file of its
    /// kind a European person is sent.
    #[test]
    fn an_older_character_set_is_the_finding() {
        assert_eq!(
            of(b"Name;Stra\xdfe;Gr\xf6\xdfe\r\nM\xfcller;Hauptstra\xdfe;42\r\n"),
            Texted::OlderCharacterSet
        );
    }

    /// UTF-16 is text when it says so, and not when it is odd, broken or
    /// holds a character text does not.
    #[test]
    fn utf16_is_text_only_when_whole() {
        let mut little = vec![0xff, 0xfe];
        for unit in "Grüße, 😀".encode_utf16() {
            little.extend_from_slice(&unit.to_le_bytes());
        }
        assert_eq!(of(&little), Texted::Text);

        let mut big = vec![0xfe, 0xff];
        for unit in "Grüße".encode_utf16() {
            big.extend_from_slice(&unit.to_be_bytes());
        }
        assert_eq!(of(&big), Texted::Text);

        let mut odd = little.clone();
        odd.push(b'a');
        assert_eq!(of(&odd), Texted::NotText);

        let mut unpaired = vec![0xff, 0xfe];
        unpaired.extend_from_slice(&0xd83d_u16.to_le_bytes());
        assert_eq!(of(&unpaired), Texted::NotText);

        assert_eq!(of(b"\xff\xfe\x00\x00h\x00"), Texted::NotText);
    }

    /// **Text that turns to binary part way is not text**, however long its
    /// first page was — the whole reason this rule reads to the end.
    #[test]
    fn text_that_turns_to_binary_after_its_first_page_is_not_text() {
        let mut file = vec![b'a'; A_PIECE * 3];
        file.extend_from_slice(b"\0\0\x01\x02");
        assert_eq!(of(&file), Texted::NotText);
    }

    /// A character cut across two pieces is still one character, and a file
    /// that ends in the middle of one is not text.
    #[test]
    fn a_character_across_two_pieces_is_one_character() {
        let mut file = vec![b'a'; A_PIECE - 1];
        file.extend_from_slice("ł".as_bytes());
        file.extend_from_slice(b" end");
        assert_eq!(of(&file), Texted::Text);

        assert_eq!(of(b"caf\xc3"), Texted::OlderCharacterSet);
        assert_eq!(of(b"caf\xc3\x01"), Texted::NotText);
    }

    /// Control bytes are not text in any character set.
    #[test]
    fn control_bytes_are_not_text() {
        assert_eq!(of(b"hello\0world"), Texted::NotText);
        assert_eq!(of(b"\x00\x00\x00\x18ftypmp42"), Texted::NotText);
        assert_eq!(of(b"a\x7fb"), Texted::NotText);
        assert_eq!(of(b"\x1b[32mgreen\x1b[0m\n"), Texted::Text);
    }
}
