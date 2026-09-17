//! Given this file, what can this machine do with it — against files written to
//! a real disk and opened the way whoever holds a grant would open them.
//!
//! Each test here is one clause of the acceptance for *What this machine can do
//! with a file, and what it cannot* in
//! `docs/autonomy/v0-5-documents-and-paper-plan.md`: what a file is comes from
//! its content and not its name; what the machine can do is one of three
//! outcomes and never a *probably*; a file that is not what its name says is
//! that finding; and every outcome carries a sentence from the vocabulary the
//! machine really has.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::ffi::OsStr;
use std::fs::{self, File};

use alo_opening::{
    Appears, Cannot, Container, Decided, Kind, Macros, Named, Outcome, ThisMachine, decide,
};
use alo_strings::Strings;

use making::{
    Stored, a_folder_of_our_own, a_word_document, a_zip, an_older_office_file, an_open_document,
};

/// A machine with a viewer for PDFs, images and text, and a conversion from
/// each office format into a PDF — the shape task 2 of the plan gives a real
/// one. What it has is handed in, as it always is.
fn a_machine() -> ThisMachine {
    let mut machine = ThisMachine::with_nothing();
    for kind in [
        Kind::Pdf,
        Kind::PngImage,
        Kind::JpegImage,
        Kind::GifImage,
        Kind::Text,
    ] {
        machine = machine.opens(kind).expect("a kind to open");
    }
    for kind in [
        Kind::WordDocument,
        Kind::ExcelWorkbook,
        Kind::PowerPointPresentation,
        Kind::OpenDocumentText,
        Kind::OlderWordDocument,
    ] {
        machine = machine.converts(kind, Kind::Pdf).expect("a conversion");
    }
    machine
}

/// These bytes, written to a file of this name on the disk, then opened and
/// decided about.
fn decided_on_disk(name: &str, bytes: &[u8], machine: &ThisMachine) -> Decided {
    let folder = a_folder_of_our_own("decided");
    let path = folder.join(name);
    fs::write(&path, bytes).expect("the file to be written");
    let mut file = File::open(&path).expect("the file to open");
    let decided = decide(&mut file, OsStr::new(name), machine).expect("the file to be read");
    drop(file);
    drop(fs::remove_dir_all(&folder));
    decided
}

/// What the outcome is, whatever the name said.
fn outcome_of(decided: Decided) -> Outcome {
    match decided {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    }
}

/// The machine's one vocabulary, in English.
fn what_this_machine_says() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    )
}

/// A PNG's first bytes and a little after them.
const A_PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x02\0\0\0";

/// A small PDF with its end where it belongs.
const A_PDF: &[u8] = b"%PDF-1.7\n1 0 obj << /Type /Catalog >> endobj\ntrailer << >>\n%%EOF\n";

