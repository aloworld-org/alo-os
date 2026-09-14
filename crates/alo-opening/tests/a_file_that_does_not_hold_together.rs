//! A damaged file is said to be damaged, and a file nothing recognises is said
//! to be unrecognised — and the two are never mistaken for each other.
//!
//! They send a person in opposite directions: *damaged* back to whoever sent
//! it, for another copy; *not recognised* to another machine, or a different
//! format. So every way a file can fail to hold together is shown here beside
//! a good file of the same kind that does, and beside bytes that merely look
//! like nothing — because a rule that called everything damaged would pass
//! every refusal and send every person the wrong way.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::ffi::OsStr;
use std::io::Cursor;

use alo_opening::{Appears, Container, Decided, Kind, Macros, ThisMachine, decide};

use making::{
    Finish, Stored, a_word_document, a_zip, a_zip_finished, an_older_office_file, an_open_document,
};

/// What these bytes were decided as, under a name that claims nothing.
fn appears(bytes: &[u8]) -> Appears {
    appears_with_macros(bytes).0
}

/// What these bytes were decided as, and whether macros were seen — on a
/// machine that opens every kind, so that a kind always reaches an outcome that
/// says.
fn appears_with_macros(bytes: &[u8]) -> (Appears, Option<Macros>) {
    let mut machine = ThisMachine::with_nothing();
    for kind in Kind::EVERY {
        machine = machine
            .opens(kind)
            .expect("a machine that opens every kind");
    }
    let decided = decide(
        &mut Cursor::new(bytes.to_vec()),
        OsStr::new("unnamed"),
        &machine,
    )
    .expect("bytes in memory to be read");
    let outcome = match decided {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    };
    let macros = match outcome {
        alo_opening::Outcome::OpensAsItIs { macros, .. } => Some(macros),
        alo_opening::Outcome::Converts { costs, .. } => Some(costs.macros()),
        alo_opening::Outcome::CannotOpen(_) => None,
    };
    (outcome.appears(), macros)
}

/// **A damaged file is said to be damaged, not unrecognised** — a download
/// cut short in each storage this crate reads, next to the whole file it was
/// cut from.
#[test]
fn a_damaged_file_is_said_to_be_damaged_not_unrecognised() {
    let word = a_word_document(4096);
    assert_eq!(appears(&word), Appears::A(Kind::WordDocument));
    let half = word.get(..word.len() / 2).expect("half a document");
    assert_eq!(appears(half), Appears::Damaged(Container::Compressed));

    let pdf = b"%PDF-1.4\n1 0 obj << >> endobj\nxref\ntrailer << >>\n%%EOF\n".to_vec();
    assert_eq!(appears(&pdf), Appears::A(Kind::Pdf));
    let cut = pdf.get(..pdf.len() - 7).expect("a PDF without its end");
    assert_eq!(appears(cut), Appears::Damaged(Container::Pdf));

    let older = an_older_office_file(&Stored::with(&["Workbook"]));
    assert_eq!(appears(&older), Appears::A(Kind::OlderExcelWorkbook));
    let after_the_header = an_older_office_file(&Stored {
        cut_after_the_header: true,
        ..Stored::with(&["Workbook"])
    });
    assert_eq!(
        appears(&after_the_header),
        Appears::Damaged(Container::OlderOffice)
    );

    // And bytes that are nothing at all are not damaged: nothing began.
    assert_eq!(appears(b"\x00\x13\x37garbage"), Appears::Unrecognised);
}

/// A zip whose list does not add up is damaged, every way it can fail to.
#[test]
fn a_zip_whose_list_does_not_add_up_is_damaged() {
    let whole = a_zip(&[("a.txt", b"one"), ("b.txt", b"two")]);
    assert_eq!(appears(&whole), Appears::A(Kind::ZipArchive));

    // Its end record claims one more entry than the list holds.
    let mut one_more = whole.clone();
    let end = one_more.len() - 22;
    if let Some(count) = one_more.get_mut(end + 10..end + 12) {
        count.copy_from_slice(&3_u16.to_le_bytes());
    }
    assert_eq!(appears(&one_more), Appears::Damaged(Container::Compressed));

    // Its end record points the list past the end of the file.
    let mut elsewhere = whole.clone();
    if let Some(offset) = elsewhere.get_mut(end + 16..end + 20) {
        offset.copy_from_slice(&0x00ff_ffff_u32.to_le_bytes());
    }
    assert_eq!(appears(&elsewhere), Appears::Damaged(Container::Compressed));

    // An entry of its list is not an entry.
    let mut scribbled = whole.clone();
    let list_at = whole
        .windows(4)
        .position(|four| four == b"PK\x01\x02")
        .expect("the list is in the file");
    if let Some(signature) = scribbled.get_mut(list_at..list_at + 4) {
        signature.copy_from_slice(b"XXXX");
    }
    assert_eq!(appears(&scribbled), Appears::Damaged(Container::Compressed));

    // Only its first entry's header is there.
    let first_entry = whole.get(..40).expect("the start of a zip");
    assert_eq!(
        appears(first_entry),
        Appears::Damaged(Container::Compressed)
    );
}

