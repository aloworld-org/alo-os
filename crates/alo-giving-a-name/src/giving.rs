//! Reading one person's name from the account store and giving it to them.
//!
//! # Written beside, given away, then put in place
//!
//! The file is replaced whole, never edited where it is:
//!
//! 1. a sibling, `.<number>.new`, is created fresh by this process at `0600`;
//! 2. the name is written into it and synced;
//! 3. it is given to the person — the one step that needs `CAP_CHOWN`;
//! 4. it is renamed over `<number>`.
//!
//! So a desktop reading at any moment reads the old name or the new one, and
//! never half of either. **And the second sign-in needs no more than the
//! first**: renaming over a file in a folder this process owns needs no
//! ownership of the file being replaced, so a name the person already owns is
//! replaced without taking it back — where `alo-giving-a-folder` has to take its
//! directory back first because it changes a mode in place.
//!
//! The order of 3 and 4 is the safe one: the file reaches its name already the
//! person's, so there is no moment in which `<number>` is root's and the
//! person's desktop cannot read it.

use std::{
    fs, io,
    io::Write as _,
    os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _},
    path::{Path, PathBuf},
};

use alo_accounts::{A_NAMES_MODE, THE_NAMES_MODE, as_written, where_their_name_is};

use crate::failing::NotGiven;

/// Give account `person` their own name, read from the store at `accounts`,
/// in a file under `under`.
///
/// `under` is [`alo_accounts::THE_NAMES`] and `accounts` is
/// [`alo_accounts::THE_ACCOUNTS`] on a machine, and two temporary paths in a
/// test. Returns where the name now is.
///
/// **Started at every sign-in, so a changed name is picked up at the next
/// one**, and giving the same name twice is not a failure: it writes the same
/// line again.
///
/// # Errors
///
/// [`NotGiven`], which is always the machine being unable — the store would not
/// read, it has nobody with this number, or the file could not be written or
/// given away.
pub fn given_to(under: &Path, accounts: &Path, person: u32) -> Result<PathBuf, NotGiven> {
    if !under.is_absolute() {
        return Err(NotGiven::NotAnAbsoluteFolder(under.to_path_buf()));
    }
    let store = alo_accounts::found(accounts).map_err(NotGiven::TheAccounts)?;
    let name = store
        .name_of(person)
        .ok_or(NotGiven::NobodyNumbered(person))?;

    // The folder is root's and passable by everybody. systemd's
    // `StateDirectory=` makes it on a machine; it is made here as well so that
    // the mode is this crate's statement and not only the unit's.
    let at_the_folder = |why| NotGiven::TheFile {
        at: under.to_path_buf(),
        why,
    };
    fs::DirBuilder::new()
        .recursive(true)
        .mode(THE_NAMES_MODE)
        .create(under)
        .map_err(at_the_folder)?;
    fs::set_permissions(under, fs::Permissions::from_mode(THE_NAMES_MODE))
        .map_err(at_the_folder)?;

    let theirs = where_their_name_is(under, person);
    let beside = under.join(format!(".{person}.new"));
    let at_the_sibling = |why| NotGiven::TheFile {
        at: beside.clone(),
        why,
    };

    // A sibling left by a run that died is removed, so `create_new` below can
    // insist the one being written is this run's own.
    if let Err(why) = fs::remove_file(&beside)
        && why.kind() != io::ErrorKind::NotFound
    {
        return Err(at_the_sibling(why));
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(A_NAMES_MODE)
        .open(&beside)
        .map_err(at_the_sibling)?;
    // The mode is set explicitly as well, because `mode` above is masked by
    // the process `umask`.
    file.set_permissions(fs::Permissions::from_mode(A_NAMES_MODE))
        .and_then(|()| file.write_all(as_written(name).as_bytes()))
        .and_then(|()| file.sync_all())
        .map_err(at_the_sibling)?;
    drop(file);

    given(&beside, person).map_err(|why| NotGiven::NotTheirs {
        at: beside.clone(),
        why,
    })?;

    fs::rename(&beside, &theirs).map_err(|why| NotGiven::TheFile {
        at: theirs.clone(),
        why,
    })?;
    Ok(theirs)
}

/// Give `at` to account `owner`, and leave its group as it was.
///
/// **One of the two places in this repository that change a file's owner**,
/// and `crates/alo-giving-a-folder/tests/only_here_may_give_a_directory_away.rs`
/// holds both to that. The group is left alone for `alo-giving-a-folder`'s
/// reason: at `0600` the group bits grant nothing, and setting one would mean
/// assuming an account's group number equals its account number, which a first
/// version of that crate assumed and was refused for.
fn given(at: &Path, owner: u32) -> io::Result<()> {
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
