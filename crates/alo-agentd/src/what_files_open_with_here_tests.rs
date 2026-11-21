//! What a file opens with here, read from real entries and a real file — the
//! seam between `alo_turn`'s question and the machine's own answer.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use super::*;

/// A folder of this test's own, holding these files.
fn a_place(called: &str, files: &[(&str, &[u8])]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-agentd-opens-with-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, bytes) in files {
        std::fs::write(at.join(named), bytes).unwrap();
    }
    at
}

const DOLPHIN: &[u8] = b"[Desktop Entry]
Type=Application
Name=Dolphin
Exec=dolphin %u
MimeType=inode/directory;
";

const PAPERS: &[u8] = b"[Desktop Entry]
Type=Application
Name=Papers
Exec=papers %U
MimeType=application/pdf;
";

/// The machine's own words.
fn the_words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// **A PDF opens with what declares PDFs, and the file manager opens
/// folders** — from the entries as written, and the file's own bytes.
#[test]
fn a_pdf_opens_with_what_declares_it_and_the_file_manager_opens_folders() {
    let entries = a_place(
        "entries",
        &[
            ("org.kde.dolphin.desktop", DOLPHIN),
            ("org.gnome.Papers.desktop", PAPERS),
        ],
    );
    // A small PDF with its end where it belongs, as `alo-opening`'s own tests
    // make one: a PDF cut short is refused as damaged, which is right.
    let files = a_place(
        "files",
        &[(
            "report.pdf",
            b"%PDF-1.7\n1 0 obj << /Type /Catalog >> endobj\ntrailer << >>\n%%EOF\n",
        )],
    );
    let words = the_words();
    let here = WhatFilesOpenWithHere::in_these(vec![entries], None, &words);
    assert!(here.opens_folders("org.kde.dolphin"));
    assert!(!here.opens_folders("org.gnome.Papers"));
    assert!(here.is_a_folder(&files));
    assert!(!here.is_a_folder(&files.join("report.pdf")));
    // **Named as the entries name them**, so a card never shows an identifier.
    assert_eq!(here.called("org.gnome.Papers").as_deref(), Some("Papers"));
    assert_eq!(here.called("org.kde.dolphin").as_deref(), Some("Dolphin"));
    assert_eq!(here.called("org.example.Nothing"), None);
    assert_eq!(
        here.opens_with(&files.join("report.pdf")),
        Ok("org.gnome.Papers".to_owned())
    );
}

/// **The refusal cases**: a program opens with nothing, a file that is not
/// there opens with nothing, and each says why in the machine's words.
#[test]
fn what_nothing_opens_is_said_and_never_guessed() {
    let entries = a_place(
        "entries-refused",
        &[
            ("org.kde.dolphin.desktop", DOLPHIN),
            ("org.gnome.Papers.desktop", PAPERS),
        ],
    );
    let files = a_place("files-refused", &[("run.pdf", b"#!/bin/sh\necho hello\n")]);
    let words = the_words();
    let here = WhatFilesOpenWithHere::in_these(vec![entries], None, &words);
    let program = here.opens_with(&files.join("run.pdf"));
    assert!(program.is_err(), "a program named .pdf opened: {program:?}");
    let missing = here.opens_with(&files.join("not-there.pdf"));
    assert!(missing.is_err(), "{missing:?}");
    assert!(missing.unwrap_err().said().text().contains("not-there.pdf"));
}
