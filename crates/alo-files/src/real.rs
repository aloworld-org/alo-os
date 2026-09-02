//! The path the machine would really open, as opposed to the one that was
//! asked for.
//!
//! Two paths can name one file and one path can name two files over its life.
//! A grant is over a place, so the question a grant has to be asked is about
//! the place — and that is what a [`Real`] is: what came back after every link
//! was followed.
//!
//! **It cannot be made outside this crate.** There is no public constructor,
//! which is what seals [`crate::Resolving`]: the only way to obtain one is to
//! ask [`crate::OnThisMachine`], so nothing anywhere can hand the grant check a
//! "real" path that it made up. A second way of resolving would be a second
//! answer to the question reach is decided on, and two answers that can
//! disagree are worse than none — the same reasoning that keeps the clock out
//! of [`alo_capability`].
//!
//! A `Real` is not promised to be usable: a path is compared against a grant by
//! [`alo_capability::path`], which refuses a relative path or one with `..` in
//! it, so an implausible answer is refused rather than trusted. Failing towards
//! a refusal is the only direction this crate is allowed to fail in.

use std::path::{Path, PathBuf};

/// A path with every link followed: what this machine would really open.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Real(PathBuf);

impl Real {
    /// What the resolver found.
    ///
    /// Crate-private on purpose — see this module's documentation. It is the
    /// whole of the seal, so moving it would quietly make [`crate::Resolving`]
    /// something anybody can implement.
    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// The path itself, for the code that will open it.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// The path, taken.
    #[must_use]
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }

    /// Where it really leads, in words a person reads in a refusal.
    #[must_use]
    pub fn describe(&self) -> String {
        self.0.display().to_string()
    }
}

/// Why a path could not be made real.
///
/// Both messages are read by somebody holding a call that did not run, so both
/// say what to do rather than which call failed. Neither is ever reached for a
/// path the grants have not already permitted — [`crate::Touching`] asks them
/// first, so a refusal here can only ever be about somewhere the agent was
/// already allowed to look.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RealError {
    /// Nothing is at that path.
    #[error(
        "there is nothing at {path} — a verb reaches what is there, so name something that exists"
    )]
    Nothing {
        /// The path as it was asked about.
        path: String,
    },
    /// Something is there, and this machine would not say where it leads.
    #[error(
        "{path} could not be followed ({why}) — nothing is done to a path this machine cannot resolve"
    )]
    Unreadable {
        /// The path as it was asked about.
        path: String,
        /// What the machine said, in its own words.
        why: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn march() -> Real {
        Real::new(PathBuf::from("/home/anna/Invoices/march.pdf"))
    }

    /// A real path is a path, and it says where it leads in the words a refusal
    /// will use.
    #[test]
    fn a_real_path_is_the_path_and_the_words_for_it() {
        assert_eq!(
            march().as_path(),
            Path::new("/home/anna/Invoices/march.pdf")
        );
        assert_eq!(march().describe(), "/home/anna/Invoices/march.pdf");
        assert_eq!(
            march().into_path_buf(),
            PathBuf::from("/home/anna/Invoices/march.pdf")
        );
    }

    /// Two paths that look alike are not alike, which is the comparison the
    /// grants are about to make.
    #[test]
    fn two_real_paths_are_the_same_only_when_they_are() {
        assert_eq!(
            march(),
            Real::new(PathBuf::from("/home/anna/Invoices/march.pdf"))
        );
        assert_ne!(
            march(),
            Real::new(PathBuf::from("/home/anna/Invoices2/march.pdf"))
        );
    }

    /// The refusals say what to do, and name the path they are about — a
    /// refusal that named neither is one somebody has to guess at.
    #[test]
    fn the_refusals_say_what_to_do() {
        let nothing = RealError::Nothing {
            path: "/home/anna/Invoices/april.pdf".to_owned(),
        };
        assert!(nothing.to_string().contains("april.pdf"), "{nothing}");
        assert!(nothing.to_string().contains("name something that exists"));

        let unreadable = RealError::Unreadable {
            path: "/home/anna/Invoices".to_owned(),
            why: "too many levels of symbolic links".to_owned(),
        };
        assert!(unreadable.to_string().contains("could not be followed"));
        assert!(unreadable.to_string().contains("too many levels"));
    }
}
