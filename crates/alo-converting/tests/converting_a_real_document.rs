//! `.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost, against the
//! owner's real documents and the real service running the pinned engine.
//!
//! Task 2 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, all the way
//! through: a declared verb, a grant over the folders, an approval redeemed,
//! the paths resolved, and then `alo-convertd` — the binary this crate builds,
//! started the way systemd starts it, with its listening socket on standard
//! input — converting through the engine at `alo_converting::engine::THE_ENGINE`.
//!
//! # A machine that cannot run the engine says so, and the rest still fails
//!
//! ADR 0039 said these must never skip: *a test that skips itself when the
//! engine is missing reports green on exactly the machines where nothing
//! converts.* ADR 0063 amends that for the one case the fleet then met — a
//! machine where the engine **cannot** exist, because it is an x86_64 build and
//! that machine is aarch64. Nine tests here failed there on every run, for a
//! reason nobody on that machine could fix, and a permanently red suite is one
//! people learn to read past.
//!
//! So each of those nine asks [`asking::this_machine_cannot_run_the_engine`]
//! first — which **runs** the engine rather than looking for its file — and
//! where it cannot, says what was missing and skips itself. Nothing else
//! changes: every assertion below is what it was, every other test in this file
//! still runs everywhere, and on a machine with the engine all nine still run
//! and still pass. An engine that starts and then converts badly is still a
//! loud failure, because the ask is only whether the engine runs at all.
//!
//! # The refusals are tested as carefully
//!
//! A name already in the folder, a copy nobody granted, a program named as a
//! document, no service at all, and a compression bomb: each ends with no copy
//! left behind and the original exactly as it was.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod asking;
mod making;

use std::collections::BTreeSet;
use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Given, Grant, Grantee, Grants, Proposal, Reach, Refused};
use alo_converting::recording::entry;
use alo_converting::{
    Carried, ConvertingService, Done, Field, FontName, Linked, NotCarried, NotConverted, Refusal,
    convert, converting_verbs,
};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_strings::{Strings, Vocabulary};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the approval last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn agent() -> Grantee {
    Grantee::named("@documents")
}

/// Every sentence a conversion can be told in, from every crate that words one.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_converting::declare_into(&mut vocabulary).unwrap();
    alo_opening::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, resolved.
fn a_folder(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-converting-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// One of the owner's three documents, as its bytes.
fn the_document(named: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("documents")
            .join(named),
    )
    .unwrap()
}

/// The one real Pages document, as its bytes.
///
/// It belongs to `alo-opening`, whose `tests/files/README.md` records where it
/// came from — saved by Pages 15.3.1 on a Mac, its heading set in Helvetica and
/// its body in HelveticaNeue on purpose, so that what a conversion substitutes
/// could be measured. It is read from there rather than copied here: two copies
/// of a 227 KB document would be two things to keep true.
fn the_pages_document() -> Vec<u8> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("this crate is two directories below the repository")
        .join("crates")
        .join("alo-opening")
        .join("tests")
        .join("files")
        .join("document.pages");
    fs::read(&at).unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The converting service, started as its socket unit starts it, and stopped
/// when the test ends.
struct Running {
    /// The service.
    child: Child,
    /// Its socket.
    socket: PathBuf,
}

impl Running {
    /// Start one, in a folder of its own.
    fn started() -> Self {
        let folder = a_folder("service");
        let socket = folder.join("socket");
        let scratch = folder.join("scratch");
        fs::create_dir_all(&scratch).unwrap();
        let listener = UnixListener::bind(&socket).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_alo-convertd"))
            .stdin(Stdio::from(OwnedFd::from(listener)))
            .env_clear()
            .env("TMPDIR", &scratch)
            .spawn()
            .expect("the converting service starts");
        Self { child, socket }
    }

    /// The service, as the verb reaches it.
    fn service(&self) -> ConvertingService {
        ConvertingService::at(&self.socket)
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _killed = self.child.kill();
        let _reaped = self.child.wait();
    }
}

/// Grants to the agent over these places, made at noon for an hour.
fn granting(reaches: Vec<Reach>) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(Grant::checked("@documents", reach, noon(), hour()).unwrap());
    }
    grants
}

