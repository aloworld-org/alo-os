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
//!
//! # A folder the record already names is never renamed
//!
//! The record is read first, and a folder it names is that folder, in
//! whatever language it was made in. When the person's language now calls it
//! something else, that is said in [`Made::named_differently`] — for a log
//! today, and for the screen that will *offer* the rename once one is designed
//! — and nothing on the disk moves. The owner's decision of 2026-10-28: an
//! existing home is never renamed without asking.

use std::path::{Path, PathBuf};

use crate::catalogue::Catalogue;
use crate::folders::Folder;
use crate::naming::Named;
use crate::recording::{AlreadyRecorded, Points, THE_RECORD, the_record, under_home};
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

    /// Read this file, or [`None`] when there is no file there.
    ///
    /// # Errors
    /// What the operating system said, for a file that is there and would not
    /// be read.
    fn read(&self, at: &Path) -> Result<Option<String>, String>;
}

/// A folder the record names, which the person's language now calls
/// something else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedDifferently {
    /// Which folder.
    pub folder: Folder,
    /// Where it is, which is where it stays.
    pub at: PathBuf,
    /// What their language calls it now.
    pub now: String,
}

/// What a person got, and what they did not.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Made {
    /// Each folder that is now on the disk, and where it is.
    folders: Vec<(Folder, PathBuf)>,
    /// Where the record is, when it says what it should.
    recorded: Option<PathBuf>,
    /// Folders kept under a name the person's language no longer uses.
    named_differently: Vec<NamedDifferently>,
    /// Everything that did not work, in the order it was found.
    refusals: Vec<NotMade>,
}

impl Made {
    /// Each folder that is now on the disk, and where it is.
    #[must_use]
    pub fn folders(&self) -> &[(Folder, PathBuf)] {
        &self.folders
    }

    /// Where the record naming the folders is, when it says what it should.
    #[must_use]
    pub fn recorded(&self) -> Option<&Path> {
        self.recorded.as_deref()
    }

    /// **Folders kept under a name the person's language no longer uses** —
    /// what a rename would be offered for, and never done here.
    #[must_use]
    pub fn named_differently(&self) -> &[NamedDifferently] {
        &self.named_differently
    }

    /// Everything that did not work, for whoever looks after the machine.
    #[must_use]
    pub fn refusals(&self) -> &[NotMade] {
        &self.refusals
    }

    /// Whether the person got every folder the record does not say they have
    /// none of, and the record.
    #[must_use]
    pub fn everything_was_made(&self) -> bool {
        self.refusals.is_empty() && self.recorded.is_some()
    }
}

