//! **A document saved by Pages, recognised from its bytes** — against a real
//! one, with its provenance in `tests/files/README.md`.
//!
//! Task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, and the second of
//! its three formats. `docs/features.md`: ★ *"I can't open this file." A
//! `.pages`, a `.heic`, a `.dwg`: the system converts it where it can, and where
//! it cannot says plainly what will open it, instead of shrugging.*
//!
//! # A Pages document is a zip, which is the whole difficulty
//!
//! So are a Word document, a spreadsheet, a presentation, their OpenDocument
//! equivalents and every plain archive anybody was ever sent. **`PK\x03\x04`
//! says nothing.** What separates them is which parts they hold, and this file
//! holds the rule to that in both directions: the real document is recognised,
//! and the real Office documents in `alo-converting/tests/documents/` — which
//! are zips too — are still themselves.
//!
//! # And so are Keynote and Numbers
//!
//! All three iWork applications write the same container. `Index/Document.iwa`
//! means *one of these three* and not which, so the rule also requires the
//! absence of the parts the other two have. **That half is reasoned and not
//! measured**: this team has no Keynote or Numbers document to check it
//! against, and `tests/files/README.md` records the ask. What is tested here is
//! that the exclusion does what it says — a container carrying those parts is
//! refused rather than called a Pages document — which is the logic, not the
//! format.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::io::{Cursor, Write as _};
use std::path::{Path, PathBuf};

use alo_opening::{Appears, Cannot, Decided, Kind, Outcome, ThisMachine, Would, decide};
use sha2::{Digest as _, Sha256};

/// The real document, and where it came from.
const THE_DOCUMENT: &str = "document.pages";

/// The digest its provenance records.
const ITS_DIGEST: &str = "1b01189904934a7c5d6b59199c9719eab111759cfea5e2a7de17d3e45ee09c4d";

/// What it weighs.
const ITS_SIZE: usize = 227_583;

/// Its provenance, which is evidence rather than a note beside it.
const THE_PROVENANCE: &str = "README.md";

/// Where both live.
fn the_files() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("files")
}

/// One of them, read.
fn the_file(named: &str) -> Vec<u8> {
    fs::read(the_files().join(named)).expect("a file this crate is measured against")
}

/// One of the real Office documents the converting crate is measured against.
///
/// Borrowed rather than copied: they are real files with their own provenance,
/// and a second copy of them here would be a second thing to keep true.
fn an_office_document(named: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crates/")
            .join("alo-converting")
            .join("tests")
            .join("documents")
            .join(named),
    )
    .expect("a real Office document")
}

/// What this machine would do with these bytes, its name claiming nothing.
///
/// The name is empty on purpose: a name that agreed with the bytes would let a
/// rule pass by reading the extension, which is what this task forbids.
fn outcome_of(bytes: &[u8], machine: &ThisMachine) -> Outcome {
    match decide(&mut Cursor::new(bytes), OsStr::new(""), machine)
        .expect("bytes in memory are readable")
    {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    }
}

/// What this machine says these bytes are.
fn appears(bytes: &[u8]) -> Appears {
    outcome_of(bytes, &ThisMachine::with_nothing()).appears()
}