/// `convert_document`, proposed, approved once, redeemed, resolved, and
/// carried out through `service`.
fn converting(
    service: &ConvertingService,
    file: &Path,
    into: &Path,
    grants: &Grants,
) -> Result<Done, Refused> {
    let strings = in_english();
    let call = converting_verbs()
        .unwrap()
        .call(
            "convert_document",
            &[
                ("file", Given::text(file.to_string_lossy().into_owned())),
                ("into", Given::text(into.to_string_lossy().into_owned())),
            ],
        )
        .unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(&call, &agent(), grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap();
    let touching = Touching::of(authorised, grants, &OnThisMachine, &strings)?;
    convert(service, touching, grants)
}

/// A document the owner sent, in a folder of its own, and an empty folder the
/// person chose for the copy — both granted.
fn sent(named: &str, bytes: &[u8]) -> (PathBuf, PathBuf, Grants) {
    let received = a_folder("received");
    let chosen = a_folder("chosen");
    let file = received.join(named);
    fs::write(&file, bytes).unwrap();
    let grants = granting(vec![Reach::Folder(received), Reach::Folder(chosen.clone())]);
    (file, chosen, grants)
}

/// Everything a copy did not carry, as a set.
fn lost(carried: &Carried) -> BTreeSet<NotCarried> {
    carried.not_carried().cloned().collect()
}

/// A font's name.
fn font(named: &str) -> NotCarried {
    NotCarried::FontSubstituted(FontName::from_document(named).unwrap())
}

/// Convert one of the owner's documents and hold what it lost to exactly the
/// list its README gives, the original to its bytes, and the record to what the
/// person was told.
fn a_real_document_loses_exactly(named: &str, expected: &[NotCarried]) {
    a_document_loses_exactly(named, &the_document(named), expected, "Garamond");
}

/// The same, for a document that is not one of the three in `tests/documents/`:
/// its bytes, and a family the sentences a person reads must name.
fn a_document_loses_exactly(
    named: &str,
    original: &[u8],
    expected: &[NotCarried],
    named_in_a_sentence: &str,
) {
    let running = Running::started();
    let (file, chosen, grants) = sent(named, original);

    let done = converting(&running.service(), &file, &chosen, &grants).unwrap();
    let strings = in_english();
    let converted = done.converted().unwrap_or_else(|| {
        panic!(
            "{named} was not converted: {:?}",
            done.said(&strings)
                .iter()
                .map(|said| said.text().to_owned())
                .collect::<Vec<_>>()
        )
    });

    let copy = chosen.join(Path::new(named).with_extension("pdf").file_name().unwrap());
    assert_eq!(converted.copy(), copy);
    assert!(fs::read(&copy).unwrap().starts_with(b"%PDF-"));
    assert_eq!(fs::read(&file).unwrap(), original, "the original changed");
    assert_eq!(
        lost(converted.carried()),
        expected.iter().cloned().collect::<BTreeSet<_>>()
    );

    let said: Vec<String> = done
        .said(&strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect();
    assert_eq!(said.len(), 2 + expected.len(), "{said:?}");
    assert!(
        said.iter().any(|line| line.contains(named_in_a_sentence)),
        "{said:?}"
    );

    let recorded = entry(&done, &strings);
    assert!(recorded.happened().ran());
    let told: Vec<String> = recorded.told().iter().map(ToString::to_string).collect();
    assert_eq!(
        told, said,
        "the record keeps something other than what was said"
    );
}

/// **A Word document is converted on this machine, and what the copy could not
/// carry is said by name**: Garamond, the date field, the picture linked from
/// the owner's own disk, and the comment.
#[test]
fn a_word_document_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly(
        "sample.docx",
        &[
            font("Garamond"),
            NotCarried::FieldFixed(Field::Date),
            NotCarried::LinkedNotFetched(Linked::Picture),
            NotCarried::Comments,
        ],
    );
}

/// **An Excel workbook**: Garamond, `=NOW()`, and the threaded comment.
#[test]
fn an_excel_workbook_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly(
        "sample.xlsx",
        &[
            font("Garamond"),
            NotCarried::FieldFixed(Field::TheCurrentMoment),
            NotCarried::Comments,
        ],
    );
}

