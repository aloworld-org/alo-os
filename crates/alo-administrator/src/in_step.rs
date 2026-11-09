//! **Whether the system's list holds the person's password**, written by the
//! keeper after every sign-in and read by the desktop's password sheet.
//!
//! The sheet needs this for one refusal. polkit's helper answers a password
//! with *SUCCESS* or *FAILURE* and nothing else, so on a machine whose keeper
//! could not write the password at sign-in the right password would read as
//! *not right* — a refusal that is false. With this file the sheet can say
//! what is true instead: the password was not given to the system when they
//! signed in, so it cannot be checked here.
//!
//! One file per person, `/run/alo-the-system-password/<number>`, mode `0644`
//! in a folder `tmpfiles.d` makes for root: it holds a word, never a password,
//! and `/run` is gone at the next start, which is when the next sign-in writes
//! it again.

use std::path::{Path, PathBuf};

use crate::keeping::NotKept;

/// The folder the keeper writes in.
pub const THE_FOLDER: &str = "/run/alo-the-system-password";

/// What the system's list holds, as far as this boot knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InStep {
    /// The keeper wrote the password the person signed in with.
    Kept,
    /// The keeper was asked and did not write it.
    NotKept(NotKept),
    /// Nothing was written this boot, or what was written would not read.
    NotKnown,
}

/// The file the keeper writes for `person`, under `folder`.
#[must_use]
pub fn where_for(folder: &Path, person: u32) -> PathBuf {
    folder.join(person.to_string())
}

/// The text the keeper writes: the same line it answers the screen with.
#[must_use]
pub fn as_written(kept: Result<(), NotKept>) -> String {
    crate::keeping::answer_of(kept)
}

/// What the file says.
#[must_use]
pub fn as_read(text: &str) -> InStep {
    match crate::keeping::answered(text) {
        Ok(()) => InStep::Kept,
        Err(NotKept::NoAnswer) => InStep::NotKnown,
        Err(why) => InStep::NotKept(why),
    }
}

/// What the file under `folder` says for `person`, now.
#[must_use]
pub fn now(folder: &Path, person: u32) -> InStep {
    std::fs::read_to_string(where_for(folder, person))
        .map_or(InStep::NotKnown, |text| as_read(&text))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **What the keeper writes reads back as what happened**, and a file
    /// nobody wrote is not known rather than kept.
    #[test]
    fn what_is_written_reads_back() {
        assert_eq!(as_read(&as_written(Ok(()))), InStep::Kept);
        assert_eq!(
            as_read(&as_written(Err(NotKept::Refused))),
            InStep::NotKept(NotKept::Refused)
        );
        assert_eq!(as_read("something else"), InStep::NotKnown);
        let folder = tempfile::tempdir().unwrap();
        assert_eq!(now(folder.path(), 1000), InStep::NotKnown);
        std::fs::write(where_for(folder.path(), 1000), as_written(Ok(()))).unwrap();
        assert_eq!(now(folder.path(), 1000), InStep::Kept);
        assert_eq!(now(folder.path(), 1001), InStep::NotKnown);
    }
}
