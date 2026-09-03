//! The whole journey an application verb takes, end to end: a call arrives, it
//! is proposed, one person approves it once, the grants are asked again, the
//! machine is asked whether the application is even here — and whatever
//! happened is written down.
//!
//! The crate's own tests take one door at a time. This is the other half: the
//! real verbs, the real grants and the real record, walked the way a daemon
//! would walk them. It crosses into `alo-record` on purpose, because
//! `CLAUDE.md`'s gate names two guarantees that no single crate can demonstrate
//! — **one approval causes exactly one execution**, and **every execution and
//! every refusal leaves a record**.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! opened a window: there is no compositor, and the acting half is Linux's.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_applications::{Application, Installed, Reaching, application_verbs, application_words};
use alo_capability::{
    Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach, Refused,
};
use alo_record::{Asking, Entry, Only, Record};
use alo_strings::Strings;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the questions here last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent everything here is granted to.
fn agent() -> Grantee {
    Grantee::named("@applications")
}

/// The words this machine reads: this crate's beside the capability model's,
/// which is the arrangement a shell has.
fn in_english() -> Strings {
    let mut vocabulary = application_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A grant to that agent over one application, made at noon, lasting an hour.
fn granting(application: &str) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@applications",
            Reach::Application(application.to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

/// This machine, with Blender on it.
fn this_machine() -> Installed {
    Installed::holding([Application::called("org.blender.Blender", "Blender").unwrap()])
}

/// A call of one of the four, naming one application — and, for the one that
/// asks for it, an arrangement.
fn calling(verb: &str, application: &str) -> Call {
    let verbs = application_verbs().unwrap();
    let given = [
        ("application", Given::text(application)),
        ("where", Given::text("left_half")),
    ];
    let takes_where = verbs
        .of(verb)
        .is_some_and(|declared| declared.arg("where").is_some());
    verbs
        .call(verb, if takes_where { &given } else { &given[..1] })
        .unwrap()
}

/// **The ordinary path, all of it.** A change is proposed with the sentence a
/// person reads, approved once, checked against the grants at the moment it
/// runs, found on this machine, and recorded with all four of ADR 0001 §7's
/// answers.
#[test]
fn a_change_travels_from_a_call_to_a_window_and_into_the_record() {
    let strings = in_english();
    let grants = granting("org.blender.Blender");
    let call = calling("close_application", "org.blender.Blender");

    // What the person is asked. The sentence says *ask*, not *close*.
    let mut approvals = Approvals::default();
    let proposal = Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap();
    assert_eq!(
        proposal.sentence(&strings).text(),
        "ask org.blender.Blender to close"
    );
    let id = approvals.propose(proposal);
    assert_eq!(approvals.waiting_at(noon()).count(), 1);

    // One approval, redeemed once, against the grants as they are now.
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert_eq!(authorised.from_approval(), Some(id));
    assert_eq!(authorised.against().len(), 1);

    // And the machine, which is the question only this crate can ask.
    let reaching = Reaching::of(authorised, &grants, &this_machine(), &strings).unwrap();
    assert_eq!(reaching.verb(), "close_application");
    assert_eq!(
        reaching
            .application("application")
            .map(Application::identifier),
        Some("org.blender.Blender")
    );

    // Whatever the compositor then did, what was authorised is written down.
    let mut record = Record::default();
    record.keep(Entry::ran(&reaching.into_authorised(), &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), Some(id.as_u64()));
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("close_application"));
    assert!(
        what.sentence().is("ask org.blender.Blender to close"),
        "the record kept a different sentence from the one that was approved: {}",
        what.sentence().as_str()
    );
}

/// **One approval, one execution.** The approval is spent redeeming it, and
/// there is no second authorisation to be had — the second attempt is not a
/// program that compiles, and what a daemon *can* try is answering the same
/// proposal twice.
#[test]
fn one_approval_is_worth_exactly_one_execution() {
    let grants = granting("org.blender.Blender");
    let call = calling("open_application", "org.blender.Blender");
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap());

    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert!(Reaching::of(authorised, &grants, &this_machine(), &in_english()).is_ok());

    // The question has been answered, and answering it again is refused.
    assert!(approvals.approve(id, noon()).is_err());
    assert!(approvals.of(id).is_none());
}

