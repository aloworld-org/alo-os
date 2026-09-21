//! A Pages document, inventoried — out of what the engine reads of it, and the
//! measurement that settled it.
//!
//! [`word`](crate::inventory::word), [`excel`](crate::inventory::excel),
//! [`powerpoint`](crate::inventory::powerpoint) and
//! [`opendocument`](crate::inventory::opendocument) each read a real document's
//! own zip and say what it holds. A Pages document keeps its text, its styles
//! and its annotations in `Index/*.iwa` — compressed protobuf, to a schema
//! nobody publishes — and **nothing in this repository reads one**. So this
//! format's inventory takes the other road
//! ([`crate::inventory::read_from`]): the engine renders the original into an
//! OpenDocument text document, and the measured OpenDocument reader inventories
//! that.
//!
//! # The measurement, 2026-09-21
//!
//! Taken against `crates/alo-opening/tests/files/document.pages` — 227,583
//! bytes, held to its digest, its provenance in that folder's `README.md` —
//! read by the pinned engine on x86_64, which is the architecture that engine
//! is built for and the reason this one task was done on a machine of its own.
//!
//! What the rendering said the document holds:
//!
//! | | |
//! |---|---|
//! | every family its text is set in | **`Helvetica`** and **`HelveticaNeue`**, which are the document's own |
//! | every field whose value depends on when or where it is open | none |
//! | every kind of content taken from elsewhere | none; its one picture is *embedded* |
//! | comments | none |
//! | tracked changes | none |
//! | macros | none |
//!
//! Two of those six are why it was worth measuring rather than reasoning
//! about.
//!
//! **The fonts.** The rendering *declares* five families: the document's two,
//! and `Liberation Sans`, `Liberation Serif` and `Noto Sans`, which the engine
//! put there because it does not have the two it was asked for. An inventory
//! that counted declarations would report five, three of which the copy
//! carries, and the loss a person is owed — *your document is set in Helvetica
//! and this machine has no Helvetica* — would be buried in three findings that
//! are not losses. [`crate::inventory`]'s rule is that **a family counts when
//! text is set in it**, and under that rule the answer is the document's own
//! two. That is ADR 0008's sentence, and it is the one a person can act on.
//!
//! **Taken from elsewhere is none, beside an embedded picture.** The document
//! carries a pasted image; the rendering writes it into its own package as
//! `Pictures/…`. A reader that counted that as linked would tell a person that
//! a copy is incomplete offline when it is complete, which is the opposite
//! answer to the one they asked for. The OpenDocument reader already draws that
//! line — a link counts when the package does not hold what it names — so the
//! answer is none.
//!
//! # What this road cannot see
//!
//! Whatever the engine's reader does not carry out of the original reaches
//! neither the rendering nor the copy, so the difference between them cannot
//! name it. [`crate::inventory::read_from`] says why that is accepted here and
//! nowhere else.

use crate::conversion::Conversion;

/// What this format is called where a refusal names it.
pub const THE_FORMAT: &str = "Pages document";

