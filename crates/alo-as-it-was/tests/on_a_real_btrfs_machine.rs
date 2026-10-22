//! A folder put back on a real `btrfs` filesystem, from a place a person's own
//! command kept — the whole road, with nothing standing in for anything.
//!
//! Everything in this crate's own suite runs against ordinary directories,
//! which is enough for the copy, the swap and every refusal. **Three things only
//! a real filesystem settles**, and all three are what the feature rests on:
//!
//! - `alo-agentd`'s own handler, called as a person's shell reaches it, really
//!   leaves a place behind — a read-only snapshot **and** the note beside it;
//! - `alo_letting_go::found`, which is the privileged remover's eyes, **finds**
//!   that place rather than stepping over it. Until 2026-10-22 it stepped over
//!   every one, because no note was written, so nothing could ever be removed
//!   and a person who turned this on filled their disk;
//! - a folder goes back **out of a read-only snapshot**, which is the one source
//!   this act is allowed to trust and the one an ordinary directory cannot be.
//!
//! # What it needs, and why it is not in the suite
//!
//! Root, `mkfs.btrfs`, `btrfs`, `mount`, `truncate`, `cp`, a loop device and a
//! gibibyte of room. It is `#[ignore]`d and run by name, and it **fails** rather
//! than skips when something it needs is missing, because a test that passed on
//! a machine that could not run it would be evidence of nothing.
//!
//! ```text
//! cargo test -p alo-as-it-was --test on_a_real_btrfs_machine -- --ignored --nocapture
//! ```
//!
//! # What it is not
//!
//! Not a certified machine and not a booted alo OS. It is this crate's code,
//! and `alo-agentd`'s handler, against a real filesystem on a loop device.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_as_it_was::{Places, go_back, what_would_go_back};

/// Big enough for a filesystem, a snapshot of a small home and a copy back.
const ROOM_TO_SPARE: &str = "1G";

/// The moment the place is kept at, so the note is checkable by eye.
const KEPT_AT: u64 = 1_700_000_000;

