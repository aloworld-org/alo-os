//! The places this person can return to, read off the disk, newest first.
//!
//! One directory under `/var/lib/alo-undo/<uid>` per place, each holding a
//! read-only snapshot called `before` and a note called `kept.json` saying when
//! it was taken and what it was taken for. `docs/contracts/kept-undo-folder.md`
//! gives a person's own directory to that person, so this reads it as an
//! ordinary call with no privilege at all — which is the whole reason
//! [ADR 0127](../../../docs/decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
//! could refuse a privileged helper.
//!
//! # A directory's name is never read
//!
//! The contract calls bracket names **opaque**. Everything a person is shown
//! about a place comes from the note beside the snapshot, and the name is only
//! ever passed back through as the thing they chose. A reader that learned the
//! moment by parsing a name would be a second answer about one place, and the
//! first thing to go stale when the naming changes.
//!
//! # A place with no note is not a place
//!
//! It is also not news. The privileged remover's own reader —
//! `crates/alo-letting-go/src/found.rs`, named by file rather than by its Rust
//! path, because that crate's own guard forbids anything a turn runs inside from
//! spelling it and this crate is in a turn's graph — steps such a directory over
//! rather than sweep something it does not understand, so a half-written place
//! stays on the disk; it is left out here for the same reason, and counted in
//! [`Places::stepped_over`] so that nothing disappears silently.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_kept_undo::{BEFORE, THE_FOLDER, THE_TURN, TheTurn};

/// Where this person's own places are kept.
///
/// Their own directory under the machine's undo folder, named by their user
/// identifier — which is what `docs/contracts/kept-undo-folder.md` says and
/// what `alo-agentd` writes into.
#[must_use]
pub fn where_this_person_keeps(person: u32) -> PathBuf {
    Path::new(THE_FOLDER).join(person.to_string())
}

/// One place a person can return to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APlaceKept {
    /// The directory, which is what a person names to choose it.
    at: PathBuf,
    /// When the snapshot inside it was taken.
    taken: SystemTime,
    /// What it was taken for, in the words written down at the time.
    did: String,
}

impl APlaceKept {
    /// The directory this place is, which is what a person names to choose it.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// The snapshot inside it — the files as they were.
    #[must_use]
    pub fn before(&self) -> PathBuf {
        self.at.join(BEFORE)
    }

    /// When it was taken.
    #[must_use]
    pub const fn taken(&self) -> SystemTime {
        self.taken
    }

    /// What it was taken for, in the words written down at the time.
    ///
    /// Never reworded here. A place kept in March says what it said in March,
    /// in the language it was written in — the same rule `alo-recounting`
    /// follows, and for the same reason: two wordings of one moment are two
    /// accounts of it.
    #[must_use]
    pub fn did(&self) -> &str {
        &self.did
    }
}

/// Every place one person can return to, and everything that was not read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Places {
    /// Newest first.
    kept: Vec<APlaceKept>,
    /// What was there and was left exactly where it was, with why.
    stepped_over: Vec<String>,
}

impl Places {
    /// Read one person's own directory.
    ///
    /// A directory that is not there at all is **no places and nothing wrong**:
    /// it is the state of every machine on which nobody has turned this on, and
    /// of every machine whose filesystem cannot keep one.
    #[must_use]
    pub fn under(whose: &Path) -> Self {
        let mut places = Self::default();
        let read = match std::fs::read_dir(whose) {
            Ok(read) => read,
            Err(why) => {
                if whose.exists() {
                    places
                        .stepped_over
                        .push(format!("{} was not read: {why}", whose.display()));
                }
                return places;
            }
        };
        let mut found = Vec::new();
        for entry in read {
            match entry {
                Ok(entry) => found.push(entry.path()),
                Err(why) => places
                    .stepped_over
                    .push(format!("{} was not read: {why}", whose.display())),
            }
        }
        // **Sorted before it is read.** `read_dir` has no order and the order
        // differs by filesystem, so a walk that read entries as they came would
        // be a test that passes on one disk and not another.
        found.sort();
        for at in found {
            match one_place(&at, &mut places.stepped_over) {
                Some(place) => places.kept.push(place),
                None => continue,
            }
        }
        places
            .kept
            .sort_by_key(|place| std::cmp::Reverse(place.taken));
        places
    }

    /// Every place, newest first.
    #[must_use]
    pub fn kept(&self) -> &[APlaceKept] {
        &self.kept
    }

    /// The one a person chose, by the directory's name.
    ///
    /// By name rather than by position, because a position means something
    /// different the moment another command runs — and a command runs between
    /// a person reading the list and typing their answer.
    #[must_use]
    pub fn named(&self, named: &str) -> Option<&APlaceKept> {
        self.kept
            .iter()
            .find(|place| place.at.file_name().is_some_and(|had| had == named))
    }

    /// What was in the folder and was not read, with why — never thrown away
    /// silently.
    #[must_use]
    pub fn stepped_over(&self) -> &[String] {
        &self.stepped_over
    }
}

/// One directory, if it is a place a person can return to.
fn one_place(at: &Path, stepped_over: &mut Vec<String>) -> Option<APlaceKept> {
    if !at.is_dir() {
        return None;
    }
    let says = at.join(THE_TURN);
    let text = match std::fs::read_to_string(&says) {
        Ok(text) => text,
        Err(why) => {
            stepped_over.push(format!("{} was not read: {why}", says.display()));
            return None;
        }
    };
    let turn = match TheTurn::read(&text, &says) {
        Ok(turn) => turn,
        Err(why) => {
            stepped_over.push(why.to_string());
            return None;
        }
    };
    // **The snapshot has to actually be there.** A note without one is a place
    // that was being made when something stopped: going back to it would put
    // back nothing and report success, which is the worst of the three outcomes.
    let before = at.join(BEFORE);
    if !before.is_dir() {
        stepped_over.push(format!("{} has nothing in it to return to", at.display()));
        return None;
    }
    Some(APlaceKept {
        at: at.to_owned(),
        taken: turn.taken(),
        did: turn.did().to_owned(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "places_tests.rs"]
mod tests;
