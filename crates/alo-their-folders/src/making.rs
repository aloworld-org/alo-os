//! Making a person's folders, and saying what could not be made.
//!
//! # It makes what it can and never gives up part way
//!
//! A machine that would not make a Pictures folder is still a machine somebody
//! has to be able to sign in to. So this returns [`Made`] — what was made and
//! what was not — rather than an error in place of the work: stopping at the
//! first refusal would mean a bad translation of one word costs a person all
//! three folders, and a full disk costs them their sign-in.
//!
//! That is a deliberate departure from how the rest of this repository treats
//! a failure, and the reason is which way the mistake falls. Everywhere else,
//! refusing the whole operation leaves the machine as it was. Here the
//! operation **is** the setting-up: there is no state to protect and nothing
//! to roll back, and a person with two of their three folders is strictly
//! better off than a person with none.
//!
//! # The disk is a trait, so every rule is tested without one
//!
//! `alo-picking` keeps the same division and for the same reason: the rules
//! about what is made, in what order, and what a refusal does are decided
//! here, and [`Disk`] is the only thing that touches a filesystem. The tests
//! below walk every path — a name that cannot be a folder, a disk that refuses
//! one folder, a disk that refuses the record — on every platform the suite
//! runs on, with no privilege and no temporary directory.

use std::path::{Path, PathBuf};

use alo_strings::Strings;

use crate::folders::Folder;
use crate::naming::Named;
use crate::recording::{THE_RECORD, the_record};
use crate::refusing::NotMade;

/// The one thing in this crate that touches a filesystem.
pub trait Disk {
    /// Make this directory, and every directory above it that is missing.
    ///
    /// A directory that is already there is **not** a failure: a person who
    /// made their own Documents folder before alo OS got to it keeps it, and
    /// keeps what is in it.
    ///
    /// # Errors
    /// What the operating system said, for a log. It is in the machine's
    /// language and is never shown to a person.
    fn make_folder(&self, at: &Path) -> Result<(), String>;

    /// Write this file whole, replacing whatever was there.
    ///
    /// # Errors
    /// What the operating system said, for a log.
    fn write(&self, at: &Path, contents: &str) -> Result<(), String>;
}

/// What a person got, and what they did not.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Made {
    /// Each folder that is now on the disk, and where it is.
    folders: Vec<(Folder, PathBuf)>,
    /// Where the record was written, when it was.
    recorded: Option<PathBuf>,
    /// Everything that did not work, in the order it was found.
    refusals: Vec<NotMade>,
}

impl Made {
    /// Each folder that is now on the disk, and where it is.
    #[must_use]
    pub fn folders(&self) -> &[(Folder, PathBuf)] {
        &self.folders
    }

    /// Where the record naming the folders was written, when it was.
    #[must_use]
    pub fn recorded(&self) -> Option<&Path> {
        self.recorded.as_deref()
    }

    /// Everything that did not work, for whoever looks after the machine.
    #[must_use]
    pub fn refusals(&self) -> &[NotMade] {
        &self.refusals
    }

    /// Whether the person got all of their folders and the record.
    #[must_use]
    pub fn everything_was_made(&self) -> bool {
        self.refusals.is_empty()
            && self.recorded.is_some()
            && self.folders.len() == Folder::ALL.len()
    }
}

