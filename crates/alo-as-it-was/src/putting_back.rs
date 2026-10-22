//! Putting one folder back as it was, and every reason it is refused instead.
//!
//! [ADR 0127](../../../docs/decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
//! term 2 decided three things about when a rewind happens, and this is the
//! first of them: **a folder, not the home, now.** The home is the thing a
//! person is living in while they ask to rewind it — their shell, their editor
//! and their applications hold files open in it — so the whole home waits for a
//! sign-in, and a privileged helper that swaps the subvolume was refused
//! outright. A folder needs none of that: it is their own file in their own
//! home, read from their own snapshot, by their own process.
//!
//! # What was there is set aside, never thrown away
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s third
//! term is *an undo never overwrites a later change*, and it is enforced there
//! by comparing the two sides of a turn. **Nothing compares a person's own
//! command** — ADR 0127 says so in as many words, *restoring reads a snapshot
//! and compares nothing* — so that term cannot be met the same way here. It is
//! met the other way instead: what is in the folder now is **renamed beside it**
//! rather than deleted, and the person is told where it went. A later change is
//! therefore never lost, only moved, and a person who went back by mistake has
//! somewhere to go back from.
//!
//! That is
//! [ADR 0131](../../../docs/decisions/0131-what-happens-to-what-is-in-a-folder-when-it-goes-back.md),
//! which is where the three alternatives are written down with what each costs —
//! including the one this is: a directory in somebody's home that only they can
//! tidy away.
//!
//! # The one moment that has to be atomic
//!
//! There is exactly one: the instant the folder stops being what it is and
//! starts being what it was. `renameat2` with `RENAME_EXCHANGE` swaps two
//! directory entries in one call, so there is no instant at which the folder is
//! missing. Everything either side of it is a copy or a rename that can be
//! interrupted and leaves something a person can see and name.
//!
//! # Why the base's own `cp`
//!
//! ADR 0045's chosen road says *by reflinked copy*, and `--reflink=auto` is what
//! asks for one and falls back to a full copy where the filesystem has no such
//! thing. `-a` keeps the modes, the times, the symbolic links as links and the
//! extended attributes. Writing that in Rust would be reimplementing a program
//! the image already ships, which the standing rules forbid for the same reason
//! they forbid patching one.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use rustix::fs::{CWD, RenameFlags, renameat_with};

use crate::{removing, when};

/// The program that copies, and the arguments it is given.
///
/// `--no-target-directory` is the one that matters: without it a copy into a
/// name that already exists lands *inside* it, and this copy's whole point is
/// that the name does not exist yet.
const THE_PROGRAM: &str = "cp";

/// What is asked of it, before the two paths.
const THE_ARGUMENTS: [&str; 3] = ["-a", "--reflink=auto", "--no-target-directory"];

/// The suffix a folder being put back is built under.
///
/// Visible on purpose. A person who looks while this is running sees a name
/// that says what it is, and one left behind by an interrupted go-back is found
/// by the same name rather than being a directory nobody can explain.
pub const BEING_PUT_BACK: &str = "alo-as-it-was";

/// The suffix what was there is set aside under.
pub const WHAT_WAS_HERE: &str = "alo-was-here";

/// Every reason a folder is not put back.
#[derive(Debug, thiserror::Error)]
pub enum NotGoingBack {
    /// The folder named is not inside this person's home.
    ///
    /// Checked after both paths are resolved, so a link pointing out of the home
    /// is caught rather than followed. There is no grant to `/`, and there is no
    /// road here to one either.
    #[error("{folder} is not inside {home}")]
    NotInTheirHome {
        /// What was named, as it resolves.
        folder: PathBuf,
        /// Their home, as it resolves.
        home: PathBuf,
    },

    /// The folder named **is** the home.
    ///
    /// The whole home is the second road in ADR 0127's table and happens at the
    /// next sign-in, not now: nothing can swap a directory somebody is living
    /// in. Refused by name rather than attempted, because attempting it would
    /// half-work.
    #[error("{home} is the whole home, which cannot be put back while you are signed in")]
    TheWholeHome {
        /// Their home.
        home: PathBuf,
    },

    /// What was named is not a folder, or is not there.
    #[error("{folder} is not a folder on this machine")]
    NotAFolder {
        /// What was named.
        folder: PathBuf,
    },

