//! *Restart into Windows* sets the next start a person approved and nothing
//! else — and on every refusal the firmware is not told, nothing about how the
//! machine starts changes, and the refusal is in the record.
//!
//! Task 16 of `docs/autonomy/v0-5-the-installer-plan.md`, and
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
//! *what stays as it was*. Every request here crosses the broker's real
//! decision (`alo_broker::Broker::heard`) under a genuine token, so what is
//! tested is the door and the carrier together, against a firmware that records
//! everything it was asked and everything it was told.
//!
//! **The default provably untouched** is the point of the firmware fixture: it
//! keeps the machine's start-up order beside its entries, and every test below
//! reads that order back after the act.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::Path;
use std::time::SystemTime;

use alo_broker::{Answer, ApprovingKey, Broker, Door, Identity, Request, SystemVerb, our_user};
use alo_brokerd::{Carriers, Network, NextStart, Proxy, Storage, the_identity_of};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};
use alo_record::{AtTheBroker, Happened, Record};
use alo_starting::testing::as_a_firmware_reports_it;
use alo_starting::{Entry, Firmware, NotAnswering, NotDone, THE_WINDOWS_LOADER_IN_A_PATH};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [16; 32];

/// The order a machine ordinarily starts in, which nothing here may change.
const THE_ORDER: [u16; 2] = [3, 1];

/// Nothing to change, and a panic if asked to.
#[derive(Debug)]
struct Nothing;

impl Networks for Nothing {
    fn now(&self) -> Result<TheNetworks, alo_networks::NotAnswering> {
        panic!("a start-up verb asked the network manager")
    }
}

impl NetworkService for Nothing {
    fn join(&self, _: &Visible) -> Result<(), alo_networks::NotDone> {
        panic!("a start-up verb joined a network")
    }

    fn forget(&self, _: &Saved) -> Result<(), alo_networks::NotDone> {
        panic!("a start-up verb forgot a network")
    }

    fn switch_wireless(&self, _: bool) -> Result<(), alo_networks::NotDone> {
        panic!("a start-up verb switched the radio")
    }
}

impl Drives for Nothing {
    fn now(&self) -> Result<TheDrives, alo_drives::NotAnswering> {
        panic!("a start-up verb asked the disk service")
    }
}

impl DriveService for Nothing {
    fn mount(&self, _: &Drive, _: &Filesystem, _: &LoginName) -> Result<(), alo_drives::NotDone> {
        panic!("a start-up verb mounted a drive")
    }

    fn eject(&self, _: &Drive) -> Result<(), alo_drives::NotDone> {
        panic!("a start-up verb ejected a drive")
    }
}

/// A firmware that reports entries, keeps an ordinary start-up order, and
/// remembers what it was told.
#[derive(Debug)]
struct AFirmware {
    /// What it reports, or the reason it will not.
    reports: Result<Vec<Entry>, NotAnswering>,
    /// Whether it refuses to be told.
    refuses: bool,
    /// The order it ordinarily starts in. Nothing here has a way to change it.
    order: RefCell<Vec<u16>>,
    /// What it was told to start next.
    told: RefCell<Vec<u16>>,
}

impl Firmware for AFirmware {
    fn entries(&self) -> Result<Vec<Entry>, NotAnswering> {
        self.reports.clone()
    }

    fn start_next(&self, entry: u16) -> Result<(), NotDone> {
        if self.refuses {
            return Err(NotDone("this firmware is read-only".to_owned()));
        }
        self.told.borrow_mut().push(entry);
        Ok(())
    }
}

/// What this test's broker is, spelt once.
type ABroker = Broker<
    Record,
    Carriers<Nothing, Nothing, alo_printing::PrintingService, alo_brokerd::NoUnits, AFirmware>,
>;

/// A start-up entry a firmware reports.
fn an_entry(number: u16, named: &str, path: &str) -> Entry {
    Entry::reported(number, &as_a_firmware_reports_it(named, path)).unwrap()
}

