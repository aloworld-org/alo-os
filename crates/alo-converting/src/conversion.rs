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
//! # The seventh, and why [`Conversion::HELD_BACK`] is empty
//!
//! A Pages document was written here whole and kept out of the set, because
//! ADR 0039 §4 makes the inventory of the original the step before any copy and
//! nobody had inventoried one: the engine that reads a Pages document is an
//! x86_64 build and this plan gates on aarch64. It is in the set now, on the
//! measurement of 2026-09-21 recorded in `inventory::pages`, taken on a machine
//! with the engine against the one real Pages document this repository holds.
//!
//! **It is the first conversion whose original this crate does not read for
//! itself**, and `inventory::read_from` is where that is said rather than here:
//! what a conversion *is* and where its inventory comes *from* are two
//! questions, and this file answers the first.
//!
//! [`Conversion::HELD_BACK`] stays, empty, because empty is the finished state
//! and the tests holding a held-back conversion unreachable are what the next
//! one will need. A conversion leaves it by being measured, never by being
//! decided to be fine.

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
    /// A Pages document into a PDF.
    PagesDocument,
    /// A Word document saved in the format before 2007, into a PDF.
    OlderWordDocument,
    /// An Excel workbook saved in the format before 2007, into a PDF.
    OlderExcelWorkbook,
    /// A PowerPoint presentation saved in the format before 2007, into a PDF.
    OlderPowerPointPresentation,
}

impl Conversion {
    /// Every conversion this machine makes, in one order.
    ///
    /// A conversion is in here when a real document of its kind has been
    /// inventoried; until then it is in [`Self::HELD_BACK`]. Everything that
    /// decides what this machine offers reads this and not the variants:
    /// `crate::machine` announces these, `Self::asked` answers to these, and a
    /// request naming any other word is not a request.
    pub const EVERY: [Self; 10] = [
        Self::WordDocument,
        Self::ExcelWorkbook,
        Self::PowerPointPresentation,
        Self::OpenDocumentText,
        Self::OpenDocumentSpreadsheet,
        Self::OpenDocumentPresentation,
        Self::PagesDocument,
        Self::OlderWordDocument,
        Self::OlderExcelWorkbook,
        Self::OlderPowerPointPresentation,
    ];

    /// Every conversion that is written here and that this machine does not
    /// make, because no document of its kind has been inventoried.
    ///
    /// Empty is the finished state. A conversion leaves here by being measured,
    /// never by being decided to be fine.
    pub const HELD_BACK: [Self; 0] = [];

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
            Kind::PagesDocument => Some(Self::PagesDocument),
            Kind::OlderWordDocument => Some(Self::OlderWordDocument),
            Kind::OlderExcelWorkbook => Some(Self::OlderExcelWorkbook),
            Kind::OlderPowerPointPresentation => Some(Self::OlderPowerPointPresentation),
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
            Self::OlderWordDocument => Kind::OlderWordDocument,
            Self::OlderExcelWorkbook => Kind::OlderExcelWorkbook,
            Self::OlderPowerPointPresentation => Kind::OlderPowerPointPresentation,
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
            Self::OlderWordDocument => "older-word-document",
            Self::OlderExcelWorkbook => "older-excel-workbook",
            Self::OlderPowerPointPresentation => "older-powerpoint-presentation",
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
            // The ending is what tells the engine which reader opens the file,
            // and these three are read by a different reader from the 2007
            // formats above even though they come out of the same writer.
            Self::OlderWordDocument => "document.doc",
            Self::OlderExcelWorkbook => "document.xls",
            Self::OlderPowerPointPresentation => "document.ppt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every conversion is from its own kind and into a PDF**, and a word
    /// that names none of them names nothing.
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
    const EVERY_VARIANT: [Conversion; 10] = [
        Conversion::WordDocument,
        Conversion::ExcelWorkbook,
        Conversion::PowerPointPresentation,
        Conversion::OpenDocumentText,
        Conversion::OpenDocumentSpreadsheet,
        Conversion::OpenDocumentPresentation,
        Conversion::PagesDocument,
        Conversion::OlderWordDocument,
        Conversion::OlderExcelWorkbook,
        Conversion::OlderPowerPointPresentation,
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
    ///
    /// **[`Conversion::HELD_BACK`] is empty today**, so this walks nothing and
    /// is kept for the next conversion that waits on a measurement rather than
    /// deleted with the one that stopped waiting. Deleting it would mean the
    /// next worker writing the holding rule again from the report of the one
    /// who wrote it first, which is how a rule comes back weaker.
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

    /// **Nothing is held back**, which is the finished state — and the Pages
    /// document, which was the one that had been, is a conversion this machine
    /// makes, from a Pages document, under the scratch name whose ending tells
    /// the engine to read it as one.
    ///
    /// The empty list is asserted rather than left to be noticed: a conversion
    /// slipped back into [`Conversion::HELD_BACK`] by a later change is
    /// something a person is not offered, and the test that says so should be
    /// this one rather than a surprise at the far end of the machine.
    #[test]
    fn nothing_is_held_back_and_the_pages_document_is_one_this_machine_makes() {
        assert_eq!(Conversion::HELD_BACK, []);
        assert!(Conversion::EVERY.contains(&Conversion::PagesDocument));
        assert_eq!(
            Conversion::of(Kind::PagesDocument),
            Some(Conversion::PagesDocument)
        );
        assert_eq!(Conversion::PagesDocument.from(), Kind::PagesDocument);
        assert_eq!(Conversion::PagesDocument.into(), Kind::Pdf);
        assert_eq!(Conversion::PagesDocument.asked_as(), "pages-document");
        assert_eq!(Conversion::PagesDocument.scratch_name(), "document.pages");
        assert_eq!(
            Conversion::asked("pages-document"),
            Some(Conversion::PagesDocument)
        );
    }
}
