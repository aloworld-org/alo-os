//! The copy, inventoried after it is converted: the fonts the PDF contains.
//!
//! A PDF names every font it draws text with in a font dictionary, as
//! `/BaseFont /ABCDEF+Garamond-Bold` — a six-letter tag when only the glyphs
//! used were embedded, then the font's own name, then its style. So that is
//! what is read, and a family is the name before its style.
//!
//! # A PDF this cannot read is not a copy that lost nothing
//!
//! A PDF may pack its dictionaries into compressed object streams, where no
//! `/BaseFont` can be seen without decompressing them. The pinned engine does
//! not write those (measured against the owner's three documents on
//! 2026-09-16), and one that did would be a copy whose fonts were not read: that
//! is [`NotACopy::FontsNotReadable`], a refusal to show the copy, and never an
//! empty list of fonts that would compare as every family substituted — or, the
//! other way, as nothing to compare at all.

use std::collections::BTreeSet;

use crate::carried::compared;

/// Why a copy could not be inventoried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotACopy {
    /// It is not a PDF.
    #[error("the copy is not a PDF")]
    NotAPdf,
    /// Its fonts are packed where they cannot be read.
    #[error("the copy's fonts cannot be read")]
    FontsNotReadable,
}

/// What a copy contains that an original's losses are measured against.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Copy {
    /// Every font family it contains, by its compared name.
    families: BTreeSet<String>,
}

/// What a PDF name ends at.
const DELIMITERS: &[u8] = b" \t\r\n\x0c\0/[]<>(){}%";

/// The endings a font's own name carries after its family, which are not the
/// family: `TimesNewRomanPSMT` is Times New Roman.
const NOT_THE_FAMILY: [&str; 3] = ["psmt", "mt", "ps"];

impl Copy {
    /// Inventory a PDF.
    ///
    /// # Errors
    /// [`NotACopy`].
    pub fn of(bytes: &[u8]) -> Result<Self, NotACopy> {
        if !bytes.starts_with(b"%PDF-") {
            return Err(NotACopy::NotAPdf);
        }
        if contains(bytes, b"/ObjStm") {
            return Err(NotACopy::FontsNotReadable);
        }
        let mut families = BTreeSet::new();
        let marker = b"/BaseFont";
        let mut from = 0;
        while let Some(found) = bytes.get(from..).and_then(|rest| position(rest, marker)) {
            let after = from + found + marker.len();
            from = after;
            let Some(rest) = bytes.get(after..) else {
                break;
            };
            let start = rest
                .iter()
                .position(|byte| !b" \t\r\n\x0c\0".contains(byte))
                .unwrap_or(rest.len());
            let Some(name) = rest.get(start..).and_then(|rest| rest.strip_prefix(b"/")) else {
                continue;
            };
            let end = name
                .iter()
                .position(|byte| DELIMITERS.contains(byte))
                .unwrap_or(name.len());
            if let Some(family) = name.get(..end).map(|name| family(&decoded(name)))
                && !family.is_empty()
            {
                families.insert(family);
            }
        }
        Ok(Self { families })
    }

    /// Whether the copy contains a font of the family an original set text in,
    /// given as its compared name.
    #[must_use]
    pub fn contains_family(&self, compared_family: &str) -> bool {
        let wanted = without_endings(compared_family);
        self.families.iter().any(|family| *family == wanted)
    }

    /// Every family it contains, by compared name.
    #[cfg(test)]
    pub fn families(&self) -> impl Iterator<Item = &str> {
        self.families.iter().map(String::as_str)
    }
}

/// A PDF name with its `#xx` escapes undone.
fn decoded(name: &[u8]) -> String {
    let mut bytes = Vec::with_capacity(name.len());
    let mut at = 0;
    while let Some(byte) = name.get(at) {
        if *byte == b'#'
            && let Some(hex) = name.get(at + 1..at + 3)
            && let Ok(hex) = std::str::from_utf8(hex)
            && let Ok(value) = u8::from_str_radix(hex, 16)
        {
            bytes.push(value);
            at += 3;
        } else {
            bytes.push(*byte);
            at += 1;
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// The family a font's name in a PDF is, compared.
fn family(name: &str) -> String {
    let untagged = match name.split_once('+') {
        Some((tag, rest))
            if tag.len() == 6 && tag.chars().all(|letter| letter.is_ascii_uppercase()) =>
        {
            rest
        }
        _ => name,
    };
    let family = untagged.split(['-', ',']).next().unwrap_or(untagged);
    without_endings(&compared(family))
}

/// A compared family without the endings that are not the family.
fn without_endings(compared_family: &str) -> String {
    NOT_THE_FAMILY
        .iter()
        .find_map(|ending| {
            compared_family
                .strip_suffix(ending)
                .filter(|family| !family.is_empty())
        })
        .unwrap_or(compared_family)
        .to_owned()
}

/// Whether some bytes contain others.
fn contains(bytes: &[u8], wanted: &[u8]) -> bool {
    position(bytes, wanted).is_some()
}

/// Where some bytes first contain others.
fn position(bytes: &[u8], wanted: &[u8]) -> Option<usize> {
    bytes
        .windows(wanted.len())
        .position(|window| window == wanted)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A font's family is read past its subset tag and its style.**
    #[test]
    fn a_family_is_read_past_its_tag_and_style() {
        let copy = Copy::of(
            b"%PDF-1.7\n<</Type/Font/BaseFont/BAAAAA+NotoSerif-Regular>>\n<</BaseFont /TimesNewRomanPS-BoldMT>>\n<</BaseFont/Aptos#20Narrow,Bold>>",
        )
        .unwrap();
        assert_eq!(
            copy.families().collect::<Vec<_>>(),
            ["aptosnarrow", "notoserif", "timesnewroman"]
        );
        assert!(copy.contains_family(&compared("Noto Serif")));
        assert!(copy.contains_family(&compared("Times New Roman")));
        assert!(copy.contains_family(&compared("Aptos Narrow")));
        assert!(!copy.contains_family(&compared("Garamond")));
        assert!(!copy.contains_family(&compared("Aptos")));
    }

    /// **What is not a PDF, or hides its fonts, is not inventoried.**
    #[test]
    fn a_copy_whose_fonts_cannot_be_read_is_refused() {
        assert_eq!(Copy::of(b"PK\x03\x04"), Err(NotACopy::NotAPdf));
        assert_eq!(
            Copy::of(b"%PDF-1.7\n1 0 obj <</Type /ObjStm /N 4>> stream"),
            Err(NotACopy::FontsNotReadable)
        );
        assert_eq!(
            Copy::of(b"%PDF-1.7\n/BaseFont").unwrap().families().count(),
            0
        );
    }
}
