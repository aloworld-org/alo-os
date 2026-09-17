//! What a file is, as a closed list of the kinds this crate can recognise.
//!
//! A [`Kind`] is decided from a file's bytes by `crate::looking` and never from
//! its name. It is the list of things this machine could be *able* to open or
//! convert: a program, a locked document, an empty file and bytes of no known
//! kind are deliberately not kinds, because no machine is ever told it opens
//! one — they are [`crate::Appears`] instead, and `crate::machine` has no way
//! to be handed them.
//!
//! # Why the list is the list
//!
//! It is what a person is sent: the three current office formats and the three
//! older ones they replaced, their open-standard equivalents, PDF, rich and
//! plain text, the four image formats a mail or a web page carries, a zip
//! archive — and, since
//! [ADR 0052's sibling work on media](../../../docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md),
//! the films and sound recordings people send each other. A kind that is not
//! here is not refused because it is bad; it is *not recognised*, and the
//! sentence for that says so about the machine rather than about the file.
//! Adding one is adding a variant, a word and a rule in `crate::looking`, in one
//! change.
//!
//! # A media kind is the wrapping, not what is inside it
//!
//! [`Kind::MatroskaVideo`] says a file is a Matroska file. It does **not** say
//! which video is inside it, and it cannot: one Matroska file holds AV1 that
//! this machine plays and the next holds something it may not, and telling them
//! apart means reading the tracks rather than the first four bytes.
//!
//! That is deliberate and it is the split ADR 0051 needs. This crate answers
//! *what is this file*, from its bytes, for every file a person opens;
//! `alo-playing` answers *can this machine play it*, by looking inside the
//! wrapping at the codecs, against the list ADR 0051 settles. A person who is
//! sent a film this machine cannot play meets `crate::Cannot::NothingHereOpens`
//! — the same sentence as for a document nothing here opens, which is the point:
//! **one sentence for *I cannot open this*, whether it is a document or a
//! film.**

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What a file is, from its own bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// A PDF document.
    Pdf,
    /// A Word document in the format Word has saved in since 2007.
    WordDocument,
    /// An Excel workbook in the format Excel has saved in since 2007.
    ExcelWorkbook,
    /// A PowerPoint presentation in the format PowerPoint has saved in since
    /// 2007.
    PowerPointPresentation,
    /// An OpenDocument text document.
    OpenDocumentText,
    /// An OpenDocument spreadsheet.
    OpenDocumentSpreadsheet,
    /// An OpenDocument presentation.
    OpenDocumentPresentation,
    /// A Word document in the format before 2007.
    OlderWordDocument,
    /// An Excel workbook in the format before 2007.
    OlderExcelWorkbook,
    /// A PowerPoint presentation in the format before 2007.
    OlderPowerPointPresentation,
    /// A rich text document.
    RichText,
    /// Plain text in Unicode — UTF-8, or UTF-16 that says so at its start.
    Text,
    /// Plain text in a character set older than Unicode, which the file does
    /// not name.
    TextInAnOlderCharacterSet,
    /// A PNG image.
    PngImage,
    /// A JPEG image.
    JpegImage,
    /// A GIF image.
    GifImage,
    /// A WebP image.
    WebpImage,
    /// A zip archive that is not a document stored as one.
    ZipArchive,
    /// A film in a Matroska file, which is what `.mkv` is.
    MatroskaVideo,
    /// A film in a WebM file, which is Matroska under another name and is what
    /// a web page sends.
    WebmVideo,
    /// A film in the container MP4 names, which is most of what a telephone
    /// records and most of what people send each other.
    Mp4Video,
    /// Sound in that same container, with no picture in it: what `.m4a` is.
    Mp4Audio,
    /// A film in an AVI file, which is older and still arrives.
    AviVideo,
    /// An Ogg file, which usually holds sound and can hold a picture.
    OggMedia,
    /// A sound recording in an MP3 file.
    Mp3Audio,
    /// A sound recording in a WAVE file, which is what `.wav` is.
    WaveAudio,
    /// A sound recording in a FLAC file.
    FlacAudio,
}

impl Kind {
    /// Every kind, in the order [`crate::words::THE_NAMES`] names them.
    pub const EVERY: [Self; 27] = [
        Self::Pdf,
        Self::WordDocument,
        Self::ExcelWorkbook,
        Self::PowerPointPresentation,
        Self::OpenDocumentText,
        Self::OpenDocumentSpreadsheet,
        Self::OpenDocumentPresentation,
        Self::OlderWordDocument,
        Self::OlderExcelWorkbook,
        Self::OlderPowerPointPresentation,
        Self::RichText,
        Self::Text,
        Self::TextInAnOlderCharacterSet,
        Self::PngImage,
        Self::JpegImage,
        Self::GifImage,
        Self::WebpImage,
        Self::ZipArchive,
        Self::MatroskaVideo,
        Self::WebmVideo,
        Self::Mp4Video,
        Self::Mp4Audio,
        Self::AviVideo,
        Self::OggMedia,
        Self::Mp3Audio,
        Self::WaveAudio,
        Self::FlacAudio,
    ];

