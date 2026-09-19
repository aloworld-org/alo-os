//! Printer dispatch through the shared carriers retains the broker's refusals.

#![cfg(unix)]
#![expect(
    clippy::panic,
    reason = "an unexpected answer fails the acceptance test"
)]

#[path = "../../alo-printing/tests/serving/mod.rs"]
mod serving;
mod with_printers;

use std::time::{Duration, SystemTime};

use alo_broker::{Answer, ApprovingKey, Broker, Carrying, Door, Identity, Request, SystemVerb};
use alo_printing::PrintingService;
use alo_record::{AtTheBroker, Happened, Record};

use serving::{
    Answer as Served, DELETE_PRINTER, GET_DEVICES, GET_PRINTERS, a_printing_service, ok, status,
    with_device, with_printer,
};

/// The key shared by this test's turn and broker.
const KEY: [u8; 32] = [31; 32];

/// A queue reported by the printing service, never supplied as a verb argument.
const QUEUE: &str = "alo-canon-0f1e2d3c4b5a6978";

/// The discovered device whose digest an add approval names.
const DEVICE: &str = "ippusb://Canon%20G3560/?serial=X";

/// The same carriers the process assembles, against a private printing service.
fn broker(service: PrintingService) -> Broker<Record, impl Carrying> {
    Broker::new(
        Door::handed_to(1000),
        ApprovingKey::of(&KEY),
        Record::default(),
        with_printers::carriers(service),
    )
}

/// Only the digest of reported bytes crosses the broker's door.
fn identity(reported: &str) -> Identity {
    Identity::of_what_was_reported(reported.as_bytes())
}

/// The broker's recorded decisions, in order.
fn decisions(record: &Record) -> Vec<Option<AtTheBroker>> {
    record
        .everything()
        .map(|entry| match entry.happened() {
            Happened::Brokered { refused, .. } => *refused,
            other => panic!("unexpected record: {other:?}"),
        })
        .collect()
}

/// Existing callers retain both their constructor and their printer refusal.
#[test]
fn the_published_constructor_refuses_printers_until_a_service_is_supplied() {
    let mut carriers = with_printers::without_printers();
    assert!(carriers.printers().is_none());
    for verb in [
        SystemVerb::AddPrinter(identity(DEVICE)),
        SystemVerb::RemovePrinter(identity(QUEUE)),
        SystemVerb::SetDefaultPrinter(identity(QUEUE)),
    ] {
        assert!(carriers.carry(verb, 1).is_err());
    }
}

