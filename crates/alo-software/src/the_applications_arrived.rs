//! The mark that says this machine's applications have already been walked.
//!
//! One question: **has `alo-shipping` run on this machine before?**
//!
//! # Why this exists, when systemd already answered it
//!
//! `alo-shipping.service` asked `ConditionFirstBoot=yes`, and its comment gave
//! the reason in as many words: *systemd already knows what a first boot is —
//! `/etc/machine-id` unpopulated — and inventing a second answer to a question
//! systemd already answers is how two of them come to disagree.* That
//! reasoning is sound, and **it is wrong about the one install path alo OS
//! uses.**
//!
//! **Measured, after a first attempt at this paragraph got the cause wrong.**
//! `/etc/machine-id` is 0 bytes in the container image, **0 bytes on a freshly
//! written disk that has never been booted**, and 33 bytes after one boot with
//! that boot's own mtime. So the *boot* populates it, not the install — the
//! first version of this said `bootc install to-disk` did, from a 33-byte file
//! on a disk that had already booted.
//!
//! **And `ConditionFirstBoot=yes` was unmet on that very boot.** On a disk
//! proven fresh, with an empty `machine-id` in the deployment,
//! `first-boot-complete.target - First Boot Complete skipped, unmet condition
//! check ConditionFirstBoot=yes` at 171 seconds — before anything of ours ran.
//! systemd had already decided this was not a first boot.
//!
//! Why it decides that is not measured here and is not load-bearing: the
//! likely mechanism is that systemd settles first-boot from the `/etc` it sees
//! in the initrd, and on an ostree deployment the `/etc` carrying this
//! `machine-id` is not that one. **What is measured is that it never fires**,
//! on a disk that could not be fresher, which is all this file needs to be
//! true. `alo-shipping.service` was skipped the same way: *The applications a
//! fresh machine has skipped, unmet condition check ConditionFirstBoot=yes*.
//!
//! So a machine written this way shipped with **none** of its applications and
//! reported success. There are not two answers to disagree: systemd's is
//! unusable here, and this is the only one.
//!
//! # What it is marked after, and what it is not
//!
//! **After the walk, not after a success.** `alo-shipping` exits zero when
//! some applications did not install, because *incomplete* is a state this
//! machine is allowed to be in (ADR 0073, and that program's own header). The
//! mark means *the list was walked once*, which is exactly what
//! `ConditionFirstBoot` meant, so a machine that had no network on its first
//! boot does not quietly try again for ever — ADR 0073 rejected a unit that
//! retries, and a person asks again through Software in Settings.
//!
//! **Not after a defect.** The two non-zero exits — the built-in list would
//! not parse, the places could not be read — mean the program could not do its
//! job at all. Those are faults here rather than conditions out there, and a
//! machine must not be marked done for one. This is strictly better than what
//! `ConditionFirstBoot` gave, which spent the one chance either way.
//!
//! # Why not under `/var/lib/alo`
//!
//! That directory is one person's: `image/usr/lib/tmpfiles.d/alo.conf` makes
//! it `0700 alo alo` because *what an agent did on somebody's machine is
//! theirs*. The applications a machine arrived with are the machine's — they
//! are installed into the system-wide flatpak installation, `Bound::Nobodys` —
//! and on a machine with two people there is one of these and two of those.
//! So it sits beside that directory rather than inside it, and the service
//! that writes it runs as root.

use std::io;
use std::path::{Path, PathBuf};

/// Where the mark lives on a running machine.
///
/// **Under `/var` and not `/etc`**, because an ostree deployment keeps both
/// and `/etc` is a place a person edits: a machine's applications having been
/// walked is not a setting anybody should be editing by hand, and a stray
/// `/etc` file surviving a reinstall would hide the applications from a fresh
/// machine. Made by the program rather than by `tmpfiles.d`, so that adding
/// this needed no second file to agree with.
const WHERE_IT_LIVES: &str = "/var/lib/alo-shipping/the-applications-arrived";

/// The mark on this machine, or on a root given for a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheApplicationsArrived {
    /// The file whose existence is the whole of the answer.
    at: PathBuf,
}

impl TheApplicationsArrived {
    /// The mark where a running machine keeps it.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            at: PathBuf::from(WHERE_IT_LIVES),
        }
    }

    /// The mark beneath some other root, for a test or a mounted disk.
    ///
    /// `WHERE_IT_LIVES` is absolute, so it is joined by its own components
    /// rather than with [`Path::join`] — joining an absolute path replaces the
    /// root instead of going beneath it, which would make every test read the
    /// real machine's file and pass on a machine that had already run.
    #[must_use]
    pub fn beneath(root: &Path) -> Self {
        Self {
            at: WHERE_IT_LIVES
                .split('/')
                .filter(|part| !part.is_empty())
                .fold(root.to_path_buf(), |path, part| path.join(part)),
        }
    }

    /// Where this mark is, which is what the unit's condition names.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// Whether the list has been walked on this machine.
    #[must_use]
    pub fn already(&self) -> bool {
        self.at.exists()
    }

    /// Record that the list has now been walked.
    ///
    /// The directory is made if it is not there, which is the one place this
    /// crate makes a directory of the machine's.
    ///
    /// # Errors
    /// [`io::Error`] from making the directory or writing the file — a
    /// read-only `/var`, or a full disk. **The caller says so and carries on
    /// rather than failing**: the applications really were walked, and a
    /// machine that could not write the mark walks them again next boot, which
    /// is a better failure than reporting a walk that did not happen.
    pub fn record(&self) -> Result<(), io::Error> {
        if let Some(holding) = self.at.parent() {
            std::fs::create_dir_all(holding)?;
        }
        std::fs::write(
            &self.at,
            b"the applications a fresh machine has were walked\n",
        )
    }
}

#[cfg(test)]
#[path = "the_applications_arrived_tests.rs"]
mod tests;
