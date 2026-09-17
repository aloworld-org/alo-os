//! **What a fine-tune will be trained on, listed before anything trains.**

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_picking::Picked;
use serde::{Deserialize, Serialize};

/// **The files a fine-tune will learn from, and the ones it will not.**
///
/// # A snapshot, not a folder
///
/// The list is taken **once**, before training, and training reads only what is
/// on it. A file added to the folder while the model trains is not learned, and
/// a file removed is not reached.
///
/// That is the whole reason a person is shown a list: what they approved is
/// *these documents*, and a fine-tune that followed the folder instead would
/// train on whatever arrived afterwards — a mail that landed at the wrong
/// moment, a file somebody else dropped in a shared folder. A person cannot
/// approve a list that is still changing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dataset {
    /// The folders a person granted, as they granted them.
    folders: Vec<PathBuf>,
    /// The files this fine-tune will learn from, in the order they were found.
    files: Vec<PathBuf>,
    /// What was found and will not be trained on, each with its reason.
    not_trained_on: Vec<NotTrainedOn>,
    /// When the list was taken. Everything after this moment is outside it.
    taken_at: SystemTime,
}

impl Dataset {
    /// **Take the list**, from folders a person granted.
    ///
    /// `readable` answers whether a file can be read and trained on; it is the
    /// index's answer where there is one, so this crate does not open files and
    /// does not decide what a document is.
    ///
    /// # Errors
    /// [`NoDataset::NothingGranted`] where no folder was granted — a fine-tune
    /// over nothing is not a fine-tune, and *all my documents* is not a grant.
    pub fn taken_from(
        granted: &[Picked],
        found: &dyn Fn(&Path) -> Vec<PathBuf>,
        readable: &dyn Fn(&Path) -> Result<(), Skipped>,
        at: SystemTime,
    ) -> Result<Self, NoDataset> {
        if granted.is_empty() {
            return Err(NoDataset::NothingGranted);
        }
        let mut files = Vec::new();
        let mut not_trained_on = Vec::new();
        for folder in granted {
            for path in found(folder.folder()) {
                match readable(&path) {
                    Ok(()) => files.push(path),
                    Err(why) => not_trained_on.push(NotTrainedOn { file: path, why }),
                }
            }
        }
        if files.is_empty() {
            return Err(NoDataset::NothingToLearnFrom {
                looked_at: not_trained_on.len(),
            });
        }
        Ok(Self {
            folders: granted
                .iter()
                .map(|folder| folder.folder().to_owned())
                .collect(),
            files,
            not_trained_on,
            taken_at: at,
        })
    }

    /// The folders this was taken from.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// **Every file the model will learn from** — the whole list, so a person
    /// sees it rather than a number.
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// How many files it will learn from.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.files.len()
    }

    /// **What was found and will not be trained on, each named with why** — a
    /// file skipped silently is a person believing their model read something it
    /// never saw.
    #[must_use]
    pub fn not_trained_on(&self) -> &[NotTrainedOn] {
        &self.not_trained_on
    }

    /// When the list was taken.
    #[must_use]
    pub fn taken_at(&self) -> SystemTime {
        self.taken_at
    }

    /// Whether this file is one the model will learn from.
    ///
    /// The road every reader of a dataset takes, so that *the snapshot decides*
    /// is a fact about the type rather than a rule somebody remembers.
    #[must_use]
    pub fn will_learn_from(&self, file: &Path) -> bool {
        self.files.iter().any(|listed| listed == file)
    }
}

/// One file that will not be trained on, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotTrainedOn {
    /// Which file.
    pub file: PathBuf,
    /// Why it is not in the dataset.
    pub why: Skipped,
}

/// Why a file found in a granted folder is not trained on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Skipped {
    /// The index could not read it — a format nothing here opens, or a file
    /// that is damaged.
    TheIndexCouldNotReadIt,
    /// A kind of file this does not train on: a photograph, a video, a
    /// database. Not a judgement about the file, a statement about the method.
    NotAKindItTrainsOn,
    /// Empty, so there is nothing in it to learn.
    NothingInIt,
    /// It is not the person's to train on: something the machine itself wrote
    /// into the folder, or another account's.
    NotTheirsToTrainOn,
}

/// Why there is no dataset at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NoDataset {
    /// No folder was granted.
    #[error("no folder was granted to train on, and a fine-tune over nothing is not one")]
    NothingGranted,
    /// Folders were granted and nothing in them can be learned from.
    #[error(
        "nothing in the granted folders can be trained on: {looked_at} file(s) were found and \
         every one of them is named with its reason"
    )]
    NothingToLearnFrom {
        /// How many were looked at.
        looked_at: usize,
    },
}

/// What another module's tests build a dataset with.
#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
pub(crate) mod tests_support {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use super::{Dataset, tests};

