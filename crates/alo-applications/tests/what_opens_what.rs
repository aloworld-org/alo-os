//! What opens what, changeable by a person.
//!
//! Task 4 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance that belongs to this crate a test here (the
//! open-with portal's clause is `alo-portals`'
//! `tests/open_with_is_answered_from_what_opens_what.rs`):
//!
//! - **`alo-applications` answers *what opens this* for a file by its kind — the
//!   kind read from the file's bytes, never from the extension** —
//!   [`the_kind_is_read_from_the_files_bytes_and_never_from_its_name`];
//! - **from an association the person set, or from the application's own
//!   declaration where the person set none, and says which of the two it was**
//!   — [`the_answer_says_whether_the_person_chose_it_or_the_application_declared_it`];
//! - **a person's association is written into their own settings, read back,
//!   and wins over any declaration** —
//!   [`a_persons_choice_is_written_read_back_and_wins_over_every_declaration`];
//! - **the answer for a kind nothing opens is a sentence in the vocabulary rather
//!   than a fallback to a text editor** —
//!   [`a_kind_nothing_opens_is_a_sentence_and_never_a_text_editor`];
//!
//! and the constraint: **no association is set by an application on its own
//! behalf** — [`no_application_sets_what_opens_what_on_its_own_behalf`] — nor
//! by a file that did not read —
//! [`a_file_of_choices_that_does_not_read_is_refused_whole_and_never_written_over`].
//!
//! Files are written to a real disk, under a misleading name, and opened the
//! way the portal backend will hand them over. Nothing is launched.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_applications::keeping::{self, THE_FILE};
use alo_applications::{
    Application, Because, Chosen, Declared, Installed, NothingOpens, WhatOpensWhat,
    application_verbs,
};
use alo_opening::{Cannot, Kind};
use alo_strings::{Key, Strings};

/// A PDF, as its first and last bytes make it one.
const A_PDF: &[u8] = b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n";

/// A PNG image's signature and the start of its header.
const A_PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01";

/// Plain text.
const SOME_TEXT: &[u8] = b"Dear Anna,\n\tthe contract is attached.\n";

/// The start of a program.
const A_PROGRAM: &[u8] = b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0";

fn papers() -> Application {
    Application::called("org.gnome.Papers", "Papers").unwrap()
}

fn loupe() -> Application {
    Application::called("org.gnome.Loupe", "Image Viewer").unwrap()
}

fn text_editor() -> Application {
    Application::called("org.gnome.TextEditor", "Text Editor").unwrap()
}

fn okular() -> Application {
    Application::called("org.kde.okular", "Okular").unwrap()
}

/// The machine these tests answer on: three applications, each declaring what
/// its desktop entry would.
fn a_machine() -> (Installed, Declared) {
    let installed = Installed::holding([papers(), loupe(), text_editor(), okular()]);
    let mut declared = Declared::nothing();
    declared.declares_media_types(&papers(), ["application/pdf"]);
    declared.declares_media_types(&loupe(), ["image/png", "image/jpeg"]);
    declared.declares_media_types(&text_editor(), ["text/plain"]);
    declared.declares_media_types(&okular(), ["application/pdf", "image/png"]);
    (installed, declared)
}

/// The machine's one vocabulary, in English.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-applications-what-opens-what-{}-{what}",
        std::process::id()
    ));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// These bytes, written under this name, and opened.
fn a_file(folder: &Path, named: &str, bytes: &[u8]) -> File {
    let at = folder.join(named);
    std::fs::write(&at, bytes).unwrap();
    File::open(&at).unwrap()
}

/// **The kind is the file's bytes.** A PDF named as a picture opens where PDFs
/// do, a picture named as a PDF opens where pictures do, and a program named as
/// a PDF is opened by nothing — `alo-opening`'s finding, carried whole.
#[test]
fn the_kind_is_read_from_the_files_bytes_and_never_from_its_name() {
    let (installed, declared) = a_machine();
    let chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let folder = a_folder_of_our_own("bytes");

    let opener = asking
        .what_opens(&mut a_file(&folder, "holiday.png", A_PDF))
        .unwrap();
    assert_eq!(opener.kind(), Kind::Pdf);
    assert_eq!(opener.application(), &papers());

    let opener = asking
        .what_opens(&mut a_file(&folder, "invoice.pdf", A_PNG))
        .unwrap();
    assert_eq!(opener.kind(), Kind::PngImage);
    assert_eq!(opener.application(), &loupe());

    let opener = asking
        .what_opens(&mut a_file(&folder, "scan.jpg", SOME_TEXT))
        .unwrap();
    assert_eq!(opener.kind(), Kind::Text);
    assert_eq!(opener.application(), &text_editor());

    // A program is never a kind anything opens, whatever it is called.
    let refused = asking
        .what_opens(&mut a_file(&folder, "invoice.pdf", A_PROGRAM))
        .unwrap_err();
    assert_eq!(refused, NothingOpens::TheFile(Cannot::AProgram));
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("never runs a program"), "{said}");

    // And an empty file is not text for a text editor to open.
    assert_eq!(
        asking.what_opens(&mut a_file(&folder, "notes.txt", b"")),
        Err(NothingOpens::TheFile(Cannot::Empty))
    );
}

