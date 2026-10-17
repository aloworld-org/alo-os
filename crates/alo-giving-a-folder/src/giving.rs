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
    os::unix::fs::{DirBuilderExt as _, MetadataExt as _, PermissionsExt as _},
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

/// Whether the directory has to be taken back before its mode can be set.
///
/// **Setting a mode needs ownership**, or `CAP_FOWNER`, which this unit
/// deliberately does not hold. So a directory this program has already given to
/// somebody has to be taken back first — which `CAP_CHOWN` permits — and a
/// directory this program still owns does not.
///
/// A function rather than the condition written at the call site, because it is
/// the whole of the decision and it is arithmetic on two numbers: it can
/// therefore be held to every case in the gate, on a host where nobody is root
/// and the road it guards never runs.
///
/// It is also what keeps the capability unexercised when it is not needed. The
/// first call for a person creates the directory and owns it, so nothing is
/// taken back; only a repeat call reaches the `chown`.
#[must_use]
const fn taken_back_first(whose: u32, us: u32) -> bool {
    whose != us
}

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
/// A person signs in more than once, and whatever starts this will start it
/// again. What it does on a second call is put the **mode and the owner back**,
/// so a folder somebody widened by hand is narrowed again rather than staying
/// wide for ever.
///
/// **That is what lets a caller ask every time** rather than having to remember
/// whether it already has — which is what a sign-in road needs, since the
/// alternative is a privileged component keeping a list of who has a folder.
///
/// *The second call failed with `EPERM` until 2026-10-17*, and the test that
/// catches it is `a_second_call_puts_a_widened_mode_back`. Setting the mode on a
/// directory this program has already given away needs `CAP_FOWNER`, which the
/// unit does not hold; the fix is to take it back first, which `CAP_CHOWN`
/// permits. The property was documented before it was true — in this comment,
/// in ADR 0120 and in the unit — which is why the test is named after the second
/// call rather than after the mode.
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

        // **Taken back before the mode is set, and given away after.**
        //
        // A `DirBuilder`'s `mode` is masked by the process `umask` and is
        // **ignored entirely for a directory that already exists**, so the mode
        // has to be set explicitly — and the second call for a person is a
        // directory that already exists *and is already theirs*. `chmod` on a
        // file you do not own needs `CAP_FOWNER`, which this unit deliberately
        // does not hold, so the second call was refused with `EPERM` until this
        // took the directory back first.
        //
        // `CAP_CHOWN` permits giving a file to anybody, including back to
        // oneself, so all three steps are inside the one capability
        // [ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)
        // grants. Measured: a `0777` directory owned by the person was narrowed
        // to `0700` and returned, holding `cap_chown` with `cap_fowner`,
        // `cap_dac_override` and `cap_dac_read_search` dropped — and with
        // `cap_chown` dropped as well the first step is refused and the
        // directory stays as it was.
        //
        // The window in which the folder is root's is why the order is this way
        // round rather than mode-then-owner: for those three syscalls the
        // person cannot reach their own directory, which is strictly safer than
        // the alternative window, in which it is theirs and still wide.
        if what.owner.is_some() {
            let whose = fs::metadata(&what.at)
                .map_err(|why| NotMade::TheFolder {
                    at: what.at.clone(),
                    why,
                })?
                .uid();
            let us = rustix::process::geteuid().as_raw();
            if taken_back_first(whose, us) {
                given_to(&what.at, us).map_err(|why| NotMade::NotTheirs {
                    at: what.at.clone(),
                    why,
                })?;
            }
        }

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