/// **Give this person the folders they expect, under this home folder.**
///
/// A folder the record already names is that folder: made again if it went
/// missing, never renamed, and never doubled. A folder it does not name is
/// made under the name `catalogue` gives it — `xdg-user-dirs`' own, in the
/// person's language — or its English key where there is no catalogue, as
/// `xdg-user-dirs` itself does. The record is then written with the folders
/// that are on the disk, keeping every line it had that is not this crate's,
/// and is not written at all when it already says exactly that.
///
/// `home` is the caller's to supply and is not checked for being a full path:
/// on this repository's own hosts `Path::is_absolute` answers about the host
/// the code was compiled for rather than about the machine, so a check here
/// would be wrong on one of them (`docs/quirks.md`).
#[must_use]
pub fn made_for(home: &Path, catalogue: Option<&Catalogue>, disk: &dyn Disk) -> Made {
    let mut made = Made::default();
    let record = home.join(THE_RECORD);
    let already = match disk.read(&record) {
        Ok(text) => AlreadyRecorded::read(text.as_deref().unwrap_or_default()),
        // **A record that is there and would not be read is not written
        // over**: it is somebody's, and this cannot see what is in it.
        Err(said) => {
            made.refusals.push(NotMade::TheRecordWasNotWritten { said });
            return made;
        }
    };

    let mut ours: Vec<(Folder, String)> = Vec::new();
    for folder in Folder::ALL {
        let in_their_language = Named::of(folder, catalogue);
        match already.where_it_points(folder, home) {
            Some(Points::Nowhere) => {}
            Some(Points::At(at)) => {
                if let Err(said) = disk.make_folder(&at) {
                    made.refusals.push(NotMade::TheDiskWouldNot { said });
                    continue;
                }
                if let Ok(named) = &in_their_language
                    && at.file_name().and_then(|it| it.to_str()) != Some(named.name())
                {
                    made.named_differently.push(NamedDifferently {
                        folder,
                        at: at.clone(),
                        now: named.name().to_owned(),
                    });
                }
                if let Some(value) = already.value_of(folder) {
                    ours.push((folder, value.to_owned()));
                }
                made.folders.push((folder, at));
            }
            None => {
                let named = match in_their_language {
                    Ok(named) => named,
                    Err(why) => {
                        made.refusals.push(why);
                        continue;
                    }
                };
                let at = home.join(named.name());
                if let Err(said) = disk.make_folder(&at) {
                    made.refusals.push(NotMade::TheDiskWouldNot { said });
                    continue;
                }
                ours.push((folder, under_home(named.name())));
                made.folders.push((folder, at));
            }
        }
    }

    // Nothing to record and nothing recorded: a record naming no folder at all
    // would only be a file somebody has to explain later.
    let unchanged = ours
        .iter()
        .all(|(folder, value)| already.value_of(*folder) == Some(value.as_str()));
    if ours.is_empty() && already == AlreadyRecorded::default() {
        return made;
    }
    if unchanged {
        made.recorded = Some(record);
        return made;
    }

    let under = record
        .parent()
        .map_or_else(|| home.to_owned(), Path::to_owned);
    let written = disk
        .make_folder(&under)
        .and_then(|()| disk.write(&record, &the_record(&ours, &already)));
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
    use crate::testing::{ADisk, in_german};

    fn home() -> PathBuf {
        PathBuf::from("/var/home/alo")
    }

    fn the_record_at() -> PathBuf {
        home().join(".config/user-dirs.dirs")
    }

    fn german() -> Catalogue {
        in_german(&[
            (Folder::Documents, "Dokumente"),
            (Folder::Pictures, "Bilder"),
        ])
    }

    /// The ordinary machine with no catalogue: three folders and a record.
    #[test]
    fn a_person_gets_three_folders_and_the_record_naming_them() {
        let disk = ADisk::that_works();
        let made = made_for(&home(), None, &disk);

        assert!(made.everything_was_made(), "{made:?}");
        assert_eq!(made.folders().len(), 3);
        assert_eq!(made.recorded().unwrap(), the_record_at());
        assert!(disk.made(&home().join("Documents")));
        assert!(disk.made(&home().join("Downloads")));
        assert!(disk.made(&home().join("Pictures")));
    }

    /// **A German person gets Dokumente**, and the record says so.
    #[test]
    fn a_german_person_gets_dokumente() {
        let disk = ADisk::that_works();
        let made = made_for(&home(), Some(&german()), &disk);

        assert!(made.everything_was_made(), "{made:?}");
        assert!(disk.made(&home().join("Dokumente")));
        assert!(disk.made(&home().join("Bilder")));
        assert!(!disk.made(&home().join("Documents")));
        let written = disk.written(&the_record_at()).unwrap();
        assert!(
            written.contains("XDG_DOCUMENTS_DIR=\"$HOME/Dokumente\""),
            "{written}"
        );
    }

    /// **An existing home is never renamed without asking.** Made in English,
    /// signed into in German: `Documents` stays, no `Dokumente` appears beside
    /// it, the record is not touched — and what a rename would be is said.
    #[test]
    fn an_existing_home_is_never_renamed_without_asking() {
        let english = ADisk::that_works();
        let _ = made_for(&home(), None, &english);
        let record = english.written(&the_record_at()).unwrap();

        let disk = ADisk::that_works().holding(&the_record_at(), &record);
        let made = made_for(&home(), Some(&german()), &disk);

        assert!(made.everything_was_made(), "{made:?}");
        assert!(disk.made(&home().join("Documents")));
        assert!(
            !disk.made(&home().join("Dokumente")),
            "a second folder was made"
        );
        assert_eq!(
            disk.written(&the_record_at()),
            None,
            "the record was rewritten"
        );
        let differently: Vec<(Folder, &str)> = made
            .named_differently()
            .iter()
            .map(|it| (it.folder, it.now.as_str()))
            .collect();
        assert_eq!(
            differently,
            [
                (Folder::Documents, "Dokumente"),
                (Folder::Pictures, "Bilder")
            ]
        );
    }

    /// **A folder the record says the person has none of stays absent**, and
    /// the lines it does not own are kept.
    #[test]
    fn a_folder_the_record_says_they_have_none_of_stays_absent() {
        let disk = ADisk::that_works().holding(
            &the_record_at(),
            "XDG_PICTURES_DIR=\"$HOME\"\nXDG_MUSIC_DIR=\"$HOME/Musik\"\n",
        );
        let made = made_for(&home(), Some(&german()), &disk);

        assert!(!disk.made(&home().join("Bilder")));
        assert!(!disk.made(&home().join("Pictures")));
        assert!(disk.made(&home().join("Dokumente")));
        let written = disk.written(&the_record_at()).unwrap();
        assert!(
            written.contains("XDG_MUSIC_DIR=\"$HOME/Musik\""),
            "{written}"
        );
        assert!(!written.contains("XDG_PICTURES_DIR"), "{written}");
        assert!(made.everything_was_made(), "{made:?}");
    }

    /// **A record that would not be read is not written over.**
    #[test]
    fn a_record_that_would_not_be_read_is_left_alone() {
        let disk = ADisk::that_works().unreadable(&the_record_at());
        let made = made_for(&home(), Some(&german()), &disk);
        assert!(made.folders().is_empty(), "{made:?}");
        assert_eq!(disk.written(&the_record_at()), None);
        assert!(matches!(
            made.refusals().first(),
            Some(NotMade::TheRecordWasNotWritten { .. })
        ));
    }

    /// One bad name costs that folder and nothing else, and is not recorded.
    #[test]
    fn a_name_that_cannot_be_a_folder_does_not_cost_the_other_two() {
        let disk = ADisk::that_works();
        let made = made_for(
            &home(),
            Some(&in_german(&[(Folder::Documents, ".hidden")])),
            &disk,
        );

        assert_eq!(made.folders().len(), 2, "{made:?}");
        assert_eq!(made.refusals().len(), 1);
        let written = disk.written(&the_record_at()).unwrap();
        assert!(!written.contains("XDG_DOCUMENTS_DIR"), "{written}");
        assert!(written.contains("XDG_PICTURES_DIR"), "{written}");
    }

    /// A disk that refuses one folder still gives the person the others.
    #[test]
    fn a_disk_that_refuses_one_folder_does_not_cost_the_other_two() {
        let disk = ADisk::that_works().refusing(&home().join("Downloads"));
        let made = made_for(&home(), None, &disk);

        assert_eq!(made.folders().len(), 2, "{made:?}");
        assert!(matches!(
            made.refusals().first(),
            Some(NotMade::TheDiskWouldNot { .. })
        ));
    }

    /// The folders survive a record that could not be written.
    #[test]
    fn a_record_that_was_not_written_leaves_the_folders_made() {
        let disk = ADisk::that_works().refusing_to_write();
        let made = made_for(&home(), None, &disk);

        assert_eq!(made.folders().len(), 3, "{made:?}");
        assert!(made.recorded().is_none());
        assert!(!made.everything_was_made());
    }

    /// Nothing made means nothing recorded.
    #[test]
    fn a_machine_that_could_make_nothing_writes_no_record() {
        let disk = ADisk::that_refuses_everything();
        let made = made_for(&home(), None, &disk);

        assert!(made.folders().is_empty(), "{made:?}");
        assert!(made.recorded().is_none());
        assert_eq!(made.refusals().len(), Folder::ALL.len());
        assert!(disk.written(&the_record_at()).is_none());
    }
}
