//! The network's verbs change exactly the network a person approved, as the
//! network manager reports it now — and nothing else, ever.
//!
//! Task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *the broker's
//! network verbs — join a network the machine can see, forget a network, turn
//! the radio on or off — take closed types (a network by the identity the rented
//! network manager reported, never a typed name).* Every request here crosses
//! the broker's real decision (`alo_broker::Broker::heard`) under a genuine
//! token, so what is tested is the door and the carrier together, against a
//! network manager that records what it was asked.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::Path;
use std::time::SystemTime;

use alo_broker::{
    Answer, ApprovingKey, Broker, Door, Identity, Request, Switch, SystemVerb, our_user,
};
use alo_brokerd::{Carriers, Network, Proxy, Storage};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::{
    NetworkName, NetworkService, Networks, NotAnswering, NotDone, Primary, Protection, Saved,
    TheNetworks, Visible,
};
use alo_record::{AtTheBroker, Happened, Record};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [41; 32];

/// A network manager that reports what it is given and remembers what it was
/// asked to change.
#[derive(Debug)]
struct Reporting {
    /// What it reports.
    networks: TheNetworks,
    /// What it was asked to change, in order.
    asked: RefCell<Vec<String>>,
}

impl Networks for Reporting {
    fn now(&self) -> Result<TheNetworks, NotAnswering> {
        Ok(self.networks.clone())
    }
}

impl NetworkService for Reporting {
    fn join(&self, network: &Visible) -> Result<(), NotDone> {
        self.asked
            .borrow_mut()
            .push(format!("join {}", network.name().called().unwrap_or("?")));
        Ok(())
    }

    fn forget(&self, network: &Saved) -> Result<(), NotDone> {
        self.asked
            .borrow_mut()
            .push(format!("forget {}", network.uuid()));
        Ok(())
    }

    fn switch_wireless(&self, on: bool) -> Result<(), NotDone> {
        self.asked.borrow_mut().push(format!("wireless {on}"));
        Ok(())
    }
}

/// A disk service with no drives, which must never be asked to change one.
#[derive(Debug)]
struct NoDrives;

impl Drives for NoDrives {
    fn now(&self) -> Result<TheDrives, alo_drives::NotAnswering> {
        Ok(TheDrives::default())
    }
}

impl DriveService for NoDrives {
    fn mount(&self, _: &Drive, _: &Filesystem, _: &LoginName) -> Result<(), alo_drives::NotDone> {
        panic!("a network test mounted a drive")
    }

    fn eject(&self, _: &Drive) -> Result<(), alo_drives::NotDone> {
        panic!("a network test ejected a drive")
    }
}

/// A name.
fn named(name: &str) -> NetworkName {
    NetworkName::announced(name.as_bytes()).unwrap()
}

/// What a machine in a flat sees.
fn a_flat() -> TheNetworks {
    TheNetworks {
        visible: vec![
            Visible::reported(named("Home"), Protection::Password, 80),
            Visible::reported(named("Home"), Protection::Open, 99),
            Visible::reported(named("Cafe"), Protection::Open, 40),
            Visible::reported(named("Work"), Protection::Enterprise, 60),
        ],
        saved: vec![
            Saved::reported(named("Home"), "home-uuid"),
            Saved::reported(named("Office"), "office-uuid"),
        ],
        wireless_on: true,
        primary: Some(Primary::reported(true, "home-uuid")),
    }
}

/// A broker whose door hears this test, carrying the network's verbs out
/// against `networks`.
fn a_broker(networks: TheNetworks) -> Broker<Record, Carriers<Reporting, NoDrives>> {
    Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carriers::of(
            Network::against(Reporting {
                networks,
                asked: RefCell::default(),
            }),
            Proxy::handed_over(
                Path::new("/nonexistent-alo-brokerd/wanted.json"),
                Path::new("/nonexistent-alo-brokerd/proxy-password"),
                Path::new("/nonexistent-alo-brokerd/proxy.json"),
                alo_proxy::TheMachinesCredentials::at(
                    Path::new("/nonexistent-alo-brokerd/credstore.encrypted"),
                    Path::new("/nonexistent-alo-brokerd/systemd-creds"),
                ),
                our_user(),
            ),
            Storage::against(
                NoDrives,
                Path::new("/nonexistent-alo-brokerd/passwd"),
                our_user(),
            ),
        ),
    )
}

/// Ask `broker` for `verb`, under a genuine token for approval `approval`.
fn ask(
    broker: &mut Broker<Record, Carriers<Reporting, NoDrives>>,
    verb: SystemVerb,
    approval: u64,
) -> Answer {
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
    broker.heard(Some(our_user()), request.written().as_bytes(), now)
}

/// What the network manager was asked to change.
fn asked(broker: &Broker<Record, Carriers<Reporting, NoDrives>>) -> Vec<String> {
    broker.carrying().network().service().asked.borrow().clone()
}

