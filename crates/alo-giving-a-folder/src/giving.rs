//! Making a person's directory and giving it to them.
//!
//! # One pure function, and a maker around it
//!
//! [`what_to_make`] decides **what** should exist and with which mode and
//! owner, from two numbers and a path — so every case is decided in the gate on
//! any host, including one with no `/var/lib`. [`made_for`] is the part that
//! touches the disk. It is `alo-accounts`' and `alo-sessiond`'s shape in this
//! repository, and for their reason: what a gate can decide anywhere should not
//! need a machine to decide it.
//!
//! # `0711` on the folder and not `0755`
//!
//! Every person must **traverse** the folder to reach their own directory, and
//! nobody but root may **list** it, because listing would tell one person who
//! else uses the machine.
//!
//! [ADR 0118](../../../docs/decisions/0118-more-than-one-person-on-a-machine.md)
//! records that this mode "exists for a case the sign-in screen cannot yet
//! reach" — it was chosen when a machine held one person, against a case that is
//! now accepted rather than hypothetical.

use std::{
    fs, io,
    os::unix::fs::{DirBuilderExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
};

use crate::failing::NotMade;

/// Where a machine keeps what an undo would put back.
///
/// `alo-letting-go` declares the same path and owns the layout. **This crate
/// does not depend on it**, because that crate carries the machine's one
/// privileged remover, and ADR 0119 chose this component for being small: a
/// giver that could reach a remover would be two powers in the place chosen to
/// hold one.
///
/// So the fact is in two places and held to one value by
/// `tests/the_contract_describes_where_an_undo_keeps.rs`, which reads the
/// published contract rather than the other crate's constant — two constants
/// compared against each other pass while both are wrong together.
pub const THE_FOLDER: &str = "/var/lib/alo-undo";

/// The mode the folder itself gets: traverse for everybody, list for root.
pub const THE_FOLDERS_MODE: u32 = 0o711;

/// The mode one person's directory gets: theirs and nobody else's.
pub const A_PERSONS_MODE: u32 = 0o700;

/// Something that should exist, with the mode and owner it should have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToMake {
    /// Where it goes.
    pub at: PathBuf,
    /// The mode it gets.
    pub mode: u32,
    /// The account that should own it, or [`None`] for root's own.
    pub owner: Option<u32>,
}

/// What should exist for this person, in the order it has to be made.
///
/// The folder first and their directory second, because the second goes inside
/// the first. Both are returned rather than only what is missing: deciding what
/// *should* be true is separate from asking the disk what is, and a step that
/// looked first would answer differently on a machine somebody had edited by
/// hand.
#[must_use]
pub fn what_to_make(under: &Path, person: u32) -> [ToMake; 2] {
    [
        ToMake {
            at: under.to_path_buf(),
            mode: THE_FOLDERS_MODE,
            owner: None,
        },
        ToMake {
            at: under.join(person.to_string()),
            mode: A_PERSONS_MODE,
            owner: Some(person),
        },
    ]
}

/// Make this person's place for what an undo keeps, under `under`, and give it
/// to them.
///
/// `under` is [`THE_FOLDER`] on a machine and a temporary folder in a test.
/// Returns where this person's own directory is.
///
/// # Making one that is already there is not a failure
///
/// A person signs in more than once, and ADR 0118 adds a second road that may
/// ask for the same person twice. What this does on a second call is put the
/// **mode and the owner back**, so a folder somebody widened by hand is
/// narrowed again rather than staying wide for ever.
///
/// **That is what makes two callers safe**, which ADR 0119 relies on: the first
/// account's road and the later route both start this, and neither has to know
/// whether the other already did.
///
/// # Errors
/// [`NotMade`], which is always the machine being unable rather than a rule
/// refusing. [`NotMade::NotTheirs`] is the one that happens on a machine where
/// this unit has lost `CAP_CHOWN`, and its sentence says so.
pub fn made_for(under: &Path, person: u32) -> Result<PathBuf, NotMade> {
    if !under.is_absolute() {
        return Err(NotMade::NotAnAbsoluteFolder(under.to_path_buf()));
    }

    let mut theirs = under.to_path_buf();
    for what in what_to_make(under, person) {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(what.mode)
            .create(&what.at)
            .map_err(|why| NotMade::TheFolder {
                at: what.at.clone(),
                why,
            })?;

        // **Set after creating, not only in the builder.** A `DirBuilder`'s
        // `mode` is masked by the process `umask` and is **ignored entirely for
        // a directory that already exists** — and the second case is every call
        // after the first. Without this, a folder somebody widened by hand would
        // stay wide for ever.
        fs::set_permissions(&what.at, fs::Permissions::from_mode(what.mode)).map_err(|why| {
            NotMade::TheFolder {
                at: what.at.clone(),
                why,
            }
        })?;

        if let Some(owner) = what.owner {
            given_to(&what.at, owner).map_err(|why| NotMade::NotTheirs {
                at: what.at.clone(),
                why,
            })?;
            theirs = what.at;
        }
    }
    Ok(theirs)
}

/// Give `at` to account `owner`, and leave its group exactly as it was.
///
/// **This is the one place in this repository that changes a file's owner**, and
/// `tests/only_here_may_give_a_directory_away.rs` holds it to that.
///
/// # The group is not set, and that is the decision rather than an omission
///
/// A person's directory here is `0700`, so the group bits grant nothing to
/// anybody: setting it would be a change with no effect bought with an
/// assumption about the machine.
///
/// The assumption would be that an account's group number equals its account
/// number. *A first version did assume that and was refused* — on the machine
/// this was written on, account 501's primary group is not 501, and `chown`
/// answered `EPERM` for a directory the account already owned. A person's real
/// group is field three of the user database; reading it would be the
/// alternative, and it would be reading a thing in order to put it somewhere it
/// does nothing.
fn given_to(at: &Path, owner: u32) -> io::Result<()> {
    rustix::fs::chown(at, Some(rustix::fs::Uid::from_raw(owner)), None)
        .map_err(|errno| io::Error::from_raw_os_error(errno.raw_os_error()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "giving_tests.rs"]
mod tests;