/// **Chosen or declared, and the answer says which** — including when the
/// person's choice could not be used, which is said rather than hidden.
#[test]
fn the_answer_says_whether_the_person_chose_it_or_the_application_declared_it() {
    let (installed, declared) = a_machine();
    let strings = in_english();

    let nothing_chosen = Chosen::untouched();
    let declared_answer = WhatOpensWhat::on(&installed, &declared, &nothing_chosen)
        .what_opens(&mut Cursor::new(A_PDF.to_vec()))
        .unwrap();
    assert_eq!(declared_answer.application(), &papers());
    assert_eq!(declared_answer.because(), &Because::ItDeclaresIt);
    assert!(!declared_answer.was_chosen());
    let said = declared_answer.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("org.gnome.Papers"), "{said}");
    assert!(said.text().contains("says it can"), "{said}");

    let mut chosen = Chosen::untouched();
    chosen.choose(Kind::Pdf, &okular());
    let chosen_answer = WhatOpensWhat::on(&installed, &declared, &chosen)
        .what_opens(&mut Cursor::new(A_PDF.to_vec()))
        .unwrap();
    assert_eq!(chosen_answer.application(), &okular());
    assert_eq!(chosen_answer.because(), &Because::ThePersonChoseIt);
    let said = chosen_answer.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("you chose"), "{said}");

    // Chosen, and since uninstalled: the declaration answers and says so.
    let without_okular = Installed::holding([papers(), loupe(), text_editor()]);
    let passed_over = WhatOpensWhat::on(&without_okular, &declared, &chosen)
        .what_opens(&mut Cursor::new(A_PDF.to_vec()))
        .unwrap();
    assert_eq!(passed_over.application(), &papers());
    assert_eq!(
        passed_over.because(),
        &Because::ItDeclaresItAndTheChoiceIsNotInstalled {
            chosen: "org.kde.okular".to_owned()
        }
    );
    let said = passed_over.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("org.kde.okular"), "{said}");
    assert!(said.text().contains("no longer installed"), "{said}");
}

/// **A person's choice goes into their own folder, comes back off the disk at
/// the next sign-in, and wins over every declaration** — including one made by
/// an application installed after they chose.
#[test]
fn a_persons_choice_is_written_read_back_and_wins_over_every_declaration() {
    let folder = a_folder_of_our_own("written");
    let at = folder.join(THE_FILE);
    assert_eq!(THE_FILE, "what-opens-what.toml");

    // Before anything is chosen there is no file, and that is not an error.
    assert_eq!(keeping::at_sign_in(&at), (Chosen::untouched(), None));
    assert!(!at.exists());

    // The person picks Okular for PDFs and Papers for PNG images — which Papers
    // does not even declare.
    let mut chosen = Chosen::untouched();
    chosen.choose(Kind::Pdf, &okular());
    chosen.choose(Kind::PngImage, &papers());
    keeping::keep(&at, &chosen).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "format = 1\n\n[kinds]\npdf = \"org.kde.okular\"\npng-image = \"org.gnome.Papers\"\n"
    );

    // Read back, as the next sign-in reads it.
    let (read_back, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(read_back, chosen);

    // Every application on the machine declares PDFs and PNGs, one of them
    // installed after the choice was made. The person's choice still answers.
    let (mut installed, mut declared) = a_machine();
    let latecomer = Application::called("com.example.OpensEverything", "Opens Everything").unwrap();
    installed.add(latecomer.clone());
    declared.declares(&latecomer, Kind::EVERY);
    let asking = WhatOpensWhat::on(&installed, &declared, &read_back);
    for (bytes, kind, application) in [
        (A_PDF, Kind::Pdf, okular()),
        (A_PNG, Kind::PngImage, papers()),
    ] {
        let opener = asking.what_opens(&mut Cursor::new(bytes.to_vec())).unwrap();
        assert_eq!(opener.kind(), kind);
        assert_eq!(opener.application(), &application, "{kind:?}");
        assert!(opener.was_chosen(), "{kind:?}");
    }

    // Forgetting a choice, kept, hands the kind back to the declarations.
    let mut changed = read_back;
    assert!(changed.forget(Kind::Pdf));
    keeping::keep(&at, &changed).unwrap();
    let (again, _) = keeping::at_sign_in(&at);
    let opener = WhatOpensWhat::on(&installed, &declared, &again)
        .what_opens_a(Kind::Pdf)
        .unwrap();
    assert_eq!(opener.application(), &papers());
    assert_eq!(opener.because(), &Because::ItDeclaresIt);

    // Put back as shipped, nothing is chosen.
    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    assert_eq!(keeping::read(&at).unwrap(), Chosen::untouched());
}

