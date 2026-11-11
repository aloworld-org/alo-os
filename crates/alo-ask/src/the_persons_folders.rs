//! **Where this person's home and standard folders are**, read as they press
//! Enter and handed to the agent with what they typed.
//!
//! The walk of `f028d5ec` typed *alo make a folder called demo in my
//! Documents* and the model ran `mkdir /Users/username/Documents/demo`, because
//! nothing told it that this person's home is `/var/home/alo`. This reads the
//! home from the session and the folders from the record `alo-their-folders`
//! wrote when the account was made, **at the moment of asking and for that
//! request only** — the machine's layout, never what is in a folder
//! (`alo_instructing::ThePersonsFolders` has the whole account).

use std::path::{Path, PathBuf};

use alo_instructing::ThePersonsFolders;

/// **This person's folders, now**: their home as the session names it, and
/// what its record says. [`None`] for a session with no absolute home, which
/// sends the marker alone and tells the model nothing.
#[must_use]
pub fn now() -> Option<ThePersonsFolders> {
    let home = std::env::var_os(alo_choosing::HOME).map(PathBuf::from)?;
    of_this_home(&home)
}

/// The folders of the home at `home`, read off this disk.
#[must_use]
pub fn of_this_home(home: &Path) -> Option<ThePersonsFolders> {
    let found =
        alo_their_folders::where_they_are(home, &alo_their_folders::OnThisDisk, &Path::is_dir);
    ThePersonsFolders::of(
        home,
        found
            .iter()
            .map(|(folder, at)| (folder.xdg_name(), at.as_path())),
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A path the model is told is a folder that is there**: the account's
    /// folders are made as a real account's are, and every path the model
    /// would be shown — the home and each folder — resolves on the disk.
    #[test]
    fn every_path_the_model_is_told_resolves() {
        let home =
            std::env::temp_dir().join(format!("alo-ask-their-folders-{}", std::process::id()));
        std::fs::create_dir_all(&home).expect("a home for this test");
        let made = alo_their_folders::made_for(&home, None, &alo_their_folders::OnThisDisk);
        let told = of_this_home(&home);
        let resolved: Vec<bool> = told
            .as_ref()
            .map(|told| {
                told.folders()
                    .iter()
                    .map(|(_, at)| Path::new(at).is_dir())
                    .collect()
            })
            .unwrap_or_default();
        let _ = std::fs::remove_dir_all(&home);

        assert!(made.everything_was_made(), "{made:?}");
        let told = told.expect("an absolute home is told");
        assert_eq!(
            told.home(),
            home.to_str().expect("a home with a printable name")
        );
        assert_eq!(resolved, [true, true, true], "{told:?}");
        let documents = home.join("Documents");
        assert!(
            told.told().contains(&format!(
                "Documents: {}",
                documents.to_str().expect("printable")
            )),
            "{}",
            told.told()
        );
    }

    /// **A home with no record is still told**, with no folders: the home is
    /// a fact the session gives, and the folders are only ever the record's.
    #[test]
    fn a_home_with_no_record_is_told_with_no_folders() {
        let home =
            std::env::temp_dir().join(format!("alo-ask-their-folders-none-{}", std::process::id()));
        std::fs::create_dir_all(&home).expect("a home for this test");
        let told = of_this_home(&home);
        let _ = std::fs::remove_dir_all(&home);
        let told = told.expect("an absolute home is told");
        assert!(told.folders().is_empty(), "{told:?}");
        assert_eq!(of_this_home(Path::new("relative/home")), None);
    }
}