    /// The place holds no copy of that folder.
    ///
    /// The folder did not exist when the snapshot was taken. Putting it *back*
    /// would mean deleting it, which is a different act and a larger one, and is
    /// refused rather than quietly done.
    #[error("{folder} was not there when that place was kept, so there is nothing to put back")]
    NotInThatPlace {
        /// What was named.
        folder: PathBuf,
    },

    /// The copy inside the place is a symbolic link.
    ///
    /// A link in a read-only snapshot still points wherever it pointed, and
    /// following one would copy from outside the snapshot. The folder itself is
    /// never followed; links *inside* it are copied as links, which is `cp -a`.
    #[error("{at} is a link rather than a folder, and this does not follow one")]
    ALinkRatherThanAFolder {
        /// Where in the place it was found.
        at: PathBuf,
    },

    /// A previous go-back was interrupted and its working name is still there.
    #[error("{at} is still there from a go-back that did not finish — look at it, then move it")]
    OneIsHalfDone {
        /// The working name that is still there.
        at: PathBuf,
    },

    /// The name what-was-there would be set aside under is taken.
    ///
    /// Only reachable when two go-backs of one folder land in the same second,
    /// which is a name collision rather than anything about the folder — and it
    /// is refused before anything is copied, because the alternative is finding
    /// out after the swap, when the folder has already changed.
    #[error("{at} is already there, so what is in the folder now has nowhere to go")]
    NowhereToSetAsideWhatIsThere {
        /// The name that is taken.
        at: PathBuf,
    },

    /// The copy out of the snapshot did not finish.
    #[error("the folder could not be copied out of that place: {why}")]
    NotCopied {
        /// What the program said, or why it would not run.
        why: String,
    },

    /// The swap did not happen, and nothing changed.
    #[error("the folder was not put back and nothing was changed: {why}")]
    NotSwapped {
        /// What the kernel said.
        why: String,
    },

    /// The folder went back and what was there could not be renamed.
    ///
    /// The folder **is** as it was — the swap succeeded. What failed is giving
    /// the old contents their final name, so they are still under
    /// [`BEING_PUT_BACK`], and the person is told that rather than told it
    /// worked.
    #[error("{folder} is as it was, but what was there is still called {at}: {why}")]
    SetAsideUnderTheWorkingName {
        /// The folder, which **did** go back.
        folder: PathBuf,
        /// Where the old contents are.
        at: PathBuf,
        /// What the kernel said.
        why: String,
    },
}

/// Exactly what going back would do — the thing a person approves.
///
/// Built before anything is touched and executed by [`go_back`], so that the
/// sentence a person reads and the act that follows are one object rather than
/// two descriptions of the same intention. *What a person approves is that
/// sentence.*
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatGoesBack {
    /// The folder that will be as it was.
    folder: PathBuf,
    /// The copy it will be taken from.
    from: PathBuf,
    /// Where it is built before the swap.
    staging: PathBuf,
    /// Where what is there now will be found afterwards.
    what_was_here: PathBuf,
    /// When the place was taken.
    taken: SystemTime,
}

impl WhatGoesBack {
    /// The folder that will be as it was.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// Where what is there now will be found afterwards.
    #[must_use]
    pub fn what_was_here(&self) -> &Path {
        &self.what_was_here
    }

    /// When the place was taken, as a person reads it.
    #[must_use]
    pub fn taken_as_text(&self) -> String {
        when::as_text(self.taken)
    }
}

