//! What is actually in the folder, read off a disk: who has kept turns, where
//! each of them keeps their settings, and when each turn ran.
//!
//! The only module in this crate that opens anything. [`crate::the_folder`] is
//! the shapes and their refusals; this is the walk.
//!
//! # Nothing that did not read is ever removed
//!
//! A person's directory that will not read is **stepped over**, and so is one
//! kept turn whose `kept.json` will not read. Both leave the snapshots exactly
//! where they are, and both are said — to the journal, for whoever administers
//! the machine.
//!
//! That is the careful answer rather than the tidy one, and the reason is
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s second
//! accepted term. When an undo is let go, the record has to name *which turn*
//! lost it, in the words the person approved. A turn the machine cannot
//! describe cannot be named, so removing it would take a person's undo away and
//! leave nothing able to tell them it had gone. A snapshot left behind costs
//! disk; a snapshot removed silently costs the only account there is.
//!
//! **And one person's broken directory does not stop anybody else's disk being
//! tidied.** The walk goes on, which is why what it stepped over is a list
//! beside the answer rather than an error instead of it.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::the_folder::{THE_TURN, THEIRS, TheTurn, Theirs};

/// One kept turn, found on the disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The directory holding its two snapshots and what describes them.
    at: PathBuf,
    /// When it ran, and what it did.
    turn: TheTurn,
}

/// One person's kept turns, and where they keep their settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whose {
    /// Their directory under the folder.
    at: PathBuf,
    /// Their own settings folder, as their session wrote it down.
    settings: PathBuf,
    /// Their kept turns, **newest first** — the order
    /// [`alo_keeping_up::HowFarBack::how_many_it_still_reaches`] asks for.
    kept: Vec<Found>,
}

/// Everything under the folder, and what the walk would not read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Everyone {
    /// One per person whose directory read.
    whose: Vec<Whose>,
    /// What was stepped over, in English, for the journal.
    stepped_over: Vec<String>,
}

impl Found {
    /// The directory holding this kept turn.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// When it ran, and what it did.
    #[must_use]
    pub const fn turn(&self) -> &TheTurn {
        &self.turn
    }
}

impl Whose {
    /// Their directory under the folder.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// Their own settings folder.
    #[must_use]
    pub fn settings(&self) -> &Path {
        &self.settings
    }

    /// Their kept turns, newest first.
    #[must_use]
    pub fn kept(&self) -> &[Found] {
        &self.kept
    }

    /// How many days ago each of them was, newest first — exactly the list
    /// [`crate::WhatGoes::decided`] takes.
    #[must_use]
    pub fn days_ago(&self, now: SystemTime) -> Vec<u32> {
        self.kept
            .iter()
            .map(|found| found.turn.days_ago(now))
            .collect()
    }
}

impl Everyone {
    /// Everyone with kept turns under `folder`.
    ///
    /// A folder that is not there at all is nobody, which is the ordinary state
    /// of a machine on which no agent has ever changed a file — and of every
    /// machine that cannot keep anything.
    #[must_use]
    pub fn under(folder: &Path) -> Self {
        let mut everyone = Self::default();
        let directories = match directories_in(folder) {
            Ok(directories) => directories,
            Err(why) => {
                if !folder.exists() {
                    return everyone;
                }
                everyone
                    .stepped_over
                    .push(format!("{} was not read: {why}", folder.display()));
                return everyone;
            }
        };
        for at in directories {
            match Self::one_person(&at, &mut everyone.stepped_over) {
                Some(whose) => everyone.whose.push(whose),
                None => continue,
            }
        }
        everyone
    }

    /// Everyone found, one per person.
    #[must_use]
    pub fn whose(&self) -> &[Whose] {
        &self.whose
    }

    /// What the walk would not read and left exactly where it was.
    #[must_use]
    pub fn stepped_over(&self) -> &[String] {
        &self.stepped_over
    }

    /// One person's directory, or nothing and a sentence saying why.
    fn one_person(at: &Path, stepped_over: &mut Vec<String>) -> Option<Whose> {
        let says = at.join(THEIRS);
        let text = match std::fs::read_to_string(&says) {
            Ok(text) => text,
            Err(why) => {
                stepped_over.push(format!("{} was not read: {why}", says.display()));
                return None;
            }
        };
        let theirs = match Theirs::read(&text, &says) {
            Ok(theirs) => theirs,
            Err(why) => {
                stepped_over.push(why.to_string());
                return None;
            }
        };

        let directories = match directories_in(at) {
            Ok(directories) => directories,
            Err(why) => {
                stepped_over.push(format!("{} was not read: {why}", at.display()));
                return None;
            }
        };
        let mut kept = Vec::new();
        for turn_at in directories {
            if let Some(found) = one_turn(&turn_at, stepped_over) {
                kept.push(found);
            }
        }
        // Newest first, and a moment is the only order there is: the directory
        // names are opaque on purpose (`crate::the_folder`).
        kept.sort_by_key(|found| std::cmp::Reverse(found.turn.taken()));

        Some(Whose {
            at: at.to_owned(),
            settings: theirs.settings().to_owned(),
            kept,
        })
    }
}

