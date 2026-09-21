//! Snapshots made, the clock moved past the window, the unit run, and `btrfs
//! subvolume list` asked afterwards — on a real `btrfs` filesystem, with the
//! base's own program doing the removing.
//!
//! Everything else in this crate's suite runs against ordinary directories and
//! a stand-in for the one thing a developer's machine cannot do. This is that
//! one thing, measured: a filesystem with subvolumes, read-only snapshots that
//! `rm -rf` will not clear, and `btrfs subvolume delete` — which needs
//! `CAP_SYS_ADMIN` and is the whole reason
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s
//! seventh term puts expiry in a privileged unit rather than in `alo-turn`.
//!
//! It measures both directions, because the refusal is the half that matters:
//! with the capability the expired snapshots go and the ones inside the window
//! stay; **without it nothing goes at all**, and the machine is exactly as it
//! was.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `mkfs.btrfs`, `btrfs`, `mount`, `losetup`, `capsh`, a loop device and a
//! gibibyte of room. It is `#[ignore]`d and run by name, and it **fails** rather
//! than skips when something it needs is missing, because a test that passed on
//! a machine that could not run it would be evidence of nothing.
//!
//! ```text
//! cargo test -p alo-letting-go --test on_a_real_btrfs_machine -- --ignored --nocapture
//! ```
//!
//! # What it is not
//!
//! Not the certified machine and not a booted alo OS: it is this crate's code
//! against a real `btrfs` filesystem on a loop device. What it settles is the
//! thing a stand-in cannot — that the program alo OS ships really removes a
//! read-only snapshot, and really cannot without the capability its unit names.

#![cfg(target_os = "linux")]
#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use alo_letting_go::the_folder::{AFTER, BEFORE, THE_TURN, THEIRS, TheTurn, Theirs};
use alo_letting_go::{
    AskingTheDisk, Changes, OnThisMachine, Swept, WhatItIs, WithTheBase, WritingItDown, keeping,
    sweep,
};
use alo_record::{Entry, Happened, WhyLetGo};

/// How long a day is where this test moves a clock.
const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// How big a filesystem is when it has room to spare — comfortably over
/// [`alo_letting_go::THE_FLOOR`], and sparse, so it costs the machine running
/// this nothing until something is written to it.
const ROOM_TO_SPARE: &str = "16G";

/// And how big one is that is under the floor from the moment it is made.
const SHORT_OF_ROOM: &str = "1G";

/// A record that keeps what it was given in memory.
#[derive(Default)]
struct InHand(Vec<Entry>);

impl WritingItDown for InHand {
    fn keep(&mut self, entry: Entry) -> Result<(), String> {
        self.0.push(entry);
        Ok(())
    }
}

/// One program, run, with everything it said — and a panic naming it when it
/// could not be run at all, because a missing tool is this test failing rather
/// than passing quietly.
fn run(program: &str, args: &[&str]) -> (bool, String) {
    let said = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|why| panic!("{program} could not be run: {why}"));
    let mut text = String::from_utf8_lossy(&said.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&said.stderr));
    (said.status.success(), text)
}

/// The same, insisting it worked.
fn must(program: &str, args: &[&str]) -> String {
    let (worked, said) = run(program, args);
    assert!(worked, "{program} {args:?} did not work: {said}");
    said
}

/// A `btrfs` filesystem on a loop device, mounted, and taken down when the
/// test is over however it ends.
struct AFilesystem {
    /// Where the backing file and the mount point are.
    under: PathBuf,
    /// Where it is mounted.
    at: PathBuf,
}

