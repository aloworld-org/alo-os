//! How a kind of file is spelled in `what-opens-what.toml`.
//!
//! [`alo_opening::Kind`] is what a file is, read from its bytes, and it has no
//! spelling of its own on a disk: nothing in that crate writes a file. A
//! person's choice of what opens a kind does get written, into a file they may
//! open in an editor, so each kind needs a name that is plain to type, stable
//! across releases, and one-to-one with the kinds. This is that name.
//!
//! # Why not the media type
//!
//! `application/pdf` is what a desktop entry declares, and [`crate::media_types`]
//! reads it. It is not what a person's file is keyed by, because it is not
//! one-to-one with what a file is: `text/plain` is both [`Kind::Text`] and
//! [`Kind::TextInAnOlderCharacterSet`], and a key that meant two kinds would be
//! a choice about one of them silently made about the other.
//!
//! The `match` below names every kind, so a kind `alo-opening` adds is a
//! compile error here rather than a kind a person cannot choose for.

use alo_opening::Kind;

/// The name this kind is written under.
#[must_use]
pub const fn spelled(kind: Kind) -> &'static str {
    match kind {
        Kind::Pdf => "pdf",
        Kind::WordDocument => "word-document",
        Kind::ExcelWorkbook => "excel-workbook",
        Kind::PowerPointPresentation => "powerpoint-presentation",
        Kind::OpenDocumentText => "opendocument-text",
        Kind::OpenDocumentSpreadsheet => "opendocument-spreadsheet",
        Kind::OpenDocumentPresentation => "opendocument-presentation",
        Kind::OlderWordDocument => "older-word-document",
        Kind::OlderExcelWorkbook => "older-excel-workbook",
        Kind::OlderPowerPointPresentation => "older-powerpoint-presentation",
        Kind::RichText => "rich-text",
        Kind::Text => "text",
        Kind::TextInAnOlderCharacterSet => "text-in-an-older-character-set",
        Kind::PngImage => "png-image",
        Kind::JpegImage => "jpeg-image",
        Kind::GifImage => "gif-image",
        Kind::WebpImage => "webp-image",
        Kind::HeicPhoto => "heic-photo",
        Kind::ZipArchive => "zip-archive",
        Kind::MatroskaVideo => "matroska-video",
        Kind::WebmVideo => "webm-video",
        Kind::Mp4Video => "mp4-video",
        Kind::Mp4Audio => "mp4-audio",
        Kind::AviVideo => "avi-video",
        Kind::OggMedia => "ogg-media",
        Kind::Mp3Audio => "mp3-audio",
        Kind::WaveAudio => "wave-audio",
        Kind::FlacAudio => "flac-audio",
    }
}

/// The kind written under this name, matched exactly — or [`None`] for a name
/// that is not one.
#[must_use]
pub fn kind_spelled(name: &str) -> Option<Kind> {
    Kind::EVERY.into_iter().find(|kind| spelled(*kind) == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **Every kind has a name of its own, and the name reads back as it.**
    #[test]
    fn every_kind_is_spelled_once_and_reads_back() {
        let names: BTreeSet<&str> = Kind::EVERY.into_iter().map(spelled).collect();
        assert_eq!(names.len(), Kind::EVERY.len());
        for kind in Kind::EVERY {
            assert_eq!(kind_spelled(spelled(kind)), Some(kind));
            // Lower case, digits and hyphens: plain to type in an editor and
            // the same on every keyboard. Digits are here because the formats
            // people are sent have them in their names — MP4, MP3 — and a key
            // spelled `mpeg-four-video` would be a name nobody would guess.
            assert!(
                spelled(kind)
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{kind:?}"
            );
        }
    }

    /// **A name is matched exactly**, so a key typed in another case is not
    /// quietly taken for a kind the person may not have meant.
    #[test]
    fn a_name_that_is_not_one_is_no_kind() {
        for name in ["PDF", " pdf", "application/pdf", "docx", ""] {
            assert_eq!(kind_spelled(name), None, "{name:?}");
        }
    }
}
