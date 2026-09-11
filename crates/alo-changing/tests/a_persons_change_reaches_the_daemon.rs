//! A person's change to the grants reaches the file the daemon re-reads —
//! measured, refusals beside the answers.
//!
//! Everything here goes through one [`Changing`]: a real folder picked
//! through `alo-picking` on a real disk, a real row derived through
//! `alo-granted`, the real `alo-remembering` file, and — where the test is
//! about the wire — a real Unix socket with a thread on the other end
//! speaking `alo-protocol`. The change is always read **back off the disk**
//! through `alo_remembering::remembered`, because the acceptance is about
//! what the daemon would find there, not about what a value in memory
//! believes.
//!
//! The order — the knock after the write, never before — is measured twice:
//! by a door of this file's own that reads the file at the moment it is
//! knocked on, and over a real socket by the far thread doing the same before
//! it answers.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};
use alo_changing::{Changing, Gone, Knocking, Made, NotChanged, Stood, TheDaemonsDoor};
use alo_granted::{Listing, Seen};
use alo_picking::{Chosen, Granting, OnThisDisk, Picker};
use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::Strings;

/// The agent every test here grants to.
const HERS: &str = "@files";

/// The moment this file is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant made in these tests lasts.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The ordinary granting these tests make.
fn for_an_hour() -> Granting {
    Granting::to(HERS, hour())
}

/// A folder of this test's own, empty, on the real disk.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir()
        .join("alo-changing-tests")
        .join(format!("{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A home with an `Invoices` folder in it, and that folder picked the one way
/// a folder can be: a person standing in it.
fn invoices_picked_in(folder: &Path) -> (PathBuf, Chosen) {
    let home = folder.join("home");
    let invoices = home.join("Invoices");
    std::fs::create_dir_all(&invoices).unwrap();
    let mut picker = Picker::standing_in(&home, &OnThisDisk).unwrap();
    picker.go_into("Invoices", &OnThisDisk).unwrap();
    (invoices, picker.pick().unwrap())
}

/// Whether `@files` may reach this path under this list, at noon.
fn may_reach(grants: &Grants, at: &Path) -> bool {
    grants.permits(&Grantee::named(HERS), &Ask::path(at), noon())
}

/// The machine's one vocabulary, with this crate's words collected in it.
fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's words collect"))
}

/// The one row a list of one grant shows.
fn the_row(grants: &Grants) -> Seen {
    Listing::of(grants, noon()).rows().first().unwrap().clone()
}

/// A door of this file's own: it counts the knocks, and at each one it reads
/// the grants file — so *the knock came after the write* is a measurement,
/// not a reading of `Changing`'s source.
#[derive(Debug)]
struct AtTheDoor {
    /// The file the daemon would re-read.
    reading: PathBuf,
    /// How many knocks arrived.
    knocks: Cell<u32>,
    /// What the file held at the moment of each knock.
    seen: RefCell<Vec<Grants>>,
}

impl AtTheDoor {
    /// A door that will read this file when knocked on.
    fn watching(reading: &Path) -> Self {
        Self {
            reading: reading.to_owned(),
            knocks: Cell::new(0),
            seen: RefCell::new(Vec::new()),
        }
    }
}

impl Knocking for AtTheDoor {
    fn knock(&self) -> Stood {
        self.knocks.set(self.knocks.get() + 1);
        let grants = alo_remembering::remembered(&self.reading, noon()).unwrap_or_default();
        let holding = u64::try_from(grants.active_at(noon()).count()).unwrap();
        self.seen.borrow_mut().push(grants);
        Stood::Heard { holding }
    }
}

/// **A grant made through the person's half is on the disk before the knock,
/// and the knock is sent exactly once.** The change is then read back off the
/// disk through `alo_remembering::remembered` — the daemon's own road in.
#[test]
fn a_grant_made_through_it_is_on_the_disk_before_the_one_knock() {
    let folder = a_folder_of_our_own("grant-then-knock");
    let (invoices, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let made = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &picked, noon())
        .unwrap();

    let invoice = invoices.join("march.pdf");
    assert!(matches!(
        made,
        Made::Granted {
            stood: Stood::Heard { holding: 1 },
            ..
        }
    ));
    assert_eq!(door.knocks.get(), 1, "one change, one knock");
    assert!(
        may_reach(door.seen.borrow().first().unwrap(), &invoice),
        "the knock arrived before the grant was on the disk"
    );
    assert!(
        may_reach(&alo_remembering::remembered(&at, noon()).unwrap(), &invoice),
        "the grant did not reach the file the daemon re-reads"
    );
    assert!(may_reach(&grants, &invoice), "the caller's own list lagged");
}

/// **A revocation made through the person's half reaches the disk, and its
/// knock arrives after the shorter list is written.** A grant missing from
/// the file is a grant revoked — the daemon's own reading — so what the door
/// sees at the second knock is the revocation, already kept.
#[test]
fn a_revocation_made_through_it_is_off_the_disk_before_its_knock() {
    let folder = a_folder_of_our_own("revoke-then-knock");
    let (invoices, picked) = invoices_picked_in(&folder);
    let invoice = invoices.join("march.pdf");
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let mut changing = Changing::of(&mut grants, &at, &door);
    changing.granted(&for_an_hour(), &picked, noon()).unwrap();

    let row = the_row(changing.holding());
    let gone = changing.revoked(&row, noon()).unwrap();

    assert!(matches!(
        gone,
        Gone::Revoked {
            stood: Stood::Heard { holding: 0 },
        }
    ));
    assert_eq!(door.knocks.get(), 2, "two changes, two knocks");
    assert!(
        !may_reach(door.seen.borrow().get(1).unwrap(), &invoice),
        "the second knock arrived before the revocation was on the disk"
    );
    assert!(
        !may_reach(&alo_remembering::remembered(&at, noon()).unwrap(), &invoice),
        "the revocation did not reach the file the daemon re-reads"
    );
    assert!(!may_reach(&grants, &invoice));
}

/// **Over a real socket, the knock is the empty one and the grant is already
/// readable when it arrives.** The far side is a thread standing where
/// `alo-agentd` stands: it reads one line, holds it to being
/// `FromAPerson::Granted` with nothing in it, re-reads the file the way the
/// daemon does, and answers with what it found — which the person's side
/// reads back as [`Stood::Heard`].
#[test]
fn a_running_daemon_hears_the_knock_after_the_file_says_so() {
    let folder = a_folder_of_our_own("real-socket");
    let (invoices, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let socket = folder.join("agentd.sock");
    let listener = UnixListener::bind(&socket).unwrap();

    let reading = at.clone();
    let invoice = invoices.join("march.pdf");
    let daemon = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        assert_eq!(
            FromAPerson::read(line.trim_end()).unwrap(),
            FromAPerson::Granted,
            "the knock carried something, or was not the knock"
        );
        // The daemon's own answer to the knock: read the file again. The
        // grant must already be there — the knock came after the write.
        let grants = alo_remembering::remembered(&reading, noon()).unwrap();
        assert!(may_reach(&grants, &invoice));
        let holding = u64::try_from(grants.active_at(noon()).count()).unwrap();
        let answer = ToAPerson::granted(holding).written().unwrap();
        (&stream).write_all(answer.as_bytes()).unwrap();
        (&stream).write_all(b"\n").unwrap();
    });

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let made = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &picked, noon())
        .unwrap();
    daemon.join().unwrap();

    assert!(matches!(
        made,
        Made::Granted {
            stood: Stood::Heard { holding: 1 },
            ..
        }
    ));
}

