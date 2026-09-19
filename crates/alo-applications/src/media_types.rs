//! Which kinds of file a media type in a desktop entry declares.
//!
//! An application says what it opens in its desktop entry, as media types:
//! `MimeType=application/pdf;image/png;`. [`crate::Declared`] holds those
//! declarations as [`alo_opening::Kind`]s, because a kind is what a file is read
//! as, and this is the table between the two.
//!
//! # Deliberately partial
//!
//! A media type this table does not know declares nothing. That is not a
//! refusal of the application — it is still installed and still grantable —
//! it is a type naming a kind of file this machine does not recognise from its
//! bytes, and an association for a kind nothing can recognise is a choice that
//! could never be used. Media types are compared without regard to case,
//! because RFC 2045 says they are.

use alo_opening::Kind;

/// Each media type this machine reads, and the kinds it declares.
///
/// `text/plain` declares both kinds of plain text: an application that opens
/// text says nothing about which character set, and a file in an older one is
/// still text to it.
const TABLE: [(&str, &[Kind]); 40] = [
    ("application/pdf", &[Kind::Pdf]),
    (
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        &[Kind::WordDocument],
    ),
    (
        "application/vnd.ms-word.document.macroenabled.12",
        &[Kind::WordDocument],
    ),
    (
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        &[Kind::ExcelWorkbook],
    ),
    (
        "application/vnd.ms-excel.sheet.macroenabled.12",
        &[Kind::ExcelWorkbook],
    ),
    (
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        &[Kind::PowerPointPresentation],
    ),
    (
        "application/vnd.ms-powerpoint.presentation.macroenabled.12",
        &[Kind::PowerPointPresentation],
    ),
    (
        "application/vnd.oasis.opendocument.text",
        &[Kind::OpenDocumentText],
    ),
    (
        "application/vnd.oasis.opendocument.spreadsheet",
        &[Kind::OpenDocumentSpreadsheet],
    ),
    (
        "application/vnd.oasis.opendocument.presentation",
        &[Kind::OpenDocumentPresentation],
    ),
    ("application/msword", &[Kind::OlderWordDocument]),
    ("application/vnd.ms-excel", &[Kind::OlderExcelWorkbook]),
    (
        "application/vnd.ms-powerpoint",
        &[Kind::OlderPowerPointPresentation],
    ),
    ("application/rtf", &[Kind::RichText]),
    ("text/rtf", &[Kind::RichText]),
    ("text/plain", &[Kind::Text, Kind::TextInAnOlderCharacterSet]),
    ("image/png", &[Kind::PngImage]),
    ("image/jpeg", &[Kind::JpegImage]),
    ("image/gif", &[Kind::GifImage]),
    ("image/webp", &[Kind::WebpImage]),
    // Both, because a viewer declares whichever it was written against: `heic`
    // is the brand a telephone stamps and `heif` the family it belongs to, and
    // an application that declared only the one it had heard of would open
    // nothing.
    ("image/heic", &[Kind::HeicPhoto]),
    ("image/heif", &[Kind::HeicPhoto]),
    ("application/zip", &[Kind::ZipArchive]),
    ("application/x-zip-compressed", &[Kind::ZipArchive]),
    ("video/x-matroska", &[Kind::MatroskaVideo]),
    ("video/webm", &[Kind::WebmVideo]),
    ("audio/webm", &[Kind::WebmVideo]),
    ("video/mp4", &[Kind::Mp4Video]),
    ("video/quicktime", &[Kind::Mp4Video]),
    ("audio/mp4", &[Kind::Mp4Audio]),
    ("audio/x-m4a", &[Kind::Mp4Audio]),
    ("video/x-msvideo", &[Kind::AviVideo]),
    ("audio/ogg", &[Kind::OggMedia]),
    ("video/ogg", &[Kind::OggMedia]),
    ("application/ogg", &[Kind::OggMedia]),
    ("audio/mpeg", &[Kind::Mp3Audio]),
    ("audio/wav", &[Kind::WaveAudio]),
    ("audio/x-wav", &[Kind::WaveAudio]),
    ("audio/flac", &[Kind::FlacAudio]),
    ("audio/x-flac", &[Kind::FlacAudio]),
];

/// The kinds this media type declares — none for a type this machine does not
/// read.
#[must_use]
pub fn kinds_declared_by(media_type: &str) -> &'static [Kind] {
    let media_type = media_type.trim();
    TABLE
        .iter()
        .find(|(named, _)| named.eq_ignore_ascii_case(media_type))
        .map_or(&[], |(_, kinds)| kinds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **Every kind this machine reads from bytes can be declared**, so no
    /// kind is one only a person's choice could ever open.
    #[test]
    fn every_kind_is_declared_by_some_media_type() {
        let declared: BTreeSet<Kind> = TABLE
            .iter()
            .flat_map(|(_, kinds)| kinds.iter().copied())
            .collect();
        assert_eq!(declared, Kind::EVERY.into_iter().collect());
    }

    /// A type is read without regard to case, and one nobody knows declares
    /// nothing rather than something near it.
    #[test]
    fn a_type_is_matched_without_case_and_an_unknown_one_declares_nothing() {
        assert_eq!(kinds_declared_by("Application/PDF"), [Kind::Pdf]);
        assert_eq!(
            kinds_declared_by("text/plain"),
            [Kind::Text, Kind::TextInAnOlderCharacterSet]
        );
        for unknown in ["application/x-shellscript", "text/html", "", "pdf"] {
            assert!(kinds_declared_by(unknown).is_empty(), "{unknown:?}");
        }
    }

    /// No type is listed twice, so no type can declare two different answers.
    #[test]
    fn no_type_is_listed_twice() {
        let named: BTreeSet<String> = TABLE
            .iter()
            .map(|(named, _)| named.to_ascii_lowercase())
            .collect();
        assert_eq!(named.len(), TABLE.len());
    }
}
