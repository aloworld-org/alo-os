//! Removing one kept turn: two read-only snapshots and the small directory
//! that described them.
//!
//! # The base's own program, with fixed arguments and no shell
//!
//! [ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md):
//! the filesystem is rented and never patched, and it is spoken to through its
//! own tools. `btrfs subvolume delete` is that tool, run with exactly the
//! arguments written below — a fixed program, a fixed switch and one path — and
//! never through a shell. This is the same shape `alo-updating` gives `bootc`,
//! and law 2 is why: nothing here assembles a command line out of anything a
//! request carried, because nothing here can be asked for at all.
//!
//! # Why a read-only snapshot needs this at all
//!
//! Measured on the disk the pinned release installs and written down in
//! `docs/quirks.md`: **taking a read-only snapshot needs no capability and
//! removing one needs `CAP_SYS_ADMIN`**, and a read-only snapshot cannot be
//! cleared with `rm -rf` either — that answers `Read-only file system`, which
//! reads like a mount fault and is not one. That asymmetry is the whole reason
//! this crate has a privileged unit rather than a line inside `alo-turn`.
//!
//! # `--commit-after`, and why it is not a nicety
//!
//! `btrfs subvolume delete` returns as soon as the subvolume is unlinked; the
//! space comes back later, when the cleaner thread has walked it. Under
//! [`crate::THE_FLOOR`] this crate removes **one at a time and asks the
//! filesystem again**, so a deletion that had not yet freed anything would read
//! as *that did not help* and take the next one, and the next — a machine
//! removing a person's whole history because it asked too early. `-C` waits for
//! the commit, which makes the answer afterwards mean what the loop reads it
//! to mean.
//!
//! # The order: the snapshots, then what described them
//!
//! `after` and `before` go first and `kept.json` last, so a run interrupted
//! anywhere leaves a directory that still says which turn it was. The opposite
//! order would leave snapshots on the disk that nothing could name, which is
//! the one state from which a person can never be told what they lost.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::the_folder::{BOTH_SNAPSHOTS, THE_TURN};

/// The base's own program for a subvolume.
pub const THE_PROGRAM: &str = "btrfs";

/// The arguments it is given, before the one path.
///
/// `subvolume delete --commit-after`, and there is no other list. A test reads
/// this file for any argument that is not one of these.
pub const THE_ARGUMENTS: [&str; 3] = ["subvolume", "delete", "--commit-after"];

/// Why a kept turn was not removed.
///
/// The machine is as it was — or, where a snapshot went and the one beside it
/// did not, with that one gone and the directory still describing the turn, so
/// the next run finishes it.
#[derive(Debug, thiserror::Error)]
pub enum NotRemoved {
    /// The program could not be run at all.
    #[error("{THE_PROGRAM} could not be run for {at}: {why}")]
    NotRun {
        /// What was to be removed.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
    /// It ran and refused.
    #[error("{THE_PROGRAM} would not remove {at}: {said}")]
    Refused {
        /// What was to be removed.
        at: PathBuf,
        /// What it printed, which on a machine with no capability is
        /// `Operation not permitted`.
        said: String,
    },
    /// The directory that described the turn would not go.
    #[error("{at} was not removed: {why}")]
    NotCleared {
        /// The directory.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
}

/// Something that removes one kept turn — this machine, and a test's stand-in
/// that writes down what it was asked and removes nothing.
pub trait Removing {
    /// Remove the kept turn whose directory is `at`: both snapshots, then what
    /// described them.
    ///
    /// # Errors
    /// [`NotRemoved`]. Nothing is recorded about a turn this refused, because a
    /// record saying an undo is gone while the snapshot is still on the disk is
    /// the one line a person cannot check.
    fn remove(&self, at: &Path) -> Result<(), NotRemoved>;
}

/// This machine, with the base's own program.
#[derive(Debug, Clone, Copy, Default)]
pub struct WithTheBase;

impl Removing for WithTheBase {
    fn remove(&self, at: &Path) -> Result<(), NotRemoved> {
        for snapshot in BOTH_SNAPSHOTS {
            let snapshot = at.join(snapshot);
            if !snapshot.exists() {
                continue;
            }
            let said = Command::new(THE_PROGRAM)
                .args(THE_ARGUMENTS)
                .arg(&snapshot)
                .output()
                .map_err(|why| NotRemoved::NotRun {
                    at: snapshot.clone(),
                    why,
                })?;
            if !said.status.success() {
                return Err(NotRemoved::Refused {
                    at: snapshot,
                    said: String::from_utf8_lossy(&said.stderr).trim().to_owned(),
                });
            }
        }
        let described = at.join(THE_TURN);
        if described.exists() {
            std::fs::remove_file(&described)
                .map_err(|why| NotRemoved::NotCleared { at: described, why })?;
        }
        std::fs::remove_dir(at).map_err(|why| NotRemoved::NotCleared {
            at: at.to_owned(),
            why,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What this file ships, without the tests at the bottom of it — so that a
    /// test naming a thing it forbids does not read as the file doing it.
    fn what_this_file_ships() -> &'static str {
        let whole = include_str!("removing.rs");
        match whole.find("#[cfg(test)]") {
            Some(tests) => whole.split_at(tests).0,
            None => whole,
        }
    }

    /// **The program is given three arguments and one path, and there is no
    /// other list.** Read off this file, so an argument added later is a
    /// failing test rather than a command line nobody reviewed.
    #[test]
    fn the_only_arguments_are_the_three_written_down() {
        assert_eq!(THE_ARGUMENTS, ["subvolume", "delete", "--commit-after"]);
        let source = what_this_file_ships();
        for line in source.lines() {
            let line = line.trim();
            assert!(
                !line.starts_with(".arg(\"") && !line.starts_with(".args([\""),
                "an argument is spelt in place rather than taken from THE_ARGUMENTS: {line}"
            );
        }
    }

    /// **Nothing here reaches a shell, and one program is started.** The
    /// program is named by [`THE_PROGRAM`] and run directly; a shell would be
    /// law 2 broken at the one place in alo OS that removes a person's own
    /// history.
    #[test]
    fn one_program_is_started_and_it_is_named_by_its_constant() {
        let source = what_this_file_ships();
        let started = source.matches("Command::new(").count();
        assert_eq!(started, 1, "more than one program is started here");
        assert!(
            source.contains("Command::new(THE_PROGRAM)"),
            "the program started is not the one this file names"
        );
        assert_eq!(THE_PROGRAM, "btrfs");
    }

    /// **The snapshots go before what described them**, so a run interrupted
    /// anywhere leaves a directory that can still say which turn it was.
    #[test]
    fn the_snapshots_go_before_what_described_them() {
        let source = what_this_file_ships();
        let snapshots = source.find("for snapshot in BOTH_SNAPSHOTS");
        let described = source.find("let described = at.join(THE_TURN)");
        assert!(
            matches!((snapshots, described), (Some(first), Some(then)) if first < then),
            "the removal must take both snapshots before what described them"
        );
    }
}