/// **A PowerPoint presentation**: Garamond, and the modern comment.
#[test]
fn a_powerpoint_presentation_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly("sample.pptx", &[font("Garamond"), NotCarried::Comments]);
}

/// **An OpenDocument text document**: Garamond, the date field, the picture it
/// links rather than carries, and the comment.
///
/// # What these four hold and what they do not
///
/// The originals are **measured**: `crate::inventory::opendocument` reads them
/// here, on this gate, and `inventory::original`'s own tests hold each to what
/// it contains — including the two things only a real file said, the family that
/// lives on a spreadsheet's column and the presentation comment written in a
/// namespace of its own.
///
/// What is **not** measured here is the copy, because these tests need the
/// engine ADR 0039 pins and that engine is an x86_64 build, so on an aarch64
/// gate they join the four that already cannot run. The loss lists below are
/// therefore held by rules this repository has already measured three times
/// over on the Office documents beside them — a family the engine does not have
/// is substituted, a field that is not fixed is fixed, a link is not fetched, a
/// comment is not shown, a macro is not run — and by nothing about these three
/// files in particular. **They become measurements the first time this suite
/// runs on a machine with the engine**, and until then that is what they are.
#[test]
fn an_opendocument_text_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly(
        "sample.odt",
        &[
            font("Garamond"),
            NotCarried::FieldFixed(Field::Date),
            NotCarried::LinkedNotFetched(Linked::Picture),
            NotCarried::Comments,
        ],
    );
}

/// **An OpenDocument spreadsheet**: Garamond, `NOW()`, and the comment.
#[test]
fn an_opendocument_spreadsheet_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly(
        "sample.ods",
        &[
            font("Garamond"),
            NotCarried::FieldFixed(Field::TheCurrentMoment),
            NotCarried::Comments,
        ],
    );
}

/// **An OpenDocument presentation**: Garamond, and the comment.
#[test]
fn an_opendocument_presentation_is_converted_and_what_it_lost_is_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly("sample.odp", &[font("Garamond"), NotCarried::Comments]);
}

/// **A document carrying a macro library says the macros were not run.**
///
/// The one thing none of the other documents in this folder can show: the three
/// Office files deliberately carry no macro, and a `.docm` was left to a later
/// change. An OpenDocument keeps its macros in the open, as a module under
/// `Basic/`, where `alo-opening` already sees them — so this is the format that
/// can prove the sentence exists, against a real file rather than a constructed
/// one.
#[test]
fn a_document_with_a_macro_library_says_the_macros_were_not_run() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_real_document_loses_exactly(
        "sample-with-a-macro.odt",
        &[
            font("Garamond"),
            NotCarried::FieldFixed(Field::Date),
            NotCarried::LinkedNotFetched(Linked::Picture),
            NotCarried::Comments,
            NotCarried::Macros,
        ],
    );
}

/// **A Pages document is converted on this machine, and the two families it is
/// set in are named as substituted.**
///
/// The seventh conversion, and the first whose original this crate does not
/// read for itself: `inventory::read_from` sends it through a rendering the
/// engine makes, and `inventory::opendocument` — measured against four real
/// files — inventories that.
///
/// # What this test is, exactly
///
/// It is **the measurement of 2026-09-21**, run again on every gate that has
/// the engine. Taken on this machine, LibreOffice 26.2 on x86_64, against this
/// document held to its digest: the rendering names five families and text is
/// set in two of them, `Helvetica` and `HelveticaNeue`, which are the
/// document's own; the other three are the engine's substitutes and are
/// declared and used by nothing. The copy contains `NimbusSans-Bold` and
/// `NotoSans-Regular`, so neither family the document was set in survives, and
/// both are named.
///
/// **Exactly two findings and no others** is the half that matters. The
/// document has no field that changes, takes nothing from elsewhere — its one
/// picture is embedded — and carries no comment, no tracked change and no
/// macro, and a reader that counted the engine's substituted declarations
/// would report five losses of which three are not losses at all. That is why
/// the list is asserted whole rather than searched for Helvetica.
#[test]
fn a_pages_document_is_converted_and_the_families_it_is_set_in_are_named() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    a_document_loses_exactly(
        "notes.pages",
        &the_pages_document(),
        &[font("Helvetica"), font("HelveticaNeue")],
        "Helvetica",
    );
}