/// **A zip that holds together is read wherever its end is**: after a comment,
/// and in the larger form a zip over four gigabytes is written in.
#[test]
fn a_zip_that_holds_together_is_read_however_it_ends() {
    let entries: [(&str, &[u8]); 2] = [
        ("[Content_Types].xml", b"<Types/>"),
        ("xl/workbook.xml", b"<workbook/>"),
    ];
    let commented = a_zip_finished(
        &entries,
        &Finish {
            comment: b"made by somebody's spreadsheet program".to_vec(),
            as_zip64: false,
        },
    );
    assert_eq!(appears(&commented), Appears::A(Kind::ExcelWorkbook));

    let large_form = a_zip_finished(
        &entries,
        &Finish {
            comment: Vec::new(),
            as_zip64: true,
        },
    );
    assert_eq!(appears(&large_form), Appears::A(Kind::ExcelWorkbook));
}

/// An older Office file whose directory loops, or points at entries that are
/// not there, is damaged — and deciding it ends.
#[test]
fn an_older_office_file_whose_directory_does_not_hold_together_is_damaged() {
    let looping = an_older_office_file(&Stored {
        looping: true,
        ..Stored::with(&["WordDocument", "1Table", "Data", "ObjectPool"])
    });
    assert_eq!(appears(&looping), Appears::Damaged(Container::OlderOffice));

    let nowhere = an_older_office_file(&Stored {
        pointing_nowhere: true,
        ..Stored::with(&["WordDocument"])
    });
    assert_eq!(appears(&nowhere), Appears::Damaged(Container::OlderOffice));

    let mut wrong_version = an_older_office_file(&Stored::with(&["WordDocument"]));
    if let Some(major) = wrong_version.get_mut(0x1a..0x1c) {
        major.copy_from_slice(&7_u16.to_le_bytes());
    }
    assert_eq!(
        appears(&wrong_version),
        Appears::Damaged(Container::OlderOffice)
    );
}

/// **What is not a document of a known kind is unrecognised, not guessed at**:
/// storage files that are not Office documents, zips declaring a type nobody
/// here knows, and an Office-shaped zip holding two documents' parts at once.
#[test]
fn what_is_not_a_known_kind_is_unrecognised_rather_than_guessed() {
    // A saved mail message is stored the older Office way and is not a
    // document.
    let message = an_older_office_file(&Stored::with(&[
        "__substg1.0_0037001F",
        "__properties_version1.0",
    ]));
    assert_eq!(appears(&message), Appears::Unrecognised);

    // An electronic book declares its own type the way an OpenDocument does.
    let book = an_open_document("application/epub+zip", &[]);
    assert_eq!(appears(&book), Appears::Unrecognised);

    // Parts of a Word document and of a presentation in one file.
    let both = a_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("word/document.xml", b"<w:document/>"),
        ("ppt/presentation.xml", b"<p:presentation/>"),
    ]);
    assert_eq!(appears(&both), Appears::Unrecognised);

    // An older Office file naming two documents at its top.
    let two = an_older_office_file(&Stored::with(&["WordDocument", "Workbook"]));
    assert_eq!(appears(&two), Appears::Unrecognised);

    // `MZ` is two letters a text file can begin with.
    assert_eq!(
        appears(b"MZ is where the story starts.\n"),
        Appears::A(Kind::Text)
    );
}

/// **Macros are a finding where they are kept, and absent ones are only not
/// seen.** A document that keeps an empty macro listing — which every
/// OpenDocument does — is not said to carry macros.
#[test]
fn macros_are_found_where_each_kind_keeps_them() {
    let open_with = an_open_document(
        "application/vnd.oasis.opendocument.text",
        &[
            ("Basic/script-lc.xml", b"<library:libraries/>"),
            ("Basic/Standard/script-lb.xml", b"<library:library/>"),
            (
                "Basic/Standard/Module1.xml",
                b"<script:module>Sub Main</script:module>",
            ),
        ],
    );
    assert_eq!(
        appears_with_macros(&open_with),
        (Appears::A(Kind::OpenDocumentText), Some(Macros::Inside))
    );

    let open_listing_only = an_open_document(
        "application/vnd.oasis.opendocument.text",
        &[
            ("Basic/script-lc.xml", b"<library:libraries/>"),
            ("Basic/Standard/script-lb.xml", b"<library:library/>"),
        ],
    );
    assert_eq!(
        appears_with_macros(&open_listing_only),
        (Appears::A(Kind::OpenDocumentText), Some(Macros::NoneSeen))
    );

    let older_with = an_older_office_file(&Stored {
        top: vec![
            ("WordDocument".to_owned(), false),
            ("Macros".to_owned(), true),
        ],
        ..Stored::default()
    });
    assert_eq!(
        appears_with_macros(&older_with),
        (Appears::A(Kind::OlderWordDocument), Some(Macros::Inside))
    );

    let current_with = a_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("ppt/presentation.xml", b"<p:presentation/>"),
        ("ppt/vbaProject.bin", b"\xd0\xcf\x11\xe0"),
    ]);
    assert_eq!(
        appears_with_macros(&current_with),
        (
            Appears::A(Kind::PowerPointPresentation),
            Some(Macros::Inside)
        )
    );
    assert_eq!(
        appears_with_macros(&a_word_document(16)),
        (Appears::A(Kind::WordDocument), Some(Macros::NoneSeen))
    );
}
