//! Which folder a person's shell starts in.
//!
//! # Their home, as every terminal they have used
//!
//! A terminal opens in the person's home folder. That is what every terminal
//! on every desktop does, and `docs/features.md` asks this one to be ordinary
//! everywhere it is not the point. **Until 2026-10-28 it opened in `/`**: the
//! shell inherited the folder the compositor was started in, which under
//! systemd is the top of the disk. Found walking the boot demo — `pwd` printed
//! `/`, so `ls Documents` answered *No such file or directory* on a machine
//! that had made that folder for the person at their first sign-in.
//!
//! # A home that is not there still gets a shell
//!
//! When the session names no home, names one that is not an absolute path, or
//! names one that is not a folder on this disk, the shell **still starts** —
//! in `/`, which every account can enter — and [`WhereItStarts::NotTheirHome`]
//! says why, so the caller can say it rather than leave a person in the wrong
//! place without a word. Refusing to open the terminal instead would take the
//! shell away at the moment a person most needs one: a broken home folder is
//! something you mend from a shell. `/` is also what `login` itself does, in
//! its own words: *No directory, logging in with HOME=/*.
//!
//! **`/` is named rather than inherited**, because what the compositor happens
//! to be standing in is an accident of how it was started — the top of the
//! disk on a machine, a source tree on a developer's desk — and a fallback
//! nobody can predict is not one anybody can check.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Where a shell starts when the person's home cannot be used.
const THE_TOP_OF_THE_DISK: &str = "/";

/// Which folder a shell starts in, and whether that is the person's home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhereItStarts {
    /// The person's home folder, which is the ordinary answer.
    TheirHome(PathBuf),
    /// Not their home, for this reason; the shell starts in `/` instead.
    NotTheirHome(WhyNotHome),
}

/// Why a shell could not start in the person's home.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WhyNotHome {
    /// The session named no home at all, or named an empty one.
    #[error("this session names no home folder, so the shell starts in /")]
    NoneWasNamed,
    /// The home named is not an absolute path, so where it is would depend on
    /// where something else happened to be standing.
    #[error(
        "the home folder this session names, {0:?}, is not a full path, so the shell starts in /"
    )]
    NotAFullPath(PathBuf),
    /// The home named is not a folder on this disk: it is missing, or it is
    /// something other than a folder.
    #[error(
        "the home folder this session names, {0:?}, is not a folder on this disk, so the shell starts in /"
    )]
    NotAFolder(PathBuf),
}

impl WhereItStarts {
    /// Where a shell starts for a session whose `$HOME` is `home`.
    #[must_use]
    pub fn for_a_home(home: Option<&OsStr>) -> Self {
        let Some(home) = home.filter(|named| !named.is_empty()) else {
            return Self::NotTheirHome(WhyNotHome::NoneWasNamed);
        };
        let home = PathBuf::from(home);
        if !home.is_absolute() {
            return Self::NotTheirHome(WhyNotHome::NotAFullPath(home));
        }
        if !home.is_dir() {
            return Self::NotTheirHome(WhyNotHome::NotAFolder(home));
        }
        Self::TheirHome(home)
    }

    /// The folder the shell is started in.
    #[must_use]
    pub fn the_folder(&self) -> &Path {
        match self {
            Self::TheirHome(home) => home,
            Self::NotTheirHome(_) => Path::new(THE_TOP_OF_THE_DISK),
        }
    }
}

#[cfg(test)]
#[path = "where_a_shell_starts_tests.rs"]
mod tests;
