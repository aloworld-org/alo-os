//! Only a verb a person approved in a turn is handed on, and only once.
//!
//! The plan: the broker accepts a request *only for a verb a person approved in
//! a turn, which it verifies by a token the turn issues rather than trusting the
//! asker*. The asker here is always the agent's service — the door lets it in —
//! and what it sends is everything a compromised caller could try: a token for
//! a different verb, for a different argument, under a guessed key, renumbered,
//! backdated, from the future, and the same genuine token twice. Each is
//! refused, each leaves an entry, and nothing is carried out for any of them.

use std::time::{Duration, SystemTime};

use alo_broker::{
    Answer, ApprovingKey, Broker, Carrying, Door, Identity, LIFETIME, NotCarried, Request, Switch,
    SystemVerb,
};
use alo_record::{AtTheBroker, Happened, Record};

/// The user `alo-agentd` runs as on the image.
const THE_AGENT_SERVICE: u32 = 1000;

/// The bytes of the key the broker in these tests holds.
const THE_KEY: [u8; 32] = [42; 32];

/// Noon on a day in 2025.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Everything handed to be carried out, in order.
#[derive(Debug, Default)]
struct Carried(Vec<(SystemVerb, u64)>);

impl Carrying for Carried {
    fn carry(&mut self, verb: SystemVerb, approval: u64) -> Result<(), NotCarried> {
        self.0.push((verb, approval));
        Ok(())
    }
}

/// A broker at the agent's service's door, keeping its record in memory.
fn broker() -> Broker<Record, Carried> {
    Broker::new(
        Door::handed_to(THE_AGENT_SERVICE),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carried::default(),
    )
}

/// The printer the person approved adding.
fn the_printer() -> SystemVerb {
    SystemVerb::AddPrinter(Identity::of_what_was_reported(
        b"ipp://office-printer.local",
    ))
}

/// What the turn sends when approval `approval` of `verb` is spent at `at`.
fn approved(verb: SystemVerb, approval: u64, at: SystemTime) -> Vec<u8> {
    Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, at))
        .written()
        .into_bytes()
}

/// **One approval of one verb is carried out exactly once**, under the approval
/// the turn named — and the same request again is refused as spent.
#[test]
fn a_request_under_its_own_approval_is_carried_out_once() {
    let mut broker = broker();
    let request = approved(the_printer(), 7, noon());

    let first = broker.heard(Some(THE_AGENT_SERVICE), &request, noon());
    let again = broker.heard(
        Some(THE_AGENT_SERVICE),
        &request,
        noon() + Duration::from_secs(1),
    );

    assert_eq!(first, Answer::Carried);
    assert_eq!(again, Answer::Refused(AtTheBroker::ApprovalSpent));
    assert_eq!(broker.carrying().0, [(the_printer(), 7)]);
}

/// **Every verb on the list is carried out under its own approval**, so the
/// happy path is shown for the whole list and not one verb of it.
#[test]
fn every_verb_on_the_list_is_carried_out_under_its_own_approval() {
    let mut broker = broker();
    let every = SystemVerb::one_of_each(Identity::of_what_was_reported(b"a drive"), Switch::Off);
    for (verb, approval) in every.into_iter().zip(0..) {
        let answer = broker.heard(
            Some(THE_AGENT_SERVICE),
            &approved(verb, approval, noon()),
            noon(),
        );
        assert_eq!(answer, Answer::Carried, "{}", verb.name());
    }
    assert_eq!(broker.carrying().0.len(), every.len());
}

/// **Nothing that is not exactly an approval the turn issued is carried out.**
#[test]
fn a_request_under_no_genuine_approval_is_refused_and_nothing_is_carried() {
    let mut broker = broker();
    let printer = the_printer();
    let another_printer =
        SystemVerb::AddPrinter(Identity::of_what_was_reported(b"ipp://somebody-else.local"));
    let genuine = ApprovingKey::of(&THE_KEY).issue(&printer, 7, noon());
    let written = Request::of(printer, genuine).written();
    let words: Vec<&str> = written.split(' ').collect();
    let [name, argument, approval, issued, proof] = words.as_slice() else {
        return assert_eq!(words.len(), 5);
    };

    let not_approved: Vec<(&str, String)> = vec![
        (
            "a token for adding one printer, sent for adding another",
            Request::of(another_printer, genuine).written(),
        ),
        (
            "a token for adding a printer, sent for removing it",
            written.replacen("printers.add", "printers.remove", 1),
        ),
        (
            "a token issued under a key the broker does not hold",
            Request::of(
                printer,
                ApprovingKey::of(&[0; 32]).issue(&printer, 7, noon()),
            )
            .written(),
        ),
        (
            "a genuine token renumbered to another approval",
            format!("{name} {argument} 8 {issued} {proof}"),
        ),
        (
            "a genuine token redated",
            format!("{name} {argument} {approval} 1760000001 {proof}"),
        ),
        (
            "a proof of the right shape made up",
            format!("{name} {argument} {approval} {issued} {}", "0".repeat(64)),
        ),
        (
            "a switch's token sent for the other position",
            Request::of(
                SystemVerb::SetRadio(Switch::Off),
                ApprovingKey::of(&THE_KEY).issue(&SystemVerb::SetRadio(Switch::On), 9, noon()),
            )
            .written(),
        ),
    ];
    for (what, line) in &not_approved {
        assert_eq!(
            broker.heard(Some(THE_AGENT_SERVICE), line.as_bytes(), noon()),
            Answer::Refused(AtTheBroker::NotApproved),
            "{what}"
        );
    }

    let old = approved(printer, 10, noon() - LIFETIME - Duration::from_secs(1));
    let from_tomorrow = approved(printer, 11, noon() + Duration::from_secs(86_400));
    for (what, line) in [
        ("an old approval", old),
        ("one from tomorrow", from_tomorrow),
    ] {
        assert_eq!(
            broker.heard(Some(THE_AGENT_SERVICE), &line, noon()),
            Answer::Refused(AtTheBroker::ApprovalLapsed),
            "{what}"
        );
    }

    assert!(broker.carrying().0.is_empty(), "{:?}", broker.carrying().0);

    // **A forged token names no approval in the record**; a genuine one that
    // lapsed names the one it really was.
    let entries: Vec<&Happened> = broker
        .recording()
        .everything()
        .map(|e| e.happened())
        .collect();
    assert_eq!(entries.len(), not_approved.len() + 2);
    for entry in entries.iter().take(not_approved.len()) {
        assert_eq!(entry.from_approval(), None, "{entry:?}");
    }
    let lapsed: Vec<Option<u64>> = entries
        .iter()
        .skip(not_approved.len())
        .map(|entry| entry.from_approval())
        .collect();
    assert_eq!(lapsed, [Some(10), Some(11)]);

    // And the genuine token was never spent by any of that, so it still works.
    assert_eq!(
        broker.heard(Some(THE_AGENT_SERVICE), written.as_bytes(), noon()),
        Answer::Carried
    );
    assert_eq!(broker.carrying().0, [(printer, 7)]);
    assert!(broker.recording().everything().last().is_some_and(|last| {
        matches!(
            last.happened(),
            Happened::Brokered {
                refused: None,
                from_approval: Some(7),
                ..
            }
        )
    }));
}