/// **Each verb changes exactly what was approved**: the protected network
/// called Home and not the open one calling itself Home, the saved network by
/// its own identifier, and the radio — each carried once, each written down.
#[test]
fn each_network_verb_changes_exactly_what_was_approved() {
    let flat = a_flat();
    let home = flat
        .visible
        .iter()
        .find(|network| network.protection() == Protection::Password)
        .unwrap()
        .as_reported();
    let office = flat
        .saved
        .iter()
        .find(|saved| saved.uuid() == "office-uuid")
        .unwrap()
        .as_reported();
    let mut broker = a_broker(flat);

    let join = SystemVerb::JoinNetwork(Identity::of_what_was_reported(&home));
    let forget = SystemVerb::ForgetNetwork(Identity::of_what_was_reported(&office));
    let off = SystemVerb::SetRadio(Switch::Off);
    assert_eq!(ask(&mut broker, join, 1), Answer::Carried);
    assert_eq!(ask(&mut broker, forget, 2), Answer::Carried);
    assert_eq!(ask(&mut broker, off, 3), Answer::Carried);
    assert_eq!(
        asked(&broker),
        ["join Home", "forget office-uuid", "wireless false"]
    );
    let handed_on = broker
        .recording()
        .everything()
        .filter(|entry| matches!(entry.happened(), Happened::Brokered { refused: None, .. }))
        .count();
    assert_eq!(handed_on, 3);
}

/// **A network not reported now is not changed, and nothing else is in its
/// place**: one that went out of range, a name typed where an identity goes,
/// the identity of a visible network offered to forget, and a saved one
/// offered to join.
#[test]
fn a_network_not_reported_now_is_not_changed_and_nothing_else_is() {
    let flat = a_flat();
    let cafe = flat
        .visible
        .iter()
        .find(|network| network.name() == &named("Cafe"))
        .unwrap()
        .as_reported();
    let saved_home = flat.saved.first().unwrap().as_reported();
    let mut broker = a_broker(flat);
    let gone = Visible::reported(named("Library"), Protection::Open, 10).as_reported();

    for verb in [
        SystemVerb::JoinNetwork(Identity::of_what_was_reported(&gone)),
        SystemVerb::JoinNetwork(Identity::of_what_was_reported(b"Home")),
        SystemVerb::ForgetNetwork(Identity::of_what_was_reported(&cafe)),
        SystemVerb::JoinNetwork(Identity::of_what_was_reported(&saved_home)),
    ] {
        assert_eq!(
            ask(&mut broker, verb, 7),
            Answer::Refused(AtTheBroker::NotCarried),
            "{verb:?}"
        );
    }
    assert!(asked(&broker).is_empty(), "{:?}", asked(&broker));
}

/// **An identity two networks answer to names neither**, and a network asking
/// for an organisation's sign-in is not joined.
#[test]
fn two_networks_answering_to_one_identity_are_neither_and_a_sign_in_is_not_joined() {
    let mut flat = a_flat();
    flat.saved
        .push(Saved::reported(named("Home again"), "home-uuid"));
    let twice = flat.saved.first().unwrap().as_reported();
    let work = flat
        .visible
        .iter()
        .find(|network| network.protection() == Protection::Enterprise)
        .unwrap()
        .as_reported();
    let mut broker = a_broker(flat);

    for verb in [
        SystemVerb::ForgetNetwork(Identity::of_what_was_reported(&twice)),
        SystemVerb::JoinNetwork(Identity::of_what_was_reported(&work)),
    ] {
        assert_eq!(
            ask(&mut broker, verb, 8),
            Answer::Refused(AtTheBroker::NotCarried)
        );
    }
    assert!(asked(&broker).is_empty(), "{:?}", asked(&broker));
}

/// **A verb that is not the network's is not carried out here**, and says so
/// in the record rather than pretending: printers and updates, and storage verbs
/// naming no drive the disk service reports.
#[test]
fn a_verb_that_is_not_the_networks_is_not_carried_out_here() {
    let mut broker = a_broker(a_flat());
    let something = Identity::of_what_was_reported(b"something");
    for (verb, approval) in SystemVerb::one_of_each(something, Switch::On)
        .into_iter()
        .filter(|verb| !verb.name().starts_with("network."))
        .zip(20..)
    {
        assert_eq!(
            ask(&mut broker, verb, approval),
            Answer::Refused(AtTheBroker::NotCarried),
            "{}",
            verb.name()
        );
    }
    assert!(asked(&broker).is_empty());
}

/// **A request under no genuine approval changes no network** — the door's own
/// refusal, held here for the network's verbs.
#[test]
fn a_network_change_under_no_genuine_approval_changes_nothing() {
    let flat = a_flat();
    let home = flat.saved.first().unwrap().as_reported();
    let mut broker = a_broker(flat);
    let verb = SystemVerb::ForgetNetwork(Identity::of_what_was_reported(&home));
    let now = SystemTime::now();
    let forged = Request::of(verb, ApprovingKey::of(&[0; 32]).issue(&verb, 1, now));
    assert_eq!(
        broker.heard(Some(our_user()), forged.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotApproved)
    );
    let genuine = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, 2, now));
    assert_eq!(
        broker.heard(Some(our_user()), genuine.written().as_bytes(), now),
        Answer::Carried
    );
    assert_eq!(
        broker.heard(Some(our_user()), genuine.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    assert_eq!(asked(&broker), ["forget home-uuid"]);
}
