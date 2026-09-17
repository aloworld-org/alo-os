//! Every request, permitted or refused, is written down before it is answered.
//!
//! The plan: *every request, permitted or refused, is recorded through
//! `alo-record` before it answers*. So these tests watch the order and not only
//! the result: the record is shared with what carries a verb out, and a verb
//! handed on must find its own entry already there. Every kind of refusal is
//! sent once and must leave exactly one entry saying so. And a record that
//! cannot be written stops the broker — nothing is carried out without
//! evidence, then or afterwards.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_broker::{
    Answer, ApprovingKey, Broker, Carrying, Door, Identity, LIFETIME, NotCarried, NotKept,
    Recording, Request, SystemVerb,
};
use alo_record::{AtTheBroker, Entry, Happened, Line, Record};

/// The user `alo-agentd` runs as on the image.
const THE_AGENT_SERVICE: u32 = 1000;

/// The agent's own login, which is never the agent's service.
const THE_AGENT: u32 = 60989;

/// The bytes of the key the broker in these tests holds.
const THE_KEY: [u8; 32] = [5; 32];

/// Noon on a day in 2025.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A record two sides can see: the broker writes to it, the verb reads it.
#[derive(Debug, Default, Clone)]
struct Shared(Rc<RefCell<Record>>);

impl Recording for Shared {
    fn keep(&mut self, entry: Entry) -> Result<(), NotKept> {
        self.0.borrow_mut().keep(entry);
        Ok(())
    }
}

impl Shared {
    /// Every entry's `happened`, oldest first.
    fn happened(&self) -> Vec<Happened> {
        self.0
            .borrow()
            .everything()
            .map(|entry| entry.happened().clone())
            .collect()
    }
}

/// A verb that looks at the record at the moment it is carried out, and can be
/// told to fail.
#[derive(Debug)]
struct LooksFirst {
    /// The record the broker writes to.
    record: Shared,
    /// How many entries the record held each time a verb was carried out.
    entries_when_carried: Vec<usize>,
    /// Whether the machine could carry it out.
    works: bool,
}

impl Carrying for LooksFirst {
    fn carry(&mut self, _: SystemVerb, _: u64) -> Result<(), NotCarried> {
        self.entries_when_carried.push(self.record.happened().len());
        if self.works {
            Ok(())
        } else {
            Err(NotCarried("the print service did not answer".to_owned()))
        }
    }
}

/// A broker whose record the test can read while it runs.
fn broker(works: bool) -> (Broker<Shared, LooksFirst>, Shared) {
    let record = Shared::default();
    let carrying = LooksFirst {
        record: record.clone(),
        entries_when_carried: Vec::new(),
        works,
    };
    let broker = Broker::new(
        Door::handed_to(THE_AGENT_SERVICE),
        ApprovingKey::of(&THE_KEY),
        record.clone(),
        carrying,
    );
    (broker, record)
}

/// The verb every request here asks for.
fn eject() -> SystemVerb {
    SystemVerb::EjectDrive(Identity::of_what_was_reported(b"usb-SanDisk_4C53-0:0"))
}

/// What the turn sends when approval `approval` is spent at `at`.
fn approved(approval: u64, at: SystemTime) -> String {
    Request::of(
        eject(),
        ApprovingKey::of(&THE_KEY).issue(&eject(), approval, at),
    )
    .written()
}

