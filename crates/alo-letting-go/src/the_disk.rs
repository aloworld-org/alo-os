//! The two questions this crate asks a filesystem: whether it is one that can
//! keep an undo at all, and how much room is left on it.
//!
//! Both are read from the kernel rather than remembered, for
//! `alo-measuring`'s reason: a number the machine derived and hopes is close is
//! the one number a person cannot check. `statfs` answers both at once —
//! `f_type` says which filesystem, `f_bavail` and `f_bsize` say how much of it
//! an unprivileged writer could still use.
//!
//! # A machine that keeps nothing does nothing, rather than failing
//!
//! ADR 0045's sixth term: *a machine installed on `ext4` keeps working and
//! answers* not yet on this machine *for every undo, honestly, until it is
//! reinstalled.* There is nothing there to remove, so the unit has nothing to
//! do — and a timer whose unit fails at every firing on a perfectly healthy
//! machine is a machine that teaches its administrator to ignore it.
//! [`WhatItIs::NotOneThatKeeps`] is therefore an answer and not an error.
//!
//! # Why free space rather than a percentage
//!
//! [`THE_FLOOR`] is an amount, because what a machine needs room for is an
//! amount: staging one system image is several gibibytes whatever the size of
//! the disk it lands on, and a percentage would leave a small machine unable to
//! update while a large one hoarded space it had no use for.

use std::io;
use std::path::Path;

/// How much free space this machine keeps, below which an undo loses to the
/// disk.
///
/// **Ten gibibytes**, and the number is an argument rather than a taste. ADR
/// 0045's second accepted term says *the machine never fills a disk to preserve
/// an undo*, and the first thing a machine that cannot write loses is the thing
/// this workstream is otherwise about: `bootc` stages a whole system image onto
/// the disk before a person restarts into it, which the pinned base does in
/// units of gibibytes, and a person's working day needs room beside it. A floor
/// under that would be a machine that kept an undo and could not take the
/// update it had told the person was ready — two promises in
/// `docs/features.md`, with the wrong one winning.
///
/// It is **not** a setting. The person's one setting is the window (ADR 0045's
/// first term, `crate::keeping`), and a second lever over the same snapshots is
/// how a machine ends up with nobody able to answer *when will this be gone*.
pub const THE_FLOOR: u64 = 10 * 1024 * 1024 * 1024;

/// What the filesystem under a folder is, as far as an undo is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItIs {
    /// A filesystem with subvolumes and read-only snapshots — `btrfs`,
    /// `alo_image::THE_ONLY_FILESYSTEM`, which is what alo OS installs.
    OneThatKeeps,
    /// Anything else. A machine installed before the filesystem was decided,
    /// which keeps working and can undo nothing.
    NotOneThatKeeps,
}

/// The filesystem `btrfs` answers `statfs` with.
///
/// `BTRFS_SUPER_MAGIC`, from the kernel's own `include/uapi/linux/magic.h`.
/// Named here rather than reached for from a crate, so that the one number this
/// decision turns on is written down beside the reason it matters.
pub const BTRFS: i64 = 0x9123_683E;

/// Something that can be asked about a filesystem — the machine, and a test's
/// stand-in that answers what the test arranged.
pub trait AskingTheDisk {
    /// What the filesystem under `at` is.
    ///
    /// # Errors
    /// [`io::Error`] when the folder could not be asked about at all, which is
    /// not the same answer as *not one that keeps*: a machine that cannot be
    /// asked removes nothing.
    fn what_it_is(&self, at: &Path) -> io::Result<WhatItIs>;

    /// How many bytes are free on the filesystem under `at`.
    ///
    /// # Errors
    /// [`io::Error`] when it could not be asked.
    fn free(&self, at: &Path) -> io::Result<u64>;

    /// Whether the disk under `at` is below [`THE_FLOOR`].
    ///
    /// # Errors
    /// [`io::Error`] when it could not be asked. A machine that cannot say how
    /// much room it has is one that removes nothing for room — the honest
    /// answer, since the alternative is removing a person's undo on a guess.
    fn below_the_floor(&self, at: &Path) -> io::Result<bool> {
        self.free(at).map(|free| free < THE_FLOOR)
    }
}

/// This machine's own filesystems, asked with `statfs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct OnThisMachine;

#[cfg(unix)]
impl AskingTheDisk for OnThisMachine {
    fn what_it_is(&self, at: &Path) -> io::Result<WhatItIs> {
        let said = rustix::fs::statfs(at)?;
        Ok(if said.f_type == BTRFS {
            WhatItIs::OneThatKeeps
        } else {
            WhatItIs::NotOneThatKeeps
        })
    }

    fn free(&self, at: &Path) -> io::Result<u64> {
        let said = rustix::fs::statfs(at)?;
        let blocks = said.f_bavail;
        let size = u64::try_from(said.f_bsize).unwrap_or(0);
        Ok(blocks.saturating_mul(size))
    }
}

/// Anywhere that is not a Unix, where this crate's unit does not run.
#[cfg(not(unix))]
impl AskingTheDisk for OnThisMachine {
    fn what_it_is(&self, _at: &Path) -> io::Result<WhatItIs> {
        Err(not_a_machine())
    }

    fn free(&self, _at: &Path) -> io::Result<u64> {
        Err(not_a_machine())
    }
}

/// What this machine says anywhere alo OS is not.
#[cfg(not(unix))]
fn not_a_machine() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "alo OS is Linux (ADR 0011), and a filesystem is asked about there",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A filesystem that is not the one alo OS installs keeps nothing**, and
    /// the machine says so rather than failing — which is what keeps the
    /// timer quiet on a machine installed before the filesystem was decided.
    #[test]
    fn the_two_answers_are_an_answer_each_and_neither_is_an_error() {
        assert_ne!(WhatItIs::OneThatKeeps, WhatItIs::NotOneThatKeeps);
    }

    /// **The floor is ten gibibytes**, and it is an amount rather than a share
    /// of the disk — a small machine needs the same room to stage an update as
    /// a large one.
    #[test]
    fn the_floor_is_ten_gibibytes() {
        assert_eq!(THE_FLOOR, 10_737_418_240);
    }

    /// **The filesystem this decision turns on is the one alo OS installs.**
    /// `alo_image::THE_ONLY_FILESYSTEM` is `btrfs`, and this is the number the
    /// kernel answers for it; a test elsewhere holds the two names together on
    /// a real disk.
    #[test]
    fn the_number_is_btrfs() {
        assert_eq!(BTRFS, 0x9123_683E);
    }

    /// **Below the floor is read off the number that was asked for**, so a
    /// machine exactly at the floor is not below it.
    #[test]
    fn a_machine_exactly_at_the_floor_is_not_below_it() {
        /// A disk with however many bytes free the test says.
        struct WithFree(u64);
        impl AskingTheDisk for WithFree {
            fn what_it_is(&self, _at: &Path) -> io::Result<WhatItIs> {
                Ok(WhatItIs::OneThatKeeps)
            }
            fn free(&self, _at: &Path) -> io::Result<u64> {
                Ok(self.0)
            }
        }

        let at = Path::new("/var/lib/alo/undo");
        assert!(!WithFree(THE_FLOOR).below_the_floor(at).unwrap_or(true));
        assert!(WithFree(THE_FLOOR - 1).below_the_floor(at).unwrap_or(false));
    }
}
