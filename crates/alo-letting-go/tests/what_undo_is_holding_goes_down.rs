//! **What is filling the disk counts what undo is holding, by name — and the
//! number goes down after the unit has run.**
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s fourth
//! accepted term: *`alo-measuring`'s answer includes what undo is holding, as
//! its own line, because an answer that hid it would send a person hunting for
//! space the machine itself was keeping.* The term was accepted on 2026-09-16
//! and until now it was a promise about a number nothing ever reduced: the
//! window decided that a snapshot had expired and nothing removed it, so *what
//! undo is holding* could only ever go up.
//!
//! This is the other end of that sentence, measured rather than argued. It asks
//! `alo-measuring` — the crate whose answer the person's window draws, asked
//! rather than imitated, and which this lane does not own and does not edit
//! (ADR 0028) — what the undo folder is holding; runs one firing of the timer
//! over it; and asks again.
//!
//! **By name**, in both directions: the folder is named
//! [`alo_letting_go::THE_FOLDER`] and each person's directory under it is a
//! node of its own in the tree `alo-measuring` answers with, so a person
//! clicking through *what is filling the disk* meets it where they are looking
//! rather than having to know it exists.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_letting_go::the_folder::{AFTER, BEFORE, THE_TURN, THEIRS, TheTurn, Theirs};
use alo_letting_go::{
    AskingTheDisk, NotRemoved, Removing, Swept, THE_FOLDER, WhatItIs, WritingItDown, sweep,
};
use alo_measuring::Holding;
use alo_record::Entry;

/// How long a day is where this test moves a clock.
const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// A moment far enough from the epoch to read as a real one.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A directory of this test's own, on a real disk.
fn a_folder(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-letting-go-holding-{what}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).expect("a directory of this test's own");
    folder
}

/// One kept turn holding `bytes` in each of its two snapshots.
fn one_kept_turn(whose: &Path, named: &str, taken: SystemTime, did: &str, bytes: usize) {
    let at = whose.join(named);
    for side in [BEFORE, AFTER] {
        let side = at.join(side);
        std::fs::create_dir_all(&side).expect("a snapshot");
        std::fs::write(side.join("March.pdf"), vec![b'a'; bytes]).expect("what it holds");
    }
    let says = at.join(THE_TURN);
    let turn = TheTurn::done(taken, did, &says).expect("a turn");
    std::fs::write(&says, serde_json::to_string(&turn).expect("json")).expect("what describes it");
}

/// A disk that keeps and has room to spare, so nothing goes for room and the
/// window is the only thing deciding.
struct WithRoom;

impl AskingTheDisk for WithRoom {
    fn what_it_is(&self, _at: &Path) -> std::io::Result<WhatItIs> {
        Ok(WhatItIs::OneThatKeeps)
    }

    fn free(&self, _at: &Path) -> std::io::Result<u64> {
        Ok(u64::MAX)
    }
}

/// A remover that really removes the ordinary directories this test made,
/// standing in for the one thing a developer's machine cannot do.
struct Really;

impl Removing for Really {
    fn remove(&self, at: &Path) -> Result<(), NotRemoved> {
        std::fs::remove_dir_all(at).map_err(|why| NotRemoved::NotCleared {
            at: at.to_owned(),
            why,
        })
    }
}

/// A record that keeps entries in memory.
#[derive(Default)]
struct InHand(Vec<Entry>);

impl WritingItDown for InHand {
    fn keep(&mut self, entry: Entry) -> Result<(), String> {
        self.0.push(entry);
        Ok(())
    }
}

/// **What undo is holding is its own line, by name, and it goes down after the
/// unit runs.**
#[test]
fn what_undo_is_holding_is_a_line_of_its_own_and_goes_down_after_the_unit_runs() {
    let under = a_folder("goes-down").join("undo");
    let settings = a_folder("goes-down-settings");
    let ada = under.join("ada");
    std::fs::create_dir_all(&ada).expect("a person's directory");
    let says = ada.join(THEIRS);
    let theirs = Theirs::at(&settings, &says).expect("where their settings are");
    std::fs::write(&says, serde_json::to_string(&theirs).expect("json")).expect("theirs.json");

    // Two kept turns of a hundred kibibytes each side: one the window still
    // reaches, one it does not.
    const EACH_SIDE: usize = 100 * 1024;
    one_kept_turn(&ada, "recent", noon() - A_DAY, "move March.pdf", EACH_SIDE);
    one_kept_turn(
        &ada,
        "old",
        noon() - A_DAY * 30,
        "archive Old letters",
        EACH_SIDE,
    );

    // `alo-measuring`'s own answer, asked the way a person's window asks it.
    let before = Holding::of(&under).expect("what the undo folder is holding");
    assert!(before.finished);
    assert!(
        before.tree.size >= (4 * EACH_SIDE) as u64,
        "what undo is holding was not counted: {}",
        before.tree.size
    );

    // **By name**: the person's directory is a line of its own under it, so a
    // person clicking through meets it rather than having to know it is there.
    let named: Vec<&str> = before
        .tree
        .children
        .iter()
        .map(|child| child.name.as_str())
        .collect();
    assert!(named.contains(&"ada"), "{named:?}");

    let swept = sweep(&under, &WithRoom, &Really, &mut InHand::default(), noon());
    assert!(
        matches!(swept, Swept::Done(ref did) if did.outside_the_window() == 1),
        "{swept:?}"
    );

    let after = Holding::of(&under).expect("what the undo folder is holding now");
    assert!(
        after.tree.size < before.tree.size,
        "what undo is holding did not go down: {} then {}",
        before.tree.size,
        after.tree.size
    );
    assert!(
        after.tree.size >= (2 * EACH_SIDE) as u64,
        "the turn still inside the window was removed too: {}",
        after.tree.size
    );
}

/// **The folder it counts is the folder the unit sweeps**, named once in this
/// repository — so *what is filling the disk* and *what the machine lets go of*
/// can never come to mean two different places.
#[test]
fn the_folder_counted_is_the_folder_swept() {
    assert_eq!(THE_FOLDER, "/var/lib/alo/undo");
    assert!(
        Path::new(THE_FOLDER).is_absolute(),
        "a person's window is pointed at an absolute folder"
    );
}
