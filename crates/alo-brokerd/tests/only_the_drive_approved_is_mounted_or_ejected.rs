//! The storage verbs mount or eject exactly the removable drive a person
//! approved, as the disk service reports it now, for the signed-in person — and
//! nothing else, ever.
//!
//! Task 4 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *the storage
//! verbs — mount a removable drive a person plugged in, eject it — take the drive
//! by its stable identity, never format, repartition or erase anything, and a
//! removable drive mounts for the signed-in person only, with no grant made to an
//! agent by plugging it in.* Every request here crosses the broker's real
//! decision (`alo_broker::Broker::heard`) under a genuine token, so what is
//! tested is the door and the carrier together, against a disk service that
//! records what it was asked.

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
use alo_brokerd::{Carriers, Network, Proxy, Storage};
use alo_drives::{
    Drive, DriveService, Drives, Filesystem, Health, LoginName, NotAnswering, NotDone, Plugged,
    TheDrives,
};
use alo_networks::{NetworkService, Networks, Saved, TheNetworks, Visible};
use alo_record::{AtTheBroker, Happened, Record};

/// The key this test's turn and broker share.
const THE_KEY: [u8; 32] = [43; 32];

/// A disk service that reports what it is given and remembers what it was asked
/// to change.
#[derive(Debug)]
struct Reporting {
    /// What it reports.
    drives: TheDrives,
    /// What it was asked to change, in order.
    asked: RefCell<Vec<String>>,
}

impl Drives for Reporting {
    fn now(&self) -> Result<TheDrives, NotAnswering> {
        Ok(self.drives.clone())
    }
}

impl DriveService for Reporting {
    fn mount(
        &self,
        drive: &Drive,
        filesystem: &Filesystem,
        for_the_person: &LoginName,
    ) -> Result<(), NotDone> {
        self.asked.borrow_mut().push(format!(
            "mount {} {} for {}",
            drive.identifier(),
            filesystem.uuid(),
            for_the_person.as_str()
        ));
        Ok(())
    }

    fn eject(&self, drive: &Drive) -> Result<(), NotDone> {
        self.asked
            .borrow_mut()
            .push(format!("eject {}", drive.identifier()));
        Ok(())
    }
}

/// A network manager with nothing, which must never be asked to change it.
#[derive(Debug)]
struct NoNetworks;

impl Networks for NoNetworks {
    fn now(&self) -> Result<TheNetworks, alo_networks::NotAnswering> {
        Ok(TheNetworks {
            visible: Vec::new(),
            saved: Vec::new(),
            wireless_on: false,
            primary: None,
        })
    }
}

impl NetworkService for NoNetworks {
    fn join(&self, _: &Visible) -> Result<(), alo_networks::NotDone> {
        panic!("a storage test joined a network")
    }

    fn forget(&self, _: &Saved) -> Result<(), alo_networks::NotDone> {
        panic!("a storage test forgot a network")
    }

    fn switch_wireless(&self, _: bool) -> Result<(), alo_networks::NotDone> {
        panic!("a storage test switched the radio")
    }
}

/// A stick a person plugged in, with one filesystem mounted and one not.
fn a_stick() -> Drive {
    Drive::reported(
        "Kingston-DataTraveler-1C1B",
        Plugged::Removable,
        Health::NotKnown,
    )
    .unwrap()
    .with(Filesystem::reported("1234-ABCD", false).unwrap())
    .with(Filesystem::reported("5678-EF01", true).unwrap())
}

/// The machine's own drive, holding the system.
fn the_system_drive() -> Drive {
    Drive::reported("Samsung-SSD-970-S4EW", Plugged::BuiltIn, Health::Good)
        .unwrap()
        .with(Filesystem::reported("root-uuid", true).unwrap())
        .with(Filesystem::reported("data-uuid", false).unwrap())
}

/// An account file in a folder of this test's own, removed when it is dropped,
/// pass or fail.
struct Accounts {
    /// The folder.
    folder: PathBuf,
    /// The file in it, which may not have been written.
    file: PathBuf,
}

impl std::ops::Deref for Accounts {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.file
    }
}

impl Drop for Accounts {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.folder));
    }
}