/// Give this person the folders they expect, under this home folder.
///
/// The folders are named in the language `strings` reads, made in the order
/// [`Folder::ALL`] gives, and then recorded so that other software can find
/// them. The record names only the folders that were actually made.
///
/// `home` is the caller's to supply and is not checked for being a full path:
/// on this repository's own hosts `Path::is_absolute` answers about the host
/// the code was compiled for rather than about the machine, so a check here
/// would be wrong on one of them (`docs/quirks.md`).
#[must_use]
pub fn made_for(home: &Path, strings: &Strings, disk: &dyn Disk) -> Made {
    let mut made = Made::default();
    let mut named = Vec::new();

    for folder in Folder::ALL {
        let name = match Named::of(folder, strings) {
            Ok(name) => name,
            Err(why) => {
                made.refusals.push(why);
                continue;
            }
        };
        let at = home.join(name.name());
        if let Err(said) = disk.make_folder(&at) {
            made.refusals.push(NotMade::TheDiskWouldNot { said });
            continue;
        }
        made.folders.push((folder, at));
        named.push(name);
    }

    // Nothing was made, so there is nothing to record — and a record naming no
    // folder at all would only be a file somebody has to explain later.
    if named.is_empty() {
        return made;
    }

    let record = home.join(THE_RECORD);
    let under = record
        .parent()
        .map_or_else(|| home.to_owned(), Path::to_owned);
    let written = disk
        .make_folder(&under)
        .and_then(|()| disk.write(&record, &the_record(&named)));
    match written {
        Ok(()) => made.recorded = Some(record),
        Err(said) => made.refusals.push(NotMade::TheRecordWasNotWritten { said }),
    }
    made
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{ADisk, in_english, translated};
    use crate::words::{DOCUMENTS, PICTURES};

    fn home() -> PathBuf {
        PathBuf::from("/var/home/alo")
    }

    /// The ordinary machine: three folders and a record.
    #[test]
    fn a_person_gets_three_folders_and_the_record_naming_them() {
        let disk = ADisk::that_works();
        let made = made_for(&home(), &in_english(), &disk);

        assert!(made.everything_was_made(), "{made:?}");
        assert_eq!(made.folders().len(), 3);
        assert_eq!(
            made.recorded().unwrap(),
            home().join(".config/user-dirs.dirs")
        );
        assert!(disk.made(&home().join("Documents")));
        assert!(disk.made(&home().join("Downloads")));
        assert!(disk.made(&home().join("Pictures")));
    }

    /// The whole reason the crate exists: a German machine, German folders.
    #[test]
    fn the_folders_are_made_in_the_language_the_machine_was_set_up_in() {
        let disk = ADisk::that_works();
        let strings = translated(&[(DOCUMENTS, "Dokumente"), (PICTURES, "Bilder")]);
        let made = made_for(&home(), &strings, &disk);

        assert!(made.everything_was_made(), "{made:?}");
        assert!(disk.made(&home().join("Dokumente")));
        assert!(disk.made(&home().join("Bilder")));
        assert!(!disk.made(&home().join("Documents")));
    }

    /// One bad translation costs that folder and nothing else.
    #[test]
    fn a_name_that_cannot_be_a_folder_does_not_cost_the_other_two() {
        let disk = ADisk::that_works();
        let made = made_for(&home(), &translated(&[(DOCUMENTS, ".hidden")]), &disk);

        assert_eq!(made.folders().len(), 2, "{made:?}");
        assert_eq!(made.refusals().len(), 1);
        assert!(made.recorded().is_some());
        assert!(!made.everything_was_made());
    }

    /// And the folder that was refused is not in the record, because it is
    /// not on the disk.
    #[test]
    fn a_folder_that_was_not_made_is_not_named_in_the_record() {
        let disk = ADisk::that_works();
        let _made = made_for(&home(), &translated(&[(DOCUMENTS, ".hidden")]), &disk);
        let written = disk
            .written(&home().join(".config/user-dirs.dirs"))
            .unwrap();
        assert!(!written.contains("XDG_DOCUMENTS_DIR"), "{written}");
        assert!(written.contains("XDG_PICTURES_DIR"), "{written}");
    }

    /// A disk that refuses one folder still gives the person the others.
    #[test]
    fn a_disk_that_refuses_one_folder_does_not_cost_the_other_two() {
        let disk = ADisk::that_works().refusing(&home().join("Downloads"));
        let made = made_for(&home(), &in_english(), &disk);

        assert_eq!(made.folders().len(), 2, "{made:?}");
        assert_eq!(made.refusals().len(), 1);
        assert!(matches!(
            made.refusals().first(),
            Some(NotMade::TheDiskWouldNot { .. })
        ));
    }

    /// The folders survive a record that could not be written, and the
    /// refusal says so rather than claiming the folders failed.
    #[test]
    fn a_record_that_was_not_written_leaves_the_folders_made() {
        let disk = ADisk::that_works().refusing_to_write();
        let made = made_for(&home(), &in_english(), &disk);

        assert_eq!(made.folders().len(), 3, "{made:?}");
        assert!(made.recorded().is_none());
        assert!(matches!(
            made.refusals().first(),
            Some(NotMade::TheRecordWasNotWritten { .. })
        ));
        assert!(!made.everything_was_made());
    }

    /// Nothing made means nothing recorded: a record naming no folder is a
    /// file somebody would later have to explain.
    #[test]
    fn a_machine_that_could_make_nothing_writes_no_record() {
        let disk = ADisk::that_refuses_everything();
        let made = made_for(&home(), &in_english(), &disk);

        assert!(made.folders().is_empty(), "{made:?}");
        assert!(made.recorded().is_none());
        assert_eq!(made.refusals().len(), Folder::ALL.len());
        assert!(
            disk.written(&home().join(".config/user-dirs.dirs"))
                .is_none()
        );
    }
}
