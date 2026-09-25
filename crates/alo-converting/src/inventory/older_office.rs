//! The three formats Office saved in before 2007, and why nothing here reads
//! their bytes.
//!
//! A `.docx` is a zip: `crate::inventory::word` opens it, walks its parts and
//! reads what the document says about its own fonts, fields, links and
//! comments. A `.doc` is not. It is an **OLE2 compound file** — a little
//! filesystem of streams inside one file, written by Word in the nineteen
//! nineties, holding its text in a format with no schema anybody publishes a
//! reader for.
//!
//! # So this crate does not write one
//!
//! ADR 0011: engines are configured, never written in. A reader of our own for
//! a format that old would be a second implementation of somebody else's
//! undocumented file layout, kept correct by us, standing between a person and
//! a document they can already open. The engine ADR 0039 pins reads all three
//! and has done for twenty years.
//!
//! These conversions therefore take the same road the Pages document takes
//! (`crate::inventory::pages`): the service asks the engine to render the
//! original into the kind this crate *does* read, inventories that rendering,
//! and makes the copy from the **original** so nobody's PDF is a conversion of
//! a conversion.
//!
//! # What that costs, said plainly
//!
//! Whatever the engine's own reader does not carry out of the original reaches
//! neither the rendering nor the copy, so the difference between them cannot
//! name it. `crate::inventory::read_from` says why that is accepted here: the
//! alternative is not a better inventory, it is no conversion at all.

use crate::conversion::Conversion;

/// What each of these formats is called where a refusal names it.
#[must_use]
pub const fn the_format(conversion: Conversion) -> &'static str {
    match conversion {
        Conversion::OlderExcelWorkbook => "Excel workbook saved before 2007",
        Conversion::OlderPowerPointPresentation => "PowerPoint presentation saved before 2007",
        _ => "Word document saved before 2007",
    }
}

/// What the engine is asked to render one as, so that it can be inventoried.
///
/// Each into the OpenDocument kind of its own shape — prose as prose, a
/// spreadsheet as a spreadsheet, slides as slides — because the engine opens
/// each with its own reader and a spreadsheet rendered as prose would lose the
/// columns before anything counted them.
#[must_use]
pub const fn rendered_as(conversion: Conversion) -> Conversion {
    match conversion {
        Conversion::OlderExcelWorkbook => Conversion::OpenDocumentSpreadsheet,
        Conversion::OlderPowerPointPresentation => Conversion::OpenDocumentPresentation,
        _ => Conversion::OpenDocumentText,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::read_from::{ReadFrom, read_from};

    /// The three, for a test to walk.
    const THE_THREE: [Conversion; 3] = [
        Conversion::OlderWordDocument,
        Conversion::OlderExcelWorkbook,
        Conversion::OlderPowerPointPresentation,
    ];

    /// **None of the three is inventoried from its own bytes**, and each is
    /// rendered as the OpenDocument kind of its own shape.
    ///
    /// A spreadsheet rendered as prose would be a copy whose losses were
    /// counted against a document with no columns in it.
    #[test]
    fn each_is_rendered_as_its_own_shape_and_never_read_directly() {
        for conversion in THE_THREE {
            assert_eq!(
                read_from(conversion),
                ReadFrom::WhatTheEngineReadsOfIt(rendered_as(conversion)),
                "{conversion:?} is not inventoried out of a rendering"
            );
            // Prose renders as prose, a spreadsheet as a spreadsheet and
            // slides as slides. `crate::engine` decides which writer each
            // shape uses and holds that itself; what this asserts is that the
            // rendering asked for is the OpenDocument of the same shape, which
            // is the half this file decides.
            assert_eq!(
                rendered_as(conversion),
                match conversion {
                    Conversion::OlderExcelWorkbook => Conversion::OpenDocumentSpreadsheet,
                    Conversion::OlderPowerPointPresentation => Conversion::OpenDocumentPresentation,
                    _ => Conversion::OpenDocumentText,
                },
                "{conversion:?} is rendered as a different shape of document"
            );
        }
    }

    /// **Each has a name of its own where a refusal says which format it was.**
    #[test]
    fn no_two_of_the_three_are_named_the_same() {
        for (one, first) in THE_THREE.iter().enumerate() {
            for second in THE_THREE.iter().skip(one + 1) {
                assert_ne!(the_format(*first), the_format(*second));
            }
        }
    }
}