/// **A refusal leaves a record too**, and the words in it are the words the
/// person was shown rather than a second rendering. Both of this half's
/// refusals are here: the grants', and this crate's.
#[test]
fn every_refusal_is_written_down_in_the_words_the_person_read() {
    let strings = in_english();
    let mut record = Record::default();

    // The grants', after the grant is taken away between approval and
    // execution.
    let call = calling("focus_application", "org.blender.Blender");
    let mut grants = granting("org.blender.Blender");
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    grants.revoke_everything_for(&agent());
    let refused: Refused =
        Reaching::of(authorised, &grants, &this_machine(), &strings).unwrap_err();
    record.keep(Entry::refused(&refused, &agent(), &strings, noon()));

    // And this crate's, for an application that is granted and is not here.
    let grants = granting("com.example.Payroll");
    let call = calling("open_application", "com.example.Payroll");
    let mut approvals = Approvals::default();
    let id =
        approvals.propose(Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap());
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    let missing = Reaching::of(authorised, &grants, &this_machine(), &strings).unwrap_err();
    record.keep(Entry::refused(&missing, &agent(), &strings, noon()));

    assert_eq!(record.len(), 2);
    let refusals_only = Asking::anything().only(Only::Refusals);
    let stopped: Vec<_> = record.answering(&refusals_only).collect();
    assert_eq!(stopped.len(), 2);
    let said: Vec<String> = stopped
        .iter()
        .map(|entry| entry.happened().why_stopped().unwrap().as_str().to_owned())
        .collect();
    assert!(
        said.iter().any(|why| why.contains("has not been granted")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|why| why.contains("nothing installed on this machine")),
        "{said:?}"
    );
    // Neither of them ran, and the record says which verb was stopped.
    for entry in stopped {
        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().ran());
    }
}

/// **What a person approved and what the record keeps are one sentence, with
/// the arrangement in it as words** — and underneath it the record still holds
/// the name the model sent, which is the half of item 11a that a security
/// review reads rather than a person.
#[test]
fn an_arrangement_is_approved_as_words_and_recorded_as_both() {
    let strings = in_english();
    let grants = granting("org.blender.Blender");
    let call = calling("arrange_application", "org.blender.Blender");

    let mut approvals = Approvals::default();
    let proposal = Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap();
    assert_eq!(
        proposal.sentence(&strings).text(),
        "put org.blender.Blender on the left half of the screen",
        "the approval sentence is the one place an identifier must never appear"
    );
    let id = approvals.propose(proposal);
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();

    // The grant is over the application and over nothing else: an arrangement
    // is not a thing anybody grants.
    assert_eq!(authorised.against().len(), 1);

    let reaching = Reaching::of(authorised, &grants, &this_machine(), &strings).unwrap();
    let mut record = Record::default();
    record.keep(Entry::ran(&reaching.into_authorised(), &strings));

    let entry = record.everything().next().unwrap();
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("arrange_application"));
    assert!(
        what.sentence()
            .is("put org.blender.Blender on the left half of the screen"),
        "the record kept a different sentence from the one that was approved: {}",
        what.sentence().as_str()
    );
    // And the argument itself, kept by the name that was sent rather than by
    // the words somebody happened to be reading.
    assert!(
        what.arguments()
            .get("where")
            .is_some_and(|written| written.describe().is("left_half")),
        "the record lost the arrangement that was chosen"
    );
}

/// **A change cannot take the read door**, whichever verb it is. Every one of
/// the four is a change, so none of them ever answers inside a turn.
#[test]
fn nothing_here_runs_without_somebody_approving_it() {
    let grants = granting("org.blender.Blender");
    for verb in [
        "open_application",
        "focus_application",
        "close_application",
        "arrange_application",
    ] {
        let call = calling(verb, "org.blender.Blender");
        assert!(call.waits_for_approval(), "{verb}");
        let refused = Authorised::read(&call, &agent(), &grants, noon()).unwrap_err();
        let said = refused.said(&in_english());
        assert!(said.text().contains("approve"), "{verb}: {said}");
    }
}

/// A call for an application nobody granted never reaches this crate at all:
/// it is refused where it is proposed, and what is recorded is that nobody was
/// ever asked.
#[test]
fn an_application_nobody_granted_is_stopped_before_anybody_is_asked() {
    let strings = in_english();
    let grants = granting("org.blender.Blender");
    let call = calling("open_application", "com.example.Payroll");

    let why = Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap_err();
    let said = why.said(&strings);
    assert!(said.text().contains("has not been granted"), "{said}");
    assert!(said.text().contains("com.example.Payroll"), "{said}");

    let mut record = Record::default();
    record.keep(Entry::never_asked(
        &call,
        &agent(),
        said.text(),
        &strings,
        noon(),
    ));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().was_stopped());
    assert!(
        entry
            .happened()
            .what()
            .is_some_and(|what| what.verb().is("open_application"))
    );
}
