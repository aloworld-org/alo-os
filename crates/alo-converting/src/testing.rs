//! What this crate's own tests are written against.
//!
//! The strings a person would read, the owner's three real documents, and the
//! zips `tests/making/` builds — one builder for the unit tests and the
//! integration tests alike, so two fixtures cannot pass two rules.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_strings::{Strings, Vocabulary};

use crate::zip::Zipped;

/// The zips the tests are shown.
#[path = "../tests/making/mod.rs"]
mod making;

pub(crate) use making::a_zip;

/// The strings in English, with the words of every crate a sentence here
/// borrows from.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::words::declare_into(&mut vocabulary).unwrap();
    alo_opening::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// One of the owner's three documents, as its bytes.
pub(crate) fn the_document(named: &str) -> Vec<u8> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("documents")
        .join(named);
    std::fs::read(&at).unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// One part of a document, decompressed.
pub(crate) fn a_part(document: &[u8], named: &str) -> Vec<u8> {
    Zipped::of(document).unwrap().read(named).unwrap().unwrap()
}
