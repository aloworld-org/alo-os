//! A Pages document, inventoried — the shape of it, with none of it measured.
//!
//! [`word`](crate::inventory::word), [`excel`](crate::inventory::excel) and
//! [`powerpoint`](crate::inventory::powerpoint) each read a real document and
//! say what it holds. **This one reads nothing and refuses**, and the refusal is
//! the honest answer rather than a placeholder: what a Pages document holds has
//! never been read on this machine, because the engine that reads one is built
//! for an architecture this repository is not gated on, and the only way to
//! know what is in [`THE_PARTS`] is to run it.
//!
//! # Why it is written before it is measured
//!
//! Everything around an inventory can be decided without one: which kind it is
//! from, what the engine is asked to export it with, what the document is
//! called in the scratch folder, and that the service does not offer it. Those
//! are in [`crate::conversion`] and [`crate::engine`], and they are settled.
//! What cannot be decided without one is *this document sets text in these
//! families and carries these fields*, and a crate that guessed those would
//! report losses that are not there and miss ones that are — which is the same
//! lie [`crate::inventory`] already refuses to tell in the other direction.
//!
//! So the shape is here, [`Measured`] says nobody has filled it in, and
//! [`inventory`] refuses. `a_pages_document_is_not_inventoried_yet` fails on the
//! day somebody does, which is the reminder to move
//! [`Conversion::PagesDocument`](crate::Conversion::PagesDocument) out of
//! [`Conversion::HELD_BACK`](crate::Conversion::HELD_BACK) in the same change.
//!
//! # What is measured, and it is only this
//!
//! [`THE_PARTS`] is the list of parts in the one real Pages document this
//! repository holds, read off that file. It is not a list of what a Pages
//! document has in general — two of its names carry a number that belongs to
//! that document — and **a part's name is not a reading of it**: that
//! `Index/DocumentStylesheet.iwa` is where the families are is a guess until
//! somebody opens it, and nothing here claims it.

// The shape is not read by anything that runs, because nothing here inventories
// a Pages document; what reads it is this file's own tests, and the person who
// fills `inventory` in. So it is dead code when the tests are not compiled and
// live code when they are, which is why the expectation is conditional — an
// unconditional one is unfulfilled in the test build and fails the clippy gate.
//
// An expectation rather than an allowance, because of what it does when the
// work is finished: the day the code that runs uses the last of these, this
// stops being fulfilled and becomes the reminder that the file is no longer a
// shape.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the shape of an inventory nobody has taken; every item here is used by \
                  the change that takes it"
    )
)]

use crate::inventory::original::{NotInventoried, Original};
use crate::zip::Zipped;

/// What this format is called where a refusal names it.
pub const THE_FORMAT: &str = "Pages document";

/// Where the one measured Pages document is, and where its provenance is
/// written down.
pub const THE_ONE_MEASURED: &str = "crates/alo-opening/tests/files/document.pages";

/// Every part of the document at [`THE_ONE_MEASURED`], in the order its zip
/// lists them.
///
/// Measured by reading that file. Two of these names end in a number that
/// belongs to that document and not to the format, which is itself a reason
/// this is a reading of one file rather than a description of Pages.
pub const THE_PARTS: [&str; 15] = [
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

/// Everything an [`Original`] holds, which an inventory of a Pages document
/// would have to answer out of [`THE_PARTS`].
///
/// The list is here so that whoever measures one knows when they are finished:
/// an inventory that answered five of these and left the sixth would be a
/// partial [`Original`], which [`crate::inventory::original`] does not have.
pub const WHAT_AN_INVENTORY_ANSWERS: [&str; 6] = [
    "every family its text is set in",
    "every field whose value depends on when or where it is open",
    "every kind of content taken from elsewhere",
    "whether it has comments",
    "whether it has tracked changes",
    "whether it carries macros",
];

/// How much of [`WHAT_AN_INVENTORY_ANSWERS`] a Pages document has answered.
///
/// One value, the way `alo_playing::right::SoftwareDecoders` has one: a
/// question held open is not a `bool` with a default, because a default is an
/// answer somebody would eventually read as measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measured {
    /// None of it. No Pages document has been opened by the engine on a machine
    /// this repository gates on, so nothing about one is known from reading it.
    NothingOnThisMachine,
}

/// What has been measured of a Pages document's contents.
#[must_use]
pub const fn measured() -> Measured {
    Measured::NothingOnThisMachine
}

/// Inventory a Pages document — which nothing here can do.
///
/// The arguments are the ones the other three formats' files take, so that
/// filling this in is writing a body rather than changing a shape.
///
/// # Errors
/// Always [`NotInventoried::NotMeasured`], and then nothing about the document
/// is claimed — which is what every other refusal in this module means too.
pub fn inventory(_zipped: &mut Zipped, _original: &mut Original) -> Result<(), NotInventoried> {
    Err(NotInventoried::NotMeasured(THE_FORMAT))
}

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

    use crate::conversion::Conversion;

    /// The one real Pages document, as its bytes.
    ///
    /// It belongs to `alo-opening`, whose `tests/files/README.md` records where
    /// it came from. It is read from there rather than copied here: two copies
    /// of a 227 KB document would be two things to keep true, and the one that
    /// drifted would be this one.
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
    /// is a reading of a file and stays one.
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

    /// **A real Pages document is not inventoried**, and says so as a refusal
    /// rather than as an empty inventory.
    ///
    /// **This test fails the day somebody measures one**, and that is what it
    /// is for: whoever runs the engine over this document and writes down what
    /// it holds deletes this, fills in [`inventory`], and moves
    /// `Conversion::PagesDocument` into `Conversion::EVERY` — three parts of
    /// one change, with this as the thing that will not let them ship one
    /// without the others.
    #[test]
    fn a_pages_document_is_not_inventoried_yet() {
        assert_eq!(measured(), Measured::NothingOnThisMachine);
        assert_eq!(
            Original::of(
                &the_one_measured(),
                Conversion::PagesDocument,
                Macros::NoneSeen
            ),
            Err(NotInventoried::NotMeasured(THE_FORMAT))
        );
    }

    /// **An inventory that has answered nothing has answered nothing about
    /// every one of the six**, rather than defaulting five of them.
    ///
    /// The count is the guard: [`Original`] holds six things, and a seventh
    /// added there without a line here would leave a question nobody knew was
    /// open.
    #[test]
    fn nothing_is_answered_rather_than_most_of_it() {
        assert_eq!(WHAT_AN_INVENTORY_ANSWERS.len(), 6);
        let mut said = WHAT_AN_INVENTORY_ANSWERS.to_vec();
        said.sort_unstable();
        said.dedup();
        assert_eq!(said.len(), WHAT_AN_INVENTORY_ANSWERS.len());
    }
}
