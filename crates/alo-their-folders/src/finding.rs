//! **Where a person's folders are**, read back from the record
//! [`crate::made_for`] wrote — so that something asked on their behalf can name
//! the folder that is really there.
//!
//! Measured on a booted machine on 2026-11-10: asked for a folder *in my
//! Documents*, the agent's model named `/Users/username/Documents`, because
//! nothing had told it where a person's documents are on this computer. The
//! record answers exactly that, in the names the person's language gave the
//! folders (`Dokumente`, `Bilder`), and it is the record every other program on
//! the machine asks. So this reads the same record rather than guessing a
//! second time.
//!
//! **Only what the record names, and only folders that are there.** A folder
//! the record says the person has none of, or names somewhere that is not a
//! folder, is left out rather than replaced by the English default: a path
//! told to somebody as *your documents* that does not exist is worse than none.

use std::path::{Path, PathBuf};

use crate::folders::Folder;
use crate::making::Disk;
use crate::recording::{AlreadyRecorded, Points, THE_RECORD};

/// **Each of the person's folders that the record under `home` names**, in
/// [`Folder::ALL`]'s order, where it is.
///
/// A record that is not there, or will not be read, names nothing, and the
/// answer is empty: where the folders are is a hint, and no hint is the honest
/// answer for a home nobody recorded. `is_a_folder` is asked of each, so a
/// record naming a folder since deleted does not send anybody to it.
#[must_use]
pub fn where_they_are(
    home: &Path,
    disk: &dyn Disk,
    is_a_folder: &dyn Fn(&Path) -> bool,
) -> Vec<(Folder, PathBuf)> {
    let Ok(Some(text)) = disk.read(&home.join(THE_RECORD)) else {
        return Vec::new();
    };
    let recorded = AlreadyRecorded::read(&text);
    Folder::ALL
        .into_iter()
        .filter_map(|folder| match recorded.where_it_points(folder, home)? {
            Points::At(at) => is_a_folder(&at).then_some((folder, at)),
            Points::Nowhere => None,
        })
        .collect()
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
        PathBuf::from("/var/home/anna")
    }

    /// Every path is a folder, for the tests about what the record says.
    fn every_path_is_a_folder(_: &Path) -> bool {
        true
    }

    /// **A German person's documents are where their record says**, under the
    /// name it gives — never the English default.
    #[test]
    fn the_folders_are_where_the_record_says_in_their_own_names() {
        let disk = ADisk::that_works().holding(
            &home().join(THE_RECORD),
            "XDG_DOCUMENTS_DIR=\"$HOME/Dokumente\"\nXDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n\
             XDG_PICTURES_DIR=\"$HOME\"\nXDG_MUSIC_DIR=\"$HOME/Musik\"\n",
        );
        assert_eq!(
            where_they_are(&home(), &disk, &every_path_is_a_folder),
            [
                (Folder::Documents, home().join("Dokumente")),
                (Folder::Downloads, home().join("Downloads")),
            ],
            "Pictures is recorded as none, and Music is not one of these"
        );
    }

    /// **No record, or one that will not read, names nothing** — and a folder
    /// that is not there is left out.
    #[test]
    fn no_record_names_nothing_and_a_missing_folder_is_left_out() {
        assert!(where_they_are(&home(), &ADisk::that_works(), &every_path_is_a_folder).is_empty());
        let unreadable = ADisk::that_works().unreadable(&home().join(THE_RECORD));
        assert!(where_they_are(&home(), &unreadable, &every_path_is_a_folder).is_empty());
        let disk = ADisk::that_works().holding(
            &home().join(THE_RECORD),
            "XDG_DOCUMENTS_DIR=\"$HOME/Dokumente\"\n",
        );
        assert!(where_they_are(&home(), &disk, &|_: &Path| false).is_empty());
    }

    /// **On a real disk, what [`crate::made_for`] made is what this finds**, and
    /// every path it answers is a folder that exists — so a path told to a
    /// model from here is one that resolves.
    #[test]
    fn on_a_real_disk_every_folder_it_finds_is_there() {
        let home = std::env::temp_dir().join(format!(
            "alo-their-folders-finding-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&home).unwrap();
        let german = in_german(&[(Folder::Documents, "Dokumente")]);
        let made = crate::made_for(&home, Some(&german), &crate::OnThisDisk);
        assert!(made.everything_was_made(), "{made:?}");

        let found = where_they_are(&home, &crate::OnThisDisk, &Path::is_dir);
        let _ = std::fs::remove_dir_all(&home);
        assert_eq!(found.len(), 3, "{found:?}");
        assert_eq!(found.first().unwrap(), &(Folder::Documents, home.join("Dokumente")));
    }
}