/// Run a program and say whether it worked and what it said.
fn run(program: &str, args: &[&str]) -> (bool, String) {
    match Command::new(program).args(args).output() {
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
            "alo-as-it-was-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
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
}

impl Drop for AFilesystem {
    fn drop(&mut self) {
        let _ = run("umount", &[self.at.to_str().unwrap()]);
        let _ = std::fs::remove_dir_all(&self.under);
    }
}

/// The person this test is about, by the identifier their directory is named
/// after — which is what `docs/contracts/kept-undo-folder.md` says and what
/// `WhereBracketsGo` joins on, so this test never builds that path itself.
const THE_PERSON: u32 = 1000;

/// A home subvolume with a folder in it, and the undo folder it keeps under.
///
/// Answers the home and the **undo folder**, not the person's directory inside
/// it: making that directory is the handler's own job through
/// `where_their_settings_are`, and a test that made it would be testing a path
/// it had decided on.
fn a_person_on(mounted: &Path) -> (PathBuf, PathBuf) {
    let home = mounted.join("home").join("ada");
    std::fs::create_dir_all(mounted.join("home")).unwrap();
    must("btrfs", &["subvolume", "create", home.to_str().unwrap()]);
    std::fs::create_dir_all(home.join("src")).unwrap();
    std::fs::create_dir_all(home.join(".config").join("alo")).unwrap();
    std::fs::write(home.join("src").join("main.rs"), b"fn main() {}").unwrap();
    std::fs::write(home.join("src").join("notes"), b"as it was").unwrap();

    let folder = mounted.join("alo-undo");
    std::fs::create_dir_all(&folder).unwrap();
    (home, folder)
}

/// Keep a place the way a person's own shell does, through `alo-agentd`'s own
/// handler, and answer whether it said one was kept and where it put it.
fn a_place_kept_the_way_a_shell_asks(folder: &Path, home: &Path, named: &str) -> (bool, PathBuf) {
    let goes = alo_agentd::the_bracket_this_turn_takes::WhereBracketsGo::of_a_session(
        folder,
        THE_PERSON,
        Some(home.as_os_str()),
    )
    .expect("an absolute home is a session");
    let whose = goes.whose().to_owned();
    // **The person's own directory inside the undo folder is the machine's to
    // make, not the handler's.** `docs/contracts/kept-undo-folder.md` and
    // ADR 0045 term 8: the folder is root's and `0711`, and *a session cannot
    // make a directory in a root-owned folder* — so it is made for them, and
    // `where_their_settings_are` writes `theirs.json` into one that is already
    // there. A test that did not make it would be measuring a machine nobody
    // had ever signed in to.
    std::fs::create_dir_all(&whose).unwrap();
    let strings = alo_strings::Strings::of(every_word_the_daemon_says());
    let answered = alo_agentd::a_place_to_return_to::kept(
        Some(&goes),
        named,
        UNIX_EPOCH + Duration::from_secs(KEPT_AT),
        &strings,
    );
    // Printed, because the handler answers *no place was kept* for every reason
    // there is and a test that only saw the boolean would have nothing to go on.
    println!("the handler answered: {answered:?}");
    let kept = matches!(
        answered,
        alo_protocol::ToAPerson::APlaceToReturnTo { kept: true, .. }
    );
    (kept, whose)
}

/// The daemon's own words, which is what its sentence is rendered from.
fn every_word_the_daemon_says() -> alo_strings::Vocabulary {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_agentd::words::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// **A place is kept, the remover can see it, and a folder goes back out of
/// it.**
///
/// One walk, in the order a person meets it: they turn the row on and run a
/// command, so a place appears; they change the folder; they ask to go back; the
/// folder is as it was and what they had is beside it. With the privileged
/// remover's own reader asked in the middle, because a place it cannot see is a
/// place that never goes away.
#[test]
#[ignore = "needs root, a loop device and btrfs; run by name"]
fn a_place_is_kept_the_remover_sees_it_and_a_folder_goes_back() {
    let disk = AFilesystem::made();
    let (home, folder) = a_person_on(&disk.at);

    let (kept, whose) = a_place_kept_the_way_a_shell_asks(&folder, &home, "1700000000-7-their-own");
    assert!(kept, "the handler did not keep a place");
    let at = whose.join("1700000000-7-their-own");

    // **The snapshot is a real subvolume and it is read-only.**
    let said = must(
        "btrfs",
        &[
            "property",
            "get",
            "-ts",
            at.join("before").to_str().unwrap(),
        ],
    );
    assert!(
        said.contains("ro=true"),
        "the snapshot is not read-only: {said}"
    );

    // **The note is beside it**, which is what the privileged remover reads and
    // what carries the moment a person chooses by.
    assert!(
        at.join("kept.json").is_file(),
        "no note was written beside the snapshot, so nothing can ever remove it \
         and nothing can say when it was taken"
    );

    // **The remover's own reader finds it.** This is the assertion that would
    // have failed before 2026-10-22: it stepped every such place over, because
    // there was no note to read. The handler wrote `theirs.json` itself, which
    // is the other file that reader needs, so nothing here writes one.
    let everyone = alo_letting_go::found::Everyone::under(&folder);
    let found: usize = everyone
        .whose()
        .iter()
        .map(|whose| whose.kept().len())
        .sum();
    assert_eq!(
        found,
        1,
        "the privileged remover's reader did not find the place it has to be able to \
         remove. it stepped over: {:?}",
        everyone.stepped_over()
    );

    // **The person changes the folder, then asks to go back.**
    std::fs::write(home.join("src").join("notes"), b"changed since").unwrap();
    std::fs::write(home.join("src").join("made-since"), b"new").unwrap();

    let places = Places::under(&whose);
    let place = places
        .named("1700000000-7-their-own")
        .unwrap_or_else(|| panic!("the place was not read back: {:?}", places.stepped_over()));
    assert_eq!(
        place.taken(),
        UNIX_EPOCH + Duration::from_secs(KEPT_AT),
        "the moment in the note is not the moment the place was kept"
    );

    let goes = what_would_go_back(
        &place.before(),
        &home.join("src"),
        &home,
        place.taken(),
        SystemTime::now(),
    )
    .unwrap();
    go_back(&goes).unwrap();

    // **The folder is as it was**, out of a read-only snapshot.
    assert_eq!(
        std::fs::read(home.join("src").join("notes")).unwrap(),
        b"as it was",
        "the folder did not go back"
    );
    assert!(
        !home.join("src").join("made-since").exists(),
        "a file made after the place was kept is still in the folder"
    );

    // **And what was there is beside it, not gone.**
    assert_eq!(
        std::fs::read(goes.what_was_here().join("notes")).unwrap(),
        b"changed since",
        "what was in the folder was not set aside"
    );
    assert!(
        goes.what_was_here().join("made-since").exists(),
        "and neither was the file nothing else holds a copy of"
    );

    // **The folder a person works in is writable**, which the snapshot it came
    // out of is not. A go-back that handed back a read-only folder would be a
    // person unable to use their own files.
    std::fs::write(home.join("src").join("notes"), b"working again").unwrap();
}