impl AFilesystem {
    /// A `btrfs` filesystem of this size, made and mounted.
    fn made(how_big: &str) -> Self {
        let under = std::env::temp_dir().join(format!(
            "alo-letting-go-btrfs-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let at = under.join("mnt");
        std::fs::create_dir_all(&at).unwrap();
        let disk = under.join("disk.img");

        must("truncate", &["-s", how_big, disk.to_str().unwrap()]);
        must("mkfs.btrfs", &["-q", disk.to_str().unwrap()]);
        must(
            "mount",
            &["-o", "loop", disk.to_str().unwrap(), at.to_str().unwrap()],
        );
        Self { under, at }
    }

    /// What `btrfs subvolume list` says about it now.
    fn subvolumes(&self) -> String {
        must("btrfs", &["subvolume", "list", self.at.to_str().unwrap()])
    }
}

impl Drop for AFilesystem {
    fn drop(&mut self) {
        let _ = run("umount", &[self.at.to_str().unwrap()]);
        let _ = std::fs::remove_dir_all(&self.under);
    }
}

/// One kept turn on that filesystem: a read-only snapshot either side of the
/// home, and what describes it.
fn one_kept_turn(under: &Path, home: &Path, named: &str, taken: SystemTime, did: &str) -> PathBuf {
    let at = under.join(named);
    std::fs::create_dir_all(&at).unwrap();
    for side in [BEFORE, AFTER] {
        // A destination that does not exist yet, and the result checked —
        // `docs/quirks.md`: a snapshot into a destination that already exists
        // is made *inside* it and answers `Read-only file system`.
        let side = at.join(side);
        assert!(!side.exists());
        must(
            "btrfs",
            &[
                "subvolume",
                "snapshot",
                "-r",
                home.to_str().unwrap(),
                side.to_str().unwrap(),
            ],
        );
    }
    let says = at.join(THE_TURN);
    let turn = TheTurn::done(taken, did, &says).unwrap();
    std::fs::write(&says, serde_json::to_string(&turn).unwrap()).unwrap();
    at
}

/// A person, a home subvolume, their own window on the disk, and their
/// directory under the undo folder — everything up to the kept turns.
fn a_machine_with_a_person_on_it(mounted: &Path) -> (PathBuf, PathBuf) {
    let homes = mounted.join("home");
    std::fs::create_dir_all(&homes).unwrap();
    let home = homes.join("ada");
    must("btrfs", &["subvolume", "create", home.to_str().unwrap()]);
    std::fs::write(home.join("March.pdf"), b"an invoice").unwrap();

    // The person's own window: what alo OS ships, written as their own file so
    // that the one setting really is read off a disk here.
    let settings = mounted.join("settings");
    std::fs::create_dir_all(&settings).unwrap();
    keeping::keep(&settings.join(keeping::THE_FILE), &Changes::untouched()).unwrap();

    let ada = mounted.join("undo").join("ada");
    std::fs::create_dir_all(&ada).unwrap();
    let says = ada.join(THEIRS);
    let theirs = Theirs::at(&settings, &says).unwrap();
    std::fs::write(&says, serde_json::to_string(&theirs).unwrap()).unwrap();
    (home, ada)
}

/// This measurement is root's, and says so rather than passing quietly.
fn as_root() {
    assert_eq!(
        must("id", &["-u"]).trim(),
        "0",
        "this measurement is root's: removing a read-only snapshot needs CAP_SYS_ADMIN"
    );
}

/// **A real `btrfs` machine: what the window no longer reaches is removed, what
/// it reaches is kept, and `btrfs subvolume list` says so afterwards.**
#[test]
#[ignore = "needs root, a loop device, btrfs-progs and capsh"]
fn on_a_real_btrfs_machine_the_expired_snapshots_go_and_the_rest_stay() {
    as_root();

    let filesystem = AFilesystem::made(ROOM_TO_SPARE);
    let mounted = filesystem.at.clone();

    // The filesystem alo OS installs, read the way the unit reads it.
    assert_eq!(
        OnThisMachine.what_it_is(&mounted).unwrap(),
        WhatItIs::OneThatKeeps,
        "the filesystem this test made is not one that keeps"
    );

    // **A machine with room to spare**, so that the window is the only thing
    // deciding and nothing is lost to the disk.
    assert!(
        OnThisMachine.free(&mounted).unwrap() > alo_letting_go::THE_FLOOR,
        "this filesystem is under the floor, so the disk would decide as well"
    );

    // A person's home, as the accounts lane will make one: a subvolume.
    let (home, ada) = a_machine_with_a_person_on_it(&mounted);
    let under = mounted.join("undo");

    let now = SystemTime::now();
    let recent = one_kept_turn(&ada, &home, "recent", now - A_DAY, "move March.pdf");
    let old = one_kept_turn(&ada, &home, "old", now - A_DAY * 30, "archive Old letters");

    let before = filesystem.subvolumes();
    println!("--- btrfs subvolume list, before ---\n{before}");
    assert_eq!(
        before.lines().count(),
        5,
        "the home and four snapshots were not all made: {before}"
    );

    // **A read-only snapshot does not yield to an ordinary removal**, which is
    // the whole reason this needs a privileged unit at all (`docs/quirks.md`).
    assert!(
        std::fs::remove_dir_all(old.join(BEFORE)).is_err(),
        "a read-only snapshot was cleared without the base's own program"
    );

    // **And it does not yield to the base's program without CAP_SYS_ADMIN
    // either.** Dropped from the bounding set, the removal is refused and the
    // machine is exactly as it was.
    let (worked, said) = run(
        "capsh",
        &[
            "--drop=cap_sys_admin",
            "--",
            "-c",
            &format!(
                "btrfs subvolume delete --commit-after {}",
                old.join(BEFORE).display()
            ),
        ],
    );
    println!("--- without CAP_SYS_ADMIN ---\n{said}");
    assert!(!worked, "a snapshot was removed without CAP_SYS_ADMIN");
    assert!(old.join(BEFORE).exists());
    assert_eq!(
        filesystem.subvolumes().lines().count(),
        5,
        "something went while the capability was dropped"
    );

    // One firing of the timer, with the machine's own disk and the base's own
    // program.
    let mut record = InHand::default();
    let swept = sweep(&under, &OnThisMachine, &WithTheBase, &mut record, now);
    let Swept::Done(did) = swept else {
        panic!("a btrfs machine answered that it keeps nothing")
    };
    for line in did.said() {
        println!("alo-letting-go: {line}");
    }
    assert_eq!(did.outside_the_window(), 1, "{:?}", did.said());
    assert_eq!(
        did.for_room(),
        0,
        "a machine with room lost something to it"
    );

    let after = filesystem.subvolumes();
    println!("--- btrfs subvolume list, after ---\n{after}");
    assert_eq!(
        after.lines().count(),
        3,
        "the expired snapshots are still on the disk: {after}"
    );
    assert!(
        !old.exists(),
        "what the window no longer reaches is still there"
    );
    assert!(
        recent.join(BEFORE).exists(),
        "a turn inside the window went"
    );
    assert!(recent.join(AFTER).exists(), "a turn inside the window went");
    assert!(
        home.join("March.pdf").exists(),
        "the person's own home was touched"
    );

    // **And the record names the turn that lost its undo, in the person's own
    // words** — ADR 0045's second term.
    assert_eq!(record.0.len(), 1);
    match record.0[0].happened() {
        Happened::LetGo { why, turns } => {
            assert_eq!(*why, WhyLetGo::OutsideTheWindow);
            assert_eq!(turns.len(), 1);
            assert!(turns[0].did().is("archive Old letters"));
        }
        other => panic!("the record kept {other:?}"),
    }
}

/// **A real `btrfs` machine below the floor: the oldest go first, and the
/// record says the disk needed the room** — ADR 0045's second accepted term.
///
/// The same code, on a filesystem that is under
/// [`alo_letting_go::THE_FLOOR`] from the moment it is made, with every kept
/// turn **inside** the person's window. Nothing here would go for the window's
/// sake; everything that goes, goes because the disk is short.
///
/// **What this cannot show, and where it is shown instead.** A loop device on
/// a developer's machine cannot climb back over a ten-gibibyte floor by
/// removing a few snapshots, so *the machine stops the moment there is room* is
/// not measured here — it is held by
/// `sweeping::tests::under_the_floor_the_oldest_go_first_and_it_stops_when_there_is_room`,
/// against a disk that gives room back. What is measured here is the half a
/// stand-in cannot show: that the order really is oldest first on a real
/// filesystem, and that the base's own program really removes them.
#[test]
#[ignore = "needs root, a loop device, btrfs-progs and capsh"]
fn on_a_real_btrfs_machine_under_the_floor_the_oldest_go_first() {
    as_root();

    let filesystem = AFilesystem::made(SHORT_OF_ROOM);
    let mounted = filesystem.at.clone();
    let free = OnThisMachine.free(&mounted).unwrap();
    assert!(
        free < alo_letting_go::THE_FLOOR,
        "this filesystem is not under the floor: {free} free"
    );

    let (home, ada) = a_machine_with_a_person_on_it(&mounted);
    let under = mounted.join("undo");

    // Three kept turns, **all of them inside the shipped window** — a day, two
    // days and three days old, against seven days and fifty changing turns. The
    // window would take none of them.
    let now = SystemTime::now();
    one_kept_turn(&ada, &home, "c-newest", now - A_DAY, "rename May.pdf");
    one_kept_turn(&ada, &home, "b-middle", now - A_DAY * 2, "move April.pdf");
    one_kept_turn(&ada, &home, "a-oldest", now - A_DAY * 3, "archive March");

    let before = filesystem.subvolumes();
    println!("--- btrfs subvolume list, before ---\n{before}");
    assert_eq!(before.lines().count(), 7, "{before}");

    let mut record = InHand::default();
    let swept = sweep(&under, &OnThisMachine, &WithTheBase, &mut record, now);
    let Swept::Done(did) = swept else {
        panic!("a btrfs machine answered that it keeps nothing")
    };
    for line in did.said() {
        println!("alo-letting-go: {line}");
    }

    let after = filesystem.subvolumes();
    println!("--- btrfs subvolume list, after ---\n{after}");

    assert_eq!(
        did.outside_the_window(),
        0,
        "the window took something, and every turn here is inside it"
    );
    assert_eq!(did.for_room(), 3, "{:?}", did.said());
    assert_eq!(after.lines().count(), 1, "only the home is left: {after}");
    assert!(
        home.join("March.pdf").exists(),
        "the person's own home was touched"
    );

    // **The oldest first, and the record says why** — named in the person's own
    // words, in the order they were let go.
    assert_eq!(record.0.len(), 1);
    match record.0[0].happened() {
        Happened::LetGo { why, turns } => {
            assert_eq!(*why, WhyLetGo::TheDiskNeededTheRoom);
            let named: Vec<&str> = turns.iter().map(|turn| turn.did().as_str()).collect();
            assert_eq!(
                named,
                ["archive March", "move April.pdf", "rename May.pdf"],
                "the oldest did not go first"
            );
        }
        other => panic!("the record kept {other:?}"),
    }
}