/// **A document that loses nothing says so, in its own sentence** — the one
/// case the owner's files cannot show, so the smallest Word document set in a
/// font the engine carries.
#[test]
fn a_document_that_loses_nothing_says_so() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    let running = Running::started();
    let (file, chosen, grants) = sent(
        "letter.docx",
        &making::a_word_document_in("Liberation Serif"),
    );
    let done = converting(&running.service(), &file, &chosen, &grants).unwrap();
    let converted = done.converted().expect("the letter was converted");
    assert_eq!(converted.carried(), &Carried::Everything);
    let said = done.said(&in_english());
    assert_eq!(said.len(), 2);
    assert!(
        said.last()
            .is_some_and(|said| said.text().starts_with("Nothing was lost"))
    );
}

/// **A file already called what the copy would be is never replaced**, and
/// nothing is converted.
#[test]
fn a_name_already_in_the_folder_is_refused_and_nothing_is_replaced() {
    let running = Running::started();
    let (file, chosen, grants) = sent("sample.docx", &the_document("sample.docx"));
    fs::write(chosen.join("sample.pdf"), b"the person's own").unwrap();
    let done = converting(&running.service(), &file, &chosen, &grants).unwrap();
    assert_eq!(
        done.not_converted(),
        Some(&NotConverted::NameTaken {
            copy: "sample.pdf".to_owned()
        })
    );
    assert_eq!(
        fs::read(chosen.join("sample.pdf")).unwrap(),
        b"the person's own"
    );
}

/// **A copy nobody granted is refused by the grants, before anything is opened
/// or created**: the folder itself is granted, and what would go inside it is
/// not.
#[test]
fn a_copy_nobody_granted_is_refused_before_anything_is_created() {
    let running = Running::started();
    let received = a_folder("received");
    let chosen = a_folder("chosen-but-not-inside");
    let file = received.join("sample.docx");
    fs::write(&file, the_document("sample.docx")).unwrap();
    let grants = granting(vec![Reach::Folder(received), Reach::File(chosen.clone())]);

    let refused = converting(&running.service(), &file, &chosen, &grants)
        .expect_err("a copy nobody granted was made");
    assert_eq!(refused.call().verb(), "convert_document");
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
}

/// **A program named as a document never reaches the engine**: the name's lie
/// is the finding, said first, and no copy is created — the service here is not
/// even running, and the answer is still about the file.
#[test]
fn a_program_named_as_a_document_never_reaches_the_service() {
    let nowhere = a_folder("no-service").join("socket");
    let (file, chosen, grants) = sent("invoice.docx", b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0");
    let done = converting(&ConvertingService::at(&nowhere), &file, &chosen, &grants).unwrap();
    assert!(matches!(
        done.not_converted(),
        Some(NotConverted::NotWhatItsNameSays(_))
    ));
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
}

/// **Without the service nothing is converted, nothing is sent anywhere else,
/// and the copy that was created for it is removed.**
#[test]
fn without_the_service_nothing_is_converted_and_no_copy_is_left() {
    let nowhere = a_folder("no-service").join("socket");
    let original = the_document("sample.xlsx");
    let (file, chosen, grants) = sent("sample.xlsx", &original);
    let done = converting(&ConvertingService::at(&nowhere), &file, &chosen, &grants).unwrap();
    assert_eq!(
        done.not_converted(),
        Some(&NotConverted::NothingHereConverts)
    );
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
    assert_eq!(fs::read(&file).unwrap(), original);
}

/// **A document that cannot be inventoried is not converted**: a compression
/// bomb is refused before the engine runs, and its copy is removed rather than
/// shown with an unstated cost.
#[test]
fn a_document_that_cannot_be_checked_is_not_converted() {
    let running = Running::started();
    let (file, chosen, grants) = sent("figures.docx", &making::a_compression_bomb());
    let done = converting(&running.service(), &file, &chosen, &grants).unwrap();
    assert_eq!(
        done.not_converted(),
        Some(&NotConverted::Refused(Refusal::OriginalNotChecked))
    );
    assert_eq!(fs::read_dir(&chosen).unwrap().count(), 0);
    let said = done.said(&in_english());
    assert!(
        !said
            .iter()
            .any(|said| said.text().contains("Nothing was lost"))
    );
}
