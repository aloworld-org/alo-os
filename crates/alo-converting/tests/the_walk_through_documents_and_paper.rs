//! Every sentence a person meets from *a file arrives* to *this cannot be
//! opened, and here is what would* — walked through the real verbs, and held to
//! the table in the report that records it.
//!
//! Task 5 of `docs/autonomy/v0-5-documents-and-paper-plan.md`. The four tasks
//! before it each end in sentences a person reads at the moment they are
//! already frustrated; each crate's own tests hold its sentences one at a time.
//! What none of them can show is **the sequence** — whether the sentences read
//! as one account when they arrive one after another, or as three crates
//! talking past each other. So this walks one person's afternoon:
//!
//! 1. a PDF arrives, and opens as it is;
//! 2. one of the owner's real Word documents arrives, and would be converted;
//! 3. the agent proposes converting it, and the person approves the sentence;
//! 4. the real service converts it through the pinned engine, and says what the
//!    copy could not carry;
//! 5. a real Pages document arrives from somebody on a Mac, and converts too —
//!    at the cost of the two families it is set in;
//! 6. a printer on the network is found and set up;
//! 7. the agent proposes printing the copy, the indicator shows the document
//!    leaving for the printer, and it prints;
//! 8. the printer stops for want of paper, the person puts some in, and it is
//!    ready again;
//! 9. a Word document that was cut short in the post arrives, and is explained;
//! 10. a photograph from a telephone arrives, and is explained — recognised for
//!     exactly what it is, with nothing here that opens one.
//!
//! # The table is the report's, and this test reads it
//!
//! The exact sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and
//! this test parses that table rather than a copy of it. A sentence that
//! changes without the table changing fails here, and so does a table edited to
//! say something the machine does not. A later change that moves a sentence
//! publishes the table again in a follow-up report and points [`THE_REPORT`] at
//! it, because a published report is never rewritten.
//!
//! **One substitution, and only one.** The folders a test can make live under a
//! temporary directory whose name changes every run, and the approval sentence
//! names a path. So the folder the walk runs in is written as `/home/anna` in
//! the table, and nothing else in any sentence is touched.
//!
//! # The walk skips itself where the engine cannot be run, and nowhere else
//!
//! Steps 4 and 5 run the pinned engine through `alo-convertd`, as
//! `converting_a_real_document.rs` does. It used to fail where the engine was
//! missing (ADR 0039); ADR 0063 amends that for a machine where the engine
//! **cannot** exist, and the walk asks
//! [`asking::this_machine_cannot_run_the_engine`] first and says what was
//! missing.
//!
//! **This test is the tenth**, and the one task 9's plan could not name.
//! `C:\dev\setup\which-tests-need-the-engine.sh` measured nine and the Mac
//! reported ten, and the difference was the measurement rather than the
//! machines: `cargo test` stops at the first test binary that fails, so the run
//! ended inside `converting_a_real_document.rs` and never reached this file.
//! Measured again with `--no-fail-fast` on 2026-09-22, the two agree at ten.
//!
//! Every other step is the real code against a printing service on this
//! machine's loopback that speaks the protocol; a real printer is owed, and the
//! report says so. The second test in this file reads only text and runs
//! everywhere.
//!
//! Steps 5 and 10 are real files rather than bytes assembled here: the Pages
//! document was saved by Pages on a Mac and the photograph written by a
//! telephone's own library, and both live in `alo-opening`'s `tests/files/`
//! with their provenance beside them.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod asking;

#[path = "../../alo-printing/tests/serving/mod.rs"]
mod serving;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach};
use alo_converting::{ConvertingService, Done, convert, converting_verbs, with_what_converts};
use alo_egress::{EgressPolicy, Indicator};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_opening::{ThisMachine, decide};
use alo_printing::{
    Condition, PRINT_DOCUMENT, Stopped, find, how_is, print, printing_verbs, set_up, set_up_said,
    this_machines_printer,
};
use alo_strings::{Said, Strings};