/// An account file holding `contents`, or none at all.
fn accounts(named: &str, contents: Option<String>) -> Accounts {
    let folder = std::env::temp_dir().join(format!(
        "alo-brokerd-storage-{}-{named}",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("passwd");
    if let Some(contents) = contents {
        std::fs::write(&file, contents).unwrap();
    }
    Accounts { folder, file }
}

/// The account file a machine has, as far as this test can make one: the person,
/// as whoever runs this test — root, in a build box — and the agent's own login.
fn the_machines_accounts() -> String {
    format!(
        "alo:x:{}:1000:alo OS:/home/alo:/bin/bash\n\
         alo-agent:x:60989:60989::/:/usr/sbin/nologin\n",
        our_user()
    )
}

/// A broker whose door hears this test, carrying the storage verbs out against
/// `drives`, mounting for the person the account file at `accounts` names.
fn a_broker(
    drives: Vec<Drive>,
    accounts: &Path,
) -> Broker<Record, Carriers<NoNetworks, Reporting>> {
    Broker::new(
        Door::handed_to(our_user()),
        ApprovingKey::of(&THE_KEY),
        Record::default(),
        Carriers::of(
            Network::against(NoNetworks),
            Proxy::handed_over(
                Path::new("/nonexistent-alo-brokerd/wanted.json"),
                Path::new("/nonexistent-alo-brokerd/proxy.json"),
                our_user(),
            ),
            Storage::against(
                Reporting {
                    drives: TheDrives { drives },
                    asked: RefCell::default(),
                },
                accounts,
                our_user(),
            ),
        ),
    )
}

/// Ask `broker` for `verb`, under a genuine token for approval `approval`.
fn ask(
    broker: &mut Broker<Record, Carriers<NoNetworks, Reporting>>,
    verb: SystemVerb,
    approval: u64,
) -> Answer {
    let now = SystemTime::now();
    let request = Request::of(verb, ApprovingKey::of(&THE_KEY).issue(&verb, approval, now));
    broker.heard(Some(our_user()), request.written().as_bytes(), now)
}

/// What the disk service was asked to change.
fn asked(broker: &Broker<Record, Carriers<NoNetworks, Reporting>>) -> Vec<String> {
    broker.carrying().storage().service().asked.borrow().clone()
}

/// The identity of the filesystem `uuid` on `drive`.
fn filesystem(drive: &Drive, uuid: &str) -> Identity {
    let on_it = drive
        .filesystems()
        .iter()
        .find(|filesystem| filesystem.uuid() == uuid)
        .unwrap();
    Identity::of_what_was_reported(&drive.filesystem_as_reported(on_it))
}

/// The identity of `drive`.
fn drive(drive: &Drive) -> Identity {
    Identity::of_what_was_reported(&drive.as_reported())
}

/// **A removable drive mounts for the signed-in person, and ejects**: the
/// filesystem approved and not the other on the same stick, mounted as the
/// person's own login — never the agent's, never root's — and the stick ejected
/// by its own identity; each carried once and written down, and nothing in the
/// record but what the broker answered.
#[test]
fn a_removable_drive_mounts_for_the_signed_in_person_and_ejects() {
    let stick = a_stick();
    let kept = accounts("mounts", Some(the_machines_accounts()));
    let mut broker = a_broker(vec![the_system_drive(), stick.clone()], &kept);

    let mount = SystemVerb::MountDrive(filesystem(&stick, "1234-ABCD"));
    let eject = SystemVerb::EjectDrive(drive(&stick));
    assert_eq!(ask(&mut broker, mount, 1), Answer::Carried);
    assert_eq!(ask(&mut broker, eject, 2), Answer::Carried);
    assert_eq!(
        asked(&broker),
        [
            "mount Kingston-DataTraveler-1C1B 1234-ABCD for alo",
            "eject Kingston-DataTraveler-1C1B",
        ]
    );

    let entries: Vec<&Happened> = broker
        .recording()
        .everything()
        .map(|entry| entry.happened())
        .collect();
    assert_eq!(entries.len(), 2, "{entries:?}");
    assert!(
        entries
            .iter()
            .all(|happened| matches!(happened, Happened::Brokered { refused: None, .. })),
        "mounting a drive wrote something other than the broker's answer: {entries:?}"
    );
}

/// **A drive that is part of the machine is neither mounted nor ejected**,
/// whatever identity names it: not its unmounted data partition, not the drive
/// itself.
#[test]
fn a_drive_that_is_part_of_the_machine_is_neither_mounted_nor_ejected() {
    let system = the_system_drive();
    let kept = accounts("built-in", Some(the_machines_accounts()));
    let mut broker = a_broker(vec![system.clone(), a_stick()], &kept);
    for (verb, approval) in [
        (SystemVerb::MountDrive(filesystem(&system, "data-uuid")), 1),
        (SystemVerb::MountDrive(filesystem(&system, "root-uuid")), 2),
        (SystemVerb::EjectDrive(drive(&system)), 3),
    ] {
        assert_eq!(
            ask(&mut broker, verb, approval),
            Answer::Refused(AtTheBroker::NotCarried),
            "{verb:?}"
        );
    }
    assert!(asked(&broker).is_empty(), "{:?}", asked(&broker));
}

/// **A drive not reported now, or reported twice, is not touched, and nothing
/// else is in its place**: a stick that was taken out after the approval, a
/// drive's identity offered to mount, a filesystem's offered to eject, a device
/// name where an identity goes, a filesystem already mounted, and an identity two
/// sticks answer to.
#[test]
fn a_drive_not_reported_now_or_reported_twice_is_not_touched() {
    let stick = a_stick();
    let unplugged = Drive::reported("SanDisk-Cruzer-4C53", Plugged::Removable, Health::NotKnown)
        .unwrap()
        .with(Filesystem::reported("CAFE-F00D", false).unwrap());
    let kept = accounts("not-reported", Some(the_machines_accounts()));
    let mut broker = a_broker(vec![stick.clone()], &kept);
    for (verb, approval) in [
        (
            SystemVerb::MountDrive(filesystem(&unplugged, "CAFE-F00D")),
            1,
        ),
        (SystemVerb::EjectDrive(drive(&unplugged)), 2),
        (SystemVerb::MountDrive(drive(&stick)), 3),
        (SystemVerb::EjectDrive(filesystem(&stick, "1234-ABCD")), 4),
        (
            SystemVerb::MountDrive(Identity::of_what_was_reported(b"/dev/sdb1")),
            5,
        ),
        (SystemVerb::MountDrive(filesystem(&stick, "5678-EF01")), 6),
    ] {
        assert_eq!(
            ask(&mut broker, verb, approval),
            Answer::Refused(AtTheBroker::NotCarried),
            "{verb:?}"
        );
    }
    assert!(asked(&broker).is_empty(), "{:?}", asked(&broker));

    let kept = accounts("twice", Some(the_machines_accounts()));
    let mut twice = a_broker(vec![stick.clone(), a_stick()], &kept);
    for (verb, approval) in [
        (SystemVerb::MountDrive(filesystem(&stick, "1234-ABCD")), 7),
        (SystemVerb::EjectDrive(drive(&stick)), 8),
    ] {
        assert_eq!(
            ask(&mut twice, verb, approval),
            Answer::Refused(AtTheBroker::NotCarried)
        );
    }
    assert!(asked(&twice).is_empty(), "{:?}", asked(&twice));
}

/// **Nobody the account file cannot name has a drive mounted for them**: no
/// account file, none for the person, two for the person, or a name that is not
/// a login name — and ejecting, which mounts for nobody, is unaffected.
#[test]
fn a_person_the_accounts_do_not_name_has_nothing_mounted() {
    let stick = a_stick();
    let person = our_user();
    for (named, contents) in [
        ("no-file", None),
        (
            "nobody",
            Some(format!(
                "somebody:x:{}:1::/:/bin/sh\n",
                person.wrapping_add(1)
            )),
        ),
        (
            "two",
            Some(format!("alo:x:{person}:1:::\nother:x:{person}:1:::\n")),
        ),
        ("odd", Some(format!("Alo Person:x:{person}:1:::\n"))),
    ] {
        let kept = accounts(named, contents);
        let mut broker = a_broker(vec![stick.clone()], &kept);
        assert_eq!(
            ask(
                &mut broker,
                SystemVerb::MountDrive(filesystem(&stick, "1234-ABCD")),
                1
            ),
            Answer::Refused(AtTheBroker::NotCarried),
            "{named}"
        );
        assert!(asked(&broker).is_empty(), "{named}: {:?}", asked(&broker));
    }
}

/// **A drive is changed under a genuine approval, once**: a token under another
/// key mounts nothing, and the genuine token spent once is refused the second
/// time.
#[test]
fn a_drive_is_changed_only_under_a_genuine_approval_and_once() {
    let stick = a_stick();
    let kept = accounts("approval", Some(the_machines_accounts()));
    let mut broker = a_broker(vec![stick.clone()], &kept);
    let verb = SystemVerb::MountDrive(filesystem(&stick, "1234-ABCD"));
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
    assert_eq!(
        asked(&broker),
        ["mount Kingston-DataTraveler-1C1B 1234-ABCD for alo"]
    );
}

/// **The process that mounts a drive can make no grant**: neither it nor the
/// crate that speaks to the disk service depends on anything that holds, makes
/// or offers one.
#[test]
fn the_broker_can_make_no_grant() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .unwrap_or_default();
    assert!(manifest.contains("alo-drives"), "{manifest}");
    for grants in [
        "alo-capability",
        "alo-granted",
        "alo-picking",
        "alo-portals",
    ] {
        assert!(
            !manifest.contains(&format!("{grants} =")),
            "alo-brokerd depends on {grants}"
        );
    }
}
