//! *This computer starts this system when nobody chooses* changes the one
//! place that answer is kept, to the system a person approved and no other —
//! and on every refusal nothing is written, nothing about how the machine
//! starts changes, and the refusal is in the record.
//!
//! Task 17 of `docs/autonomy/v0-5-the-installer-plan.md`, and
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md).
//! Every request here crosses the broker's real decision
//! (`alo_broker::Broker::heard`) under a genuine token, so what is tested is
//! the door and the carrier together, against a loader that records everything
//! it was asked and everything it was told.
//!
//! **The next start provably untouched** is why this broker is built with a
//! firmware as well: the two verbs on this road are two acts, and a test that
//! only watched one file could not tell them apart.

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
use alo_brokerd::{ByDefault, Carriers, Network, NextStart, Proxy, Storage, the_identity_of};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};
use alo_record::{AtTheBroker, Happened, Record};
use alo_starting::testing::as_a_firmware_reports_it;
use alo_starting::{
    Entry, EnvironmentBlock, Firmware, LENGTH, NotAnswering, NotDone, NotRead, NotWritten, System,
    THE_WINDOWS_LOADER_IN_A_PATH, TheLoader, TheStartingChoice, the_identity_of as the_system,
    to_start_by_default,
};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [17; 32];

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

/// A firmware that reports one Windows and remembers what it was told, so that
/// *the next start was not touched* is a fact this test reads rather than
/// assumes.
#[derive(Debug)]
struct AFirmware {
    /// What it was told to start next.
    told: RefCell<Vec<u16>>,
}

impl Firmware for AFirmware {
    fn entries(&self) -> Result<Vec<Entry>, NotAnswering> {
        Ok(vec![
            Entry::reported(
                1,
                &as_a_firmware_reports_it("Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH),
            )
            .unwrap(),
        ])
    }

    fn start_next(&self, entry: u16) -> Result<(), NotDone> {
        self.told.borrow_mut().push(entry);
        Ok(())
    }
}

/// What the loader's files hold and what they will do, for one machine.
#[derive(Debug)]
struct ALoader {
    /// What the menu offers.
    offers: Vec<System>,
    /// The bytes of the file the choice is kept in.
    block: RefCell<Vec<u8>>,
    /// Whether that file refuses to be written.
    refuses: bool,
    /// How many times it was written.
    written: RefCell<usize>,
}

impl ALoader {
    /// A machine offering both systems, whose file holds one setting of the
    /// base's so that *nothing else was disturbed* has something to say.
    fn with_both() -> Self {
        let mut block = EnvironmentBlock::empty();
        block.keep("boot_success", "1").unwrap();
        Self {
            offers: System::BOTH.to_vec(),
            block: RefCell::new(block.written().unwrap()),
            refuses: false,
            written: RefCell::default(),
        }
    }

    /// Which system the file says this machine starts, now.
    fn now_starts(&self) -> System {
        TheStartingChoice::read(&EnvironmentBlock::read(&self.block.borrow()).unwrap())
    }
}

impl TheLoader for ALoader {
    fn offering(&self) -> Result<Vec<System>, NotRead> {
        Ok(self.offers.clone())
    }

    fn saved(&self) -> Result<Vec<u8>, NotRead> {
        Ok(self.block.borrow().clone())
    }

    fn save(&self, bytes: &[u8]) -> Result<(), NotWritten> {
        if self.refuses {
            return Err(NotWritten("this file is read-only".to_owned()));
        }
        *self.written.borrow_mut() += 1;
        self.block.borrow_mut().clear();
        self.block.borrow_mut().extend_from_slice(bytes);
        Ok(())
    }
}

/// What this test's broker is, spelt once.
type ABroker = Broker<
    Record,
    Carriers<
        Nothing,
        Nothing,
        alo_printing::PrintingService,
        alo_brokerd::NoUnits,
        AFirmware,
        ALoader,
    >,
>;

/// A machine whose loader's files are these.
fn a_machine(loader: ALoader) -> ABroker {
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
            told: RefCell::default(),
        }))
        .with_by_default(ByDefault::against(loader)),
    )
}