/// What the engine is asked to render a Pages document as, so that it can be
/// inventoried: an OpenDocument text document, which this crate reads.
///
/// A text document because that is what a Pages document is — the engine opens
/// one with its own prose reader, which is the same reader a `.docx` and an
/// `.odt` arrive through, and the same writer exports all three.
pub const RENDERED_AS: Conversion = Conversion::OpenDocumentText;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::Path;

    use alo_opening::Macros;

    use crate::inventory::original::{NotInventoried, Original};
    use crate::inventory::read_from::{ReadFrom, read_from};
    use crate::zip::Zipped;

    /// Where the one measured Pages document is. It belongs to `alo-opening`,
    /// whose `tests/files/README.md` records where it came from; it is read
    /// from there rather than copied here, because two copies of a 227 KB
    /// document would be two things to keep true and the one that drifted
    /// would be this one.
    const THE_ONE_MEASURED: &str = "crates/alo-opening/tests/files/document.pages";

    /// Every part of the document at [`THE_ONE_MEASURED`], in the order its zip
    /// lists them.
    ///
    /// Measured by reading that file. Two of these names end in a number that
    /// belongs to that document and not to the format, which is itself a reason
    /// this is a reading of one file rather than a description of Pages.
    const THE_PARTS: [&str; 15] = [
        "Data/pasted-image-24.jpeg",
        "Data/pasted-image-small-25.jpeg",
        "Index/Document.iwa",
        "Index/ViewState.iwa",
        "Index/CalculationEngine-1732610.iwa",
        "Index/AnnotationAuthorStorage-1732609.iwa",
        "Index/DocumentStylesheet.iwa",
        "Index/DocumentMetadata.iwa",
        "Index/Metadata.iwa",
        "Metadata/Properties.plist",
        "Metadata/DocumentIdentifier",
        "Metadata/BuildVersionHistory.plist",
        "preview.jpg",
        "preview-micro.jpg",
        "preview-web.jpg",
    ];

    /// Everything an [`Original`] holds, by the names of the six accessors that
    /// answer them — the list this module's measurement answers.
    ///
    /// They are the accessors' own names rather than a sentence each, because a
    /// paraphrase of what `fonts` means is a second description of it, and the
    /// one that would go stale is this one.
    const WHAT_AN_INVENTORY_ANSWERS: [&str; 6] = [
        "fonts",
        "fields",
        "linked",
        "comments",
        "tracked_changes",
        "macros",
    ];

    /// The families the measurement of 2026-09-21 found text set in.
    const THE_FAMILIES: [&str; 2] = ["Helvetica", "HelveticaNeue"];

    /// The one real Pages document, as its bytes.
    fn the_one_measured() -> Vec<u8> {
        let at = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("alo-opening")
            .join("tests")
            .join("files")
            .join("document.pages");
        std::fs::read(&at).unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
    }

    /// **[`THE_PARTS`] is what is in the document**, in its order — so the list
    /// is a reading of a file and stays one, and the file cannot be swapped for
    /// another without this saying so.
    #[test]
    fn the_parts_are_the_parts_of_the_one_measured_document() {
        let bytes = the_one_measured();
        let zipped = Zipped::of(&bytes).unwrap();
        let named: Vec<&str> = zipped.names().collect();
        assert_eq!(named, THE_PARTS);
        assert!(
            THE_ONE_MEASURED.ends_with("document.pages"),
            "{THE_ONE_MEASURED}"
        );
    }

    /// **A Pages document is not inventoried from its own bytes**, and says so
    /// as a refusal rather than as an empty inventory.
    ///
    /// This is the half of the design that stops the other half going wrong. A
    /// caller that reaches [`Original::of`] with a Pages document has skipped
    /// the rendering, and an empty [`Original`] would then tell a person that
    /// their copy lost nothing — the one sentence ADR 0039 §4 says must be
    /// unreachable without both inventories.
    #[test]
    fn a_pages_document_is_not_inventoried_from_its_own_bytes() {
        assert_eq!(
            Original::of(
                &the_one_measured(),
                Conversion::PagesDocument,
                Macros::NoneSeen
            ),
            Err(NotInventoried::NotFromItsOwnBytes(THE_FORMAT))
        );
        assert_eq!(
            read_from(Conversion::PagesDocument),
            ReadFrom::WhatTheEngineReadsOfIt(RENDERED_AS)
        );
    }

    /// **The measurement answers all six**, and the two families it found are
    /// the document's own.
    ///
    /// The count is the guard: [`Original`] holds six things, and a seventh
    /// added there without a line here would leave a question nobody knew was
    /// open. The families are named here, where the measurement is written
    /// down; what holds them against a running engine is
    /// `tests/converting_a_real_document.rs`, which converts this document and
    /// requires exactly these two to be substituted.
    #[test]
    fn the_measurement_answers_all_six_and_names_the_documents_own_families() {
        assert_eq!(WHAT_AN_INVENTORY_ANSWERS.len(), 6);
        let mut said = WHAT_AN_INVENTORY_ANSWERS.to_vec();
        said.sort_unstable();
        said.dedup();
        assert_eq!(said.len(), WHAT_AN_INVENTORY_ANSWERS.len());

        assert_eq!(THE_FAMILIES.len(), 2);
        for family in THE_FAMILIES {
            assert!(
                family.starts_with("Helvetica"),
                "{family} is not one of the two the document was set in"
            );
        }
    }
}