/// **What a file is comes from its bytes, and never from its name.** The same
/// bytes decide the same way under every name, and the same name over
/// different bytes decides differently.
#[test]
fn what_a_file_is_comes_from_its_bytes_and_never_its_name() {
    let machine = a_machine();
    let word = a_word_document(2048);

    let cases: [(&[u8], Appears); 8] = [
        (A_PDF, Appears::A(Kind::Pdf)),
        (A_PNG, Appears::A(Kind::PngImage)),
        (&word, Appears::A(Kind::WordDocument)),
        (
            &an_open_document("application/vnd.oasis.opendocument.spreadsheet", &[]),
            Appears::A(Kind::OpenDocumentSpreadsheet),
        ),
        (
            &an_older_office_file(&Stored::with(&[
                "WordDocument",
                "1Table",
                "\u{5}SummaryInformation",
            ])),
            Appears::A(Kind::OlderWordDocument),
        ),
        (
            &a_zip(&[("photos/one.jpg", b"\xff\xd8\xff"), ("notes.txt", b"hi")]),
            Appears::A(Kind::ZipArchive),
        ),
        (
            b"Name;Stra\xdfe\r\nM\xfcller;Hauptstra\xdfe\r\n",
            Appears::A(Kind::TextInAnOlderCharacterSet),
        ),
        (b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0", Appears::AProgram),
    ];

    for (bytes, appears) in cases {
        for name in [
            "file.pdf",
            "file.docx",
            "file.png",
            "file",
            "file.exe",
            "FILE.TXT",
        ] {
            assert_eq!(
                outcome_of(decided_on_disk(name, bytes, &machine)).appears(),
                appears,
                "named {name}, the bytes of {appears:?} were decided as something else"
            );
        }
    }
}

/// **Every file lands in exactly one of three outcomes**: opened as it is,
/// converted with what that costs, or cannot be opened with the reason. There
/// is no fourth, and each is reached here from a real file.
#[test]
fn every_file_lands_in_exactly_one_of_three_outcomes() {
    let machine = a_machine();

    assert_eq!(
        decided_on_disk("scan.pdf", A_PDF, &machine),
        Decided::AsItIs(Outcome::OpensAsItIs {
            kind: Kind::Pdf,
            macros: Macros::NoneSeen
        })
    );

    let with_macros = a_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("word/document.xml", b"<w:document/>"),
        ("word/vbaProject.bin", b"\xd0\xcf\x11\xe0"),
    ]);
    match decided_on_disk("minutes.docm", &with_macros, &machine) {
        Decided::AsItIs(Outcome::Converts { from, into, costs }) => {
            assert_eq!(from, Kind::WordDocument);
            assert_eq!(into, Kind::Pdf);
            assert_eq!(costs.macros(), Macros::Inside);
        }
        other => panic!("a Word document with macros was decided as {other:?}"),
    }

    let presentation = an_older_office_file(&Stored::with(&[
        "PowerPoint Document",
        "Current User",
        "Pictures",
        "\u{5}SummaryInformation",
        "\u{5}DocumentSummaryInformation",
    ]));
    assert_eq!(
        decided_on_disk("slides.ppt", &presentation, &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
            Kind::OlderPowerPointPresentation
        )))
    );

    let locked = an_older_office_file(&Stored::with(&[
        "EncryptionInfo",
        "EncryptedPackage",
        "\u{6}DataSpaces",
    ]));
    assert_eq!(
        decided_on_disk("salaries.xlsx", &locked, &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::PasswordProtected))
    );

    assert_eq!(
        decided_on_disk("nothing.txt", b"", &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::Empty))
    );
    // A film is recognised as the film it is, and refused for the reason that
    // is true of this machine: nothing here opens it. Bytes of no kind at all
    // are the ones that are *not recognised*.
    assert_eq!(
        decided_on_disk("video", b"\0\0\0\x18ftypmp42\0\0\0\0", &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
            Kind::Mp4Video
        )))
    );
    assert_eq!(
        decided_on_disk(
            "whatever",
            b"\x07\x0e\x13\x2b\x91\xa0\x00\xff\xfe\x01",
            &machine
        ),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::Unrecognised))
    );
    assert_eq!(
        decided_on_disk("tool.exe", &a_windows_program(), &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::AProgram))
    );

    let mut cut = A_PDF.to_vec();
    cut.truncate(30);
    assert_eq!(
        decided_on_disk("contract.pdf", &cut, &machine),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::Damaged(Container::Pdf)))
    );

    // The same file on a machine that has nothing to show it with is the
    // third outcome, for the reason that is about the machine.
    assert_eq!(
        decided_on_disk("scan.pdf", A_PDF, &ThisMachine::with_nothing()),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::Pdf)))
    );
}

/// The smallest Windows program header: `MZ`, and at `0x3c` where `PE\0\0` is.
fn a_windows_program() -> Vec<u8> {
    let mut program = b"MZ".to_vec();
    program.resize(0x3c, 0);
    program.extend_from_slice(&0x40_u32.to_le_bytes());
    program.extend_from_slice(b"PE\0\0");
    program.resize(0x80, 0);
    program
}

