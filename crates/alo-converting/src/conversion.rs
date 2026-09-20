//! The conversions this machine makes, as a closed set — and one that is
//! written down and not made.
//!
//! ADR 0039 §A: the service runs the engine *with a fixed argument list chosen
//! from a closed set* of conversions. [`Conversion::EVERY`] is that set.
//! Nothing a turn sends can name one that is not in it, because a request
//! carries one of these words and a word that is not one of them is not a
//! request.
//!
//! **The set began at three and grows by measurement.** ADR 0039 fixed it at
//! the three current Office formats *and* said each further kind is "a
//! registration and a test against a real file, in a later change". The three
//! OpenDocument formats are that, made against real files saved by the engine
//! the image already pins — no new engine, and nothing about the closed-set
//! rule relaxed. What the rule forbids is a conversion a request can name
//! without one, and [`Conversion::HELD_BACK`] is where those wait.
//!
//! **The copy is always a PDF** (ADR 0039 §1): it is the one kind whose
//! rendering does not depend on another engine interpreting the copy again.
//!
//! # Why a fourth is written here and is not in the set
//!
//! [`Conversion::HELD_BACK`] is a conversion whose every part is decided — what
//! it is from, what the engine is asked to export it with, what the document is
//! called in the scratch folder — and whose original has never been
//! inventoried, because the engine that reads it runs on an architecture this
//! repository is not gated on. ADR 0039 §4 makes the inventory the step before
//! any copy, and §1 forbids *this converts* followed by a failure. A machine
//! that offered this one would say it converts and then refuse at the
//! inventory, which is the exact shape both clauses forbid.
//!
//! So it is written and held back, and the holding is what
//! [`Conversion::EVERY`] not containing it means. The day somebody measures a
//! real one on a real engine, one line moves it into the set and
//! `crate::inventory::pages` stops refusing; nothing else here changes, which
//! is the point of writing it now.

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
    /// An OpenDocument text document into a PDF.
    OpenDocumentText,
    /// An OpenDocument spreadsheet into a PDF.
    OpenDocumentSpreadsheet,
    /// An OpenDocument presentation into a PDF.
    OpenDocumentPresentation,
    /// A Pages document into a PDF. Written, and in [`Conversion::HELD_BACK`]
    /// rather than [`Conversion::EVERY`], until one is inventoried.
    PagesDocument,
}

impl Conversion {
    /// Every conversion this machine makes, in one order.
    ///
    /// A conversion is in here when a real document of its kind has been
    /// inventoried; until then it is in [`Self::HELD_BACK`]. Everything that
    /// decides what this machine offers reads this and not the variants:
    /// `crate::machine` announces these, `Self::asked` answers to these, and a
    /// request naming any other word is not a request.
    pub const EVERY: [Self; 6] = [
        Self::WordDocument,
        Self::ExcelWorkbook,
        Self::PowerPointPresentation,
        Self::OpenDocumentText,
        Self::OpenDocumentSpreadsheet,
        Self::OpenDocumentPresentation,
    ];

