//! A bracket taken on a real `btrfs` filesystem: the snapshot made, read back
//! as a subvolume, found to be read-only, and refused a write — with the base's
//! own program doing the making.
//!
//! Everything else in this crate's suite runs against ordinary directories, and
//! a directory cannot be snapshotted. This is the part a stand-in cannot
//! settle, and it settles three things no unit test can:
//!
//! - **the snapshot is really made**, by the program alo OS ships, with the
//!   arguments `the_bracket_a_turn_takes.rs` names;
//! - **it is read-only**, which is what [ADR 0045 and #752] rest on — *the
//!   agent can take a bracket and can neither alter nor destroy one*. A
//!   writable snapshot would make the first half false;
//! - **it needs no capability.** The whole test runs the taking as an ordinary
//!   call, and ADR 0045's amendment of 2026-10-16 measured the same thing from
//!   the other side: `btrfs subvolume snapshot -r` as the person, no `sudo`,
//!   exits 0.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `mkfs.btrfs`, `btrfs`, `mount`, `truncate`, a loop device and a
//! gibibyte of room. It is `#[ignore]`d and run by name, and it **fails** rather
//! than skips when something it needs is missing, because a test that passed on
//! a machine that could not run it would be evidence of nothing.
//!
//! ```text
//! cargo test -p alo-turn --test a_bracket_on_a_real_btrfs_machine -- --ignored --nocapture
//! ```
//!
//! # What it is not
//!
//! Not a certified machine and not a booted alo OS: it is this crate's code
//! against a real filesystem on a loop device. What it settles is the thing a
//! directory cannot — that the snapshot alo OS takes is real and is read-only.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use alo_turn::the_bracket_a_turn_takes::{WhereThisTurnIsKept, where_their_settings_are};

/// Big enough for a filesystem and a snapshot of an empty home.
const ROOM_TO_SPARE: &str = "1G";

/// Run a program and say whether it worked and what it said.
fn run(program: &str, args: &[&str]) -> (bool, String) {
    let said = Command::new(program).args(args).output();
    match said {
        Ok(done) => (
            done.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(&done.stdout),
                String::from_utf8_lossy(&done.stderr)
            ),
        ),
        Err(why) => (false, why.to_string()),
    }
}

/// Run a program that has to work, or fail the test naming it.
fn must(program: &str, args: &[&str]) -> String {
    let (worked, said) = run(program, args);
    assert!(worked, "`{program} {args:?}` did not work: {said}");
    said
}

/// A `btrfs` filesystem on a loop device, taken down however the test ends.
struct AFilesystem {
    /// Where the backing file and the mount point are.
    under: PathBuf,
    /// Where it is mounted.
    at: PathBuf,
}