/// **A file that is not what its name says is reported as exactly that**, and
/// the outcome for what it really is sits inside the finding rather than
/// beside it — nothing is silently corrected.
#[test]
fn a_file_that_is_not_what_its_name_says_is_that_finding_first() {
    let machine = a_machine();
    let strings = what_this_machine_says();

    // A program dressed as an invoice.
    let decided = decided_on_disk("invoice.pdf.exe.pdf", &a_windows_program(), &machine);
    assert_eq!(
        decided,
        Decided::NotWhatItsNameSays {
            named: Named::A(Kind::Pdf),
            outcome: Outcome::CannotOpen(Cannot::AProgram),
        }
    );
    let said: Vec<String> = decided
        .said(&strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect();
    assert_eq!(
        said,
        [
            "This file is named as a PDF document, but it is a program",
            "This file is a program. Opening a file never runs a program, so it has not been \
             opened",
            "If a document was expected, whoever sent it can send the document itself instead",
        ]
    );

    // A photograph saved under a document's name still opens — as the
    // photograph it is, after the person has been told.
    let decided = decided_on_disk("report.docx", A_PNG, &machine);
    assert_eq!(
        decided,
        Decided::NotWhatItsNameSays {
            named: Named::A(Kind::WordDocument),
            outcome: Outcome::OpensAsItIs {
                kind: Kind::PngImage,
                macros: Macros::NoneSeen
            },
        }
    );
    assert_eq!(
        decided
            .said(&strings)
            .first()
            .map(|said| said.text().to_owned()),
        Some("This file is named as a Word document, but it is a PNG image".to_owned())
    );

    // Bytes that are nothing at all, under a spreadsheet's name, are not one.
    let decided = decided_on_disk("budget.xlsx", b"\0\x01\x02\x03garbage\0", &machine);
    assert_eq!(
        decided
            .said(&strings)
            .first()
            .map(|said| said.text().to_owned()),
        Some("This file is named as an Excel spreadsheet, but it is not one".to_owned())
    );

    // And a name that tells the truth, or claims nothing, is no finding.
    assert!(matches!(
        decided_on_disk("photo.png", A_PNG, &machine),
        Decided::AsItIs(_)
    ));
    assert!(matches!(
        decided_on_disk("photo", A_PNG, &machine),
        Decided::AsItIs(_)
    ));
    assert!(matches!(
        decided_on_disk("export.csv", b"a;b\r\n1;2\r\n", &machine),
        Decided::AsItIs(_)
    ));
}

/// **Every outcome carries the sentence a person reads, from the vocabulary
/// `alo-saying` gathers** — so none of them reaches a person as a missing key,
/// and none of them is English written somewhere a translator cannot see.
#[test]
fn every_outcome_carries_a_sentence_from_the_machines_vocabulary() {
    let strings = what_this_machine_says();
    let vocabulary = strings.vocabulary();
    for word in alo_opening::EVERY_WORD {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected into the machine's vocabulary",
            word.named()
        );
    }

    let machine = a_machine();
    let files: [(&str, Vec<u8>); 9] = [
        ("scan.pdf", A_PDF.to_vec()),
        ("letter.docx", a_word_document(64)),
        (
            "slides.ppt",
            an_older_office_file(&Stored::with(&["PowerPoint Document"])),
        ),
        (
            "salaries.xlsx",
            an_older_office_file(&Stored::with(&["EncryptedPackage"])),
        ),
        ("nothing.txt", Vec::new()),
        ("video", b"\0\0\0\x18ftypmp42".to_vec()),
        ("tool.exe", a_windows_program()),
        ("contract.pdf", A_PDF.get(..20).unwrap_or_default().to_vec()),
        ("invoice.pdf", A_PNG.to_vec()),
    ];
    for (name, bytes) in files {
        let said = decided_on_disk(name, &bytes, &machine).said(&strings);
        assert!(!said.is_empty(), "{name} decided and said nothing");
        for sentence in &said {
            let text = sentence.text();
            assert!(
                !text.contains("opening."),
                "{name} reached a person as a key: {text}"
            );
            assert!(
                !text.contains('{') && !text.contains('}'),
                "{name} left a gap unfilled: {text}"
            );
            assert!(
                text.chars().next().is_some_and(char::is_uppercase),
                "{name} said something that does not begin a sentence: {text}"
            );
        }
    }
}