/// The Windows on a machine.
fn windows(number: u16) -> Entry {
    an_entry(number, "Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH)
}

/// alo OS on the same machine.
fn alo_os(number: u16) -> Entry {
    an_entry(number, "alo OS", "\\EFI\\fedora\\shimx64.efi")
}

/// A machine whose firmware reports these entries and answers or refuses as
/// `refuses` says.
fn a_machine(reports: Result<Vec<Entry>, NotAnswering>, refuses: bool) -> ABroker {
    Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carriers::of(
            Network::against(Nothing),
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
                Nothing,
                Path::new("/nonexistent-alo-brokerd/passwd"),
                our_user(),
            ),
        )
        .with_next_start(NextStart::against(AFirmware {
            reports,
            refuses,
            order: RefCell::new(THE_ORDER.to_vec()),
            told: RefCell::default(),
        })),
    )
}

/// Ask `broker` for `verb`, under a genuine token for approval `approval`.
fn ask(broker: &mut ABroker, verb: SystemVerb, approval: u64) -> Answer {
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
    broker.heard(Some(our_user()), request.written().as_bytes(), now)
}

/// What the firmware was told to start next.
fn told(broker: &ABroker) -> Vec<u16> {
    broker
        .carrying()
        .next_start()
        .unwrap()
        .firmware()
        .told
        .borrow()
        .clone()
}

/// The order the machine ordinarily starts in, now.
fn the_order(broker: &ABroker) -> Vec<u16> {
    broker
        .carrying()
        .next_start()
        .unwrap()
        .firmware()
        .order
        .borrow()
        .clone()
}

/// How many refusals the record holds.
fn refusals(broker: &ABroker) -> usize {
    broker
        .recording()
        .everything()
        .filter(|entry| {
            matches!(
                entry.happened(),
                Happened::Brokered {
                    refused: Some(AtTheBroker::NotCarried),
                    ..
                }
            )
        })
        .count()
}

/// **The Windows a person approved is the one the machine is set to start**,
/// once — and the order the machine ordinarily starts in is exactly as it was.
#[test]
fn the_windows_approved_is_the_next_start_and_the_default_is_untouched() {
    let approved = the_identity_of(&windows(1));
    let mut broker = a_machine(Ok(vec![alo_os(3), windows(1)]), false);
    assert_eq!(
        ask(&mut broker, SystemVerb::RestartIntoWindows(approved), 1),
        Answer::Carried
    );
    assert_eq!(told(&broker), vec![1]);
    assert_eq!(the_order(&broker), THE_ORDER.to_vec());
    assert_eq!(refusals(&broker), 0);
}

/// **One approval causes exactly one execution.** The same token again is spent
/// and the firmware is told once, which for this verb is the difference between
/// one restart into Windows and a machine that keeps going there.
#[test]
fn one_approval_sets_the_next_start_exactly_once() {
    let approved = the_identity_of(&windows(1));
    let mut broker = a_machine(Ok(vec![windows(1)]), false);
    let now = SystemTime::now();
    let verb = SystemVerb::RestartIntoWindows(approved);
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, 7, now));
    let written = request.written();

    assert_eq!(
        broker.heard(Some(our_user()), written.as_bytes(), now),
        Answer::Carried
    );
    assert_eq!(
        broker.heard(Some(our_user()), written.as_bytes(), now),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    assert_eq!(told(&broker), vec![1]);
}