impl AFilesystem {
    /// Made and mounted.
    fn made() -> Self {
        let under = std::env::temp_dir().join(format!(
            "alo-turn-bracket-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let at = under.join("mnt");
        std::fs::create_dir_all(&at).unwrap();
        let disk = under.join("disk.img");

        must("truncate", &["-s", ROOM_TO_SPARE, disk.to_str().unwrap()]);
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

/// A home subvolume with something in it, and the person's own undo directory.
fn a_person_on(mounted: &Path) -> (PathBuf, PathBuf) {
    let home = mounted.join("home").join("ada");
    std::fs::create_dir_all(mounted.join("home")).unwrap();
    must("btrfs", &["subvolume", "create", home.to_str().unwrap()]);
    std::fs::write(home.join("march.pdf"), b"an invoice").unwrap();

    let whose = mounted.join("alo-undo").join("ada");
    std::fs::create_dir_all(&whose).unwrap();
    (home, whose)
}

/// **A bracket is taken, it is a subvolume, and it is read-only.**
///
/// The three facts a directory cannot demonstrate, in one walk, with the base's
/// own program asked afterwards rather than this test's own bookkeeping.
#[test]
#[ignore = "needs root, a loop device and btrfs; run by name"]
fn a_bracket_is_taken_and_is_read_only() {
    let disk = AFilesystem::made();
    let (home, whose) = a_person_on(&disk.at);

    where_their_settings_are(&whose, Path::new("/home/ada/.config/alo")).unwrap();
    let kept = WhereThisTurnIsKept::made(&whose, "4f1c8a").unwrap();

    // Before the turn.
    let before = kept.keeping(alo_kept_undo::BEFORE, &home);
    let at = before
        .kept_at()
        .unwrap_or_else(|| panic!("the snapshot was not taken: {:?}", before.why_not()));

    // **It is really there and really a subvolume**, asked of the program
    // rather than of the filesystem's directory listing — a plain directory
    // would satisfy `is_dir` and is not a snapshot.
    assert!(at.is_dir(), "the snapshot is not on the disk at {at:?}");
    let listed = disk.subvolumes();
    assert!(
        listed.contains("4f1c8a/before"),
        "`btrfs subvolume list` does not name the snapshot: {listed}"
    );

    // **And what was in the home at the time is in it.**
    assert_eq!(
        std::fs::read(at.join("march.pdf")).unwrap(),
        b"an invoice",
        "the snapshot does not hold what the home held"
    );

    // **Read-only**, which is what #752 rests on. Asked two ways: the property
    // the program reports, and a write that must fail.
    let said = must("btrfs", &["property", "get", "-ts", at.to_str().unwrap()]);
    assert!(
        said.contains("ro=true"),
        "the snapshot is not read-only: {said}"
    );
    let refused = std::fs::write(at.join("march.pdf"), b"something else");
    assert!(
        refused.is_err(),
        "a read-only snapshot accepted a write, so the agent could alter a bracket"
    );
}

/// **Both sides are taken, and the note reads back beside them.**
///
/// What the privileged remover walks: a directory holding `kept.json`,
/// `before/` and `after/`. If this shape is wrong, nothing is ever removed and
/// the disk fills.
#[test]
#[ignore = "needs root, a loop device and btrfs; run by name"]
fn both_sides_and_the_note_are_what_the_remover_walks() {
    let disk = AFilesystem::made();
    let (home, whose) = a_person_on(&disk.at);

    where_their_settings_are(&whose, Path::new("/home/ada/.config/alo")).unwrap();
    let kept = WhereThisTurnIsKept::made(&whose, "4f1c8a").unwrap();

    assert!(
        kept.keeping(alo_kept_undo::BEFORE, &home)
            .kept_at()
            .is_some(),
        "the before side was not taken"
    );
    // The turn changes something, as a turn does.
    std::fs::write(home.join("march.pdf"), b"an archived invoice").unwrap();
    assert!(
        kept.keeping(alo_kept_undo::AFTER, &home)
            .kept_at()
            .is_some(),
        "the after side was not taken"
    );
    kept.what_it_did(SystemTime::now(), "archive march's invoices")
        .unwrap();

    for named in [alo_kept_undo::BEFORE, alo_kept_undo::AFTER] {
        assert!(
            kept.at().join(named).is_dir(),
            "{named} is not beside the note"
        );
    }
    let note = kept.at().join(alo_kept_undo::THE_TURN);
    let text = std::fs::read_to_string(&note).unwrap();
    let read = alo_kept_undo::TheTurn::read(&text, &note).unwrap();
    assert_eq!(read.did(), "archive march's invoices");

    // **And the two sides really differ**, which is what an undo is read from:
    // a bracket whose halves were identical would put nothing back.
    assert_eq!(
        std::fs::read(kept.at().join(alo_kept_undo::BEFORE).join("march.pdf")).unwrap(),
        b"an invoice"
    );
    assert_eq!(
        std::fs::read(kept.at().join(alo_kept_undo::AFTER).join("march.pdf")).unwrap(),
        b"an archived invoice"
    );
}

/// **A home that is not a subvolume is refused, and the turn is not failed.**
///
/// The control, and the shape ADR 0045's third term is about: the taking
/// answers *why not* rather than an error, so nothing on this road can turn a
/// missing snapshot into a machine that stopped working.
#[test]
#[ignore = "needs root, a loop device and btrfs; run by name"]
fn an_ordinary_directory_is_refused_rather_than_failing_the_turn() {
    let disk = AFilesystem::made();
    let (_, whose) = a_person_on(&disk.at);

    let plain = disk.at.join("not-a-subvolume");
    std::fs::create_dir_all(&plain).unwrap();

    let kept = WhereThisTurnIsKept::made(&whose, "0").unwrap();
    let side = kept.keeping(alo_kept_undo::BEFORE, &plain);

    assert!(
        side.kept_at().is_none(),
        "an ordinary directory was snapshotted"
    );
    let why = side.why_not().unwrap();
    assert!(
        format!("{why:?}").contains("ItRefused"),
        "the program was expected to refuse and the reason is {why:?}"
    );
}
