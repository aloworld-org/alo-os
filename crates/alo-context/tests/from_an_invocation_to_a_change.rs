//! The whole journey: a person presses the key with a document open, an agent
//! proposes a change to that document, they approve the sentence, and the
//! record afterwards says what happened and under which grant.
//!
//! Two of the gate's guarantees in `CLAUDE.md` cannot be demonstrated inside
//! one crate — *a verb cannot reach outside its grant* and *every execution and
//! every refusal leaves a record* — so this walks the real one, through
//! `alo-capability` and into `alo-record`, and asserts what the record kept.
//!
//! **The thing it is really asking** is the one a reader should be sceptical
//! about: that offering a context does not quietly widen anything. A person who
//! pressed the key while looking at Blender, with an invoice open and a
//! paragraph selected, has made exactly one grant — over the invoice — and the
//! record says so in the same words they read.
//!
//! Nothing here has been read by anybody and no file has been touched. There is
//! no compositor to offer a context and no daemon to hold a turn; this is the
//! portable model walked end to end.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Arg, Authorised, Call, Effect, Given, Grants, Proposal, ProposalError, Requires,
    Takes, Verb, Verbs,
};
use alo_context::{Context, Document, Focused, Selection, Turn};
use alo_record::{Asking, Entry, Line, Only, Record};
use alo_strings::{Strings, Word};

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn lasts here.
fn a_while() -> Duration {
    Duration::from_secs(5 * 60)
}

/// The document the person had open.
const MARCH: &str = "/home/anna/Invoices/march.pdf";

/// The paragraph they had selected, which is theirs and goes nowhere near the
/// record.
const SELECTED: &str = "Northstar have not paid the March invoice, account 41-8823.";

/// A change over a file, declared the way any adapter declares one.
fn verbs() -> Verbs {
    let verb = Verb::checked(
        "archive_file",
        Word::saying(
            "testing.archive-file.purpose",
            "move a file into the archive",
        ),
        Effect::Change,
        vec![Arg::taking(
            "file",
            Word::saying("testing.archive-file.file", "the file to archive"),
            Takes::Path,
        )],
        Requires::grants_over(["file"]),
        Word::saying("testing.archive-file.sentence", "archive {file}"),
    )
    .unwrap();
    let mut verbs = Verbs::default();
    verbs.declare(verb).unwrap();
    verbs
}

/// The capability model's words, this crate's, and the test verb's.
fn strings() -> Strings {
    let mut vocabulary = alo_context::context_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    for word in [
        Word::saying(
            "testing.archive-file.purpose",
            "move a file into the archive",
        ),
        Word::saying("testing.archive-file.file", "the file to archive"),
        Word::saying("testing.archive-file.sentence", "archive {file}"),
    ] {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    Strings::of(vocabulary)
}

/// What the person offered when they pressed the key: everything.
fn invoked() -> Context {
    Context::at_invocation(noon())
        .and_document(Document::open(MARCH).unwrap())
        .and_selection(Selection::of(SELECTED).unwrap())
        .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap())
}

/// A call over a path, from the verb above.
fn archiving(path: &str) -> Call {
    verbs()
        .call("archive_file", &[("file", Given::text(path))])
        .unwrap()
}

/// Why a proposal was refused, as a person would read it.
///
/// A `match` rather than a `let ... else` with a panic in it, so that a refusal
/// of the wrong shape fails the assertion it is part of and shows what it was,
/// instead of taking the test process down with a panic of its own.
fn why_not(refused: &ProposalError, strings: &Strings) -> String {
    match refused {
        ProposalError::NotGranted(why) => why.said(strings).into_text(),
        other => format!("{other:?}"),
    }
}

/// ADR 0001 §5 for a change: proposed, approved once, redeemed at the moment it
/// would run.
fn approving(call: &Call, grants: &Grants, agent: &alo_capability::Grantee) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, agent, grants, noon(), a_while()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

