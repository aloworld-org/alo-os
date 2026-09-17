//! *"I can't open this file"*, said properly — one test per reason, each
//! against a real file written to a real disk.
//!
//! Task 4 of `docs/autonomy/v0-5-documents-and-paper-plan.md`: for a file this
//! machine cannot open, what a person reads names **what it is, why this machine
//! cannot open it, and what would**. It never names a library, a media type, a
//! return code or anything this machine rents; a damaged file is said to be
//! damaged rather than unsupported, because the two send a person in opposite
//! directions; and nothing offers to send the file anywhere to find out.
//!
//! Every sentence here is the machine's collected vocabulary speaking, so a
//! word `alo-saying` did not gather would reach these tests as a key.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::ffi::OsStr;
use std::fs::{self, File};

use alo_opening::{Cannot, Container, Decided, Kind, Named, Outcome, ThisMachine, Would, decide};
use alo_strings::Strings;

use making::{Stored, a_folder_of_our_own, a_word_document, an_older_office_file};

/// A machine that opens PDFs and pictures and converts the current Word
/// format — enough for every reason to be reachable from a real file.
fn a_machine() -> ThisMachine {
    ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .and_then(|machine| machine.opens(Kind::PngImage))
        .and_then(|machine| machine.converts(Kind::WordDocument, Kind::Pdf))
        .expect("a machine that can be said")
}

/// The machine's one vocabulary, in English.
fn in_english() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    )
}

/// These bytes, written under this name to the disk, opened, and decided
/// about.
fn decided_on_disk(name: &str, bytes: &[u8], machine: &ThisMachine) -> Decided {
    let folder = a_folder_of_our_own("cannot-open");
    let path = folder.join(name);
    fs::write(&path, bytes).expect("the file to be written");
    let mut file = File::open(&path).expect("the file to open");
    let decided = decide(&mut file, OsStr::new(name), machine).expect("the file to be read");
    drop(file);
    drop(fs::remove_dir_all(&folder));
    decided
}

