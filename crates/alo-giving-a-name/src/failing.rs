//! Why a person's name was not given to them, which is always the machine
//! being unable.
//!
//! Nothing here refuses a person. A sign-in whose name could not be given goes
//! ahead, and the person's terminal says `Local` alone where it would have said
//! `Local · anna` — `alo-sessiond`'s `their_name.rs` says why that is the right
//! trade, and `CLAUDE.md` law 5 is the reason that decides it.

use std::{io, path::PathBuf};

/// Why a person's name was not given to them.
#[derive(Debug)]
pub enum NotGiven {
    /// The account number was not a number.
    NotAnAccountNumber(String),

    /// The folder to give it in is not an absolute path.
    NotAnAbsoluteFolder(PathBuf),

    /// The account store could not be read.
    TheAccounts(alo_accounts::NotKept),

    /// The store has no account with this number.
    ///
    /// `alo-sessiond` checks the number against the same store before it
    /// starts this, so on a machine this means the store changed in between.
    NobodyNumbered(u32),

    /// The folder or the file could not be made, written or put in place.
    TheFile {
        /// Which one.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },

    /// The file was written and could not be given to the person.
    ///
    /// Kept apart from the one above, because it is the one that happens on a
    /// machine where this unit has lost `CAP_CHOWN`, and the sentence says so.
    NotTheirs {
        /// Which one.
        at: PathBuf,
        /// What the machine said.
        why: io::Error,
    },
}

impl std::fmt::Display for NotGiven {
    fn fmt(&self, into: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAnAccountNumber(said) => write!(
                into,
                "{said:?} is not an account number, so there is nobody to give a name to"
            ),
            Self::NotAnAbsoluteFolder(at) => write!(
                into,
                "{} is not an absolute path, so there is nowhere to give a name",
                at.display()
            ),
            Self::TheAccounts(why) => write!(
                into,
                "the account store could not be read, so no name was given: {why}"
            ),
            Self::NobodyNumbered(person) => write!(
                into,
                "the account store has no account numbered {person}, so no name was given"
            ),
            Self::TheFile { at, why } => write!(
                into,
                "{} could not be written, so this person's desktop shows no name: {why}",
                at.display()
            ),
            Self::NotTheirs { at, why } => write!(
                into,
                "{} was written and could not be given to the person, so their desktop cannot \
                 read it — this unit needs CAP_CHOWN and nothing else: {why}",
                at.display()
            ),
        }
    }
}

impl std::error::Error for NotGiven {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotAnAccountNumber(_)
            | Self::NotAnAbsoluteFolder(_)
            | Self::NobodyNumbered(_) => None,
            Self::TheAccounts(why) => Some(why),
            Self::TheFile { why, .. } | Self::NotTheirs { why, .. } => Some(why),
        }
    }
}