use serving::{
    Answer, GET_DEFAULT, GET_DEVICES, GET_PRINTER_ATTRIBUTES, PRINT_JOB, a_printer_that_is,
    a_printing_service, ok, the_default, with_device,
};

/// The report the walk is recorded in, relative to the repository.
const THE_REPORT: &str =
    "docs/autonomy/updates/a-pages-document-is-converted-and-a-photograph-is-explained.md";

/// The heading the table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// What the folder the walk runs in is written as in the table.
const THE_PERSONS_HOME: &str = "/home/anna";

/// Where the printer is, as it announced itself on the network.
const THE_PRINTER_AT: &str = "ipp://192.168.1.20/ipp/print";

/// What the printer calls itself.
const THE_PRINTER_IS: &str = "Brother HL-L2350DW series";

/// What the printing service reports as the reason a printer is out of paper.
const OUT_OF_PAPER: &str = "media-empty-error";

/// The printer state meaning stopped.
const STOPPED: i32 = 5;

/// The printer state meaning idle: ready for a document.
const IDLE: i32 = 3;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the approvals last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the person works with.
fn agent() -> Grantee {
    Grantee::named("@documents")
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A folder of the walk's own, resolved, standing in for a person's home.
fn a_home() -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let home = std::env::temp_dir().join(format!(
        "alo-walk-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&home));
    fs::create_dir_all(home.join("Downloads")).unwrap();
    fs::create_dir_all(home.join("Documents")).unwrap();
    OnThisMachine.real(&home).unwrap().into_path_buf()
}

/// One of the owner's three documents, as its bytes.
fn the_owners(named: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("documents")
            .join(named),
    )
    .unwrap()
}

/// One of the real files `alo-opening`'s recognition is measured against, as
/// its bytes — a Pages document saved by Pages on a Mac, a photograph written
/// by a telephone's own library. Where each came from is in
/// `crates/alo-opening/tests/files/README.md`, beside the files.
fn the_recognised(named: &str) -> Vec<u8> {
    let at = the_repository()
        .join("crates")
        .join("alo-opening")
        .join("tests")
        .join("files")
        .join(named);
    fs::read(&at).unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The converting service, started as its socket unit starts it, and stopped
/// when the walk ends.
struct Running {
    /// The service.
    child: Child,
    /// Its socket.
    socket: PathBuf,
}

impl Running {
    /// Start one, in a folder of its own inside `home`'s parent.
    fn started(beside: &Path) -> Self {
        let folder = beside.with_extension("service");
        drop(fs::remove_dir_all(&folder));
        let scratch = folder.join("scratch");
        fs::create_dir_all(&scratch).unwrap();
        let socket = folder.join("socket");
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

/// A call proposed by the agent under these grants, approved once, and
/// redeemed.
fn approved(call: &Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, &agent(), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

/// What the walk has met so far: each sentence with the moment it was met at.
struct Met {
    /// The folder standing in for the person's home, as the machine resolved it.
    home: String,
    /// Every sentence, in order.
    sentences: Vec<(&'static str, String)>,
}

impl Met {
    /// Nothing met yet, in this home.
    fn in_home(home: &Path) -> Self {
        Self {
            home: home.to_string_lossy().into_owned(),
            sentences: Vec::new(),
        }
    }

    /// These sentences, met at this moment. Each one came out of the
    /// vocabulary whole: no key, no unfilled gap.
    fn at(&mut self, moment: &'static str, said: impl IntoIterator<Item = Said>) {
        for sentence in said {
            assert!(!sentence.is_a_bug(), "{moment}: {sentence}");
            assert!(sentence.unfilled().is_empty(), "{moment}: {sentence}");
            let text = sentence.text().replace(&self.home, THE_PERSONS_HOME);
            self.sentences.push((moment, text));
        }
    }
}

/// A printing service with one printer on the network, which takes every
/// document, is out of paper while `out_of_paper` is set, and is ready when it
/// is not.
fn an_office_printer(out_of_paper: Arc<AtomicBool>) -> serving::Serving {
    a_printing_service(move |request| match request.code() {
        GET_DEVICES => Answer::Chunked(with_device(ok(), THE_PRINTER_AT, THE_PRINTER_IS)),
        GET_DEFAULT => Answer::Ipp(the_default(
            "alo-brother-hl-l2350dw-series-0123456789abcdef",
            THE_PRINTER_AT,
            THE_PRINTER_IS,
        )),
        GET_PRINTER_ATTRIBUTES if out_of_paper.load(Ordering::SeqCst) => {
            Answer::Ipp(a_printer_that_is(STOPPED, &[OUT_OF_PAPER], true))
        }
        GET_PRINTER_ATTRIBUTES => Answer::Ipp(a_printer_that_is(IDLE, &["none"], true)),
        _ => Answer::Ipp(ok()),
    })
}

/// The walk, carried out: every sentence a person meets, in order.
fn the_walk() -> Vec<(&'static str, String)> {
    let strings = in_english();
    let home = a_home();
    let downloads = home.join("Downloads");
    let documents = home.join("Documents");
    let mut met = Met::in_home(&home);

    let running = Running::started(&home);
    let converting_service = running.service();
    let machine = with_what_converts(ThisMachine::with_nothing(), &converting_service).unwrap();

    // 1. A PDF arrives, and opens as it is.
    let letter = downloads.join("letter.pdf");
    fs::write(
        &letter,
        b"%PDF-1.7\n1 0 obj << >> endobj\ntrailer << >>\n%%EOF\n",
    )
    .unwrap();
    let decided = decide(
        &mut File::open(&letter).unwrap(),
        OsStr::new("letter.pdf"),
        &machine,
    )
    .unwrap();
    met.at("A PDF arrives", decided.said(&strings));

    // 2. A Word document arrives.
    let report = downloads.join("report.docx");
    fs::write(&report, the_owners("sample.docx")).unwrap();
    let decided = decide(
        &mut File::open(&report).unwrap(),
        OsStr::new("report.docx"),
        &machine,
    )
    .unwrap();
    met.at("A Word document arrives", decided.said(&strings));

    // 3. The agent proposes converting it, under grants over both folders.
    let mut grants = Grants::default();
    for folder in [&downloads, &documents] {
        grants.grant(
            Grant::checked("@documents", Reach::Folder(folder.clone()), noon(), hour()).unwrap(),
        );
    }
    let call = converting_verbs()
        .unwrap()
        .call(
            "convert_document",
            &[
                ("file", Given::text(report.to_string_lossy().into_owned())),
                (
                    "into",
                    Given::text(documents.to_string_lossy().into_owned()),
                ),
            ],
        )
        .unwrap();
    assert!(call.waits_for_approval());
    met.at("The agent asks to convert it", [call.sentence(&strings)]);

    // 4. It is converted, and the copy says what it could not carry.
    let touching = Touching::of(approved(&call, &grants), &grants, &OnThisMachine, &strings)
        .expect("the grants cover the document and the folder");
    let done: Done =
        convert(&converting_service, touching, &grants).expect("the copy's own path is granted");
    let copy = done
        .converted()
        .unwrap_or_else(|| {
            panic!(
                "the owner's document was not converted: {:?}",
                done.said(&strings)
            )
        })
        .copy()
        .to_path_buf();
    met.at("It is converted", done.said(&strings));

    // 5. A Pages document arrives from somebody on a Mac, and converts too —
    // the seventh conversion, and the one whose cost is two families this
    // machine does not have.
    let notes = downloads.join("notes.pages");
    fs::write(&notes, the_recognised("document.pages")).unwrap();
    let decided = decide(
        &mut File::open(&notes).unwrap(),
        OsStr::new("notes.pages"),
        &machine,
    )
    .unwrap();
    met.at("A Pages document arrives", decided.said(&strings));

    let call = converting_verbs()
        .unwrap()
        .call(
            "convert_document",
            &[
                ("file", Given::text(notes.to_string_lossy().into_owned())),
                (
                    "into",
                    Given::text(documents.to_string_lossy().into_owned()),
                ),
            ],
        )
        .unwrap();
    assert!(call.waits_for_approval());
    met.at(
        "The agent asks to convert the Pages document",
        [call.sentence(&strings)],
    );
    let touching = Touching::of(approved(&call, &grants), &grants, &OnThisMachine, &strings)
        .expect("the grants cover the document and the folder");
    let notes_done: Done =
        convert(&converting_service, touching, &grants).expect("the copy's own path is granted");
    notes_done.converted().unwrap_or_else(|| {
        panic!(
            "the Pages document was not converted: {:?}",
            notes_done.said(&strings)
        )
    });
    met.at("The Pages document is converted", notes_done.said(&strings));

    // 6. A printer on the network is found, and the person sets it up.
    let out_of_paper = Arc::new(AtomicBool::new(false));
    let serving = an_office_printer(Arc::clone(&out_of_paper));
    let printing_service = serving.service();
    let found = find(&printing_service).unwrap();
    let [brother] = found.as_slice() else {
        panic!("one printer is in the office: {found:?}");
    };
    met.at("The list of printers is opened", [brother.said(&strings)]);
    met.at("The person chooses it", [brother.proposal(&strings)]);
    let printer = set_up(&printing_service, brother).unwrap();
    met.at("It is set up", [set_up_said(&printer, &strings)]);
    assert_eq!(
        this_machines_printer(&printing_service).unwrap().called(),
        printer.called()
    );

    // 7. The agent proposes printing the copy, which leaves for the printer.
    let call = printing_verbs()
        .unwrap()
        .call(
            PRINT_DOCUMENT,
            &[("document", Given::text(copy.to_string_lossy().into_owned()))],
        )
        .unwrap();
    assert!(call.waits_for_approval());
    met.at(
        "The agent asks to print the copy",
        [call.sentence(&strings)],
    );
    let authorised = approved(&call, &grants);
    let leaving = printer
        .reached()
        .leaving(authorised.under())
        .unwrap()
        .expect("a printer across the network is a departure");
    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, leaving, noon())
        .unwrap();
    met.at(
        "The indicator shows it leaving",
        indicator.showing().iter().map(|shown| shown.said(&strings)),
    );
    let printed = print(
        &printing_service,
        authorised,
        &printer,
        Some(&departing),
        &mut File::open(&copy).unwrap(),
    )
    .unwrap();
    assert!(indicator.ended(departing));
    met.at("It is printed", [printed.said(&strings)]);
    assert_eq!(
        serving.operations().last(),
        Some(&PRINT_JOB),
        "the document reached the printer"
    );

    // 8. The printer stops for want of paper; paper goes in; it is ready.
    out_of_paper.store(true, Ordering::SeqCst);
    let condition = how_is(&printing_service, &printer);
    assert_eq!(condition, Condition::Stopped(Stopped::OutOfPaper));
    met.at("The printer stops", [condition.said(&strings)]);
    out_of_paper.store(false, Ordering::SeqCst);
    let condition = how_is(&printing_service, &printer);
    assert_eq!(condition, Condition::Ready);
    met.at(
        "Paper goes in, and the printer is asked again",
        [condition.said(&strings)],
    );

    // 9. A Word document cut short on its way arrives, and is explained.
    let owners = the_owners("sample.docx");
    let cut = owners.get(..owners.len() / 2).unwrap();
    let minutes = downloads.join("minutes.docx");
    fs::write(&minutes, cut).unwrap();
    let decided = decide(
        &mut File::open(&minutes).unwrap(),
        OsStr::new("minutes.docx"),
        &machine,
    )
    .unwrap();
    met.at("A damaged Word document arrives", decided.said(&strings));

    // 10. A photograph from a telephone arrives, and is explained: this
    // machine knows exactly what it is and has nothing that opens one, which
    // is a different sentence from not recognising it.
    let photo = downloads.join("IMG_4021.heic");
    fs::write(&photo, the_recognised("photo.heic")).unwrap();
    let decided = decide(
        &mut File::open(&photo).unwrap(),
        OsStr::new("IMG_4021.heic"),
        &machine,
    )
    .unwrap();
    met.at("A photo from a telephone arrives", decided.said(&strings));

    drop(running);
    drop(fs::remove_dir_all(&home));
    drop(fs::remove_dir_all(home.with_extension("service")));
    met.sentences
}

/// The table under [`THE_WALK`] in [`THE_REPORT`].
fn the_table() -> Vec<(String, String)> {
    let report = fs::read_to_string(the_repository().join(THE_REPORT))
        .unwrap_or_else(|why| panic!("{THE_REPORT} could not be read: {why}"));
    let rows = rows_under(&report);
    assert!(
        !rows.is_empty(),
        "{THE_REPORT} has no table under {THE_WALK:?}"
    );
    rows
}

/// The rows of the table under [`THE_WALK`] in this text, before the next
/// heading — each as its moment and its sentence, without the header and the
/// line under it.
fn rows_under(report: &str) -> Vec<(String, String)> {
    report
        .lines()
        .skip_while(|line| line.trim() != THE_WALK)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|row| {
            let cells: Vec<&str> = row
                .trim()
                .trim_matches('|')
                .split(" | ")
                .map(str::trim)
                .collect();
            let [_, moment, sentence] = cells.as_slice() else {
                panic!("a row is a step, a moment and a sentence: {row}");
            };
            ((*moment).to_owned(), (*sentence).to_owned())
        })
        .collect()
}

/// The walk, written as the table's rows.
fn as_a_table(walk: &[(&str, String)]) -> String {
    walk.iter()
        .enumerate()
        .map(|(step, (moment, sentence))| format!("| {} | {moment} | {sentence} |", step + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The walk as the table's rows are read.
fn as_read(walk: &[(&str, String)]) -> Vec<(String, String)> {
    walk.iter()
        .map(|(moment, sentence)| ((*moment).to_owned(), sentence.clone()))
        .collect()
}

/// **The walk produces exactly the table in the report**, sentence by
/// sentence and in order — so no sentence along it changes without the table
/// changing.
#[test]
fn the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table() {
    if asking::this_machine_cannot_run_the_engine() {
        return;
    }
    let walk = the_walk();
    assert!(
        as_read(&walk) == the_table(),
        "the walk and the table in {THE_REPORT} differ. The walk reads:\n\n{}\n",
        as_a_table(&walk)
    );
}

/// **Only the walk's own table is read, and a changed sentence is a
/// difference.** Not a table under another heading, not rows after the table
/// ends, not a table under the heading after it, and not a heading with no
/// table at all. Held against text, so the check is shown refusing without
/// editing the report.
#[test]
fn only_the_walks_own_table_is_read_and_a_changed_sentence_is_a_difference() {
    let walk = [
        ("A PDF arrives", "This is a PDF document".to_owned()),
        ("It is printed", "Printed".to_owned()),
    ];
    let header = "| Step | Moment | What a person reads |\n|---|---|---|";
    let report = format!(
        "# A report\n\n## Another table\n\n{header}\n| 1 | Elsewhere | Not the walk |\n\n\
         {THE_WALK}\n\nThe walk.\n\n{header}\n{}\n\nAfter the table.\n\n\
         | 9 | A second table | Not the walk either |\n",
        as_a_table(&walk)
    );
    let walked = as_read(&walk);
    assert_eq!(rows_under(&report), walked);

    let mut reworded = walked.clone();
    if let Some(last) = reworded.last_mut() {
        last.1 = "It printed".to_owned();
    }
    assert_ne!(rows_under(&report), reworded, "a reworded sentence");

    let mut reordered = walked;
    reordered.reverse();
    assert_ne!(rows_under(&report), reordered, "a reordered walk");

    let no_table = format!(
        "# A report\n\n{THE_WALK}\n\nNothing yet.\n\n## Next\n\n{header}\n| 1 | Elsewhere | Not the walk |\n"
    );
    assert!(
        rows_under(&no_table).is_empty(),
        "a table under the next heading"
    );
    assert!(rows_under("# A report with no walk\n").is_empty());
}