    /// One dataset over one file, and the folder it came from.
    pub(crate) fn one_file_dataset() -> (Dataset, PathBuf) {
        let (picked, inside, home) = tests::granted("Invoices");
        let dataset = Dataset::taken_from(
            &[picked],
            &|folder| vec![folder.join("march.txt")],
            &|_| Ok(()),
            SystemTime::UNIX_EPOCH,
        )
        .expect("a dataset over one file");
        // The folder outlives the dataset only for as long as this test needs
        // its path; the dataset holds paths and not handles.
        let at = inside;
        drop(home);
        (dataset, at)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
pub(crate) mod tests {
    use super::*;

    /// A folder really picked, through the picker a person uses — `Picked` is
    /// sealed, and this crate would rather build one the way the product does
    /// than be handed a path in a test it would never accept in life.
    pub(crate) fn granted(folder: &str) -> (Picked, std::path::PathBuf, tempish::Folder) {
        let home = tempish::Folder::new();
        let inside = home.path().join(folder);
        std::fs::create_dir_all(&inside).unwrap();
        let mut picker =
            alo_picking::Picker::standing_in(home.path(), &alo_picking::OnThisDisk).unwrap();
        picker.go_into(folder, &alo_picking::OnThisDisk).unwrap();
        let alo_picking::Chosen::Folder(picked) = picker.pick().unwrap() else {
            unreachable!("the picker was standing in a folder and picked it")
        };
        (picked, inside, home)
    }

    /// A folder that removes itself, so a test leaves nothing behind.
    #[expect(
        clippy::expect_used,
        reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
    )]
    pub(crate) mod tempish {
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU32, Ordering};

        /// A folder for one test.
        pub(crate) struct Folder(PathBuf);

        impl Folder {
            /// Make one.
            pub(crate) fn new() -> Self {
                static NEXT: AtomicU32 = AtomicU32::new(0);
                let path = std::env::temp_dir().join(format!(
                    "alo-adapting-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                std::fs::create_dir_all(&path).expect("a folder for this test");
                Self(path)
            }

            /// Where it is.
            pub(crate) fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for Folder {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }

    /// **A list is taken, and everything found is either learned from or named.**
    #[test]
    fn every_file_found_is_either_learned_from_or_named_with_a_reason() {
        let (picked, _inside, _home) = granted("Invoices");
        let dataset = Dataset::taken_from(
            &[picked],
            &|folder| {
                vec![
                    folder.join("march.txt"),
                    folder.join("scan.png"),
                    folder.join("empty.txt"),
                ]
            },
            &|path| match path.extension().and_then(|kind| kind.to_str()) {
                Some("png") => Err(Skipped::NotAKindItTrainsOn),
                _ if path.ends_with("empty.txt") => Err(Skipped::NothingInIt),
                _ => Ok(()),
            },
            SystemTime::UNIX_EPOCH,
        )
        .unwrap();

        assert_eq!(dataset.how_many(), 1);
        assert_eq!(dataset.not_trained_on().len(), 2);
        for skipped in dataset.not_trained_on() {
            assert!(
                matches!(
                    skipped.why,
                    Skipped::NotAKindItTrainsOn | Skipped::NothingInIt
                ),
                "a file was skipped without a reason a person could read"
            );
        }
    }

    /// **The snapshot decides**: a file that appears after the list was taken is
    /// not learned from, whatever the folder holds later.
    #[test]
    fn a_file_that_arrives_after_the_list_was_taken_is_not_learned_from() {
        let (picked, inside, _home) = granted("Invoices");
        let dataset = Dataset::taken_from(
            &[picked],
            &|folder| vec![folder.join("march.txt")],
            &|_| Ok(()),
            SystemTime::UNIX_EPOCH,
        )
        .unwrap();
        assert!(dataset.will_learn_from(&inside.join("march.txt")));
        assert!(
            !dataset.will_learn_from(&inside.join("arrived-later.txt")),
            "a fine-tune would train on a file nobody was shown"
        );
    }

    /// **No grant, no dataset** — and *all my documents* is not a grant.
    #[test]
    fn a_fine_tune_over_nothing_granted_is_refused() {
        assert_eq!(
            Dataset::taken_from(&[], &|_| Vec::new(), &|_| Ok(()), SystemTime::UNIX_EPOCH),
            Err(NoDataset::NothingGranted)
        );
    }

    /// **A folder of things it cannot read is refused**, saying how many were
    /// looked at rather than training on nothing.
    #[test]
    fn granted_folders_with_nothing_learnable_are_refused_with_the_count() {
        let (picked, _inside, _home) = granted("Photos");
        let refused = Dataset::taken_from(
            &[picked],
            &|folder| vec![folder.join("one.png"), folder.join("two.png")],
            &|_| Err(Skipped::NotAKindItTrainsOn),
            SystemTime::UNIX_EPOCH,
        );
        assert_eq!(refused, Err(NoDataset::NothingToLearnFrom { looked_at: 2 }));
    }
}
