//! Deciding reads no more of a file than deciding requires — counted.
//!
//! The plan's constraint for *What this machine can do with a file* is that
//! nothing here reads more of a file than deciding requires. A sentence in a
//! crate's documentation cannot hold that; a file that records every byte read
//! from it can. Each test hands [`decide`] a large file whose contents no rule
//! should need, and says exactly which bytes were touched.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::ffi::OsStr;

use alo_opening::{Appears, Decided, Kind, THE_HEAD, ThisMachine, decide};

use making::{Counted, Stored, a_word_document, a_zip, an_older_office_file};

/// What a counted file was decided as.
fn appears(file: &mut Counted, name: &str) -> Appears {
    match decide(file, OsStr::new(name), &ThisMachine::with_nothing()).expect("the file to be read")
    {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome.appears(),
    }
}

/// **A document is decided without reading its contents.** A Word document
/// whose body is eight megabytes is decided from its first bytes, its end
/// record and its list of contents — and not one byte of the body is read.
#[test]
fn a_document_is_decided_without_reading_its_contents() {
    const BODY: usize = 8 * 1024 * 1024;
    let bytes = a_word_document(BODY);
    let len = bytes.len() as u64;

    // Where the body is: after the first two entries' headers and contents.
    let body_starts = bytes
        .windows(b"word/document.xml".len())
        .position(|window| window == b"word/document.xml")
        .expect("the body's name is in the file") as u64
        + b"word/document.xml".len() as u64;
    let body_ends = body_starts + BODY as u64;

    let mut file = Counted::holding(bytes);
    assert_eq!(
        appears(&mut file, "letter.docx"),
        Appears::A(Kind::WordDocument)
    );
    assert!(
        !file.touched(body_starts.max(THE_HEAD as u64), body_ends),
        "deciding what a document is read its contents: {:?}",
        file.reads
    );
    assert!(
        file.read() < 2 * 1024,
        "deciding what an eight-megabyte document is read {} bytes of it",
        file.read()
    );
    assert!(file.read() as u64 <= len);
}

/// **A PDF is decided from its start and its last kilobyte**, however long it
/// is.
#[test]
fn a_pdf_is_decided_from_its_start_and_its_end() {
    let mut bytes = b"%PDF-1.7\n".to_vec();
    bytes.resize(16 * 1024 * 1024, b' ');
    bytes.extend_from_slice(b"\n%%EOF\n");
    let len = bytes.len() as u64;
    let mut file = Counted::holding(bytes);
    assert_eq!(appears(&mut file, "scan.pdf"), Appears::A(Kind::Pdf));
    assert!(file.read() <= THE_HEAD + 1024, "{:?}", file.reads);
    assert!(!file.touched(THE_HEAD as u64, len - 1024));
}

/// **An image, and a program, are decided from their first bytes alone.**
#[test]
fn an_image_or_a_program_is_decided_from_its_first_bytes() {
    let mut photograph = b"\xff\xd8\xff\xe0\0\x10JFIF\0".to_vec();
    photograph.resize(32 * 1024 * 1024, 0x55);
    let mut file = Counted::holding(photograph);
    assert_eq!(
        appears(&mut file, "holiday.jpg"),
        Appears::A(Kind::JpegImage)
    );
    assert_eq!(file.read(), THE_HEAD);

    let mut program = b"\x7fELF\x02\x01\x01".to_vec();
    program.resize(32 * 1024 * 1024, 0);
    let mut file = Counted::holding(program);
    assert_eq!(appears(&mut file, "invoice.pdf"), Appears::AProgram);
    assert_eq!(file.read(), THE_HEAD);
}

/// **The older Office format is decided from its header and its directory**,
/// following the directory's chain through the allocation table and no
/// further — even when the name that decides it is in the directory's second
/// sector.
#[test]
fn an_older_office_file_is_decided_from_its_directory() {
    let mut bytes = an_older_office_file(&Stored::with(&[
        "\u{5}SummaryInformation",
        "\u{5}DocumentSummaryInformation",
        "1Table",
        "Data",
        "ObjectPool",
        "WordDocument",
    ]));
    let directory_ends = bytes.len() as u64;
    bytes.resize(4 * 1024 * 1024, 0x77);
    let mut file = Counted::holding(bytes);
    assert_eq!(
        appears(&mut file, "old.doc"),
        Appears::A(Kind::OlderWordDocument)
    );
    assert!(
        !file.touched(directory_ends, directory_ends + 4 * 1024 * 1024),
        "deciding an older Office file read past its directory: {:?}",
        file.reads
    );
}

/// **Text is the one rule that reads to the end**, because text has no
/// signature — and it stops at the first byte that settles the question.
#[test]
fn text_reads_to_the_end_and_binary_stops_early() {
    let text = vec![b'a'; 1024 * 1024];
    let len = text.len();
    let mut file = Counted::holding(text);
    assert_eq!(appears(&mut file, "notes.txt"), Appears::A(Kind::Text));
    assert!(
        file.read() >= len,
        "text was decided without being read whole"
    );

    let mut binary = b"\x00\x01\x02\x03".to_vec();
    binary.resize(8 * 1024 * 1024, 0);
    let mut file = Counted::holding(binary);
    assert_eq!(appears(&mut file, "blob"), Appears::Unrecognised);
    // The head, the two bytes a byte-order mark would be in, and one piece.
    assert!(
        file.read() <= THE_HEAD + 2 + 64 * 1024,
        "bytes that were settled as not text in their first piece were read on: {}",
        file.read()
    );
}

/// A zip archive of large files is decided from its list, too: which files are
/// in an archive is the whole question, and none of them is read.
#[test]
fn an_archive_is_decided_from_its_list() {
    let big = vec![0x42_u8; 4 * 1024 * 1024];
    let bytes = a_zip(&[("one.bin", &big), ("two.bin", &big)]);
    let mut file = Counted::holding(bytes);
    assert_eq!(
        appears(&mut file, "files.zip"),
        Appears::A(Kind::ZipArchive)
    );
    assert!(file.read() < 2 * 1024, "{:?}", file.reads);
}