/// One kept turn's directory, or nothing and a sentence saying why.
fn one_turn(at: &Path, stepped_over: &mut Vec<String>) -> Option<Found> {
    let says = at.join(THE_TURN);
    let text = match std::fs::read_to_string(&says) {
        Ok(text) => text,
        Err(why) => {
            stepped_over.push(format!("{} was not read: {why}", says.display()));
            return None;
        }
    };
    match TheTurn::read(&text, &says) {
        Ok(turn) => Some(Found {
            at: at.to_owned(),
            turn,
        }),
        Err(why) => {
            stepped_over.push(why.to_string());
            None
        }
    }
}

/// Every directory directly inside `folder`, by name, in a settled order.
///
/// **A link is never followed**, which is `alo-files`' rule and is no different
/// here: a link under this folder would be a way to point the one privileged
/// remover on the machine at somebody else's subvolume.
fn directories_in(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            found.push(entry.path());
        }
    }
    found.sort();
    Ok(found)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{A_DAY, a_folder, noon, one_kept_turn, one_person};

    /// **A folder with nobody in it is nobody, not a refusal** — the ordinary
    /// state of a machine no agent has ever changed a file on.
    #[test]
    fn a_folder_that_is_not_there_is_nobody() {
        let under = a_folder("nobody").join("undo");
        let everyone = Everyone::under(&under);
        assert!(everyone.whose().is_empty());
        assert!(everyone.stepped_over().is_empty());
    }

    /// **A person's kept turns come back newest first**, which is the order the
    /// window is asked in, and the moment comes from the file rather than from
    /// the directory's name.
    #[test]
    fn a_persons_kept_turns_come_back_newest_first() {
        let under = a_folder("newest-first").join("undo");
        let ada = one_person(&under, "ada", "/var/home/ada/.config/alo");
        one_kept_turn(&ada, "zzz-oldest", noon() - A_DAY * 9, "archive Old");
        one_kept_turn(&ada, "aaa-newest", noon(), "move March.pdf");

        let everyone = Everyone::under(&under);
        assert_eq!(everyone.whose().len(), 1);
        let whose = &everyone.whose()[0];
        assert_eq!(whose.settings(), Path::new("/var/home/ada/.config/alo"));
        assert_eq!(
            whose
                .kept()
                .iter()
                .map(|found| found.turn().did())
                .collect::<Vec<&str>>(),
            ["move March.pdf", "archive Old"]
        );
        assert_eq!(whose.days_ago(noon()), [0, 9]);
    }

    /// **A kept turn whose file will not read is stepped over and left exactly
    /// where it is**, because an undo removed without anything able to name it
    /// is a person who can never be told what they lost.
    #[test]
    fn a_turn_that_will_not_read_is_stepped_over_and_left_alone() {
        let under = a_folder("unreadable-turn").join("undo");
        let ada = one_person(&under, "ada", "/var/home/ada/.config/alo");
        one_kept_turn(&ada, "good", noon(), "move March.pdf");
        let bad = ada.join("bad");
        std::fs::create_dir_all(bad.join("before")).unwrap();
        std::fs::write(bad.join(THE_TURN), "{\"format\":2}").unwrap();

        let everyone = Everyone::under(&under);
        let whose = &everyone.whose()[0];
        assert_eq!(whose.kept().len(), 1);
        assert_eq!(whose.kept()[0].turn().did(), "move March.pdf");
        assert_eq!(everyone.stepped_over().len(), 1);
        assert!(everyone.stepped_over()[0].contains("format 2"));
        assert!(bad.join("before").exists());
    }

    /// **One person's broken directory does not stop anybody else's disk being
    /// tidied**, which is why what was stepped over is a list beside the answer
    /// rather than an error instead of it.
    #[test]
    fn one_broken_directory_does_not_stop_the_others() {
        let under = a_folder("one-broken").join("undo");
        let ada = one_person(&under, "ada", "/var/home/ada/.config/alo");
        one_kept_turn(&ada, "one", noon(), "move March.pdf");
        let bo = under.join("bo");
        std::fs::create_dir_all(&bo).unwrap();
        std::fs::write(bo.join(THEIRS), "{\"format\":1,\"settings\":\"relative\"}").unwrap();

        let everyone = Everyone::under(&under);
        assert_eq!(everyone.whose().len(), 1);
        assert_eq!(everyone.whose()[0].at(), ada);
        assert_eq!(everyone.stepped_over().len(), 1);
        assert!(everyone.stepped_over()[0].contains("absolute"));
    }

    /// **A person's directory with no `theirs.json` is stepped over**: without
    /// it there is no road to their window, and using the shipped one instead
    /// would be the machine deciding for somebody who had already decided.
    #[test]
    fn a_person_who_never_said_where_their_settings_are_is_stepped_over() {
        let under = a_folder("no-theirs").join("undo");
        let ada = under.join("ada");
        std::fs::create_dir_all(ada.join("one")).unwrap();

        let everyone = Everyone::under(&under);
        assert!(everyone.whose().is_empty());
        assert_eq!(everyone.stepped_over().len(), 1);
        assert!(everyone.stepped_over()[0].contains(THEIRS));
    }
}
