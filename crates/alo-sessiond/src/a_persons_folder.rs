//! Where a person's folder is, read from the machine's own user database.
//!
//! # Why not from the accounts file
//!
//! [`crate::accounts`] is one question wide on purpose, and says so: *not who
//! they are, **not what they are called**, and emphatically not whether a
//! password is right.* A privileged process that knows more than it needs is a
//! privileged process with more to leak, and widening that trait to answer
//! *what is this person called* to spare this file would have spent a security
//! boundary to save forty lines.
//!
//! `/etc/passwd` is the machine's own answer to *where does account 1000 keep
//! its folder*, it is world-readable by design, it holds no secret, and it is
//! what `pam_systemd` and `logind` themselves consult. So this asks the same
//! place they do.
//!
//! # One pure function, and a reader around it
//!
//! [`the_folder_in`] takes the file's **text**, so every case this has to get
//! right — a missing account, a short line, a comment, a folder that is not a
//! path — is decided in the gate on any host. [`the_folder_of`] is the two
//! lines that read the file.

use std::path::{Path, PathBuf};

/// The machine's own user database.
pub const THE_USER_DATABASE: &str = "/etc/passwd";

/// Which field of a `passwd` line holds the account number.
const THE_NUMBER_IS_FIELD: usize = 2;

/// Which field holds the folder.
const THE_FOLDER_IS_FIELD: usize = 5;

/// How many fields a `passwd` line has.
///
/// Checked rather than assumed: a short line is a file somebody has edited by
/// hand, and reading field five of a four-field line would be reading whatever
/// happened to be there.
const A_WHOLE_LINE_HAS: usize = 7;

/// Find this person's folder in the text of a `passwd` file.
///
/// [`None`] when no line names that account number, when the line is too short
/// to have a folder, or when the folder is empty — each of which is a machine
/// that cannot say where this person keeps their things, which is different
/// from a machine that says they keep them at the root.
#[must_use]
pub fn the_folder_in(passwd: &str, person: u32) -> Option<PathBuf> {
    for line in passwd.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < A_WHOLE_LINE_HAS {
            continue;
        }
        let Some(number) = fields.get(THE_NUMBER_IS_FIELD) else {
            continue;
        };
        if number.parse::<u32>() != Ok(person) {
            continue;
        }
        let folder = fields.get(THE_FOLDER_IS_FIELD)?;
        if folder.is_empty() {
            return None;
        }
        return Some(PathBuf::from(*folder));
    }
    None
}

/// Read this machine's user database and find this person's folder.
///
/// `at` is [`THE_USER_DATABASE`] on a machine and a path a test writes in a
/// test, for the reason every other path in this crate is a parameter.
///
/// [`None`] when the file will not read as well as when it does not name them:
/// a desktop is not started either way, and the caller says which in its own
/// words.
#[must_use]
pub fn the_folder_of(at: &Path, person: u32) -> Option<PathBuf> {
    let passwd = std::fs::read_to_string(at).ok()?;
    the_folder_in(&passwd, person)
}

#[cfg(test)]
#[path = "a_persons_folder_tests.rs"]
mod tests;