/// **A kind nothing opens is a sentence, and a text editor is never the
/// answer** — not for a PDF on a machine whose only application opens text,
/// and not for bytes of no kind this machine recognises.
#[test]
fn a_kind_nothing_opens_is_a_sentence_and_never_a_text_editor() {
    let installed = Installed::holding([text_editor()]);
    let mut declared = Declared::nothing();
    declared.declares_media_types(&text_editor(), ["text/plain"]);
    let chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let strings = in_english();

    for kind in Kind::EVERY {
        let answer = asking.what_opens_a(kind);
        if matches!(kind, Kind::Text | Kind::TextInAnOlderCharacterSet) {
            assert_eq!(answer.unwrap().application(), &text_editor());
            continue;
        }
        let refused = answer.unwrap_err();
        assert_eq!(
            refused,
            NothingOpens::NoApplication { kind, chosen: None },
            "{kind:?} was answered with something"
        );
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{kind:?}: {said}");
        assert!(said.unfilled().is_empty(), "{kind:?}: {said}");
        assert!(
            said.text().starts_with("nothing on this machine opens"),
            "{said}"
        );
        assert!(!said.text().contains("org.gnome.TextEditor"), "{said}");
    }

    let pdf = asking
        .what_opens(&mut Cursor::new(A_PDF.to_vec()))
        .unwrap_err();
    assert!(
        pdf.said(&strings).text().contains("a PDF document"),
        "{}",
        pdf.said(&strings)
    );

    // Bytes of no kind at all are not text either.
    assert_eq!(
        asking.what_opens(&mut Cursor::new(b"\0\x01\x02\x03binary\0".to_vec())),
        Err(NothingOpens::TheFile(Cannot::Unrecognised))
    );

    // A person's choice for a kind is honoured even where no application
    // declares it; nothing else fills that gap.
    let mut chosen = Chosen::untouched();
    chosen.choose(Kind::Pdf, &text_editor());
    let opener = WhatOpensWhat::on(&installed, &declared, &chosen)
        .what_opens_a(Kind::Pdf)
        .unwrap();
    assert!(opener.was_chosen());

    // Every sentence this crate says is collected into the machine's one
    // vocabulary, which is where a translator finds it.
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    for word in alo_applications::EVERY_WORD {
        let key = Key::named(word.named()).unwrap();
        assert!(
            vocabulary.phrase(&key).is_some(),
            "{} is not collected",
            word.named()
        );
    }
}

