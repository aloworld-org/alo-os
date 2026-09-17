//! **Where a fine-tune works, and what happens to it when it ends.**
//!
//! The dataset a fine-tune prepares — the documents, cut into whatever the
//! rented stack reads — is the person's documents in another shape, so it is
//! written **nowhere but here**, and here is removed when the fine-tune ends.
//!
//! Unless they kept it. A person who wants to see what their model was fed, or
//! to run the same fine-tune again without re-reading a folder, says so; and
//! then it stays, where they can find it and delete it themselves.

use std::path::{Path, PathBuf};

/// **The one folder a fine-tune may write in.**
#[derive(Debug)]
pub struct WorkingFolder {
    /// Where it is.
    at: PathBuf,
    /// Whether the person asked to keep it.
    kept: bool,
}

impl WorkingFolder {
    /// A working folder at this path.
    #[must_use]
    pub fn at(path: &Path) -> Self {
        Self {
            at: path.to_owned(),
            kept: false,
        }
    }

    /// The person asked to keep it when the fine-tune ends.
    #[must_use]
    pub fn kept(mut self) -> Self {
        self.kept = true;
        self
    }

    /// Where it is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.at
    }

    /// Whether it will be kept.
    #[must_use]
    pub fn will_be_kept(&self) -> bool {
        self.kept
    }

    /// Whether this path is inside the working folder — the question every
    /// write asks, so that *the dataset is written nowhere else* is checked
    /// rather than remembered.
    #[must_use]
    pub fn holds(&self, path: &Path) -> bool {
        path.starts_with(&self.at)
    }
}

impl Drop for WorkingFolder {
    /// **Removed when the fine-tune ends**, unless the person kept it.
    fn drop(&mut self) {
        if !self.kept {
            let _ = std::fs::remove_dir_all(&self.at);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_folder() -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "alo-adapting-working-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a folder for this test");
        path
    }

    /// **It goes when the fine-tune ends.**
    #[test]
    fn a_working_folder_is_removed_when_the_fine_tune_ends() {
        let path = a_folder();
        std::fs::write(path.join("dataset.jsonl"), "a person's documents").expect("written");
        {
            let _working = WorkingFolder::at(&path);
        }
        assert!(
            !path.exists(),
            "the person's documents were left on the disk in another shape"
        );
    }

    /// **Unless they kept it**, in which case it is theirs to delete.
    #[test]
    fn a_working_folder_a_person_kept_stays() {
        let path = a_folder();
        {
            let working = WorkingFolder::at(&path).kept();
            assert!(working.will_be_kept());
        }
        assert!(path.exists());
        std::fs::remove_dir_all(&path).expect("this test's own tidying");
    }

    /// **Nothing is written outside it.**
    #[test]
    fn a_path_outside_the_working_folder_is_not_in_it() {
        let path = a_folder();
        let working = WorkingFolder::at(&path).kept();
        assert!(working.holds(&path.join("dataset.jsonl")));
        assert!(!working.holds(Path::new("/home/anna/Invoices/march.txt")));
        std::fs::remove_dir_all(&path).expect("this test's own tidying");
    }
}