    /// Every conversion that is written here and that this machine does not
    /// make, because no document of its kind has been inventoried.
    ///
    /// Empty is the finished state. A conversion leaves here by being measured,
    /// never by being decided to be fine.
    pub const HELD_BACK: [Self; 1] = [Self::PagesDocument];

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
            Kind::OpenDocumentText => Some(Self::OpenDocumentText),
            Kind::OpenDocumentSpreadsheet => Some(Self::OpenDocumentSpreadsheet),
            Kind::OpenDocumentPresentation => Some(Self::OpenDocumentPresentation),
            // A Pages document is recognised, and `Self::PagesDocument` is
            // written; naming it here is what would make a verb reach it, so
            // this stays the same answer as for any other kind until one is
            // inventoried.
            Kind::PagesDocument => None,
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
            Self::OpenDocumentText => Kind::OpenDocumentText,
            Self::OpenDocumentSpreadsheet => Kind::OpenDocumentSpreadsheet,
            Self::OpenDocumentPresentation => Kind::OpenDocumentPresentation,
            Self::PagesDocument => Kind::PagesDocument,
        }
    }

    /// What the copy is.
    #[must_use]
    pub const fn into(self) -> Kind {
        Kind::Pdf
    }

    /// The word a request names this conversion by on the service's socket.
    ///
    /// A held-back conversion has a word, and the socket does not answer to it:
    /// [`Self::asked`] reads [`Self::EVERY`], so the word is what a request
    /// *would* carry rather than one that gets a copy today.
    #[must_use]
    pub const fn asked_as(self) -> &'static str {
        match self {
            Self::WordDocument => "word-document",
            Self::ExcelWorkbook => "excel-workbook",
            Self::PowerPointPresentation => "powerpoint-presentation",
            Self::OpenDocumentText => "opendocument-text",
            Self::OpenDocumentSpreadsheet => "opendocument-spreadsheet",
            Self::OpenDocumentPresentation => "opendocument-presentation",
            Self::PagesDocument => "pages-document",
        }
    }

    /// The conversion a request's word names, and nothing for any other word —
    /// including the word of a conversion that is [`Self::HELD_BACK`].
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
            Self::OpenDocumentText => "document.odt",
            Self::OpenDocumentSpreadsheet => "document.ods",
            Self::OpenDocumentPresentation => "document.odp",
            Self::PagesDocument => "document.pages",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Three conversions, each from its own kind, each into a PDF**, and a
    /// word that names none of them names nothing.
    #[test]
    fn each_conversion_is_from_its_own_kind_and_no_other_kind_converts() {
        for conversion in Conversion::EVERY {
            assert_eq!(Conversion::of(conversion.from()), Some(conversion));
            assert_eq!(conversion.into(), Kind::Pdf);
            assert_eq!(Conversion::asked(conversion.asked_as()), Some(conversion));
        }
        // Every kind this machine can name, against the set: a kind converts
        // exactly when a conversion in `EVERY` is from it, and a kind that no
        // conversion is from converts not at all. Asked of the list rather than
        // of a property of `Kind`, so that adding a conversion cannot leave a
        // rule about which kinds convert behind in another crate.
        for kind in Kind::EVERY {
            let from_the_set = Conversion::EVERY
                .into_iter()
                .find(|conversion| conversion.from() == kind);
            assert_eq!(Conversion::of(kind), from_the_set, "{kind:?}");
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

    /// Every conversion there is, offered or held back.
    const EVERY_VARIANT: [Conversion; 7] = [
        Conversion::WordDocument,
        Conversion::ExcelWorkbook,
        Conversion::PowerPointPresentation,
        Conversion::OpenDocumentText,
        Conversion::OpenDocumentSpreadsheet,
        Conversion::OpenDocumentPresentation,
        Conversion::PagesDocument,
    ];

    /// **A conversion is offered or held back, never both and never neither.**
    ///
    /// The two assertions catch it from both sides: the first fails on a
    /// variant listed here and put in no list, the second on one put in a list
    /// and not listed here. The exhaustive matches in this file — `from`,
    /// `asked_as`, `scratch_name` — are what stop a new variant compiling until
    /// somebody decides what it is called and where its bytes go.
    #[test]
    fn every_conversion_is_offered_or_held_back_and_never_both() {
        for conversion in EVERY_VARIANT {
            assert!(
                Conversion::EVERY.contains(&conversion)
                    != Conversion::HELD_BACK.contains(&conversion),
                "{conversion:?} is in both lists or in neither"
            );
        }
        assert_eq!(
            EVERY_VARIANT.len(),
            Conversion::EVERY.len() + Conversion::HELD_BACK.len()
        );
    }

    /// **A held-back conversion is written whole and reachable by nothing.**
    ///
    /// It has a kind, a word and a scratch name, all its own — and neither road
    /// into a conversion goes near it: not the verb's, which asks
    /// [`Conversion::of`] what a document's kind converts as, and not the
    /// socket's, which asks [`Conversion::asked`] what a request's word names.
    #[test]
    fn a_held_back_conversion_is_written_and_reached_by_nothing() {
        for held in Conversion::HELD_BACK {
            assert_eq!(held.into(), Kind::Pdf);
            assert_eq!(
                Conversion::of(held.from()),
                None,
                "{held:?} is reachable from a document's kind"
            );
            assert_eq!(
                Conversion::asked(held.asked_as()),
                None,
                "{held:?} is reachable from a word on the socket"
            );
            for offered in Conversion::EVERY {
                assert_ne!(held.from(), offered.from());
                assert_ne!(held.asked_as(), offered.asked_as());
                assert_ne!(held.scratch_name(), offered.scratch_name());
            }
        }
    }

    /// **The Pages document is the held-back one**, from a Pages document, and
    /// its scratch name ends in what tells the engine to read it as one.
    #[test]
    fn the_pages_document_is_written_from_a_pages_document() {
        assert_eq!(Conversion::HELD_BACK, [Conversion::PagesDocument]);
        assert_eq!(Conversion::PagesDocument.from(), Kind::PagesDocument);
        assert_eq!(Conversion::PagesDocument.asked_as(), "pages-document");
        assert_eq!(Conversion::PagesDocument.scratch_name(), "document.pages");
    }
}
