//! A day on a machine where the person declined the agent.
//!
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! makes three promises about that machine, and two of them are only checkable
//! from outside a single crate, which is why they are checked here:
//!
//! - **turning it off removes the agent's reach at once** — `alo-capability`;
//! - **nothing further is recorded as agent activity** — `alo-record`;
//! - **the record and the egress indicator stay**, because they are not AI
//!   features and somebody who declined an agent may want *more* than average
//!   to know what left their machine — `alo-egress` and `alo-record` together.
//!
//! The third is the one that makes the first two worth writing down. A machine
//! that answered *nothing happened* to every question would satisfy the first
//! two perfectly and would be useless to the person the ADR is about.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Agent, Arg, Authorised, Call, Effect, Given, Grant, Grantee, NotGranted, Reach, Refused,
    Requires, Takes, Verb, capability_words,
};
use alo_egress::{Destination, Errand, Indicator, OnItsOwn, Underway};
use alo_record::{Asking, Entry, Only, Record};
use alo_strings::{Strings, Word};

/// A fixed moment, so that a span is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grant and the spans in this file last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent whose day this is, until the moment it is not.
fn files() -> Grantee {
    Grantee::named("@files")
}

/// What the read this file follows does.
const LISTING: Word = Word::saying("testing.list-folder.purpose", "list what is in a folder");
/// What a person is shown while it happens.
const LISTING_SENTENCE: Word =
    Word::saying("testing.list-folder.sentence", "list what is in {folder}");
/// What it lists.
const LISTING_FOLDER: Word = Word::saying("testing.list-folder.folder", "the folder to list");

/// This crate's words and the fixture verb's, in one vocabulary — the
/// arrangement a shell has.
fn in_english() -> Strings {
    let mut vocabulary = capability_words().unwrap();
    for word in [LISTING, LISTING_SENTENCE, LISTING_FOLDER] {
        vocabulary.says(word.phrase().unwrap()).unwrap();
    }
    Strings::of(vocabulary)
}

/// The read this file follows.
fn listing_invoices() -> Call {
    let verb = Verb::checked(
        "list_folder",
        LISTING,
        Effect::Read,
        vec![Arg::taking("folder", LISTING_FOLDER, Takes::Path)],
        Requires::grants_over(["folder"]),
        LISTING_SENTENCE,
    )
    .unwrap();
    Call::of(&verb, &[("folder", Given::text("/home/anna/Invoices"))]).unwrap()
}

/// A machine with an agent that has been granted one folder.
fn a_working_machine() -> Agent {
    let mut machine = Agent::present();
    machine.grants_mut().unwrap().grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour() * 24,
        )
        .unwrap(),
    );
    machine
}

/// This machine fetching a model, with nobody having asked it to — shown on the
/// indicator, which is the only maker of the type an entry can be written from.
fn fetching_a_model(at: SystemTime) -> Underway {
    Indicator::default().beginning_on_its_own(
        OnItsOwn::for_(
            Errand::FetchingAModel,
            Destination::at("models.alo.example").unwrap(),
        ),
        at,
    )
}

/// **The whole of ADR 0009's portable half, as one day.**
///
/// The agent works in the morning. At one o'clock the person turns it off. The
/// machine goes on being a machine — it fetches a model of its own accord, and
/// the record says so — and nothing after one o'clock has an agent's name on it.
#[test]
fn the_day_a_person_turns_the_agent_off() {
    let strings = in_english();
    let mut record = Record::default();
    let mut machine = a_working_machine();

    // Morning: the agent reads a folder it was granted, and that is recorded.
    let read = Authorised::read(
        &listing_invoices(),
        &files(),
        machine.grants().unwrap(),
        noon(),
    )
    .unwrap();
    record.keep(Entry::ran(&read, &strings));

    // One o'clock: the person declines the agent. One grant ends.
    let declined_at = noon() + hour();
    assert_eq!(machine.declining(declined_at), 1);

    // The reach is gone at once, and the refusal is the one about this machine
    // rather than the one about a folder nobody picked.
    let why = machine
        .permitting(
            &files(),
            &alo_capability::Ask::path("/home/anna/Invoices/march.pdf"),
            declined_at,
        )
        .unwrap_err();
    assert!(matches!(why, NotGranted::NoAgent { .. }));
    assert!(
        why.said(&strings).text().contains("has no agent"),
        "{why:?}"
    );

    // The afternoon: the machine fetches a model on its own, and that is
    // recorded, because the record and the indicator are not AI features.
    let errand = fetching_a_model(declined_at + hour());
    record.keep(Entry::left_on_its_own(&errand));

    // **Nothing further is recorded as agent activity.**
    let afterwards = Asking::anything()
        .only(Only::ByAnAgent)
        .between(declined_at, declined_at + hour() * 12);
    assert_eq!(record.answering(&afterwards).count(), 0);

    // And no spelling of any name finds one either, which is the same question
    // asked the way somebody without `Only::ByAnAgent` would have had to ask it.
    for name in ["@files", "@mail", "alo", "alo OS", "system"] {
        let asking = Asking::anything()
            .by(name)
            .between(declined_at, declined_at + hour() * 12);
        assert_eq!(record.answering(&asking).count(), 0, "{name}");
    }

    // **The record stays**, and so does what it can answer about the machine
    // itself: what left, and which of it nobody asked for.
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        1
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::OnItsOwn))
            .count(),
        1
    );

    // The morning is still there. Turning the agent off ends its reach, not the
    // evidence of what it did while it had some.
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::ByAnAgent))
            .count(),
        1
    );
    assert_eq!(record.len(), 2);
}

/// **A call that arrives anyway is refused and written down.**
///
/// *Nothing further is recorded as agent activity* is a statement about a
/// machine with no agent doing nothing, and never a licence to stop recording.
/// If something does ask — a call still in flight when the person turned the
/// agent off, a bug, or somebody trying it on — that is the entry a person who
/// declined would most want in their record, and `CLAUDE.md`'s gate says every
/// refusal leaves one.
#[test]
fn a_call_that_arrives_on_a_machine_with_no_agent_is_refused_and_recorded() {
    let strings = in_english();
    let mut record = Record::default();
    let machine = Agent::declined();
    let call = listing_invoices();

    let why = machine
        .permitting(
            &files(),
            &alo_capability::Ask::path("/home/anna/Invoices"),
            noon(),
        )
        .unwrap_err();
    let refused = Refused::not_granted(call.clone(), why);
    record.keep(Entry::refused(&refused, &files(), &strings, noon()));

    // It is a refusal, it names what was tried, and it says the machine has no
    // agent rather than telling somebody to go and grant a folder.
    let refusals = Asking::anything().only(Only::Refusals);
    assert_eq!(record.answering(&refusals).count(), 1);
    let entry = record.answering(&refusals).next().unwrap();
    assert!(entry.agent().is_some_and(|whose| whose.is("@files")));
    assert!(refused.said(&strings).text().contains("has no agent"));

    // Nothing ran, and nothing left.
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Executions))
            .count(),
        0
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        0
    );
    assert!(!call.permitted_by(&alo_capability::Grants::default(), &files(), noon()));
}
