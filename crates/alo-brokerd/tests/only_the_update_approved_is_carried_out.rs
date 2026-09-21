//! The update verbs carry out exactly what a person approved, by starting the
//! one unit that may — and on every refusal no unit is started, nothing is run,
//! and the refusal is in the record.
//!
//! Task 8 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, and
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
//! accepted option B. Every request here crosses the broker's real decision
//! (`alo_broker::Broker::heard`) under a genuine token, so what is tested is
//! the door and the carrier together, against something that records which unit
//! it was asked to start and what the unit would have read.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_broker::{Answer, ApprovingKey, Broker, Door, Identity, Request, SystemVerb, our_user};
use alo_brokerd::units::{NotDone, StartingUnits, TheUnit};
use alo_brokerd::{AnUpdate, Carriers, Handing, Network, Proxy, Storage, Updates};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_keeping_up::Digest;
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};
use alo_record::{AtTheBroker, Happened, Record};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [53; 32];

/// A whole digest made of one repeated pair.
fn whole(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// Nothing to change, and a panic if asked to.
#[derive(Debug)]
struct Nothing;

impl Networks for Nothing {
    fn now(&self) -> Result<TheNetworks, alo_networks::NotAnswering> {
        panic!("an update verb asked the network manager")
    }
}

impl NetworkService for Nothing {
    fn join(&self, _: &Visible) -> Result<(), alo_networks::NotDone> {
        panic!("an update verb joined a network")
    }

    fn forget(&self, _: &Saved) -> Result<(), alo_networks::NotDone> {
        panic!("an update verb forgot a network")
    }

    fn switch_wireless(&self, _: bool) -> Result<(), alo_networks::NotDone> {
        panic!("an update verb switched the radio")
    }
}

impl Drives for Nothing {
    fn now(&self) -> Result<TheDrives, alo_drives::NotAnswering> {
        panic!("an update verb asked the disk service")
    }
}

impl DriveService for Nothing {
    fn mount(&self, _: &Drive, _: &Filesystem, _: &LoginName) -> Result<(), alo_drives::NotDone> {
        panic!("an update verb mounted a drive")
    }

    fn eject(&self, _: &Drive) -> Result<(), alo_drives::NotDone> {
        panic!("an update verb ejected a drive")
    }
}

/// Something that starts units, remembering which it was asked for and what
/// the unit would have found waiting for it.
#[derive(Debug)]
struct Starting {
    /// The folder the broker hands things on in.
    handing: Handing,
    /// Whether the unit fails when it is started.
    failing: bool,
    /// Which units were started, and what each would have read.
    started: RefCell<Vec<(TheUnit, Vec<u8>)>>,
}

impl StartingUnits for Starting {
    fn start(&self, unit: TheUnit) -> Result<(), NotDone> {
        let at = match unit {
            TheUnit::ApplyingAnUpdate => self.handing.update(),
            TheUnit::GoingBack => self.handing.going_back(),
        };
        let found = std::fs::read(&at).unwrap_or_default();
        self.started.borrow_mut().push((unit, found));
        if self.failing {
            return Err(NotDone(format!("{} ran and failed", unit.named())));
        }
        Ok(())
    }
}

/// What this test's broker is, spelt once.
type ABroker = Broker<Record, Carriers<Nothing, Nothing, alo_printing::PrintingService, Starting>>;

/// A machine set up for one test: its folders, its broker, and what it handed
/// over.
struct AMachine {
    /// The broker.
    broker: ABroker,
    /// Where a person hands an update over.
    wanted: PathBuf,
    /// The folder the broker hands one on in.
    handing: Handing,
}

/// A machine whose person has handed `handed` over, and whose units succeed or
/// fail as `failing` says.
fn a_machine(named: &str, handed: Option<&[u8]>, failing: bool) -> AMachine {
    let root = std::env::temp_dir().join(format!(
        "alo-brokerd-updates-{}-{named}",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&root));
    std::fs::create_dir_all(root.join("wanted")).unwrap();
    std::fs::create_dir_all(root.join("approved")).unwrap();
    let wanted = root.join("wanted").join("update.json");
    if let Some(bytes) = handed {
        std::fs::write(&wanted, bytes).unwrap();
    }
    let handing = Handing::at(&root.join("approved"));
    let broker = Broker::new(
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
        .with_updates(Updates::against(
            Starting {
                handing: handing.clone(),
                failing,
                started: RefCell::default(),
            },
            &wanted,
            handing.clone(),
            our_user(),
        )),
    );
    AMachine {
        broker,
        wanted,
        handing,
    }
}

/// Ask `broker` for `verb`, under a genuine token for approval `approval`.
fn ask(broker: &mut ABroker, verb: SystemVerb, approval: u64) -> Answer {
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
    broker.heard(Some(our_user()), request.written().as_bytes(), now)
}

/// Which units were started, and what each would have read.
fn started(machine: &AMachine) -> Vec<(TheUnit, Vec<u8>)> {
    machine
        .broker
        .carrying()
        .updates()
        .unwrap()
        .starting()
        .started
        .borrow()
        .clone()
}

/// How many refusals the record holds.
fn refusals(machine: &AMachine) -> usize {
    machine
        .broker
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

/// Nothing is left in either folder.
fn nothing_is_left(machine: &AMachine) {
    assert!(
        !machine.wanted.exists(),
        "what the person handed over was left behind"
    );
    assert!(
        !machine.handing.update().exists(),
        "what was handed on to the unit was left behind"
    );
    assert!(
        !machine.handing.going_back().exists(),
        "what was handed on to the going-back unit was left behind"
    );
}

/// **The update a person approved is carried out**: the unit that may stage it
/// is started, it finds exactly the two builds that were approved, and nothing
/// is left in `/run` afterwards.
#[test]
fn the_update_approved_is_handed_to_the_unit_that_stages_it() {
    let update = AnUpdate::between(whole("aa"), whole("bb"));
    let mut machine = a_machine("carried", Some(&update.written()), false);
    assert_eq!(
        ask(
            &mut machine.broker,
            SystemVerb::ApplyStagedUpdate(update.identity()),
            1
        ),
        Answer::Carried
    );
    assert_eq!(
        started(&machine),
        vec![(TheUnit::ApplyingAnUpdate, update.written())]
    );
    nothing_is_left(&machine);
    assert_eq!(refusals(&machine), 0);
}

/// **Going back hands the unit the identity that was approved, and nothing
/// else** — there is nothing for anybody to swap on the way.
#[test]
fn going_back_hands_the_unit_the_identity_that_was_approved() {
    let approved = Identity::of_what_was_reported(b"an offer to go back");
    let mut machine = a_machine("going-back", None, false);
    assert_eq!(
        ask(&mut machine.broker, SystemVerb::RollBack(approved), 2),
        Answer::Carried
    );
    assert_eq!(
        started(&machine),
        vec![(
            TheUnit::GoingBack,
            format!("{}\n", approved.written()).into_bytes()
        )]
    );
    nothing_is_left(&machine);
}

/// **An update whose bytes are not the ones that were approved starts no
/// unit**, and the refusal is in the record.
///
/// This is the case the copy into root's folder exists for: what the person
/// left in their own folder can change after they approved it, and the broker
/// digests what is there at the moment it acts.
#[test]
fn an_update_that_is_not_the_one_approved_starts_no_unit() {
    let approved = AnUpdate::between(whole("aa"), whole("bb"));
    let swapped = AnUpdate::between(whole("aa"), whole("cc"));
    let mut machine = a_machine("swapped", Some(&swapped.written()), false);
    assert_eq!(
        ask(
            &mut machine.broker,
            SystemVerb::ApplyStagedUpdate(approved.identity()),
            3
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(started(&machine).is_empty(), "a unit was started");
    nothing_is_left(&machine);
    assert_eq!(refusals(&machine), 1);
}

/// **An approval with nothing handed over starts no unit.**
#[test]
fn an_update_with_nothing_handed_over_starts_no_unit() {
    let update = AnUpdate::between(whole("aa"), whole("bb"));
    let mut machine = a_machine("nothing", None, false);
    assert_eq!(
        ask(
            &mut machine.broker,
            SystemVerb::ApplyStagedUpdate(update.identity()),
            4
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(started(&machine).is_empty());
    assert_eq!(refusals(&machine), 1);
}

/// **Bytes that digest to the identity approved and are not a pair of builds
/// start no unit.**
///
/// A person could be got to approve the digest of anything at all; what stops
/// it reaching a privileged unit is that the broker reads what it is before it
/// hands it on.
#[test]
fn bytes_that_are_not_a_pair_of_builds_start_no_unit_even_under_their_own_digest() {
    let not_an_update = b"switch --apply ghcr.io/somebody/else\n".as_slice();
    let mut machine = a_machine("not-an-update", Some(not_an_update), false);
    assert_eq!(
        ask(
            &mut machine.broker,
            SystemVerb::ApplyStagedUpdate(Identity::of_what_was_reported(not_an_update)),
            5
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert!(started(&machine).is_empty());
    nothing_is_left(&machine);
    assert_eq!(refusals(&machine), 1);
}

/// **A unit that ran and failed is `not-carried`**, written down, and leaves
/// nothing behind for a second attempt to pick up.
#[test]
fn a_unit_that_ran_and_failed_is_not_carried_and_leaves_nothing() {
    let update = AnUpdate::between(whole("aa"), whole("bb"));
    let mut machine = a_machine("failing", Some(&update.written()), true);
    assert_eq!(
        ask(
            &mut machine.broker,
            SystemVerb::ApplyStagedUpdate(update.identity()),
            6
        ),
        Answer::Refused(AtTheBroker::NotCarried)
    );
    assert_eq!(started(&machine).len(), 1);
    nothing_is_left(&machine);
    assert_eq!(refusals(&machine), 1);
}

/// **One approval is one execution.** The same genuine token twice starts the
/// unit once; the second is `approval-spent` and nothing runs.
#[test]
fn one_approval_starts_the_unit_once() {
    let update = AnUpdate::between(whole("aa"), whole("bb"));
    let mut machine = a_machine("spent", Some(&update.written()), false);
    let verb = SystemVerb::ApplyStagedUpdate(update.identity());
    let now = SystemTime::now();
    let token = ApprovingKey::of(&THE_KEY).issue(&verb, 7, now);
    let line = Request::of(verb, token).written();
    assert_eq!(
        machine.broker.heard(Some(our_user()), line.as_bytes(), now),
        Answer::Carried
    );
    assert_eq!(
        machine.broker.heard(Some(our_user()), line.as_bytes(), now),
        Answer::Refused(AtTheBroker::ApprovalSpent)
    );
    assert_eq!(started(&machine).len(), 1, "one approval started two units");
}

/// **A broker built with nothing to start units refuses both update verbs by
/// name**, which is what the published three-argument `Carriers::of` still
/// gives a caller that has not asked for them.
#[test]
fn a_broker_that_cannot_start_a_unit_refuses_both_verbs_by_name() {
    let mut broker = Broker::new(
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
    let build = Identity::of_what_was_reported(b"a build");
    for (verb, approval) in [
        (SystemVerb::ApplyStagedUpdate(build), 8),
        (SystemVerb::RollBack(build), 9),
    ] {
        let now = SystemTime::now();
        let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
        assert_eq!(
            broker.heard(Some(our_user()), request.written().as_bytes(), now),
            Answer::Refused(AtTheBroker::NotCarried),
            "{}",
            verb.name()
        );
    }
}
