//! The one implementation that reads a real disk.
//!
//! Everything the picker decides is decided in [`crate::browsing`] against the
//! [`Folders`] port; this file is the whole of what touches a filesystem, and
//! it is deliberately small enough to read in one sitting.
//!
//! # A link is not a folder here
//!
//! A symbolic link to a folder is **not shown and cannot be picked**, and that
//! is the decision in this file worth arguing for. A grant is over a place, and
//! `alo-capability` decides reach lexically while `alo-files` resolves a path
//! before anything opens it — so a grant made over a link would name one place
//! and cover another, and the folder a person believed they were granting is
//! not the folder the agent would reach. Worse, the link can be repointed
//! afterwards by anything that can write the folder it sits in, which would
//! turn a grant somebody made on Monday into a grant over somewhere else on
//! Tuesday.
//!
//! So `std::fs::DirEntry::file_type` is asked, which does **not** follow a
//! link, and only a real directory becomes a row. A person who wants to grant
//! the folder a link points at picks the folder itself, which is the one act
//! that means what it says.
//!
//! # And what it does not do
//!
//! It does not hide anything else. A folder whose name begins with a dot is a
//! folder a person may want to grant, and deciding that a row is too technical
//! to show is a rendering decision belonging to whatever draws the picker —
//! not to the file that decides what is *there*.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::folders::{Folders, Inside, NotShown};

/// What is really on this machine's disk.
///
/// A unit struct: it holds nothing, opens nothing until it is asked, and
/// remembers nothing between questions. Two questions about one folder are two
/// answers about that folder as it was at the moment each was asked, which is
/// what a person navigating a live filesystem needs — a cache here would show
/// somebody a folder that had been deleted while they looked at it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnThisDisk;

impl Folders for OnThisDisk {
    fn inside(&self, folder: &Path) -> Result<Inside, NotShown> {
        // Asked without following a link, for this module's reason: a link to
        // a folder is not a folder the picker will stand in.
        let what = fs::symlink_metadata(folder).map_err(NotShown::from)?;
        if !what.is_dir() {
            return Err(NotShown::NotAFolder);
        }

        let mut names = Vec::new();
        let mut could_not_be_named = 0_usize;
        for entry in fs::read_dir(folder).map_err(NotShown::from)? {
            // An entry the machine will not describe is counted rather than
            // dropped: something is there, and a listing that pretended
            // otherwise would be a listing that lies about the folder.
            let Ok(entry) = entry else {
                could_not_be_named = could_not_be_named.saturating_add(1);
                continue;
            };
            let Ok(kind) = entry.file_type() else {
                could_not_be_named = could_not_be_named.saturating_add(1);
                continue;
            };
            if !kind.is_dir() {
                continue;
            }
            match entry.file_name().into_string() {
                Ok(name) => names.push(name),
                Err(_) => could_not_be_named = could_not_be_named.saturating_add(1),
            }
        }
        Ok(Inside::these(names).and_could_not_be_named(could_not_be_named))
    }
}

/// What the machine said, as one of the three things a person is told.
///
/// Anything that is not *it is not there* is *this machine would not open it*.
/// Guessing at the rest would mean showing somebody a sentence about
/// permissions for a disk that had been unplugged, and the two actions a
/// person takes are the same either way: pick something they can open, or ask
/// whoever looks after the machine.
fn why(failed: &std::io::Error) -> NotShown {
    match failed.kind() {
        ErrorKind::NotFound => NotShown::WentAway,
        _ => NotShown::WouldNotBeRead,
    }
}

impl From<std::io::Error> for NotShown {
    fn from(failed: std::io::Error) -> Self {
        why(&failed)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A folder of this test's own, gone before it starts and made fresh.
    fn a_folder_for(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-picking-{what}-{}", std::process::id()));
        let _gone = fs::remove_dir_all(&at);
        fs::create_dir_all(&at).unwrap();
        at
    }

    /// A folder holding folders and a file lists the folders, and only those:
    /// a file cannot be granted here, so showing one would be offering
    /// somebody a row that refuses them.
    #[test]
    fn folders_are_listed_and_files_are_not() {
        let at = a_folder_for("lists");
        fs::create_dir(at.join("Invoices")).unwrap();
        fs::create_dir(at.join("Photos")).unwrap();
        fs::write(at.join("notes.txt"), "hello").unwrap();

        let inside = OnThisDisk.inside(&at).unwrap();
        assert_eq!(inside.names(), ["Invoices", "Photos"]);
        assert_eq!(inside.could_not_be_named(), 0);
        assert!(!inside.there_are_more());

        let _gone = fs::remove_dir_all(&at);
    }

    /// **A file is not a folder to stand in**, which is the refusal a person
    /// meets when what they opened turns out not to be one.
    #[test]
    fn a_file_is_not_a_folder() {
        let at = a_folder_for("not-a-folder");
        let file = at.join("notes.txt");
        fs::write(&file, "hello").unwrap();

        assert_eq!(OnThisDisk.inside(&file), Err(NotShown::NotAFolder));

        let _gone = fs::remove_dir_all(&at);
    }

    /// **A folder that is not there is refused as gone**, rather than as
    /// something the machine would not open: the two send a person to
    /// different actions.
    #[test]
    fn a_folder_that_is_not_there_says_so() {
        let at = a_folder_for("went-away");
        let never = at.join("never-existed");
        assert_eq!(OnThisDisk.inside(&never), Err(NotShown::WentAway));

        let _gone = fs::remove_dir_all(&at);
    }

    /// **A link to a folder is not shown**, whatever it points at — the
    /// decision this module exists to make. Run only where a link can be made
    /// without a privilege the test may not have, which on Windows is the
    /// ordinary case rather than a rare one.
    #[cfg(unix)]
    #[test]
    fn a_link_to_a_folder_is_not_a_folder_a_person_can_pick() {
        let at = a_folder_for("links");
        fs::create_dir(at.join("Invoices")).unwrap();
        std::os::unix::fs::symlink(at.join("Invoices"), at.join("Shortcut")).unwrap();

        let inside = OnThisDisk.inside(&at).unwrap();
        assert_eq!(
            inside.names(),
            ["Invoices"],
            "a link was offered as a folder"
        );

        // And it cannot be stood in either, so nothing reaches it the long way
        // round.
        assert_eq!(
            OnThisDisk.inside(&at.join("Shortcut")),
            Err(NotShown::NotAFolder)
        );

        let _gone = fs::remove_dir_all(&at);
    }

    /// A folder whose name begins with a dot is a folder somebody may want to
    /// grant, and this file does not decide that it is too technical to show.
    #[test]
    fn nothing_is_hidden_from_the_person_looking() {
        let at = a_folder_for("hidden");
        fs::create_dir(at.join(".config")).unwrap();

        assert_eq!(OnThisDisk.inside(&at).unwrap().names(), [".config"]);

        let _gone = fs::remove_dir_all(&at);
    }

    /// Every way the machine can refuse reaches one of the three sentences,
    /// and nothing that is not *gone* is guessed at.
    #[test]
    fn what_the_machine_said_becomes_one_of_the_three() {
        assert_eq!(
            NotShown::from(std::io::Error::from(ErrorKind::NotFound)),
            NotShown::WentAway
        );
        assert_eq!(
            NotShown::from(std::io::Error::from(ErrorKind::PermissionDenied)),
            NotShown::WouldNotBeRead
        );
        assert_eq!(
            NotShown::from(std::io::Error::from(ErrorKind::Other)),
            NotShown::WouldNotBeRead
        );
    }
}
