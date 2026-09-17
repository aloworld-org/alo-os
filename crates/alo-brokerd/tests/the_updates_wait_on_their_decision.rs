//! The update verbs are carried out by nothing yet, and say so, until the
//! decision about how they are carried out is accepted.
//!
//! Task 4 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` asks the
//! broker to apply a staged update and roll back. The base's own program refuses
//! to change the machine for anything but root holding `CAP_SYS_ADMIN`, and this
//! process holds no capability (`tests/the_unit_is_the_process.rs`). How an
//! update is carried out without handing the broker that is ADR 0053, proposed.
//!
//! So two things are held here. **Until then, both verbs are answered
//! `not-carried`, written down, and nothing runs** — no program of the base's is
//! named anywhere in this crate. **And once the decision is accepted, this test
//! fails**, which is the next worker's instruction to build what it decided
//! rather than to leave the verbs refusing.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;
use std::time::SystemTime;

use alo_broker::{Answer, ApprovingKey, Broker, Door, Identity, Request, SystemVerb, our_user};
use alo_brokerd::{Carriers, Network, Proxy, Storage};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};
use alo_record::{AtTheBroker, Happened, Record};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [47; 32];

/// The decision the update verbs wait on.
const THE_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md"
);

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

/// **While the decision is proposed, neither update verb is carried out**: each
/// genuine request is written down as handed on and then refused `not-carried`,
/// and nothing else on the machine is asked anything.
#[test]
fn neither_update_verb_is_carried_out_while_its_decision_is_proposed() {
    let decision = std::fs::read_to_string(THE_DECISION).unwrap();
    let status = decision
        .lines()
        .find(|line| line.starts_with("**Status:**"))
        .unwrap();
    assert!(
        status.contains("proposed"),
        "ADR 0053 is no longer proposed ({status}). Build what it decided — the update verbs \
         carried out as it says — and replace this test with the tests of that"
    );

    let mut broker = Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carriers::of(
            Network::against(Nothing),
            Proxy::handed_over(
                Path::new("/nonexistent-alo-brokerd/wanted.json"),
                Path::new("/nonexistent-alo-brokerd/proxy.json"),
                our_user(),
            ),
            Storage::against(
                Nothing,
                Path::new("/nonexistent-alo-brokerd/passwd"),
                our_user(),
            ),
        ),
    );
    let build = Identity::of_what_was_reported(format!("sha256:{}", "ab".repeat(32)).as_bytes());
    for (verb, approval) in [
        (SystemVerb::ApplyStagedUpdate(build), 1),
        (SystemVerb::RollBack(build), 2),
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
    let refused = broker
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
        .count();
    assert_eq!(refused, 2);
}

/// **Nothing in the broker's process can run the base's program**: it does not
/// depend on the crate that runs it, and its source names no program of the
/// base's. When ADR 0053 is built, what runs the base is a unit of its own, and
/// this test is replaced by the tests that hold that unit to what it may run.
#[test]
fn nothing_in_the_brokers_process_runs_the_base() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(here.join("Cargo.toml")).unwrap();
    assert!(
        !manifest.contains("alo-updating ="),
        "alo-brokerd depends on alo-updating"
    );
    let source: Vec<(String, String)> = std::fs::read_dir(here.join("src"))
        .unwrap()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            let written = std::fs::read_to_string(&path).unwrap();
            (path.display().to_string(), written)
        })
        .collect();
    assert!(source.len() >= 8, "{source:?}");
    for (path, written) in source {
        for program in [
            "bootc",
            "rpm-ostree",
            "ostree admin",
            "std::process::Command",
        ] {
            assert!(!written.contains(program), "{path} names {program}");
        }
    }
}