/// **The whole thing.** A document offered at invocation is a grant; a change
/// to it is proposed, approved and run; and the record says what ran, against
/// which grant, in the words the person read.
#[test]
fn a_document_offered_at_invocation_is_a_change_a_person_can_approve() {
    let strings = strings();
    let mut grants = Grants::default();
    let turn = Turn::beginning(invoked(), "@files", a_while(), &mut grants).unwrap();
    let agent = turn.grantee().clone();

    let call = archiving(MARCH);
    let authorised = approving(&call, &grants, &agent);

    // It ran against the grant the context made, and against no other.
    assert_eq!(authorised.against(), &[turn.granted().unwrap()]);
    assert_eq!(
        authorised.sentence(&strings).text(),
        "archive /home/anna/Invoices/march.pdf"
    );

    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));

    assert_eq!(record.len(), 1);
    let entry = record.everything().next().unwrap();
    assert_eq!(entry.agent().map(Line::as_str), Some("@files"));
    assert_eq!(
        entry.what().unwrap().sentence().as_str(),
        "archive /home/anna/Invoices/march.pdf"
    );
}

/// **What was in front of somebody never reaches the record.** The record keeps
/// what an agent *did*; writing down what was on the screen at every invocation
/// would build the watched-context log ADR 0001 §4 exists to forbid, one entry
/// at a time.
#[test]
fn what_was_on_the_screen_is_not_what_the_record_keeps() {
    let strings = strings();
    let mut grants = Grants::default();
    let turn = Turn::beginning(invoked(), "@files", a_while(), &mut grants).unwrap();
    let agent = turn.grantee().clone();

    let mut record = Record::default();
    record.keep(Entry::ran(
        &approving(&archiving(MARCH), &grants, &agent),
        &strings,
    ));

    let everything = format!("{record:?}");
    assert!(
        !everything.contains("Northstar"),
        "the selected text reached the record"
    );
    assert!(
        !everything.contains("41-8823"),
        "the selected text reached the record"
    );
    assert!(
        !everything.contains("untitled.blend"),
        "the window title reached the record"
    );
    assert!(
        !everything.contains("org.blender.Blender"),
        "the window in front reached the record"
    );
    // What is there is what was done, and the path it was done to — which the
    // person read in the sentence they approved.
    assert!(everything.contains("archive_file"));
    assert!(everything.contains("march.pdf"));
}

/// **A context grants the document and not the folder it sits in.** The file
/// beside it was never offered, so the call over it is refused before anybody
/// is asked to approve anything — and the refusal is recorded like every other.
#[test]
fn the_invoice_beside_the_one_that_was_open_is_refused() {
    let strings = strings();
    let mut grants = Grants::default();
    let turn = Turn::beginning(invoked(), "@files", a_while(), &mut grants).unwrap();
    let agent = turn.grantee().clone();

    let call = archiving("/home/anna/Invoices/april.pdf");
    let refused = Proposal::checked(&call, &agent, &grants, noon(), a_while()).unwrap_err();
    let said = why_not(&refused, &strings);
    assert!(said.contains("has not been granted"), "{said}");
    assert!(said.contains("april.pdf"), "{said}");

    let mut record = Record::default();
    record.keep(Entry::never_asked(&call, &agent, &said, &strings, noon()));
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );
}

/// **And a window in front of somebody grants nothing at all.** The mistake
/// this crate exists not to make, asserted through the capability model rather
/// than by reading a variant: the call never becomes a proposal.
#[test]
fn the_application_that_was_in_front_cannot_be_reached_either() {
    let mut grants = Grants::default();
    let turn = Turn::beginning(invoked(), "@files", a_while(), &mut grants).unwrap();
    assert!(!grants.permits(
        turn.grantee(),
        &alo_capability::Ask::application("org.blender.Blender"),
        noon()
    ));
    assert_eq!(grants.len(), 1, "a context granted more than the document");
}

/// **When the turn is over the document is not reachable**, twice over: the
/// grant has expired at that moment anyway, and ending the turn took it out of
/// the list before then.
#[test]
fn nothing_offered_at_an_invocation_outlives_the_turn() {
    let strings = strings();
    let mut grants = Grants::default();
    let turn = Turn::beginning(invoked(), "@files", a_while(), &mut grants).unwrap();
    let agent = turn.grantee().clone();
    let call = archiving(MARCH);
    let ends = turn.ends();

    // Expired: the grant runs from the invocation for exactly the turn.
    let too_late = Proposal::checked(&call, &agent, &grants, ends, a_while()).unwrap_err();
    assert!(matches!(too_late, ProposalError::NotGranted(_)));

    // And ended: a turn that finishes early does not leave it reachable for the
    // rest of its allotted time.
    assert!(turn.ending(&mut grants));
    let after = Proposal::checked(&call, &agent, &grants, noon(), a_while()).unwrap_err();
    let said = why_not(&after, &strings);
    assert!(said.contains("has not been granted"), "{said}");
}