/// **A machine with no daemon to knock is not an error.** The change stands
/// on the disk — readable through the daemon's own road in — and the person
/// is told, in a declared sentence, that it takes effect at the next sign-in
/// and there is nothing more to do.
#[test]
fn a_machine_with_no_daemon_keeps_the_change_for_the_next_sign_in() {
    let folder = a_folder_of_our_own("no-daemon");
    let (invoices, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let door = TheDaemonsDoor::at(&folder.join("nobody-listens.sock"));

    let mut grants = Grants::default();
    let made = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &picked, noon())
        .unwrap();

    let Made::Granted { stood, .. } = made else {
        unreachable!("the grant was not made: {made:?}");
    };
    assert_eq!(stood, Stood::AtTheNextSignIn);
    let explained = stood.explained(&in_english()).unwrap();
    assert!(explained.contains("next sign-in"), "{explained}");
    assert!(
        may_reach(
            &alo_remembering::remembered(&at, noon()).unwrap(),
            &invoices.join("march.pdf")
        ),
        "the change did not stand for the next sign-in"
    );
}

/// **A daemon that could not read the list again is never read as heard.**
/// The far side answers the knock with a refusal, and what comes back is the
/// daemon's own sentence, carried — the one situation with something left to
/// look at, and the one a machine must not dress up as either of the others.
#[test]
fn a_daemon_that_turned_the_rereading_away_is_not_read_as_heard() {
    let folder = a_folder_of_our_own("turned-away");
    let (_, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let socket = folder.join("agentd.sock");
    let listener = UnixListener::bind(&socket).unwrap();

    let daemon = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        let said = in_english().say(
            &alo_changing::words::NOT_KEPT.key(),
            &alo_strings::Filling::nothing(),
        );
        let answer = ToAPerson::refused(&said).written().unwrap();
        (&stream).write_all(answer.as_bytes()).unwrap();
        (&stream).write_all(b"\n").unwrap();
    });

    let door = TheDaemonsDoor::at(&socket);
    let mut grants = Grants::default();
    let made = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &picked, noon())
        .unwrap();
    daemon.join().unwrap();

    let Made::Granted { stood, .. } = made else {
        unreachable!("the grant was not made: {made:?}");
    };
    assert!(!stood.was_heard(), "a refusal was read as heard: {stood:?}");
    let explained = stood.explained(&in_english()).unwrap();
    assert!(explained.contains("was not saved"), "{explained}");
}

