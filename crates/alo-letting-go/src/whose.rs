//! Who owns a folder or a file, as the filesystem says — the one question that
//! settles whose act an asking is.
//!
//! A person asks their machine to forget everything it was keeping for them by
//! leaving a small file in [`crate::THE_ASKING`]
//! ([`crate::asking`]). That file **names nobody and nothing**: whose act it is,
//! is the user the filesystem records as having written it, which the kernel
//! sets and a writer cannot spell for itself.
//!
//! That is the whole of why this module exists. An asking that named a person —
//! a directory under [`crate::THE_FOLDER`], a name, a number — would be an act
//! that could be *aimed*, and the one privileged remover on this machine would
//! be taking its target from a file in a folder anybody may write to. It cannot
//! be aimed, because there is nothing in it to aim. The machine works out whose
//! it is by asking the disk who owns the settings folder each person's own
//! session wrote down, and forgets for the people that answer matches.
//!
//! # A link is never followed
//!
//! Everything here asks about the name it was given rather than about whatever
//! that name may point at — `symlink_metadata` and not `metadata`. A link left
//! in a folder anybody may write to is the oldest way there is of making a
//! privileged reader answer about somebody else's file, and this is the reader
//! whose answer decides whose history goes.

use std::io;
use std::path::Path;

use crate::the_disk::OnThisMachine;

/// Something that can be asked who owns a name on a disk — this machine, and a
/// test's stand-in that answers what the test arranged.
pub trait WhoOwns {
    /// The user the filesystem records as owning `at`, **without following a
    /// link**.
    ///
    /// # Errors
    /// [`io::Error`] when the name could not be asked about at all. A machine
    /// that cannot say who owns a folder forgets nothing for whoever it belongs
    /// to: the alternative is removing a person's history on a guess about
    /// whose it was.
    fn of(&self, at: &Path) -> io::Result<u32>;
}

#[cfg(unix)]
impl WhoOwns for OnThisMachine {
    fn of(&self, at: &Path) -> io::Result<u32> {
        use std::os::unix::fs::MetadataExt;

        Ok(std::fs::symlink_metadata(at)?.uid())
    }
}

/// Anywhere that is not a Unix, where this crate's units do not run.
#[cfg(not(unix))]
impl WhoOwns for OnThisMachine {
    fn of(&self, _at: &Path) -> io::Result<u32> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "alo OS is Linux (ADR 0011), and who owns a file is asked there",
        ))
    }
}

#[cfg(test)]
mod tests {
    /// What this file ships, without the tests at the bottom of it — so that a
    /// test naming a thing it forbids does not read as the file doing it.
    fn what_this_file_ships() -> &'static str {
        let whole = include_str!("whose.rs");
        match whole.find("#[cfg(test)]") {
            Some(tests) => whole.split_at(tests).0,
            None => whole,
        }
    }

    /// **A link is asked about as itself.** Read off this file rather than
    /// argued: the one reader whose answer decides whose history goes must
    /// never be pointed at somebody else's name.
    #[test]
    fn nothing_here_follows_a_link() {
        let source = what_this_file_ships();
        assert!(source.contains("symlink_metadata"));
        for never in ["fs::metadata(", "canonicalize(", "read_link("] {
            assert!(
                !source.contains(never),
                "{never} follows a link, and this is the reader that must not"
            );
        }
    }
}
