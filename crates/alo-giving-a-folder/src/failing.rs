//! Why a person's folder was not made, which is always the machine being
//! unable.
//!
//! Nothing here is a refusal by a rule and nothing here is a person being told
//! no. A caller meeting one of these has a machine that could not do something,
//! and ADR 0045 term 8's consequence is what it should do about it: **say so and
//! carry on**, because being unable to undo is not being unable to use a
//! machine, and `CLAUDE.md` law 5 is that alo never takes a choice away.
//!
//! The honest chain is then: no folder, so no bracket, so a run that requires
//! one is refused and says so — where the person asked for something, rather
//! than where they asked to sign in.

use std::{io, path::PathBuf};

/// Why a person's place for what an undo keeps was not made.
#[derive(Debug)]
pub enum NotMade {
    /// The account number was not a number.
    ///
    /// Said rather than defaulted: a helper that read `ada` as account 0 would
    /// give the machine's own root directory to nobody and report success.
    NotAnAccountNumber(String),

    /// The folder to make it under is not an absolute path.
    ///
    /// A relative one would be resolved against wherever this process happened
    /// to be started, which is not a place anybody chose.
    NotAnAbsoluteFolder(PathBuf),

    /// A directory could not be made, or its mode could not be set.
    TheFolder {
        /// Which one.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },

    /// The directory was made and could not be given to the person.
    ///
    /// **Kept apart from the one above**, because a directory that exists and
    /// belongs to root is worse than one that does not exist: the person cannot
    /// write to it, and nothing about its presence says so. Whoever reads the
    /// journal needs to know which of the two they have.
    ///
    /// On a machine where this unit has lost `CAP_CHOWN` this is the one that
    /// happens, and the sentence names the capability so that nobody has to
    /// measure it twice.
    NotTheirs {
        /// Which one.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
}

impl std::fmt::Display for NotMade {
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAnAccountNumber(said) => write!(
                into,
                "{said:?} is not an account number, so there is nobody to give a folder to"
            ),
            Self::NotAnAbsoluteFolder(at) => write!(
                into,
                "{} is not an absolute path, so there is nowhere to make it",
                at.display()
            ),
            Self::TheFolder { at, why } => write!(
                into,
                "{} could not be made, so nothing of this person's files can be kept for an \
                 undo: {why}",
                at.display()
            ),
            Self::NotTheirs { at, why } => write!(
                into,
                "{} was made and could not be given to the person, so it exists and they \
                 cannot write to it — this unit needs CAP_CHOWN and nothing else: {why}",
                at.display()
            ),
        }
    }
}

impl std::error::Error for NotMade {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotAnAccountNumber(_) | Self::NotAnAbsoluteFolder(_) => None,
            Self::TheFolder { why, .. } | Self::NotTheirs { why, .. } => Some(why),
        }
    }
}
