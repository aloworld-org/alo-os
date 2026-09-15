//! Printing is a verb, and a document sent to a printer across the network is a
//! document leaving this machine: nothing is sent there until the indicator
//! shows it.
//!
//! One clause of the acceptance for *A printer is found, set up, and says what
//! is wrong with it* in `docs/autonomy/v0-5-documents-and-paper-plan.md`: the
//! whole journey from a declared verb, through a grant and one approval, to the
//! bytes the printing service received — and every way it is stopped.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod serving;

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Authorised, Effect, Given, Grant, GrantId, Grantee, Grants, Proposal, Reach,
    Requires, Takes, Verb, Verbs,
};
use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_printing::ipp::Group;
use alo_printing::{
    NotPrintable, NotPrinted, PRINT_DOCUMENT, Printer, PrintingService, Stopped, print,
    printing_verbs, this_machines_printer,
};
use alo_strings::Strings;

use serving::{
    Answer, GET_DEFAULT, GET_PRINTER_ATTRIBUTES, PRINT_JOB, Serving, a_printer_that_is,
    a_printing_service, ok, status, the_default,
};

/// A PDF, as small as one can be and still be one.
const A_LETTER: &[u8] = b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n";

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long grants and approvals last here.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// The machine's one vocabulary, as a shell holds it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's words"))
}

/// A folder of this test's own, resolved, holding one file of these bytes.
fn a_document(named: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-printing-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).expect("a folder of our own");
    let folder = fs::canonicalize(&folder).expect("a resolved folder");
    let file = folder.join(named);
    fs::write(&file, bytes).expect("the document");
    (folder, file)
}

/// A grant to `@files` over this folder, and the id it was given.
fn granting(folder: &Path) -> (Grants, GrantId) {
    let mut grants = Grants::default();
    let id = grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(folder.to_path_buf()),
            noon(),
            hour(),
        )
        .expect("a grant"),
    );
    (grants, id)
}

/// A call on this list, proposed, approved once and redeemed.
fn approved(verbs: &Verbs, verb: &str, file: &Path, grants: &Grants) -> Authorised {
    let call = verbs
        .call(
            verb,
            &[("document", Given::text(file.to_string_lossy().into_owned()))],
        )
        .expect("the call forms");
    let mut approvals = Approvals::default();
    let id = approvals
        .propose(Proposal::checked(&call, &files(), grants, noon(), hour()).expect("a proposal"));
    approvals
        .approve(id, noon())
        .expect("approved")
        .redeem(grants, noon())
        .expect("redeemed")
}

/// A service with a printer at `device` set up as the one this machine prints
/// on, answering a print with `printing`.
fn a_service_printing_on(device: &'static str, printing: fn() -> Answer) -> Serving {
    a_printing_service(move |request| match request.code() {
        GET_DEFAULT => Answer::Ipp(the_default(
            "alo-brother-hl-l2350dw-series-0123456789abcdef",
            device,
            "Brother HL-L2350DW series",
        )),
        PRINT_JOB => printing(),
        _ => Answer::Ipp(a_printer_that_is(5, &["media-empty-error"], true)),
    })
}

/// The printer the service keeps.
fn the_printer(service: &PrintingService) -> Printer {
    this_machines_printer(service).expect("a printer is set up")
}

/// **Printing is a verb that waits for one approval of its sentence**, under a
/// grant over the document — it is never a read that runs inside the turn.
#[test]
fn printing_is_a_verb_that_waits_for_one_approval_under_a_grant() {
    let (folder, file) = a_document("letter.pdf", A_LETTER);
    let (grants, _) = granting(&folder);
    let verbs = printing_verbs().expect("the verb declares");
    let call = verbs
        .call(
            PRINT_DOCUMENT,
            &[("document", Given::text(file.to_string_lossy().into_owned()))],
        )
        .expect("the call forms");
    assert!(call.waits_for_approval());
    assert_eq!(
        call.sentence(&in_english()).text(),
        format!("print {} on this machine's printer", file.display())
    );
    assert!(Authorised::read(&call, &files(), &grants, noon()).is_err());

    let (elsewhere, _) = a_document("other.pdf", A_LETTER);
    let (other_grants, _) = granting(&elsewhere);
    assert!(Proposal::checked(&call, &files(), &other_grants, noon(), hour()).is_err());
}

