//! **At sign-out, and at the next sign-in, the grants that last until sign-out
//! go — and nothing else does** (the owner's decision of 2026-11-16,
//! `docs/design/when-alo-asks-for-an-application.md`).
//!
//! Measured the way `a_persons_change_reaches_the_daemon.rs` measures a grant:
//! the file is read back off the disk, the daemon's own question
//! (`Grants::permits`) is asked of what it would find there, and a door of this
//! file's own reads the file at the moment it is knocked on.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{ATerminalForTheAgent, Ask, Grant, Grantee, Grants, Reach};
use alo_changing::{Changing, Knocking, RevokingPairings, Stood, Unpaired};
use alo_nearby::MachineId;

/// The agent every grant here is to.
const ALO: &str = "alo";

/// The moment this file is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A folder of this test's own, empty, on the real disk.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir()
        .join("alo-changing-session-tests")
        .join(format!("{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A door that counts the knocks and reads the grants file at each.
#[derive(Debug)]
struct AtTheDoor {
    /// The file the daemon would re-read.
    reading: PathBuf,
    /// How many knocks arrived.
    knocks: Cell<u32>,
    /// What the file held at the moment of each knock.
    seen: RefCell<Vec<Grants>>,
}

impl Knocking for AtTheDoor {
    fn knock(&self) -> Stood {
        self.knocks.set(self.knocks.get() + 1);
        self.seen
            .borrow_mut()
            .push(alo_remembering::remembered(&self.reading, noon()).unwrap());
        Stood::Heard { holding: 0 }
    }
}

impl RevokingPairings for AtTheDoor {
    fn revoke_pairing(&self, _with: &MachineId) -> Unpaired {
        Unpaired::NobodyThere
    }
}

/// Whether alo may use `identifier` under the grants the file holds now.
fn the_daemon_permits(at: &Path, identifier: &str) -> bool {
    alo_remembering::remembered(at, noon()).unwrap().permits(
        &Grantee::named(ALO),
        &Ask::application(identifier),
        noon(),
    )
}

/// VSCodium until sign-out, and Firefox until removed, kept on the disk.
fn a_session_and_a_remembered_grant(at: &Path) -> Grants {
    let alo = Grantee::named(ALO);
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_until_sign_out(
            &alo,
            Reach::Application("com.vscodium.codium".to_owned()),
            noon(),
            ATerminalForTheAgent::NotAtAll,
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked_until_removed(
            &alo,
            Reach::Application("org.mozilla.firefox".to_owned()),
            noon(),
            ATerminalForTheAgent::NotAtAll,
        )
        .unwrap(),
    );
    alo_remembering::kept(at, &grants, noon()).unwrap();
    grants
}

/// **The session grant goes, the remembered one stays, and the daemon is told
/// only after the file says so.**
#[test]
fn the_session_grant_goes_and_the_remembered_one_stays() {
    let folder = a_folder_of_our_own("goes");
    let at = folder.join("grants.toml");
    let mut grants = a_session_and_a_remembered_grant(&at);
    assert!(
        the_daemon_permits(&at, "com.vscodium.codium"),
        "the control"
    );
    let door = AtTheDoor {
        reading: at.clone(),
        knocks: Cell::new(0),
        seen: RefCell::new(Vec::new()),
    };
    let gone = Changing::of(&mut grants, &at, &door)
        .the_session_is_over(noon())
        .unwrap();
    assert_eq!(gone, 1);
    assert!(!the_daemon_permits(&at, "com.vscodium.codium"));
    assert!(the_daemon_permits(&at, "org.mozilla.firefox"));
    assert_eq!(door.knocks.get(), 1);
    let seen = door.seen.borrow();
    assert!(
        !seen.first().unwrap().permits(
            &Grantee::named(ALO),
            &Ask::application("com.vscodium.codium"),
            noon()
        ),
        "the daemon was told before the file said so"
    );
    assert_eq!(grants.len(), 1, "the list held in memory was not changed");
}

/// **A power cut, then a sign-in**: the file still holds the session grant
/// after the machine went off without signing out, and the next sign-in's
/// call takes it away.
#[test]
fn after_a_power_cut_the_next_sign_in_takes_it_away() {
    let folder = a_folder_of_our_own("power-cut");
    let at = folder.join("grants.toml");
    drop(a_session_and_a_remembered_grant(&at));
    // **The power went**: nothing ran. The file is as it was.
    assert!(the_daemon_permits(&at, "com.vscodium.codium"));
    // **The next sign-in**, reading the file fresh as a session does.
    let mut grants = alo_remembering::remembered(&at, noon()).unwrap();
    let door = AtTheDoor {
        reading: at.clone(),
        knocks: Cell::new(0),
        seen: RefCell::new(Vec::new()),
    };
    assert_eq!(
        Changing::of(&mut grants, &at, &door)
            .the_session_is_over(noon())
            .unwrap(),
        1
    );
    assert!(!the_daemon_permits(&at, "com.vscodium.codium"));
    assert!(the_daemon_permits(&at, "org.mozilla.firefox"));
}

/// **Nothing to clear writes nothing and knocks nobody**: a sign-in with no
/// session grant leaves the file byte for byte.
#[test]
fn nothing_to_clear_writes_nothing_and_knocks_nobody() {
    let folder = a_folder_of_our_own("nothing");
    let at = folder.join("grants.toml");
    let mut grants = a_session_and_a_remembered_grant(&at);
    let door = AtTheDoor {
        reading: at.clone(),
        knocks: Cell::new(0),
        seen: RefCell::new(Vec::new()),
    };
    assert_eq!(
        Changing::of(&mut grants, &at, &door)
            .the_session_is_over(noon())
            .unwrap(),
        1
    );
    let before = std::fs::read(&at).unwrap();
    assert_eq!(
        Changing::of(&mut grants, &at, &door)
            .the_session_is_over(noon())
            .unwrap(),
        0
    );
    assert_eq!(std::fs::read(&at).unwrap(), before);
    assert_eq!(door.knocks.get(), 1, "the second call knocked");
}