/// **Every answer, of every kind, is one entry written before it was given** —
/// and a verb handed on finds its entry already written when it is carried out.
#[test]
fn every_request_permitted_or_refused_is_written_down_before_its_answer() {
    let (mut broker, record) = broker(true);
    let genuine = approved(1, noon());
    let forged = Request::of(
        eject(),
        ApprovingKey::of(&[9; 32]).issue(&eject(), 2, noon()),
    );
    let asked: Vec<(Option<u32>, Vec<u8>, Answer)> = vec![
        (
            Some(THE_AGENT),
            genuine.clone().into_bytes(),
            Answer::Refused(AtTheBroker::NotTheAgentService),
        ),
        (
            None,
            genuine.clone().into_bytes(),
            Answer::Refused(AtTheBroker::NotTheAgentService),
        ),
        (
            Some(THE_AGENT_SERVICE),
            b"exec /bin/sh".to_vec(),
            Answer::Refused(AtTheBroker::NotARequest),
        ),
        (
            Some(THE_AGENT_SERVICE),
            genuine
                .replacen("storage.eject", "storage.format", 1)
                .into_bytes(),
            Answer::Refused(AtTheBroker::NotOneOfItsVerbs),
        ),
        (
            Some(THE_AGENT_SERVICE),
            forged.written().into_bytes(),
            Answer::Refused(AtTheBroker::NotApproved),
        ),
        (
            Some(THE_AGENT_SERVICE),
            approved(3, noon() - LIFETIME - Duration::from_secs(5)).into_bytes(),
            Answer::Refused(AtTheBroker::ApprovalLapsed),
        ),
        (
            Some(THE_AGENT_SERVICE),
            genuine.clone().into_bytes(),
            Answer::Carried,
        ),
        (
            Some(THE_AGENT_SERVICE),
            genuine.into_bytes(),
            Answer::Refused(AtTheBroker::ApprovalSpent),
        ),
    ];

    for (at, (caller, line, expected)) in asked.iter().enumerate() {
        let answer = broker.heard(*caller, line, noon());
        assert_eq!(answer, *expected, "request {at}");
        assert_eq!(
            record.happened().len(),
            at + 1,
            "request {at} was answered without exactly one entry"
        );
    }

    let written = record.happened();
    assert!(
        written
            .iter()
            .all(|happened| matches!(happened, Happened::Brokered { .. })),
        "the broker wrote something that is not its own kind of entry: {written:?}"
    );
    let refusals: Vec<Option<AtTheBroker>> = written
        .iter()
        .map(|happened| match happened {
            Happened::Brokered { refused, .. } => *refused,
            _ => None,
        })
        .collect();
    assert_eq!(
        refusals,
        [
            Some(AtTheBroker::NotTheAgentService),
            Some(AtTheBroker::NotTheAgentService),
            Some(AtTheBroker::NotARequest),
            Some(AtTheBroker::NotOneOfItsVerbs),
            Some(AtTheBroker::NotApproved),
            Some(AtTheBroker::ApprovalLapsed),
            None,
            Some(AtTheBroker::ApprovalSpent),
        ]
    );

    // **The verb found its own entry already written**: six refusals, then the
    // entry handing it on, so seven entries at the moment it was carried out.
    assert_eq!(broker.carrying().entries_when_carried, [7]);

    // **A caller that is not the agent's service is not read**: its entry
    // names no verb, even though its line named one. And what a caller asked
    // for that is not on the list is kept as it asked, through the record's
    // own line.
    let verbs: Vec<Option<Line>> = written
        .iter()
        .map(|happened| match happened {
            Happened::Brokered { verb, .. } => verb.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(verbs.first(), Some(&None));
    assert_eq!(verbs.get(3), Some(&Some(Line::of("storage.format"))));
    assert_eq!(verbs.get(6), Some(&Some(Line::of("storage.eject"))));
}

/// **A verb the machine could not carry out is written down as that**, after the
/// entry that handed it on, and its approval stays spent.
#[test]
fn a_verb_the_machine_could_not_carry_out_is_written_down_and_not_tried_again() {
    let (mut broker, record) = broker(false);
    let request = approved(4, noon());

    assert_eq!(
        broker.heard(Some(THE_AGENT_SERVICE), request.as_bytes(), noon()),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(
        broker.heard(Some(THE_AGENT_SERVICE), request.as_bytes(), noon()),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    assert_eq!(broker.carrying().entries_when_carried, [1]);
    let written = record.happened();
    assert!(matches!(
        written.as_slice(),
        [
            Happened::Brokered {
                refused: None,
                from_approval: Some(4),
                ..
            },
            Happened::Brokered {
                refused: Some(AtTheBroker::NotCarried),
                from_approval: Some(4),
                ..
            },
            Happened::Brokered {
                refused: Some(AtTheBroker::ApprovalSpent),
                from_approval: Some(4),
                ..
            },
        ]
    ));
}

/// A record that stops accepting entries after a number of them.
#[derive(Debug)]
struct FillsUp {
    /// How many more entries it will take.
    room: usize,
}

impl Recording for FillsUp {
    fn keep(&mut self, _: Entry) -> Result<(), NotKept> {
        match self.room.checked_sub(1) {
            Some(left) => {
                self.room = left;
                Ok(())
            }
            None => Err(NotKept("the disk is full".to_owned())),
        }
    }
}

/// Carries out everything, and counts.
#[derive(Debug, Default)]
struct Counts(usize);

impl Carrying for Counts {
    fn carry(&mut self, _: SystemVerb, _: u64) -> Result<(), NotCarried> {
        self.0 += 1;
        Ok(())
    }
}

/// **A broker that cannot write down carries nothing out — then or ever
/// again**, whether the entry it could not write was a refusal or a request
/// being handed on.
#[test]
fn a_broker_that_cannot_write_down_carries_nothing_out_again() {
    let mut refusing_first = Broker::new(
        Door::handed_to(THE_AGENT_SERVICE),
        ApprovingKey::of(&THE_KEY),
        FillsUp { room: 0 },
        Counts::default(),
    );
    assert_eq!(
        refusing_first.heard(Some(THE_AGENT), b"", noon()),
        Answer::NotKept
    );
    assert_eq!(
        refusing_first.heard(
            Some(THE_AGENT_SERVICE),
            approved(5, noon()).as_bytes(),
            noon()
        ),
        Answer::NotKept
    );
    assert_eq!(refusing_first.carrying().0, 0);

    let mut handing_on_first = Broker::new(
        Door::handed_to(THE_AGENT_SERVICE),
        ApprovingKey::of(&THE_KEY),
        FillsUp { room: 0 },
        Counts::default(),
    );
    assert_eq!(
        handing_on_first.heard(
            Some(THE_AGENT_SERVICE),
            approved(6, noon()).as_bytes(),
            noon()
        ),
        Answer::NotKept
    );
    assert_eq!(
        handing_on_first.heard(
            Some(THE_AGENT_SERVICE),
            approved(7, noon()).as_bytes(),
            noon()
        ),
        Answer::NotKept
    );
    assert_eq!(handing_on_first.carrying().0, 0);
}