/// **A document to a printer across the network lights the indicator before it
/// is sent**, as an agent sending something to that printer — and then the
/// printing service receives exactly the document, named as what its bytes are.
#[test]
fn a_document_to_a_printer_across_the_network_is_shown_leaving_before_it_is_sent() {
    let (folder, file) = a_document("letter.pdf", A_LETTER);
    let (grants, _) = granting(&folder);
    let serving = a_service_printing_on("ipp://192.168.1.20/ipp/print", || Answer::Ipp(ok()));
    let service = serving.service();
    let printer = the_printer(&service);
    let authorised = approved(
        &printing_verbs().expect("the verb"),
        PRINT_DOCUMENT,
        &file,
        &grants,
    );

    let leaving = printer
        .reached()
        .leaving(authorised.under())
        .expect("a destination")
        .expect("across the network is a departure");
    assert_eq!(leaving.why(), Why::Sending);
    let mut indicator = Indicator::default();
    let departing = indicator
        .beginning(&EgressPolicy::Anywhere, leaving, noon())
        .expect("nothing forbids it");
    assert!(!indicator.is_quiet());
    assert_eq!(
        indicator
            .showing()
            .first()
            .expect("a line on the indicator")
            .said(&in_english())
            .text(),
        "@files is sending something to 192.168.1.20"
    );

    let printed = print(
        &service,
        authorised,
        &printer,
        Some(&departing),
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect("printed");
    assert_eq!(
        printed.said(&in_english()).text(),
        "The document has been sent to the printer"
    );
    indicator.ended(departing);
    assert!(indicator.is_quiet());

    let heard = serving.heard();
    let job = heard.last().expect("the print");
    assert_eq!(job.code(), PRINT_JOB);
    assert_eq!(job.data(), A_LETTER);
    assert_eq!(
        job.attribute(Group::Operation, "document-format")
            .and_then(|format| format.text()),
        Some("application/pdf")
    );
}

/// **Without the indicator, nothing is sent across the network**: no departure,
/// a departure to another place, and one under another agent are all refused,
/// and the printing service is never handed the document.
#[test]
fn without_the_indicator_nothing_is_sent_to_a_printer_across_the_network() {
    let (folder, file) = a_document("letter.pdf", A_LETTER);
    let (grants, _) = granting(&folder);
    let serving = a_service_printing_on("ipp://192.168.1.20/ipp/print", || Answer::Ipp(ok()));
    let service = serving.service();
    let printer = the_printer(&service);
    let verbs = printing_verbs().expect("the verb");
    let mut indicator = Indicator::default();

    let elsewhere = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(
                &files(),
                Why::Sending,
                Destination::at("192.168.1.99").expect("a host"),
            ),
            noon(),
        )
        .expect("shown");
    let someone_else = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(
                &Grantee::named("@mail"),
                Why::Sending,
                Destination::at("192.168.1.20").expect("a host"),
            ),
            noon(),
        )
        .expect("shown");
    let fetching = indicator
        .beginning(
            &EgressPolicy::Anywhere,
            Leaving::because(
                &files(),
                Why::Fetching,
                Destination::at("192.168.1.20").expect("a host"),
            ),
            noon(),
        )
        .expect("shown");

    for departing in [None, Some(&elsewhere), Some(&someone_else), Some(&fetching)] {
        let refused = print(
            &service,
            approved(&verbs, PRINT_DOCUMENT, &file, &grants),
            &printer,
            departing,
            &mut File::open(&file).expect("the document, opened"),
        )
        .expect_err("not shown leaving for this print");
        assert_eq!(refused, NotPrinted::NotShownLeaving);
        let said = refused.said(&in_english());
        assert!(
            said.first()
                .is_some_and(|said| said.text().contains("had not been shown as leaving")),
            "{said:?}"
        );
    }
    assert!(!serving.operations().contains(&PRINT_JOB));
}

/// **An organisation that lets nothing leave lets no agent print across the
/// network** — the indicator refuses the departure, so there is nothing to
/// print with.
#[test]
fn a_machine_where_nothing_may_leave_lets_no_agent_print_across_the_network() {
    let serving = a_service_printing_on("ipp://192.168.1.20/ipp/print", || Answer::Ipp(ok()));
    let printer = the_printer(&serving.service());
    let leaving = printer
        .reached()
        .leaving(&files())
        .expect("a destination")
        .expect("a departure");
    let mut indicator = Indicator::default();
    assert!(
        indicator
            .beginning(&EgressPolicy::NothingLeaves, leaving, noon())
            .is_err()
    );
    assert!(indicator.is_quiet());
}

