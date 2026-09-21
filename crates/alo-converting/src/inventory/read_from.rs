//! Which bytes an inventory of an original reads.
//!
//! ADR 0039 §4 says the original is inventoried before conversion, **from the
//! documents and never from the engine's log**. For six of the seven
//! conversions that is the document itself: `crate::inventory::word`,
//! `excel`, `powerpoint` and `opendocument` each read the original's own zip.
//!
//! The seventh cannot be. A Pages document keeps its text, its styles and its
//! annotations in `Index/*.iwa` — compressed protobuf written to a schema
//! Apple publishes nowhere — and nothing in this repository reads one. So the
//! inventory of a Pages document is read from **what the engine reads of it**:
//! the engine renders the original into an OpenDocument text document, and
//! `crate::inventory::opendocument` — which is measured, against four real
//! files — inventories that.
//!
//! # Why that is a document and not a log
//!
//! A log is what the engine chose to mention. What this reads is a document
//! the engine wrote out of the original's own content: the families its text
//! is set in, the fields in it, what it links, its comments, its tracked
//! changes. Nothing is taken on the engine's word about what it did.
//!
//! # And what it cannot see, said plainly
//!
//! Whatever the engine's reader does not carry out of the original is in
//! neither the rendering nor the copy, so the difference between them cannot
//! report it. That is a real limit and it is written here rather than
//! discovered: it is the reason this road is taken only for a format nothing
//! here reads, and never as a shortcut past a reader somebody could write.
//!
//! It is also why the rendering is a separate run of the engine and never the
//! copy itself. The copy is made from the original, so a person's PDF is not a
//! conversion of a conversion.

use crate::conversion::Conversion;
use crate::inventory::pages;

/// Which bytes an inventory of an original of this conversion reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadFrom {
    /// The original's own bytes, by a reader in this crate.
    ItsOwnBytes,
    /// What the engine reads of the original, rendered first into the kind
    /// this conversion is from — because nothing here reads the original's own
    /// format.
    WhatTheEngineReadsOfIt(Conversion),
}

/// Which bytes an inventory of an original of this conversion reads.
#[must_use]
pub const fn read_from(conversion: Conversion) -> ReadFrom {
    match conversion {
        Conversion::WordDocument
        | Conversion::ExcelWorkbook
        | Conversion::PowerPointPresentation
        | Conversion::OpenDocumentText
        | Conversion::OpenDocumentSpreadsheet
        | Conversion::OpenDocumentPresentation => ReadFrom::ItsOwnBytes,
        Conversion::PagesDocument => ReadFrom::WhatTheEngineReadsOfIt(pages::RENDERED_AS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Six conversions are inventoried from the document, and the seventh is
    /// the one whose format nothing here reads.**
    ///
    /// Counted rather than listed, so a conversion added without a line in
    /// [`read_from`] cannot pass by resembling one that is already there — the
    /// match is exhaustive, so it would not compile, and this says what the
    /// answer should have been.
    #[test]
    fn only_a_pages_document_is_inventoried_from_what_the_engine_reads() {
        let from_the_document = Conversion::EVERY
            .into_iter()
            .filter(|conversion| read_from(*conversion) == ReadFrom::ItsOwnBytes)
            .count();
        assert_eq!(from_the_document, Conversion::EVERY.len() - 1);
        assert_eq!(
            read_from(Conversion::PagesDocument),
            ReadFrom::WhatTheEngineReadsOfIt(Conversion::OpenDocumentText)
        );
    }

    /// **Whatever a rendering is read as, this crate reads that kind from its
    /// own bytes** — so a rendering is never itself rendered again, and the
    /// inventory is one step and not a chain.
    #[test]
    fn a_rendering_is_a_kind_this_crate_reads_for_itself() {
        for conversion in Conversion::EVERY {
            if let ReadFrom::WhatTheEngineReadsOfIt(as_if) = read_from(conversion) {
                assert_ne!(as_if, conversion, "{conversion:?} is rendered as itself");
                assert_eq!(
                    read_from(as_if),
                    ReadFrom::ItsOwnBytes,
                    "{conversion:?} is rendered as {as_if:?}, which is not read here either"
                );
            }
        }
    }

    /// **The engine is asked for one rendering and it is the text one.**
    ///
    /// `crate::engine` has a single argument list for a rendering, and it
    /// writes the engine's own text format. A conversion rendered as a
    /// spreadsheet or as slides would come out of another writer, so the day
    /// one arrives it is a value in `crate::engine::Export` and a line here,
    /// and this is what fails until both exist.
    #[test]
    fn every_rendering_the_engine_is_asked_for_is_a_text_document() {
        for conversion in Conversion::EVERY {
            if let ReadFrom::WhatTheEngineReadsOfIt(as_if) = read_from(conversion) {
                assert_eq!(
                    as_if,
                    Conversion::OpenDocumentText,
                    "{conversion:?} asks the engine for a rendering it has no argument list for"
                );
            }
        }
    }
}