/// A zip holding exactly these names, written to real bytes.
///
/// For the one thing no real file here can show: that the exclusion of Keynote
/// and Numbers does what it says. The entries are empty because the rule reads
/// the list of names and never a part's contents.
fn a_zip_holding(names: &[&str]) -> Vec<u8> {
    let mut entries = Vec::new();
    let mut directory = Vec::new();
    let mut at: u32 = 0;
    for name in names {
        let name = name.as_bytes();
        let mut local = Vec::new();
        local.extend_from_slice(b"PK\x03\x04\x14\0\0\0\0\0\0\0\0\0");
        local.extend_from_slice(&0u32.to_le_bytes()); // crc
        local.extend_from_slice(&0u32.to_le_bytes()); // compressed
        local.extend_from_slice(&0u32.to_le_bytes()); // uncompressed
        local.extend_from_slice(&(name.len() as u16).to_le_bytes());
        local.extend_from_slice(&0u16.to_le_bytes());
        local.extend_from_slice(name);

        let mut entry = Vec::new();
        entry.extend_from_slice(b"PK\x01\x02\x14\0\x14\0\0\0\0\0\0\0\0\0");
        entry.extend_from_slice(&0u32.to_le_bytes()); // crc
        entry.extend_from_slice(&0u32.to_le_bytes()); // compressed
        entry.extend_from_slice(&0u32.to_le_bytes()); // uncompressed
        entry.extend_from_slice(&(name.len() as u16).to_le_bytes());
        entry.extend_from_slice(&0u16.to_le_bytes()); // extra
        entry.extend_from_slice(&0u16.to_le_bytes()); // comment
        entry.extend_from_slice(&0u16.to_le_bytes()); // disk
        entry.extend_from_slice(&0u16.to_le_bytes()); // internal
        entry.extend_from_slice(&0u32.to_le_bytes()); // external
        entry.extend_from_slice(&at.to_le_bytes()); // where the local header is
        entry.extend_from_slice(name);
        directory.push(entry);

        at += u32::try_from(local.len()).expect("a small fixture");
        entries.write_all(&local).expect("writing to a vector");
    }

    let directory_at = at;
    let mut whole = entries;
    let mut directory_bytes = Vec::new();
    for entry in &directory {
        directory_bytes.extend_from_slice(entry);
    }
    whole.extend_from_slice(&directory_bytes);

    let mut end = Vec::new();
    end.extend_from_slice(b"PK\x05\x06\0\0\0\0");
    end.extend_from_slice(&(names.len() as u16).to_le_bytes());
    end.extend_from_slice(&(names.len() as u16).to_le_bytes());
    end.extend_from_slice(&(directory_bytes.len() as u32).to_le_bytes());
    end.extend_from_slice(&directory_at.to_le_bytes());
    end.extend_from_slice(&0u16.to_le_bytes());
    whole.extend_from_slice(&end);
    whole
}

/// **The file is the one its provenance describes.**
///
/// Its digest, so that swapping the file fails here rather than quietly
/// changing what every other test in this file is about.
#[test]
fn the_document_is_the_one_its_provenance_records() {
    let bytes = the_file(THE_DOCUMENT);
    assert_eq!(bytes.len(), ITS_SIZE, "the file is not the one recorded");
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest, ITS_DIGEST,
        "the file's digest is not the one recorded"
    );

    let provenance =
        fs::read_to_string(the_files().join(THE_PROVENANCE)).expect("the provenance beside it");
    assert!(
        provenance.contains(THE_DOCUMENT),
        "the provenance does not mention the file it is for"
    );
    assert!(
        provenance.contains(ITS_DIGEST),
        "the provenance does not record this file's digest"
    );
}

/// **A real Pages document is a Pages document.**
#[test]
fn a_real_pages_document_is_recognised() {
    assert_eq!(
        appears(&the_file(THE_DOCUMENT)),
        Appears::A(Kind::PagesDocument)
    );
}

/// **It is recognised from its bytes, with a name that says something else.**
///
/// The whole of what this crate is for: an extension is a claim by whoever
/// named the file.
#[test]
fn it_is_recognised_under_a_name_that_lies() {
    let bytes = the_file(THE_DOCUMENT);
    let decided = decide(
        &mut Cursor::new(&bytes),
        OsStr::new("minutes.docx"),
        &ThisMachine::with_nothing(),
    )
    .expect("bytes in memory are readable");
    let outcome = match decided {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    };
    assert_eq!(outcome.appears(), Appears::A(Kind::PagesDocument));
    assert!(
        matches!(decided, Decided::NotWhatItsNameSays { .. }),
        "a Pages document named .docx is not reported as misnamed"
    );
}

/// **The zips that are not Pages documents are still themselves.**
///
/// Against the real Office documents `alo-converting` is measured on, because
/// the rule that matters is the one that separates them — a rule reading
/// `PK\x03\x04` would call all four the same thing.
#[test]
fn the_other_real_zips_are_not_claimed_to_be_pages_documents() {
    for (named, expected) in [
        ("sample.docx", Kind::WordDocument),
        ("sample.xlsx", Kind::ExcelWorkbook),
        ("sample.pptx", Kind::PowerPointPresentation),
    ] {
        assert_eq!(
            appears(&an_office_document(named)),
            Appears::A(expected),
            "{named} was read as something else"
        );
    }
}

