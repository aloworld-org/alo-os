//! The real disk: the one road out of this crate onto a machine.
//!
//! Two calls, and they are deliberately the plainest two in the standard
//! library. Everything this crate decides — which folders, what they are
//! called, what a refusal costs, what goes in the record — is decided in
//! `crate::making`, `crate::naming` and `crate::recording` against a disk that
//! is a pair of maps. What is left here is *do it*, which is the part that
//! cannot be tested without a filesystem and is therefore the part worth
//! keeping small enough to read in one go.
//!
//! # Already there is not a failure, and that is the behaviour under test
//!
//! `create_dir_all` answers `Ok` for a directory that already exists, which is
//! exactly what a person who made their own Documents folder before alo OS got
//! to it needs. **That is a promise about somebody's files**, so the test
//! below puts a file in the folder first and reads it back afterwards rather
//! than only asking whether the call returned `Ok` — a call that answered `Ok`
//! having emptied the folder would pass the weaker test.

use std::path::Path;

use crate::making::Disk;

/// The machine this is running on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OnThisDisk;

impl Disk for OnThisDisk {
    /// Make the directory and everything above it that is missing.
    fn make_folder(&self, at: &Path) -> Result<(), String> {
        std::fs::create_dir_all(at).map_err(|why| why.to_string())
    }

    /// Write the file whole.
    fn write(&self, at: &Path, contents: &str) -> Result<(), String> {
        std::fs::write(at, contents).map_err(|why| why.to_string())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::folders::Folder;
    use crate::making::made_for;
    use crate::recording::THE_RECORD;
    use crate::testing::in_english;

    /// A folder for one test, removed when the test ends.
    ///
    /// Written out rather than taken as a dependency, which is what
    /// `alo-adapting` and `alo-terminal` already do for the same need: this
    /// repository has no temporary-directory crate and adding one for two
    /// calls would be a dependency in every build to serve a test.
    mod tempish {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU32, Ordering};

        /// A folder for one test.
        pub(super) struct Folder(PathBuf);

        impl Folder {
            /// Make one.
            pub(super) fn new() -> Self {
                static NEXT: AtomicU32 = AtomicU32::new(0);
                let path = std::env::temp_dir().join(format!(
                    "alo-their-folders-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                std::fs::create_dir_all(&path).expect("a folder for this test");
                Self(path)
            }

            /// Where it is.
            pub(super) fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for Folder {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    /// The whole thing, on a filesystem: three folders and a readable record.
    #[test]
    fn a_real_home_folder_gets_three_real_folders_and_a_real_record() {
        let home = tempish::Folder::new();
        let made = made_for(home.path(), &in_english(), &OnThisDisk);

        assert!(made.everything_was_made(), "{made:?}");
        for folder in Folder::ALL {
            let (_, at) = made
                .folders()
                .iter()
                .find(|(it, _)| *it == folder)
                .expect("every folder was made");
            assert!(at.is_dir(), "{}", at.display());
        }

        let record = std::fs::read_to_string(home.path().join(THE_RECORD)).unwrap();
        assert!(
            record.contains("XDG_DOCUMENTS_DIR=\"$HOME/Documents\""),
            "{record}"
        );
        assert!(
            record.contains("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\""),
            "{record}"
        );
        assert!(
            record.contains("XDG_PICTURES_DIR=\"$HOME/Pictures\""),
            "{record}"
        );
    }

    /// **A person's own files survive this running over them.**
    ///
    /// The folder is made, a file is put in it, and the whole thing is run
    /// again — which is what a second sign-in, or a repaired machine, does.
    /// Reading the file back is the test; `Ok` on its own would pass even if
    /// the folder had been replaced.
    #[test]
    fn a_folder_that_was_already_there_keeps_what_is_in_it() {
        let home = tempish::Folder::new();
        let theirs = home.path().join("Documents");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("notes.txt"), "a person's own writing").unwrap();

        let made = made_for(home.path(), &in_english(), &OnThisDisk);

        assert!(made.everything_was_made(), "{made:?}");
        assert_eq!(
            std::fs::read_to_string(theirs.join("notes.txt")).unwrap(),
            "a person's own writing"
        );
    }

    /// Running it twice changes nothing the second time, which is what makes
    /// it safe to call on every sign-in rather than only the first.
    #[test]
    fn running_it_a_second_time_leaves_the_same_machine() {
        let home = tempish::Folder::new();
        let first = made_for(home.path(), &in_english(), &OnThisDisk);
        let again = made_for(home.path(), &in_english(), &OnThisDisk);

        assert_eq!(first, again);
        assert!(again.everything_was_made(), "{again:?}");
    }

    /// The record goes under `.config`, which this makes rather than assuming.
    #[test]
    fn the_record_is_written_even_though_nothing_made_its_folder_first() {
        let home = tempish::Folder::new();
        assert!(!home.path().join(".config").exists());

        let made = made_for(home.path(), &in_english(), &OnThisDisk);

        assert!(home.path().join(".config").is_dir());
        assert_eq!(made.recorded().unwrap(), home.path().join(THE_RECORD));
    }
}