/// What a person reads, as text.
fn read(decided: Decided, strings: &Strings) -> Vec<String> {
    decided
        .said(strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect()
}

/// **What every explanation holds to**: two sentences, the second what would;
/// nothing names the machinery; nothing is guessed; nothing offers to send the
/// file anywhere; and every word came from the vocabulary.
fn an_explanation(
    decided: Decided,
    reason: Cannot,
    would: Would,
    strings: &Strings,
) -> Vec<String> {
    assert_eq!(decided, Decided::AsItIs(Outcome::CannotOpen(reason)));
    assert_eq!(reason.would(), would);
    let said = decided.said(strings);
    assert_eq!(said.len(), 2, "{said:?}");
    assert_eq!(said.last(), Some(&would.said(strings)));
    for sentence in &said {
        assert!(!sentence.is_a_bug(), "{sentence}");
        assert!(sentence.unfilled().is_empty(), "{sentence}");
        let text = sentence.text().to_lowercase();
        for never in [
            "opening.",
            "mime",
            "application/",
            "library",
            "libreoffice",
            "code",
            "error",
            "errno",
            "upload",
            "online",
            "service",
            "probably",
            "might",
            "may ",
        ] {
            assert!(!text.contains(never), "\"{never}\" in: {text}");
        }
    }
    read(decided, strings)
}

/// **An empty file** is said to be empty — not damaged, not unrecognised —
/// and what would open it is a complete copy.
#[test]
fn an_empty_file_is_explained() {
    let strings = in_english();
    let said = an_explanation(
        decided_on_disk("minutes", b"", &a_machine()),
        Cannot::Empty,
        Would::ACompleteCopy,
        &strings,
    );
    assert_eq!(
        said,
        [
            "This file is empty: there is nothing in it to open",
            "What would open is a complete copy, which whoever has the original can send again",
        ]
    );
}

/// **A file nothing recognises** is said to be about this machine, and what
/// would open it is whoever made it saying what it is — never *damaged*, and
/// never a machine this one cannot name.
#[test]
fn a_file_nothing_recognises_is_explained() {
    let strings = in_english();
    let said = an_explanation(
        decided_on_disk("holiday", b"\0\0\0\x18ftypmp42\0\0\0\0", &a_machine()),
        Cannot::Unrecognised,
        Would::WhoeverMadeIt,
        &strings,
    );
    assert_eq!(
        said,
        [
            "This machine does not recognise what this file is, so it has not opened it",
            "Whoever sent it can say which program made it, or send a copy saved in a different \
             format",
        ]
    );
    assert!(said.iter().all(|sentence| !sentence.contains("damaged")));
}

/// **A damaged file is said to be damaged**, in each storage this machine
/// reads, and sends a person back for a complete copy — the opposite direction
/// from a file this machine has nothing for.
#[test]
fn a_damaged_file_is_explained_as_damaged_not_unsupported() {
    let strings = in_english();
    let machine = a_machine();

    let word = a_word_document(4096);
    let half = word.get(..word.len() / 2).expect("half a document");
    let pdf = b"%PDF-1.4\n1 0 obj << >> endobj\nxref\ntrailer << >>\n%%EOF\n";
    let cut_pdf = pdf.get(..pdf.len() - 7).expect("a PDF without its end");
    let older = an_older_office_file(&Stored {
        cut_after_the_header: true,
        ..Stored::with(&["WordDocument"])
    });

    for (bytes, container, what) in [
        (half, Container::Compressed, "a compressed file"),
        (cut_pdf, Container::Pdf, "a PDF document"),
        (
            older.as_slice(),
            Container::OlderOffice,
            "a file in the format older Office documents are stored in",
        ),
    ] {
        let said = an_explanation(
            decided_on_disk("sent", bytes, &machine),
            Cannot::Damaged(container),
            Would::ACompleteCopy,
            &strings,
        );
        assert_eq!(
            said,
            [
                format!(
                    "This file is damaged: it is {what}, but it has been cut short or does not \
                     hold together inside, so it cannot be opened as it is"
                ),
                "What would open is a complete copy, which whoever has the original can send \
                 again"
                    .to_owned(),
            ]
        );
    }

    // The same kind whole, on a machine with nothing for it, is sent the other
    // way: the file is fine, and the machine is what is missing.
    let whole = an_explanation(
        decided_on_disk("sent", pdf, &ThisMachine::with_nothing()),
        Cannot::NothingHereOpens(Kind::Pdf),
        Would::AnotherMachineOrFormat,
        &strings,
    );
    assert!(whole.iter().all(|sentence| !sentence.contains("damaged")));
    assert_ne!(Would::ACompleteCopy, Would::AnotherMachineOrFormat);
}

/// **A program** is never run, and what would help is the document the person
/// was expecting.
#[test]
fn a_program_is_explained() {
    let strings = in_english();
    let said = an_explanation(
        decided_on_disk(
            "setup",
            b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0",
            &a_machine(),
        ),
        Cannot::AProgram,
        Would::TheDocumentItself,
        &strings,
    );
    assert_eq!(
        said,
        [
            "This file is a program. Opening a file never runs a program, so it has not been \
             opened",
            "If a document was expected, whoever sent it can send the document itself instead",
        ]
    );
}

/// **A document locked with a password** is said to be locked, and what would
/// open is a copy without it — the person is not asked for the password.
#[test]
fn a_document_locked_with_a_password_is_explained() {
    let strings = in_english();
    let locked = an_older_office_file(&Stored::with(&[
        "EncryptionInfo",
        "EncryptedPackage",
        "\u{6}DataSpaces",
    ]));
    let said = an_explanation(
        decided_on_disk("salaries", &locked, &a_machine()),
        Cannot::PasswordProtected,
        Would::ACopyWithoutThePassword,
        &strings,
    );
    assert_eq!(
        said,
        [
            "This is an Office document protected with a password, and this machine cannot open \
             a document protected that way",
            "What would open is a copy saved without the password, which whoever sent it can make",
        ]
    );
}

/// **A kind this machine has nothing for** is named, and what would open it is
/// another machine or a different format — naming no program.
#[test]
fn a_kind_nothing_here_opens_is_explained() {
    let strings = in_english();
    let slides = an_older_office_file(&Stored::with(&["PowerPoint Document", "Current User"]));
    let said = an_explanation(
        decided_on_disk("slides", &slides, &a_machine()),
        Cannot::NothingHereOpens(Kind::OlderPowerPointPresentation),
        Would::AnotherMachineOrFormat,
        &strings,
    );
    assert_eq!(
        said,
        [
            "This is a PowerPoint presentation in the older format, and nothing on this machine \
             opens it or converts it into something that does",
            "What would open it is a machine with a program for this kind of file, or a copy saved \
             in a different format by whoever sent it",
        ]
    );
}

/// **A file whose name lies and cannot be opened** is the finding first, then
/// the whole explanation — the name never cuts the way out short.
#[test]
fn a_file_named_as_what_it_is_not_is_still_explained() {
    let strings = in_english();
    let decided = decided_on_disk(
        "invoice.pdf",
        b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0",
        &a_machine(),
    );
    assert_eq!(
        decided,
        Decided::NotWhatItsNameSays {
            named: Named::A(Kind::Pdf),
            outcome: Outcome::CannotOpen(Cannot::AProgram),
        }
    );
    assert_eq!(
        read(decided, &strings),
        [
            "This file is named as a PDF document, but it is a program",
            "This file is a program. Opening a file never runs a program, so it has not been \
             opened",
            "If a document was expected, whoever sent it can send the document itself instead",
        ]
    );
}