/// **A plain archive is a plain archive.**
#[test]
fn a_zip_that_holds_nothing_of_the_sort_is_an_archive() {
    assert_eq!(
        appears(&a_zip_holding(&["notes.txt", "pictures/one.png"])),
        Appears::A(Kind::ZipArchive)
    );
}

/// **A container carrying the other two applications' parts is refused.**
///
/// The exclusion doing what it says. These are assembled, and they are not
/// evidence about Keynote or Numbers — no real file of either exists here, and
/// `tests/files/README.md` records the ask. What they show is that a document
/// carrying those parts is **not claimed to be a Pages document**, which is the
/// half a wrong guess about those formats would otherwise turn into a confident
/// wrong answer.
#[test]
fn an_iwork_container_with_another_applications_parts_is_not_called_pages() {
    let keynote = a_zip_holding(&[
        "Index/Document.iwa",
        "Index/Slide-1234.iwa",
        "Metadata/Properties.plist",
    ]);
    assert_eq!(appears(&keynote), Appears::Unrecognised);

    let numbers = a_zip_holding(&[
        "Index/Document.iwa",
        "Index/Tables/Tile-1.iwa",
        "Metadata/Properties.plist",
    ]);
    assert_eq!(appears(&numbers), Appears::Unrecognised);

    let theme = a_zip_holding(&["Index/Document.iwa", "Index/Theme-1.iwa"]);
    assert_eq!(appears(&theme), Appears::Unrecognised);
}

/// **The index alone is what names it, and its absence is not guessed past.**
///
/// A container with the metadata and previews of an iWork document but no
/// document index is not a Pages document. The rule reads the part that is
/// always there, not the ones that happen to be.
#[test]
fn the_metadata_alone_does_not_make_a_pages_document() {
    let without = a_zip_holding(&[
        "Metadata/Properties.plist",
        "Metadata/DocumentIdentifier",
        "preview.jpg",
    ]);
    assert_eq!(appears(&without), Appears::A(Kind::ZipArchive));
}

/// **A machine with nothing explains it and says what would open it.**
///
/// The road a Pages document takes **today**, and it is not the road ADR 0057
/// decided for it. That decision says a Pages document converts through the
/// engine the image already pins; it does not yet, and the reason is in this
/// task's report: ADR 0039 requires an inventory of the original before any
/// copy is shown, the three inventories read zip-and-XML, and a Pages document
/// keeps its text in a compressed protobuf nothing here reads. Registering the
/// conversion without one would make this machine say *this converts* and then
/// refuse every time, which ADR 0039 §1 forbids by name.
///
/// So this test holds what is true rather than what was intended, and it is
/// the test that will have to change when the conversion lands.
#[test]
fn a_machine_with_nothing_explains_it_rather_than_shrugging() {
    assert_eq!(
        outcome_of(&the_file(THE_DOCUMENT), &ThisMachine::with_nothing()),
        Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::PagesDocument))
    );
    assert_eq!(
        Cannot::NothingHereOpens(Kind::PagesDocument).would(),
        Would::AnotherMachineOrFormat
    );
}

/// **A machine that converts one says so, and into a PDF.**
///
/// The road ADR 0057 decided, held here against a machine told it converts —
/// so that when the conversion is registered for real, the sentence a person
/// meets is already the one this asserts.
#[test]
fn a_machine_that_converts_one_offers_a_pdf_copy() {
    let machine = ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .expect("a machine may open a PDF")
        .converts(Kind::PagesDocument, Kind::Pdf)
        .expect("a machine may convert a Pages document");
    let outcome = outcome_of(&the_file(THE_DOCUMENT), &machine);
    assert!(
        matches!(
            outcome,
            Outcome::Converts {
                from: Kind::PagesDocument,
                into: Kind::Pdf,
                ..
            }
        ),
        "a machine that converts a Pages document did not offer a PDF copy: {outcome:?}"
    );
}

/// **It is not something to play.**
#[test]
fn a_pages_document_is_not_a_kind_that_is_played() {
    assert!(!Kind::PagesDocument.is_played());
}