/// **Every answer is written down, permitted or refused**, before it is given.
///
/// A refusal at the carrier leaves two entries rather than one — the broker
/// writes *handed on* before it hands the verb over, and the refusal after —
/// and that is the shape being held here: the record says what the broker was
/// about to do as well as what came of it.
#[test]
fn every_answer_is_in_the_record() {
    let approved = the_identity_of(&windows(1));
    let mut broker = a_machine(Ok(vec![windows(1)]), false);
    assert_eq!(
        ask(&mut broker, SystemVerb::RestartIntoWindows(approved), 2),
        Answer::Carried
    );
    assert_eq!(
        ask(
            &mut broker,
            SystemVerb::RestartIntoWindows(Identity::of_what_was_reported(b"somebody else's")),
            3
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );

    let brokered: Vec<(Option<String>, Option<AtTheBroker>)> = broker
        .recording()
        .everything()
        .filter_map(|entry| match entry.happened() {
            Happened::Brokered { verb, refused, .. } => {
                Some((verb.as_ref().map(ToString::to_string), refused.to_owned()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        brokered,
        vec![
            (Some("starting.windows-next".to_owned()), None),
            (Some("starting.windows-next".to_owned()), None),
            (
                Some("starting.windows-next".to_owned()),
                Some(AtTheBroker::NotCarried)
            ),
        ]
    );
}

/// **An entry nobody approved is not started**, and the machine is not told
/// anything at all.
#[test]
fn an_entry_nobody_approved_is_not_started() {
    let mut broker = a_machine(Ok(vec![alo_os(3), windows(1)]), false);
    assert_eq!(
        ask(
            &mut broker,
            SystemVerb::RestartIntoWindows(the_identity_of(&an_entry(
                9,
                "Windows Boot Manager",
                "\\EFI\\Microsoft\\Boot\\bootmgfw.efi.bak",
            ))),
            4
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(told(&broker).is_empty());
    assert_eq!(the_order(&broker), THE_ORDER.to_vec());
    assert_eq!(refusals(&broker), 1);
}

/// **An entry that does not start Windows is not started**, whatever it is
/// called — the refusal that makes the verb's name true.
#[test]
fn an_entry_that_does_not_start_windows_is_not_started() {
    let looks_like_it = an_entry(
        2,
        "Windows Boot Manager",
        "\\EFI\\somebody-else\\loader.efi",
    );
    let approved = the_identity_of(&looks_like_it);
    let mut broker = a_machine(Ok(vec![looks_like_it]), false);
    assert_eq!(
        ask(&mut broker, SystemVerb::RestartIntoWindows(approved), 5),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(told(&broker).is_empty());
    assert_eq!(refusals(&broker), 1);
}

/// **A machine that cannot be asked what it can start is not told anything**,
/// and the refusal is in the record rather than read as *there is no Windows
/// here*.
#[test]
fn a_machine_that_cannot_be_asked_is_not_told() {
    let mut broker = a_machine(Err(NotAnswering("nothing to read".to_owned())), false);
    assert_eq!(
        ask(
            &mut broker,
            SystemVerb::RestartIntoWindows(the_identity_of(&windows(1))),
            6
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(told(&broker).is_empty());
    assert_eq!(refusals(&broker), 1);
}

/// **A firmware that refuses is a refusal**, not an answer that the next start
/// was set.
#[test]
fn a_firmware_that_refuses_is_a_refusal() {
    let approved = the_identity_of(&windows(1));
    let mut broker = a_machine(Ok(vec![windows(1)]), true);
    assert_eq!(
        ask(&mut broker, SystemVerb::RestartIntoWindows(approved), 8),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(told(&broker).is_empty());
    assert_eq!(the_order(&broker), THE_ORDER.to_vec());
}

/// **A token that is not this verb's does not set the next start.** An approval
/// for mounting a drive is not an approval for restarting into another
/// operating system, and the proof is over the verb.
#[test]
fn an_approval_for_another_verb_does_not_set_the_next_start() {
    let approved = the_identity_of(&windows(1));
    let mut broker = a_machine(Ok(vec![windows(1)]), false);
    let now = SystemTime::now();
    let another = SystemVerb::MountDrive(approved);
    let token = ApprovingKey::of(&THE_KEY).issue(&another, 9, now);
    let request = Request::of(SystemVerb::RestartIntoWindows(approved), token);

    assert_eq!(
        broker.heard(Some(our_user()), request.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotApproved)
    );
    assert!(told(&broker).is_empty());
    assert_eq!(the_order(&broker), THE_ORDER.to_vec());
}

/// **A broker built without a firmware refuses the verb by name**, rather than
/// answering as though a machine with nothing to ask had done it.
#[test]
fn a_broker_with_no_firmware_refuses_the_verb_by_name() {
    let mut broker: Broker<Record, Carriers<Nothing, Nothing>> = Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carriers::of(
            Network::against(Nothing),
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
                Nothing,
                Path::new("/nonexistent-alo-brokerd/passwd"),
                our_user(),
            ),
        ),
    );
    let verb = SystemVerb::RestartIntoWindows(Identity::of_what_was_reported(b"a Windows"));
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, 10, now));
    assert_eq!(
        broker.heard(Some(our_user()), request.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotCarried)
    );
}
