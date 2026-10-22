//! A person's undo folder and a home, made in a temporary directory.
//!
//! No `btrfs` in here and none needed for most of what this crate does: a place
//! kept is a directory with a note and a copy in it, and *putting back* is a
//! copy and one rename. What needs a real filesystem is the snapshot being
//! read-only and shared block for block, and that is held in
//! `tests/on_a_real_btrfs_machine.rs` rather than guessed at here.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_kept_undo::{BEFORE, THE_TURN, TheTurn};

/// A temporary directory of its own, removed when it goes out of scope.
#[derive(Debug)]
pub struct AMachine {
    /// Everything this fixture made.
    at: PathBuf,
}

impl AMachine {
    /// A directory nothing else is using, named after the caller.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot make its own directory has nothing to measure"
    )]
    #[must_use]
    pub fn called(named: &str) -> Self {
        let at = std::env::temp_dir().join(format!(
            "alo-as-it-was-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("home")).expect("a fixture makes its own directory");
        std::fs::create_dir_all(at.join("undo")).expect("a fixture makes its own directory");
        Self { at }
    }

    /// The home a person's files are in.
    #[must_use]
    pub fn home(&self) -> PathBuf {
        self.at.join("home")
    }

    /// This person's own directory in the undo folder.
    #[must_use]
    pub fn whose(&self) -> PathBuf {
        self.at.join("undo")
    }

    /// A place kept at this moment, holding a copy of the home as it is now.
    ///
    /// The copy is an ordinary recursive copy rather than a snapshot, which is
    /// what a filesystem that cannot snapshot would leave — and is enough for
    /// everything about *putting back* except the sharing of blocks.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot write its own files has nothing to measure"
    )]
    pub fn a_place_kept(&self, named: &str, taken: SystemTime) -> PathBuf {
        let at = self.whose().join(named);
        let before = at.join(BEFORE);
        std::fs::create_dir_all(&before).expect("a fixture makes its own directory");
        copied(&self.home(), &before);
        let says = at.join(THE_TURN);
        let note = TheTurn::done(taken, "before a command you typed in the terminal", &says)
            .expect("a fixture writes a well-formed note");
        std::fs::write(&says, note.as_text(&says).expect("and renders it"))
            .expect("a fixture writes its own files");
        at
    }

    /// A place with a note and no copy inside it — what an interrupted keeping
    /// leaves.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot write its own files has nothing to measure"
    )]
    pub fn a_place_with_no_copy(&self, named: &str, taken: SystemTime) -> PathBuf {
        let at = self.whose().join(named);
        std::fs::create_dir_all(&at).expect("a fixture makes its own directory");
        let says = at.join(THE_TURN);
        let note = TheTurn::done(taken, "before a command you typed in the terminal", &says)
            .expect("a fixture writes a well-formed note");
        std::fs::write(&says, note.as_text(&says).expect("and renders it"))
            .expect("a fixture writes its own files");
        at
    }

    /// A place with a copy and **no note** — which is what `alo-agentd` left
    /// behind until 2026-10-22, and is why nothing could sweep or find one.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot make its own directory has nothing to measure"
    )]
    pub fn a_place_with_no_note(&self, named: &str) -> PathBuf {
        let at = self.whose().join(named);
        std::fs::create_dir_all(at.join(BEFORE)).expect("a fixture makes its own directory");
        at
    }

    /// A folder in the home, holding one file with this text in it.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot write its own files has nothing to measure"
    )]
    pub fn a_folder(&self, named: &str, holding: &str) -> PathBuf {
        let at = self.home().join(named);
        std::fs::create_dir_all(&at).expect("a fixture makes its own directory");
        std::fs::write(at.join("a-file"), holding).expect("a fixture writes its own files");
        at
    }

    /// What a file in the home says now.
    #[expect(
        clippy::expect_used,
        reason = "a fixture that cannot read back what it wrote has nothing to measure"
    )]
    #[must_use]
    pub fn what_it_says(&self, at: &Path) -> String {
        std::fs::read_to_string(at).expect("a file this fixture is asking about")
    }
}

impl Drop for AMachine {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

/// A moment, so that nothing in these tests depends on a clock.
#[must_use]
pub fn at(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

/// Copy a directory's contents into another that already exists.
#[expect(
    clippy::expect_used,
    reason = "a fixture that cannot copy its own files has nothing to measure"
)]
fn copied(from: &Path, into: &Path) {
    for entry in std::fs::read_dir(from).expect("a fixture reads its own directory") {
        let entry = entry.expect("a fixture reads its own directory");
        let to = into.join(entry.file_name());
        if entry.path().is_dir() {
            std::fs::create_dir_all(&to).expect("a fixture makes its own directory");
            copied(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).expect("a fixture copies its own files");
        }
    }
}
