//! One supervisor in one checkout, and the file that makes it one.
//!
//! Two of these running here would be two editors on one working tree, which
//! `CLAUDE.md` forbids for a reason this program would demonstrate within
//! seconds: both would rebase, and one would rebase over the other's
//! half-written commit.
//!
//! # Why a file made exclusively, and not a check
//!
//! *Is anybody running?* followed by *then I am* is the same shape as every
//! other check-then-act in this repository, and it loses the same race. The
//! file is created with `create_new`, which is one call that both refuses and
//! creates, so two loops starting together cannot both be told they are alone.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::{Path, PathBuf};

/// What the lock is called inside the loop's own directory.
const THE_LOCK: &str = "lock";

/// The lock, held for as long as this value is.
///
/// Dropping it takes the file away. That is the right shape here and the wrong
/// one for a boundary's pins: this is a program tidying up after itself, not a
/// machine deciding to stop enforcing something.
#[derive(Debug)]
pub struct Held {
    /// Where the lock file is.
    at: PathBuf,
}

impl Held {
    /// Take the lock, or say who has it.
    ///
    /// # Errors
    /// A sentence naming the file and what is in it — which is the other
    /// loop's process id — when somebody else is running, and whatever the
    /// machine said otherwise.
    pub fn taken(ours: &Path) -> Result<Self, String> {
        let at = ours.join(THE_LOCK);
        match OpenOptions::new().write(true).create_new(true).open(&at) {
            Ok(mut lock) => {
                // Best effort, and deliberately not checked: what the lock is
                // *for* is that it exists. What is written in it only helps a
                // person work out which process to look at.
                let _ = writeln!(lock, "{}", std::process::id());
                Ok(Self { at })
            }
            Err(why) if why.kind() == std::io::ErrorKind::AlreadyExists => {
                let whose = std::fs::read_to_string(&at).unwrap_or_default();
                let whose = whose.trim();
                Err(format!(
                    "another loop already holds {} (process {whose}). One supervisor per \
                     checkout: if that process is gone, remove the file and start again.",
                    at.display()
                ))
            }
            Err(why) => Err(format!(
                "the lock at {} could not be made: {why}",
                at.display()
            )),
        }
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        drop(std::fs::remove_file(&self.at));
    }
}

/// Whether a lock is being held here, for [`crate::journal`] to report.
#[must_use]
pub fn whose(ours: &Path) -> Option<String> {
    let held = std::fs::read_to_string(ours.join(THE_LOCK)).ok()?;
    Some(held.trim().to_owned())
}