/// **A printer on this machine is not a departure**, so it prints with nothing
/// on the indicator.
#[test]
fn a_printer_on_this_machine_prints_with_nothing_leaving() {
    let (folder, file) = a_document("note.txt", b"Dear Anna,\nthe invoice is attached.\n");
    let (grants, _) = granting(&folder);
    let serving = a_service_printing_on("ipp://localhost:60000/ipp/print", || Answer::Ipp(ok()));
    let service = serving.service();
    let printer = the_printer(&service);
    assert_eq!(
        printer.reached().leaving(&files()).expect("no destination"),
        None
    );

    print(
        &service,
        approved(
            &printing_verbs().expect("the verb"),
            PRINT_DOCUMENT,
            &file,
            &grants,
        ),
        &printer,
        None,
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect("printed");
    let heard = serving.heard();
    let job = heard.last().expect("the print");
    assert_eq!(
        job.attribute(Group::Operation, "document-format")
            .and_then(|format| format.text()),
        Some("text/plain")
    );
}

/// **A program named as a PDF never reaches a printer**, and the person reads
/// what the file really is before they read that nothing was printed.
#[test]
fn a_program_named_as_a_document_never_reaches_a_printer() {
    let (folder, file) = a_document("invoice.pdf", b"\x7fELF\x02\x01\x01\0program");
    let (grants, _) = granting(&folder);
    let serving = a_service_printing_on("ipp://localhost:60000/ipp/print", || Answer::Ipp(ok()));
    let service = serving.service();
    let printer = the_printer(&service);

    let refused = print(
        &service,
        approved(
            &printing_verbs().expect("the verb"),
            PRINT_DOCUMENT,
            &file,
            &grants,
        ),
        &printer,
        None,
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect_err("a program is not printed");
    assert!(matches!(
        refused,
        NotPrinted::NotPrintable(NotPrintable::NotWhatItsNameSays(_))
    ));
    let said: Vec<String> = refused
        .said(&in_english())
        .into_iter()
        .map(|said| said.into_text())
        .collect();
    assert_eq!(
        said,
        [
            "This file is named as a PDF document, but it is a program",
            "Nothing was printed, because this file is not what its name says",
        ]
    );
    assert!(!serving.operations().contains(&PRINT_JOB));
}

/// **An approval for something else prints nothing**, and a grant revoked
/// after approval stops the print at the moment it would have run.
#[test]
fn an_approval_for_something_else_or_under_a_revoked_grant_prints_nothing() {
    let (folder, file) = a_document("letter.pdf", A_LETTER);
    let (mut grants, id) = granting(&folder);
    let serving = a_service_printing_on("ipp://localhost:60000/ipp/print", || Answer::Ipp(ok()));
    let service = serving.service();
    let printer = the_printer(&service);

    let mut other = Verbs::default();
    other
        .declare(
            Verb::checked(
                "shred_document",
                alo_printing::words::VERB_PURPOSE,
                Effect::Change,
                vec![Arg::taking(
                    "document",
                    alo_printing::words::VERB_DOCUMENT,
                    Takes::Path,
                )],
                Requires::grants_over(["document"]),
                alo_printing::words::VERB_SENTENCE,
            )
            .expect("a verb"),
        )
        .expect("declared");
    let refused = print(
        &service,
        approved(&other, "shred_document", &file, &grants),
        &printer,
        None,
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect_err("an approval is for one sentence");
    assert_eq!(refused, NotPrinted::NotPrinting);

    let call = printing_verbs()
        .expect("the verb")
        .call(
            PRINT_DOCUMENT,
            &[("document", Given::text(file.to_string_lossy().into_owned()))],
        )
        .expect("the call forms");
    let mut approvals = Approvals::default();
    let proposal = approvals
        .propose(Proposal::checked(&call, &files(), &grants, noon(), hour()).expect("a proposal"));
    let approval = approvals.approve(proposal, noon()).expect("approved");
    assert!(grants.revoke(id));
    assert!(approval.redeem(&grants, noon()).is_err());
    assert!(!serving.operations().contains(&PRINT_JOB));
}

/// **A printer that does not take the document says which of the five is
/// wrong**: a kind it refuses is *refused*, and a printer not taking anything
/// because it is out of paper says *out of paper*.
#[test]
fn a_printer_that_does_not_take_the_document_says_which_thing_is_wrong() {
    let (folder, file) = a_document("letter.pdf", A_LETTER);
    let (grants, _) = granting(&folder);
    let verbs = printing_verbs().expect("the verb");

    let refusing = a_service_printing_on("ipp://localhost:60000/ipp/print", || {
        Answer::Ipp(status(0x040a))
    });
    let service = refusing.service();
    let printer = the_printer(&service);
    let refused = print(
        &service,
        approved(&verbs, PRINT_DOCUMENT, &file, &grants),
        &printer,
        None,
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect_err("refused");
    assert_eq!(refused, NotPrinted::Stopped(Stopped::RefusedTheJob));

    let empty = a_service_printing_on("ipp://localhost:60000/ipp/print", || {
        Answer::Ipp(status(0x0506))
    });
    let service = empty.service();
    let printer = the_printer(&service);
    let refused = print(
        &service,
        approved(&verbs, PRINT_DOCUMENT, &file, &grants),
        &printer,
        None,
        &mut File::open(&file).expect("the document, opened"),
    )
    .expect_err("not taking documents");
    assert_eq!(refused, NotPrinted::Stopped(Stopped::OutOfPaper));
    assert_eq!(
        empty.operations(),
        [GET_DEFAULT, PRINT_JOB, GET_PRINTER_ATTRIBUTES]
    );
    assert!(
        refused
            .said(&in_english())
            .first()
            .is_some_and(|said| said.text().starts_with("The printer is out of paper"))
    );
}