/// A valid token is tied to exactly one verb and device, expires, and is spent
/// once. Refused requests reach no printer service and every answer is recorded.
#[test]
fn composite_dispatch_refuses_wrong_stale_and_replayed_approvals_before_printing() {
    let serving = a_printing_service(|request| match request.code() {
        GET_PRINTERS => Served::Ipp(with_printer(ok(), QUEUE, DEVICE, "Canon")),
        _ => Served::Ipp(ok()),
    });
    let mut broker = broker(serving.service());
    let now = SystemTime::now();
    let remove = SystemVerb::RemovePrinter(identity(QUEUE));
    let approved = ApprovingKey::of(&KEY).issue(&remove, 1, now);
    let written = Request::of(remove, approved).written();
    let expired = now - alo_broker::LIFETIME - Duration::from_secs(1);
    for (request, refusal) in [
        (
            Request::of(
                SystemVerb::RemovePrinter(identity("another-queue")),
                approved,
            ),
            AtTheBroker::NotApproved,
        ),
        (
            Request::of(SystemVerb::SetDefaultPrinter(identity(QUEUE)), approved),
            AtTheBroker::NotApproved,
        ),
        (
            Request::of(remove, ApprovingKey::of(&[32; 32]).issue(&remove, 2, now)),
            AtTheBroker::NotApproved,
        ),
        (
            Request::of(remove, ApprovingKey::of(&KEY).issue(&remove, 3, expired)),
            AtTheBroker::ApprovalLapsed,
        ),
    ] {
        assert_eq!(
            broker.heard(Some(1000), request.written().as_bytes(), now),
            Answer::Refused(refusal)
        );
        assert!(serving.operations().is_empty());
    }
    assert_eq!(
        broker.heard(Some(1000), written.as_bytes(), now),
        Answer::Carried
    );
    assert_eq!(
        broker.heard(Some(1000), written.as_bytes(), now),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    assert_eq!(serving.operations(), [GET_PRINTERS, DELETE_PRINTER]);
    assert_eq!(
        decisions(broker.recording()),
        [
            Some(AtTheBroker::NotApproved),
            Some(AtTheBroker::NotApproved),
            Some(AtTheBroker::NotApproved),
            Some(AtTheBroker::ApprovalLapsed),
            None,
            Some(AtTheBroker::ApprovalSpent),
        ]
    );
}

/// A replacement with the same display name never acquires an old queue's approval.
#[test]
fn composite_dispatch_refuses_a_replacement_printer_with_the_same_name() {
    let serving =
        a_printing_service(|_| Served::Ipp(with_printer(ok(), "alo-replacement", DEVICE, "Canon")));
    let mut broker = broker(serving.service());
    let now = SystemTime::now();
    let remove = SystemVerb::RemovePrinter(identity(QUEUE));
    let request = Request::of(remove, ApprovingKey::of(&KEY).issue(&remove, 1, now));
    assert_eq!(
        broker.heard(Some(1000), request.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(serving.operations(), [GET_PRINTERS]);
    assert_eq!(
        decisions(broker.recording()),
        [None, Some(AtTheBroker::NotCarried)]
    );
}

/// Every printer operation's service refusal crosses the composite unchanged.
#[test]
fn composite_dispatch_records_each_printing_service_refusal() {
    let serving = a_printing_service(|request| match request.code() {
        GET_DEVICES => Served::Ipp(with_device(ok(), DEVICE, "Canon")),
        GET_PRINTERS => Served::Ipp(with_printer(ok(), QUEUE, DEVICE, "Canon")),
        _ => Served::Ipp(status(0x0403)),
    });
    let mut broker = broker(serving.service());
    for (approval, verb) in [
        (1, SystemVerb::AddPrinter(identity(DEVICE))),
        (2, SystemVerb::RemovePrinter(identity(QUEUE))),
        (3, SystemVerb::SetDefaultPrinter(identity(QUEUE))),
    ] {
        let now = SystemTime::now();
        let request = Request::of(verb, ApprovingKey::of(&KEY).issue(&verb, approval, now));
        assert_eq!(
            broker.heard(Some(1000), request.written().as_bytes(), now),
            Answer::Refused(AtTheBroker::NotCarried)
        );
    }
    assert_eq!(
        decisions(broker.recording()),
        [None, Some(AtTheBroker::NotCarried)].repeat(3)
    );
    assert_eq!(
        serving.operations(),
        [
            GET_DEVICES,
            serving::ADD_MODIFY_PRINTER,
            GET_PRINTERS,
            DELETE_PRINTER,
            GET_PRINTERS,
            serving::SET_DEFAULT,
        ]
    );
}

/// Supplying printers does not grant the broker authority to carry updates out.
#[test]
fn composite_dispatch_with_printers_still_refuses_both_update_verbs() {
    let serving = a_printing_service(|_| panic!("an update reached printing"));
    let mut broker = broker(serving.service());
    for (approval, verb) in [
        (1, SystemVerb::ApplyStagedUpdate(identity("staged"))),
        (2, SystemVerb::RollBack(identity("previous"))),
    ] {
        let now = SystemTime::now();
        let request = Request::of(verb, ApprovingKey::of(&KEY).issue(&verb, approval, now));
        assert_eq!(
            broker.heard(Some(1000), request.written().as_bytes(), now),
            Answer::Refused(AtTheBroker::NotCarried)
        );
    }
    assert!(serving.operations().is_empty());
    assert_eq!(
        decisions(broker.recording()),
        [None, Some(AtTheBroker::NotCarried)].repeat(2)
    );
}
