//! What a command a person said *try* may reach, and nothing else.
//!
//! `docs/features.md`, starred: *Try a command without letting it touch
//! anything. Paste the install script from a README and **run it**.* The owner
//! decided the gesture on 2026-10-18 — **a word before the command, plus alo
//! offering when it sees a risk** — so this is the places half of that word.
//!
//! # *Touch anything* cannot mean literally nothing, and this says why
//!
//! `alo-bounding`'s boundary permits a path **at or under** a granted place and
//! refuses everything else. A box given only `./install.sh` therefore refuses
//! `/bin/sh`, refuses the dynamic loader, and the script does not start — so a
//! sealed box bound to nothing is not a strict promise kept, it is a command
//! that cannot run.
//!
//! **What a person actually wants from `curl … | sh` is that it cannot touch
//! *their* things**, and that is what this gives:
//!
//! | granted | why |
//! |---|---|
//! | the command itself | it has to be read to be run |
//! | [`WHERE_PROGRAMS_ARE`] | so an interpreter named on a `#!` line can be found and read |
//! | the library folder, from [`where_the_libraries_are`] | so a dynamically linked program can start at all |
//! | [`THE_EMPTY_DEVICE`] | so `>/dev/null` works, which is the most ordinary line in any script |
//!
//! **And nothing of the person's.** Not their home, not the folder they are
//! standing in, not the disk. A script run this way can read the system it
//! needs in order to run and can reach no file anybody wrote.
//!
//! *These three are the set `alo-running`'s own kernel test has used since
//! #729* — `a_command_reaches_what_its_box_was_given` grants the target,
//! `/usr/bin` and the library folder. This file is that set promoted out of a
//! test, because a set a product relies on and only a test states is a set
//! nobody maintains.
//!
//! # What is deliberately not here
//!
//! **A writable place.** Many install scripts want somewhere to unpack, and
//! giving them `/tmp` would hand them a folder other programs keep things in.
//! A box with nowhere to write refuses the write, which is the honest first
//! version of *touch anything*: a person sees the script fail saying it cannot
//! write, which is true and is the thing they asked for. **Somewhere of its own
//! to write is a promise to add deliberately**, not a default to slip in, and
//! `docs/features.md` says nothing about one.
//!
//! **The network.** A different axis, and a different promise — *`--offline` so
//! it may not reach the network* is the neighbouring feature. A box here is
//! bound in the filesystem only, and this file does not pretend otherwise.

use std::path::{Path, PathBuf};

/// Where a machine keeps the programs an interpreter line can name.
pub const WHERE_PROGRAMS_ARE: &str = "/usr/bin";

/// The empty device, which almost every script opens.
///
/// **Measured on a real kernel on 2026-10-18 and not reasoned about.** A box
/// without it refused `>/dev/null`, which is the most ordinary line in any
/// shell script — and the first version of this crate's own `try` test
/// contained one, so the redirect failed, the command after it never ran, and
/// the test passed while measuring nothing. The run printed
/// `cannot create /dev/null: Permission denied` and that one line is the whole
/// evidence.
///
/// **`/dev/null` by name and never `/dev`.** The folder holds the disks, the
/// memory and every input device on the machine, and the boundary grants a
/// place and everything under it.
pub const THE_EMPTY_DEVICE: &str = "/dev/null";

/// The folders a library could be in, in the order they are tried.
///
/// Two rather than one because a merged-`/usr` machine has `/lib` as a link to
/// `/usr/lib` and an unmerged one has both as real directories — and the
/// boundary compares a device and an inode, so the link and its target are not
/// the same place. The first that is a directory wins and is resolved.
const WHERE_LIBRARIES_COULD_BE: [&str; 2] = ["/usr/lib", "/lib"];

/// Why a machine cannot run a boxed command at all.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoPlacesForIt {
    /// The command is not an absolute path.
    ///
    /// Resolved by whoever took it from the person, because only they know
    /// which folder the person was standing in. A relative path arriving here
    /// would be resolved against this process's folder, which is not theirs.
    #[error("{named} does not say where it is from the root of the machine")]
    NotAFullPath {
        /// What was asked for, as it was asked.
        named: String,
    },
    /// This machine has no library folder.
    ///
    /// Cannot happen on a machine that runs anything dynamically linked, which
    /// is every machine this product ships on. It is an error rather than a
    /// panic because a library inside a command must not take the command down
    /// with it.
    #[error("this machine has no library folder, so no dynamically linked command can run")]
    NoLibraryFolder,
}

/// The library folder this machine keeps, resolved.
///
/// # Errors
/// [`NoPlacesForIt::NoLibraryFolder`] on a machine with neither, which is not
/// a machine that runs anything.
pub fn where_the_libraries_are() -> Result<PathBuf, NoPlacesForIt> {
    for maybe in WHERE_LIBRARIES_COULD_BE {
        let path = Path::new(maybe);
        if path.is_dir() {
            // **Resolved, because the boundary compares inodes.** `/lib` as a
            // link to `/usr/lib` is a different place from `/usr/lib` as far
            // as the kernel is concerned, and granting the link grants
            // nothing the loader opens.
            if let Ok(resolved) = path.canonicalize() {
                return Ok(resolved);
            }
        }
    }
    Err(NoPlacesForIt::NoLibraryFolder)
}

/// Everywhere a command a person said *try* may reach.
///
/// Four places, in a settled order: the command, the programs folder, the
/// library folder, the empty device. See this module's header for why it is
/// four and not none.
///
/// **Four is `alo_bounding::PLACES`, so this is exactly full.** A fifth
/// place — somewhere of its own to write, most likely — cannot be added without
/// raising that bound, which is a change to the kernel side of the boundary.
/// Said here because the next person to want one will look at this function
/// first.
///
/// # Errors
/// [`NoPlacesForIt`] — a command that is not a full path, or a machine with no
/// library folder.
pub fn what_a_try_may_reach(program: &Path) -> Result<Vec<PathBuf>, NoPlacesForIt> {
    if !program.is_absolute() {
        return Err(NoPlacesForIt::NotAFullPath {
            named: program.display().to_string(),
        });
    }
    Ok(vec![
        program.to_path_buf(),
        PathBuf::from(WHERE_PROGRAMS_ARE),
        where_the_libraries_are()?,
        PathBuf::from(THE_EMPTY_DEVICE),
    ])
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected Err, or an index past a length the test just \
              asserted, is the failure being reported"
)]
#[path = "what_a_try_may_reach_tests.rs"]
mod tests;