/// **A write that fails leaves the file as it was, sends no knock, and is
/// told in words** — the grant's side. The folder the file would live in is
/// not there, which `alo-remembering` refuses rather than makes.
#[test]
fn a_grant_the_disk_would_not_take_changes_nothing_and_knocks_nobody() {
    let folder = a_folder_of_our_own("grant-not-kept");
    let (_, picked) = invoices_picked_in(&folder);
    let at = folder.join("never-made").join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let refused = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &picked, noon())
        .unwrap_err();

    assert!(matches!(refused, NotChanged::NotKept(_)), "{refused:?}");
    assert_eq!(door.knocks.get(), 0, "a change that failed was knocked for");
    assert!(!at.exists(), "the file appeared after all");
    assert!(
        grants.is_empty(),
        "the caller's list moved without the disk"
    );
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("was not saved"), "{said}");
}

/// **A write that fails leaves the grant standing** — the revocation's side,
/// which is the failure a person acts on. The staging path is squatted by a
/// directory, so the replacement cannot even begin; the file, the list and
/// the daemon all still hold the grant, and nobody was knocked about a
/// revocation that did not land.
#[test]
fn a_revocation_the_disk_would_not_take_leaves_the_grant_standing() {
    let folder = a_folder_of_our_own("revoke-not-kept");
    let (invoices, picked) = invoices_picked_in(&folder);
    let invoice = invoices.join("march.pdf");
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let mut changing = Changing::of(&mut grants, &at, &door);
    changing.granted(&for_an_hour(), &picked, noon()).unwrap();
    let row = the_row(changing.holding());

    // A directory where the staging file goes: the next write cannot happen,
    // whoever this test runs as. (Permission bits would not refuse root, and
    // the loop's checkouts run these tests as root.)
    std::fs::create_dir(folder.join("grants.toml.new")).unwrap();

    let refused = changing.revoked(&row, noon()).unwrap_err();
    assert!(matches!(refused, NotChanged::NotKept(_)), "{refused:?}");
    assert_eq!(
        door.knocks.get(),
        1,
        "the failed revocation was knocked for"
    );
    assert!(
        may_reach(&alo_remembering::remembered(&at, noon()).unwrap(), &invoice),
        "the file lost the grant although the write was refused"
    );
    assert!(
        may_reach(&grants, &invoice),
        "the caller's list dropped a grant the disk still holds"
    );
    assert!(!refused.said(&in_english()).is_a_bug());
}

/// **A stale row changes nothing, writes nothing and knocks nobody.** The
/// file already says what the person wanted said, byte for byte.
#[test]
fn a_stale_row_writes_nothing_and_knocks_nobody() {
    let folder = a_folder_of_our_own("stale-row");
    let (_, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let mut changing = Changing::of(&mut grants, &at, &door);
    changing.granted(&for_an_hour(), &picked, noon()).unwrap();
    let row = the_row(changing.holding());
    assert!(matches!(
        changing.revoked(&row, noon()).unwrap(),
        Gone::Revoked { .. }
    ));

    let file_before = std::fs::read(&at).unwrap();
    let knocks_before = door.knocks.get();
    let again = changing.revoked(&row, noon()).unwrap();

    assert_eq!(again, Gone::AlreadyGone);
    assert_eq!(
        door.knocks.get(),
        knocks_before,
        "a nothing was knocked for"
    );
    assert_eq!(
        std::fs::read(&at).unwrap(),
        file_before,
        "a nothing rewrote the file"
    );
}

/// **Picking nothing grants nothing, writes nothing and knocks nobody** — a
/// closed picker is not a change, so there is no file for a daemon to re-read
/// and no knock to send.
#[test]
fn picking_nothing_writes_nothing_and_knocks_nobody() {
    let folder = a_folder_of_our_own("picked-nothing");
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let made = Changing::of(&mut grants, &at, &door)
        .granted(&for_an_hour(), &Chosen::Nothing, noon())
        .unwrap();

    assert_eq!(made, Made::Nothing);
    assert_eq!(door.knocks.get(), 0);
    assert!(!at.exists());
    assert!(grants.is_empty());
}

/// **A grant the machine refuses never reaches the disk**, and the refusal is
/// `alo-capability`'s own sentence — the words a person reads about a grant
/// belong to the crate that decides what one is.
#[test]
fn a_grant_the_machine_refuses_is_not_written_and_not_knocked() {
    let folder = a_folder_of_our_own("refused-grant");
    let (_, picked) = invoices_picked_in(&folder);
    let at = folder.join("grants.toml");
    let door = AtTheDoor::watching(&at);

    let mut grants = Grants::default();
    let refused = Changing::of(&mut grants, &at, &door)
        .granted(&Granting::to("   ", hour()), &picked, noon())
        .unwrap_err();

    assert!(matches!(refused, NotChanged::NotAGrant(_)), "{refused:?}");
    assert_eq!(door.knocks.get(), 0);
    assert!(!at.exists());
    assert!(grants.is_empty());
    assert!(!refused.said(&in_english()).is_a_bug());
}
