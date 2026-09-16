//! The three conversions this machine makes, as a closed set.
//!
//! ADR 0039 §A: the service runs the engine *with a fixed argument list chosen
//! from a closed set of three conversions*. This is that set. Nothing a turn
//! sends can name a fourth, because a request carries one of these three words
//! and a word that is not one of them is not a request.
//!
//! **The copy is always a PDF** (ADR 0039 §1): it is the one kind whose
//! rendering does not depend on another engine interpreting the copy again.

use alo_opening::Kind;

/// One conversion this machine makes, from what a document is into a PDF copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Conversion {
    /// A Word document into a PDF.
    WordDocument,
    /// An Excel workbook into a PDF.
    ExcelWorkbook,
    /// A PowerPoint presentation into a PDF.
    PowerPointPresentation,
}

impl Conversion {
    /// Every conversion, in one order.
    pub const EVERY: [Self; 3] = [
        Self::WordDocument,
        Self::ExcelWorkbook,
        Self::PowerPointPresentation,
    ];

    /// The conversion for a file of this kind, if this machine makes one.
    ///
    /// Older Office formats and OpenDocument files are recognised by
    /// `alo-opening` and are not here: ADR 0039 leaves each further kind to a
    /// registration and a test against a real file, in a later change.
    #[must_use]
    pub const fn of(kind: Kind) -> Option<Self> {
        match kind {
            Kind::WordDocument => Some(Self::WordDocument),
            Kind::ExcelWorkbook => Some(Self::ExcelWorkbook),
            Kind::PowerPointPresentation => Some(Self::PowerPointPresentation),
            _ => None,
        }
    }

    /// What a file converted this way is.
    #[must_use]
    pub const fn from(self) -> Kind {
        match self {
            Self::WordDocument => Kind::WordDocument,
            Self::ExcelWorkbook => Kind::ExcelWorkbook,
            Self::PowerPointPresentation => Kind::PowerPointPresentation,
        }
    }

    /// What the copy is.
    #[must_use]
    pub const fn into(self) -> Kind {
        Kind::Pdf
    }

    /// The word a request names this conversion by on the service's socket.
    #[must_use]
    pub const fn asked_as(self) -> &'static str {
        match self {
            Self::WordDocument => "word-document",
            Self::ExcelWorkbook => "excel-workbook",
            Self::PowerPointPresentation => "powerpoint-presentation",
        }
    }

    /// The conversion a request's word names, and nothing for any other word.
    #[must_use]
    pub fn asked(word: &str) -> Option<Self> {
        Self::EVERY
            .into_iter()
            .find(|conversion| conversion.asked_as() == word)
    }

    /// The name the document is given in the service's scratch folder, whose
    /// ending is what tells the engine which of its readers to use.
    #[must_use]
    pub const fn scratch_name(self) -> &'static str {
        match self {
            Self::WordDocument => "document.docx",
            Self::ExcelWorkbook => "document.xlsx",
            Self::PowerPointPresentation => "document.pptx",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Three conversions, each from its own kind, each into a PDF**, and a
    /// word that names none of them names nothing.
    #[test]
    fn three_conversions_and_no_fourth() {
        for conversion in Conversion::EVERY {
            assert_eq!(Conversion::of(conversion.from()), Some(conversion));
            assert_eq!(conversion.into(), Kind::Pdf);
            assert_eq!(Conversion::asked(conversion.asked_as()), Some(conversion));
        }
        for kind in Kind::EVERY {
            if !kind.is_current_office() {
                assert_eq!(Conversion::of(kind), None, "{kind:?}");
            }
        }
        for word in [
            "",
            "pdf",
            "word-document ",
            "WORD-DOCUMENT",
            "../word-document",
        ] {
            assert_eq!(Conversion::asked(word), None, "{word:?}");
        }
    }
}