/// **No application sets what opens what on its own behalf.** Declaring a
/// kind never moves ahead of an earlier declaration and never touches a
/// person's choice; no agent verb names an association; and nothing that
/// answers the question can change it.
#[test]
fn no_application_sets_what_opens_what_on_its_own_behalf() {
    let (mut installed, mut declared) = a_machine();
    let chosen = Chosen::untouched();
    let before = chosen.clone();

    // An application installed later, declaring every kind — and declaring
    // them again, as a reinstall would.
    let grabby = Application::called("com.example.DefaultForEverything", "Documents").unwrap();
    assert!(installed.add(grabby.clone()));
    assert_eq!(declared.declares(&grabby, Kind::EVERY), Kind::EVERY.len());
    assert_eq!(declared.declares(&grabby, Kind::EVERY), 0);

    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    for (kind, first) in [
        (Kind::Pdf, papers()),
        (Kind::PngImage, loupe()),
        (Kind::Text, text_editor()),
    ] {
        assert_eq!(
            asking.what_opens_a(kind).unwrap().application(),
            &first,
            "{kind:?}"
        );
    }
    // Where nobody declared a kind before it, it answers — as a declaration,
    // never as a choice.
    let webp = asking.what_opens_a(Kind::WebpImage).unwrap();
    assert_eq!(webp.application(), &grabby);
    assert_eq!(webp.because(), &Because::ItDeclaresIt);

    // Answering changed nothing the person owns.
    assert_eq!(chosen, before);
    assert!(chosen.is_untouched());

    // Calling itself what the person picked does not make it that.
    let impostor = Application::called("com.example.Okular", "Okular").unwrap();
    installed.add(impostor.clone());
    declared.declares(&impostor, [Kind::Pdf]);
    let mut picked = Chosen::untouched();
    picked.choose(Kind::Pdf, &okular());
    assert_eq!(
        WhatOpensWhat::on(&installed, &declared, &picked)
            .what_opens_a(Kind::Pdf)
            .unwrap()
            .application(),
        &okular()
    );

    // No agent verb is about what opens what: the four application verbs,
    // and nothing else.
    let verbs = application_verbs().unwrap();
    let mut names: Vec<&str> = verbs.all().map(|verb| verb.name()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "arrange_application",
            "close_application",
            "focus_application",
            "open_application"
        ]
    );
}

/// **A file of choices that does not read is refused whole** — a kind that is
/// not one, an identifier that is not one, a key nobody declared — every kind
/// opens in what the applications declare, and the next change is not written
/// over the person's broken file.
#[test]
fn a_file_of_choices_that_does_not_read_is_refused_whole_and_never_written_over() {
    let strings = in_english();
    let (installed, declared) = a_machine();
    for (number, (text, word, key)) in [
        (
            "format = 1\n\n[kinds]\npdf = \"org.kde.okular\"\ndocx = \"org.gnome.Papers\"\n",
            "applications.kept.not-understood",
            None,
        ),
        (
            "format = 1\n\n[kinds]\npdf = \"/usr/bin/okular\"\n",
            "applications.kept.not-understood",
            None,
        ),
        (
            "format = 1\ndefault = \"org.gnome.TextEditor\"\n\n[kinds]\npdf = \"org.kde.okular\"\n",
            "applications.kept.unknown-key",
            Some("default"),
        ),
        (
            "format = 2\n\n[kinds]\npdf = \"org.kde.okular\"\n",
            "applications.kept.another-format",
            None,
        ),
        (
            "[kinds]\npdf = \"org.kde.okular\"\n",
            "applications.kept.not-understood",
            None,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let at = a_folder_of_our_own(&format!("broken-{number}")).join(THE_FILE);
        std::fs::write(&at, text).unwrap();

        let (chosen, refused) = keeping::at_sign_in(&at);
        let refused = refused.unwrap_or_else(|| panic!("this read:\n{text}"));
        assert_eq!(refused.word().named(), word, "{text}");
        assert_eq!(refused.key(), key, "{text}");
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains(&at.display().to_string()), "{said}");

        // Nothing in it was honoured, not even the line that would have read.
        assert!(chosen.is_untouched(), "{text}");
        let opener = WhatOpensWhat::on(&installed, &declared, &chosen)
            .what_opens_a(Kind::Pdf)
            .unwrap();
        assert_eq!(opener.application(), &papers(), "{text}");
        assert_eq!(opener.because(), &Because::ItDeclaresIt);

        // And a change made now is refused, leaving the person's file as it is.
        let before = std::fs::read(&at).unwrap();
        let mut change = Chosen::untouched();
        change.choose(Kind::PngImage, &papers());
        let not_written = keeping::keep(&at, &change).unwrap_err();
        assert_eq!(not_written.word().named(), "applications.kept.not-replaced");
        assert_eq!(not_written.did_not_read(), Some(refused));
        assert_eq!(std::fs::read(&at).unwrap(), before);
    }

    // A relative path is nowhere a person's settings are.
    assert!(keeping::read(Path::new("what-opens-what.toml")).is_err());
    let (_, refused) = keeping::at_sign_in(Path::new("what-opens-what.toml"));
    assert!(refused.is_some());

    // Keeping the value that failed shows the refusal is about the file, not
    // the choice: into a folder with no broken file, it is written.
    let fresh = a_folder_of_our_own("fresh").join(THE_FILE);
    let mut change = Chosen::untouched();
    change.choose(Kind::PngImage, &papers());
    keeping::keep(&fresh, &change).expect("a change over no file is written");
    assert_eq!(keeping::read(&fresh).unwrap(), change);
}