/// Ask `broker` for `verb`, under a genuine token for approval `approval`.
fn ask(broker: &mut ABroker, verb: SystemVerb, approval: u64) -> Answer {
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
    broker.heard(Some(our_user()), request.written().as_bytes(), now)
}

/// The loader's files, as this broker holds them.
fn loader(broker: &ABroker) -> &ALoader {
    broker.carrying().by_default().unwrap().loader()
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

/// **The system a person approved is the one the machine starts when nobody
/// chooses** — written into the one place that answer is kept, with the file's
/// other settings and its length exactly as they were, and the machine's next
/// start not touched at all.
#[test]
fn the_system_approved_is_the_default_and_the_next_start_is_untouched() {
    for (system, approval) in System::BOTH.into_iter().zip(1..) {
        let mut broker = a_machine(ALoader::with_both());
        assert_eq!(
            ask(&mut broker, to_start_by_default(system), approval),
            Answer::Carried
        );
        assert_eq!(loader(&broker).now_starts(), system);
        assert_eq!(*loader(&broker).written.borrow(), 1);
        assert_eq!(loader(&broker).block.borrow().len(), LENGTH);

        let block = EnvironmentBlock::read(&loader(&broker).block.borrow()).unwrap();
        assert_eq!(block.kept_as("boot_success"), Some("1"));
        assert_eq!(block.how_many(), 2);

        assert!(told(&broker).is_empty(), "the next start was touched");
        assert_eq!(refusals(&broker), 0);
    }
}

/// **One approval causes exactly one execution.** The same token again is spent
/// and the file is written once.
#[test]
fn one_approval_sets_the_default_exactly_once() {
    let mut broker = a_machine(ALoader::with_both());
    let now = SystemTime::now();
    let verb = to_start_by_default(System::Windows);
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
    assert_eq!(*loader(&broker).written.borrow(), 1);
    assert_eq!(loader(&broker).now_starts(), System::Windows);
}

/// **Every answer is written down, permitted or refused**, before it is given.
///
/// A refusal at the carrier leaves two entries rather than one — the broker
/// writes *handed on* before it hands the verb over, and the refusal after.
#[test]
fn every_answer_is_in_the_record() {
    let mut broker = a_machine(ALoader::with_both());
    assert_eq!(
        ask(&mut broker, to_start_by_default(System::Windows), 2),
        Answer::Carried
    );
    assert_eq!(
        ask(
            &mut broker,
            SystemVerb::StartByDefault(Identity::of_what_was_reported(b"something else")),
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
            (Some("starting.default".to_owned()), None),
            (Some("starting.default".to_owned()), None),
            (
                Some("starting.default".to_owned()),
                Some(AtTheBroker::NotCarried)
            ),
        ]
    );
}

/// **An identity that names no system changes nothing.** The identity of a
/// start-up entry is the case that matters: the two verbs on this road take
/// arguments of the same shape, and one made for the other reaches the door
/// with a genuine approval behind it.
#[test]
fn an_identity_that_names_no_system_changes_nothing() {
    let mut broker = a_machine(ALoader::with_both());
    let an_entry = the_identity_of(
        &Entry::reported(
            1,
            &as_a_firmware_reports_it("Windows Boot Manager", THE_WINDOWS_LOADER_IN_A_PATH),
        )
        .unwrap(),
    );
    assert_eq!(
        ask(&mut broker, SystemVerb::StartByDefault(an_entry), 4),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(*loader(&broker).written.borrow(), 0);
    assert_eq!(loader(&broker).now_starts(), System::AloOs);
    assert_eq!(refusals(&broker), 1);
}

/// **A machine with no Windows on it is not set to start one**, which is what a
/// machine alo OS replaced is — and alo OS, which it does have, is still
/// changeable on it.
#[test]
fn a_machine_with_no_windows_is_not_set_to_start_one() {
    let mut loader_of_ours = ALoader::with_both();
    loader_of_ours.offers = vec![System::AloOs];
    let mut broker = a_machine(loader_of_ours);

    assert_eq!(
        ask(&mut broker, to_start_by_default(System::Windows), 5),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(*loader(&broker).written.borrow(), 0);
    assert_eq!(refusals(&broker), 1);

    assert_eq!(
        ask(&mut broker, to_start_by_default(System::AloOs), 6),
        Answer::Carried
    );
    assert_eq!(*loader(&broker).written.borrow(), 1);
}

/// **A file that is not one this machine wrote is refused rather than
/// replaced.** It belongs to somebody, and overwriting it would be alo OS
/// taking a file it does not own.
#[test]
fn a_file_that_is_not_this_machines_is_not_written_over() {
    let loader_of_ours = ALoader::with_both();
    *loader_of_ours.block.borrow_mut() = b"something else entirely".to_vec();
    let mut broker = a_machine(loader_of_ours);

    assert_eq!(
        ask(&mut broker, to_start_by_default(System::Windows), 7),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(*loader(&broker).written.borrow(), 0);
    assert_eq!(
        *loader(&broker).block.borrow(),
        b"something else entirely".to_vec()
    );
    assert_eq!(refusals(&broker), 1);
}

/// **A file that will not hold the answer changes nothing, and nothing in it is
/// dropped to make room.** What would go is the base's.
#[test]
fn a_file_that_will_not_hold_the_answer_changes_nothing() {
    let loader_of_ours = ALoader::with_both();
    let mut full = EnvironmentBlock::empty();
    full.keep("something_of_the_bases", &"x".repeat(LENGTH - 65))
        .unwrap();
    let bytes = full.written().unwrap();
    *loader_of_ours.block.borrow_mut() = bytes.clone();
    let mut broker = a_machine(loader_of_ours);

    assert_eq!(
        ask(&mut broker, to_start_by_default(System::Windows), 8),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(*loader(&broker).written.borrow(), 0);
    assert_eq!(*loader(&broker).block.borrow(), bytes);
}

/// **A file that would not be written is a refusal**, not an answer that the
/// machine's default changed.
#[test]
fn a_file_that_would_not_be_written_is_a_refusal() {
    let mut loader_of_ours = ALoader::with_both();
    loader_of_ours.refuses = true;
    let mut broker = a_machine(loader_of_ours);

    assert_eq!(
        ask(&mut broker, to_start_by_default(System::Windows), 9),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(loader(&broker).now_starts(), System::AloOs);
    assert_eq!(refusals(&broker), 1);
}

/// **An approval to go across once is not an approval to change the default.**
/// The two verbs take the same argument and mean entirely different things, and
/// the proof is made over the verb — so a token issued for one does not open
/// the other.
#[test]
fn an_approval_for_the_next_start_does_not_change_the_default() {
    let mut broker = a_machine(ALoader::with_both());
    let now = SystemTime::now();
    let identity = the_system(System::Windows);
    let once = SystemVerb::RestartIntoWindows(identity);
    let token = ApprovingKey::of(&THE_KEY).issue(&once, 10, now);
    let request = Request::of(SystemVerb::StartByDefault(identity), token);

    assert_eq!(
        broker.heard(Some(our_user()), request.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotApproved)
    );
    assert_eq!(*loader(&broker).written.borrow(), 0);
    assert!(told(&broker).is_empty());
}

/// **A broker built without the loader's files refuses the verb by name**,
/// rather than answering as though a machine with nothing to change had done
/// it.
#[test]
fn a_broker_with_no_loader_refuses_the_verb_by_name() {
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
    let verb = to_start_by_default(System::Windows);
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, 11, now));
    assert_eq!(
        broker.heard(Some(our_user()), request.written().as_bytes(), now),
        Answer::Refused(AtTheBroker::NotCarried)
    );
}
