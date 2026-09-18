//! The printers' three verbs change exactly the printer a person approved, as
//! the printing service reports it now — and no printer at all when that one is
//! not there, when two answer to it, or when the verb is not a printer's.
//!
//! Task 2 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *the
//! broker's printer verbs take `alo-printing`'s own types, and configure the
//! rented print system without a free-form URI or driver name.* The carrier is
//! handed thirty-two bytes and nothing it could act on, so what is tested here
//! is that it acts only on the one printer those bytes digest from — through
//! the door, once, with the record written first.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::time::SystemTime;

use alo_broker::{
    Answer, ApprovingKey, Broker, Carrying, Door, Identity, NotCarried, Request, Switch, SystemVerb,
};
use alo_brokerd::{PrintService, Printers, Reported};
use alo_record::{AtTheBroker, Happened, Record};

/// A printer, by what the printing service reports it under.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Named(&'static str);

impl Reported for Named {
    fn as_reported(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// A printing service that reports what it is told to and remembers every
/// change it was asked for.
#[derive(Debug, Default)]
struct Office {
    /// What it finds.
    found: Vec<Named>,
    /// What is set up.
    set_up: Vec<Named>,
    /// Whether it says no to every change.
    refuses: bool,
    /// Every change, in order.
    changed: RefCell<Vec<String>>,
}

impl Office {
    /// A change, remembered — or refused.
    fn change(&self, what: &str, printer: &Named) -> Result<(), NotCarried> {
        if self.refuses {
            return Err(NotCarried("not permitted".to_owned()));
        }
        self.changed
            .borrow_mut()
            .push(format!("{what} {}", printer.0));
        Ok(())
    }
}

impl PrintService for Office {
    type Found = Named;
    type SetUp = Named;

    fn found(&self) -> Result<Vec<Named>, NotCarried> {
        Ok(self.found.clone())
    }

    fn set_up(&self, found: &Named) -> Result<(), NotCarried> {
        self.change("set up", found)
    }

    fn set_up_here(&self) -> Result<Vec<Named>, NotCarried> {
        Ok(self.set_up.clone())
    }

    fn remove(&self, printer: &Named) -> Result<(), NotCarried> {
        self.change("remove", printer)
    }

    fn make_default(&self, printer: &Named) -> Result<(), NotCarried> {
        self.change("make default", printer)
    }
}

/// An office with two printers nearby and two set up.
fn an_office() -> Office {
    Office {
        found: vec![
            Named("dnssd://Brother%20HL-L2350DW._ipp._tcp.local/"),
            Named("ippusb://Canon%20G3560/"),
        ],
        set_up: vec![
            Named("alo-brother-hl-l2350dw-series-5b1f0c3e9a7d2468"),
            Named("alo-canon-pixma-g3560-0f1e2d3c4b5a6978"),
        ],
        ..Office::default()
    }
}

/// The identity of what was reported.
fn identity(reported: &str) -> Identity {
    Identity::of_what_was_reported(reported.as_bytes())
}

/// **Each verb changes exactly the printer whose identity was approved**, and
/// asks nothing else of the service.
#[test]
fn each_verb_changes_exactly_the_printer_approved() {
    let mut printers = Printers::against(an_office());
    for verb in [
        SystemVerb::AddPrinter(identity("ippusb://Canon%20G3560/")),
        SystemVerb::RemovePrinter(identity("alo-brother-hl-l2350dw-series-5b1f0c3e9a7d2468")),
        SystemVerb::SetDefaultPrinter(identity("alo-canon-pixma-g3560-0f1e2d3c4b5a6978")),
    ] {
        assert_eq!(printers.carry(verb, 1), Ok(()), "{}", verb.name());
    }
    assert_eq!(
        *printers.service().changed.borrow(),
        [
            "set up ippusb://Canon%20G3560/",
            "remove alo-brother-hl-l2350dw-series-5b1f0c3e9a7d2468",
            "make default alo-canon-pixma-g3560-0f1e2d3c4b5a6978",
        ]
    );
}

/// **No printer answers to the identity: nothing is changed.** A printer found
/// cannot be removed by its address, a printer set up cannot be added again by
/// its queue, and a printer that went away is not replaced by the nearest one.
#[test]
fn a_printer_not_reported_now_is_not_changed_and_nothing_else_is() {
    let mut printers = Printers::against(an_office());
    for verb in [
        SystemVerb::AddPrinter(identity("dnssd://Somebody%20Else._ipp._tcp.local/")),
        SystemVerb::AddPrinter(identity("alo-canon-pixma-g3560-0f1e2d3c4b5a6978")),
        SystemVerb::RemovePrinter(identity("ippusb://Canon%20G3560/")),
        SystemVerb::SetDefaultPrinter(identity("alo-gone-0000000000000000")),
        SystemVerb::RemovePrinter(Identity::read(&"0".repeat(64)).unwrap()),
    ] {
        assert!(printers.carry(verb, 1).is_err(), "{}", verb.name());
    }
    assert!(printers.service().changed.borrow().is_empty());
}

/// **Two printers answering to one identity are neither of them.**
#[test]
fn two_printers_answering_to_one_identity_are_neither() {
    let mut office = an_office();
    office
        .set_up
        .push(Named("alo-canon-pixma-g3560-0f1e2d3c4b5a6978"));
    let mut printers = Printers::against(office);
    let verb = SystemVerb::RemovePrinter(identity("alo-canon-pixma-g3560-0f1e2d3c4b5a6978"));
    let refused = printers.carry(verb, 1).unwrap_err();
    assert!(refused.0.contains("more than one"), "{refused}");
    assert!(printers.service().changed.borrow().is_empty());
}

/// **A verb that is not a printer's is not carried out here**, and says so.
#[test]
fn a_verb_that_is_not_a_printers_is_not_carried_out_here() {
    let mut printers = Printers::against(an_office());
    let anything = identity("ippusb://Canon%20G3560/");
    for verb in SystemVerb::one_of_each(anything, Switch::On) {
        if verb.name().starts_with("printers.") {
            continue;
        }
        let refused = printers.carry(verb, 1).unwrap_err();
        assert!(refused.0.contains("not carried out"), "{refused}");
    }
    assert!(printers.service().changed.borrow().is_empty());
}

/// **Through the door**: an approved request to add a printer sets exactly that
/// printer up once, the same approval asked again changes nothing, a request
/// under no genuine approval changes nothing, a service that says no is written
/// down as not carried — and every one of those is in the record.
#[test]
fn through_the_door_one_approval_is_one_change_and_every_answer_is_kept() {
    let key = [41_u8; 32];
    let mut broker = Broker::new(
        Door::handed_to(1000),
        ApprovingKey::of(&key),
        Record::default(),
        Printers::against(an_office()),
    );
    let now = SystemTime::now();
    let add = SystemVerb::AddPrinter(identity("ippusb://Canon%20G3560/"));
    let approved = Request::of(add, ApprovingKey::of(&key).issue(&add, 7, now)).written();

    assert_eq!(
        broker.heard(Some(1000), approved.as_bytes(), now),
        Answer::Carried
    );
    assert_eq!(
        broker.heard(Some(1000), approved.as_bytes(), now),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    let forged = Request::of(add, ApprovingKey::of(&[42; 32]).issue(&add, 8, now)).written();
    assert_eq!(
        broker.heard(Some(1000), forged.as_bytes(), now),
        Answer::Refused(AtTheBroker::NotApproved)
    );
    assert_eq!(
        *broker.carrying().service().changed.borrow(),
        ["set up ippusb://Canon%20G3560/"]
    );

    let refused: Vec<Option<AtTheBroker>> = broker
        .recording()
        .everything()
        .map(|entry| match entry.happened() {
            Happened::Brokered { refused, .. } => *refused,
            other => panic!("the broker wrote down something else: {other:?}"),
        })
        .collect();
    assert_eq!(
        refused,
        [
            None,
            Some(AtTheBroker::ApprovalSpent),
            Some(AtTheBroker::NotApproved)
        ]
    );

    let mut saying_no = Broker::new(
        Door::handed_to(1000),
        ApprovingKey::of(&key),
        Record::default(),
        Printers::against(Office {
            refuses: true,
            ..an_office()
        }),
    );
    let again = Request::of(add, ApprovingKey::of(&key).issue(&add, 9, now)).written();
    assert_eq!(
        saying_no.heard(Some(1000), again.as_bytes(), now),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    let kept: Vec<Option<AtTheBroker>> = saying_no
        .recording()
        .everything()
        .map(|entry| match entry.happened() {
            Happened::Brokered { refused, .. } => *refused,
            other => panic!("the broker wrote down something else: {other:?}"),
        })
        .collect();
    assert_eq!(kept, [None, Some(AtTheBroker::NotCarried)]);
}