    /// The word this kind is named by.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Pdf => words::KIND_PDF,
            Self::WordDocument => words::KIND_WORD_DOCUMENT,
            Self::ExcelWorkbook => words::KIND_EXCEL_WORKBOOK,
            Self::PowerPointPresentation => words::KIND_POWERPOINT_PRESENTATION,
            Self::OpenDocumentText => words::KIND_OPENDOCUMENT_TEXT,
            Self::OpenDocumentSpreadsheet => words::KIND_OPENDOCUMENT_SPREADSHEET,
            Self::OpenDocumentPresentation => words::KIND_OPENDOCUMENT_PRESENTATION,
            Self::OlderWordDocument => words::KIND_OLDER_WORD_DOCUMENT,
            Self::OlderExcelWorkbook => words::KIND_OLDER_EXCEL_WORKBOOK,
            Self::OlderPowerPointPresentation => words::KIND_OLDER_POWERPOINT_PRESENTATION,
            Self::RichText => words::KIND_RICH_TEXT,
            Self::Text => words::KIND_TEXT,
            Self::TextInAnOlderCharacterSet => words::KIND_TEXT_IN_AN_OLDER_CHARACTER_SET,
            Self::PngImage => words::KIND_PNG_IMAGE,
            Self::JpegImage => words::KIND_JPEG_IMAGE,
            Self::GifImage => words::KIND_GIF_IMAGE,
            Self::WebpImage => words::KIND_WEBP_IMAGE,
            Self::ZipArchive => words::KIND_ZIP_ARCHIVE,
            Self::MatroskaVideo => words::KIND_MATROSKA_VIDEO,
            Self::WebmVideo => words::KIND_WEBM_VIDEO,
            Self::Mp4Video => words::KIND_MP4_VIDEO,
            Self::Mp4Audio => words::KIND_MP4_AUDIO,
            Self::AviVideo => words::KIND_AVI_VIDEO,
            Self::OggMedia => words::KIND_OGG_MEDIA,
            Self::Mp3Audio => words::KIND_MP3_AUDIO,
            Self::WaveAudio => words::KIND_WAVE_AUDIO,
            Self::FlacAudio => words::KIND_FLAC_AUDIO,
        }
    }

    /// **Whether a file of this kind is something to play** rather than
    /// something to read or look at.
    ///
    /// Read by `alo-playing`, which is the crate that decides whether this
    /// machine can play what is inside one (ADR 0051). Nothing here says it
    /// can: a media kind is the wrapping, and the codecs inside it are a
    /// different question asked of a different crate.
    #[must_use]
    pub const fn is_played(self) -> bool {
        matches!(
            self,
            Self::MatroskaVideo
                | Self::WebmVideo
                | Self::Mp4Video
                | Self::Mp4Audio
                | Self::AviVideo
                | Self::OggMedia
                | Self::Mp3Audio
                | Self::WaveAudio
                | Self::FlacAudio
        )
    }

    /// This kind, named in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// Whether a file of this kind is stored as a zip archive underneath.
    ///
    /// Read by `crate::naming`: a damaged zip is consistent with a name that
    /// claims any of these, because being cut short is what stopped this crate
    /// seeing which one it is.
    #[must_use]
    pub const fn is_stored_as_a_zip(self) -> bool {
        matches!(
            self,
            Self::WordDocument
                | Self::ExcelWorkbook
                | Self::PowerPointPresentation
                | Self::OpenDocumentText
                | Self::OpenDocumentSpreadsheet
                | Self::OpenDocumentPresentation
                | Self::ZipArchive
        )
    }

    /// Whether a file of this kind is stored in the older Office storage
    /// format underneath.
    #[must_use]
    pub const fn is_stored_as_older_office(self) -> bool {
        matches!(
            self,
            Self::OlderWordDocument | Self::OlderExcelWorkbook | Self::OlderPowerPointPresentation
        )
    }

    /// Whether a file of this kind is one of the three current Office
    /// formats, which is what a password-protected one is saved under.
    #[must_use]
    pub const fn is_current_office(self) -> bool {
        matches!(
            self,
            Self::WordDocument | Self::ExcelWorkbook | Self::PowerPointPresentation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every kind is named, by the word in its place in the list**, so the
    /// order the words are declared in and the order kinds are listed in
    /// cannot drift apart unnoticed.
    #[test]
    fn every_kind_is_named_by_its_own_word() {
        for (kind, word) in Kind::EVERY.iter().zip(words::THE_NAMES.iter()) {
            assert_eq!(kind.word(), *word, "{kind:?}");
        }
        let mut seen: Vec<&str> = Kind::EVERY.iter().map(|kind| kind.word().named()).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), Kind::EVERY.len());
    }

    /// Nothing is stored two ways at once.
    #[test]
    fn no_kind_is_stored_two_ways() {
        for kind in Kind::EVERY {
            assert!(
                !(kind.is_stored_as_a_zip() && kind.is_stored_as_older_office()),
                "{kind:?}"
            );
            if kind.is_current_office() {
                assert!(kind.is_stored_as_a_zip(), "{kind:?}");
            }
        }
    }
}