/// Work out what going back would do, touching nothing.
///
/// `before` is the snapshot inside the place — [`crate::APlaceKept::before`].
/// `folder` is what the person named. `home` is their home. `taken` is when the
/// place was taken, which is carried so that what they approve says it. `now`
/// names what is in the folder at this moment, so that going back twice does
/// not put the second old copy where the first one is.
///
/// # Errors
/// Every [`NotGoingBack`] except the four that can only happen while something
/// is being written.
pub fn what_would_go_back(
    before: &Path,
    folder: &Path,
    home: &Path,
    taken: SystemTime,
    now: SystemTime,
) -> Result<WhatGoesBack, NotGoingBack> {
    // **Resolved first, and compared afterwards.** A textual check would pass a
    // link inside the home that points out of it, which is the one shape of this
    // mistake that looks right in a test written with ordinary directories.
    let folder = folder
        .canonicalize()
        .map_err(|_| NotGoingBack::NotAFolder {
            folder: folder.to_owned(),
        })?;
    let home = home
        .canonicalize()
        .map_err(|_| NotGoingBack::NotInTheirHome {
            folder: folder.clone(),
            home: home.to_owned(),
        })?;
    if !folder.is_dir() {
        return Err(NotGoingBack::NotAFolder { folder });
    }
    if folder == home {
        return Err(NotGoingBack::TheWholeHome { home });
    }
    let inside = folder
        .strip_prefix(&home)
        .map_err(|_| NotGoingBack::NotInTheirHome {
            folder: folder.clone(),
            home: home.clone(),
        })?;

    let from = before.join(inside);
    // **`symlink_metadata`, not `metadata`.** The question is what this entry
    // *is*, not what it points at: a link is refused here rather than resolved.
    match std::fs::symlink_metadata(&from) {
        Err(_) => {
            return Err(NotGoingBack::NotInThatPlace { folder });
        }
        Ok(what) if what.file_type().is_symlink() => {
            return Err(NotGoingBack::ALinkRatherThanAFolder { at: from });
        }
        Ok(what) if !what.is_dir() => {
            return Err(NotGoingBack::NotInThatPlace { folder });
        }
        Ok(_) => {}
    }

    let staging = beside(&folder, BEING_PUT_BACK);
    if staging.symlink_metadata().is_ok() {
        return Err(NotGoingBack::OneIsHalfDone { at: staging });
    }
    let what_was_here = beside(&folder, &format!("{WHAT_WAS_HERE}-{}", seconds_of(now)));
    if what_was_here.symlink_metadata().is_ok() {
        return Err(NotGoingBack::NowhereToSetAsideWhatIsThere { at: what_was_here });
    }

    Ok(WhatGoesBack {
        folder,
        from,
        staging,
        what_was_here,
        taken,
    })
}

/// Put the folder back.
///
/// Three steps, in this order and no other: copy out of the snapshot under a
/// working name; swap that name with the folder in one call; give what came out
/// of the folder its final name.
///
/// # Errors
/// [`NotGoingBack::NotCopied`] and [`NotGoingBack::NotSwapped`] both leave the
/// folder exactly as it was. [`NotGoingBack::SetAsideUnderTheWorkingName`] means
/// the folder **is** as it was and only the last rename failed.
pub fn go_back(goes: &WhatGoesBack) -> Result<(), NotGoingBack> {
    let said = Command::new(THE_PROGRAM)
        .args(THE_ARGUMENTS)
        .arg(&goes.from)
        .arg(&goes.staging)
        .output();
    match said {
        Err(why) => {
            return Err(NotGoingBack::NotCopied {
                why: why.to_string(),
            });
        }
        Ok(done) if !done.status.success() => {
            // **Whatever was half-copied is removed**, so that a second
            // attempt is not refused by `OneIsHalfDone` over a directory this
            // call made. `crate::removing` and not `remove_dir_all`: the
            // directory `cp` leaves behind carries the source's mode, and when
            // the copy failed *because* the source could not be read that mode
            // is one the ordinary remover will not descend.
            let _ = removing::removed(&goes.staging);
            return Err(NotGoingBack::NotCopied {
                why: String::from_utf8_lossy(&done.stderr).trim_end().to_owned(),
            });
        }
        Ok(_) => {}
    }

    // **The one atomic moment.** After this call the folder is what it was and
    // the working name holds what the folder held, with no instant in between at
    // which either name is missing.
    if let Err(why) = renameat_with(CWD, &goes.staging, CWD, &goes.folder, RenameFlags::EXCHANGE) {
        let _ = removing::removed(&goes.staging);
        return Err(NotGoingBack::NotSwapped {
            why: why.to_string(),
        });
    }

    std::fs::rename(&goes.staging, &goes.what_was_here).map_err(|why| {
        NotGoingBack::SetAsideUnderTheWorkingName {
            folder: goes.folder.clone(),
            at: goes.staging.clone(),
            why: why.to_string(),
        }
    })
}

/// A name beside this one, with a suffix.
///
/// Beside rather than inside the undo folder, deliberately: the undo folder has
/// a privileged remover with an expiry on it, and a person's own work from after
/// the snapshot would be swept by it. What was there belongs to them, in their
/// own home, where they can see it and delete it themselves.
fn beside(folder: &Path, suffix: &str) -> PathBuf {
    let mut named = folder.file_name().unwrap_or_default().to_owned();
    named.push(".");
    named.push(suffix);
    folder.with_file_name(named)
}

/// This moment in whole seconds, which is what a set-aside name is told apart
/// by.
fn seconds_of(now: SystemTime) -> u64 {
    now.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "putting_back_tests.rs"]
mod tests;
