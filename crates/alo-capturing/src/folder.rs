//! The folder a person chose to keep their pictures in.
//!
//! The plan says a picture is written **to a file in a folder the person
//! chose**, and the load-bearing half of that sentence is the half this crate
//! can actually keep: **there is no folder in here that anybody did not
//! choose.** No default, no `Pictures`, nothing read from an environment
//! variable, no *we will put it beside the last one*. A [`Folder`] arrives from
//! whatever asked for the picture, and if nothing hands one over then nothing
//! is written, because [`crate::WhereItGoes`] has no third destination.
//!
//! That is a smaller claim than *this path was definitely picked in a picker*,
//! and it is deliberately the one written down. A library cannot tell how a
//! caller came by a path. What it can do is refuse the paths that cannot be
//! somebody's deliberate choice, have no default of its own to fall back on,
//! and say so plainly rather than implying more.
//!
//! # The two it refuses
//!
//! **A path that is not a full path.** *Where a relative path lands* depends on
//! which folder the process happens to be running in, which is a thing no
//! person chose and no person can see.
//!
//! **The root of a filesystem.** A person's pictures are inside their own
//! files. `alo-capability` refuses a grant to `/` for the harder version of
//! this reason (ADR 0001), and a machine that would write into the root of a
//! disk because a picker was left at the top is a machine doing something
//! nobody meant.
//!
//! # And nothing here touches a disk
//!
//! [`Folder::chosen`] asks the path two questions about its own shape and asks
//! the filesystem nothing. Whether the folder is still there, and whether it
//! can be written in, are answered by writing in it (`writing.rs`) —
//! which is the only answer that is still true a moment later.

use std::path::{Path, PathBuf};

use crate::refusing::NotTaken;

/// A folder somebody chose to keep pictures of their screen in.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Folder(PathBuf);

impl Folder {
    /// The folder at this path.
    ///
    /// # Errors
    /// [`NotTaken::NotAFolderForPictures`] for a path that is not a full path,
    /// or for the root of a filesystem. See this module's documentation for
    /// why those two and no others.
    pub fn chosen(at: &Path) -> Result<Self, NotTaken> {
        if !at.has_root() {
            return Err(NotTaken::NotAFolderForPictures);
        }
        if at.parent().is_none() {
            return Err(NotTaken::NotAFolderForPictures);
        }
        Ok(Self(at.to_path_buf()))
    }

    /// The folder itself.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.0
    }

    /// Where a file of this name would go inside it.
    ///
    /// Only ever called with a name [`crate::naming::name_for`] made, which is
    /// one path segment of digits, hyphens and a full stop — so this cannot
    /// land anywhere but inside the folder, and `naming`'s own test is what
    /// says so.
    #[must_use]
    pub fn holding(&self, named: &str) -> PathBuf {
        self.0.join(named)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A full path these tests are about, written the way this host writes one.
    fn a_full_path() -> PathBuf {
        std::env::temp_dir().join("alo-capturing-a-folder")
    }

    /// **A folder is the path somebody chose**, kept exactly as it arrived.
    #[test]
    fn a_folder_is_the_path_somebody_chose() {
        let at = a_full_path();
        let folder = Folder::chosen(&at).unwrap();
        assert_eq!(folder.at(), at);
    }

    /// **A file inside it is inside it**, which is all a name ever does.
    #[test]
    fn a_file_inside_a_folder_is_inside_it() {
        let folder = Folder::chosen(&a_full_path()).unwrap();
        let inside = folder.holding("2026-09-16-120000.png");
        assert_eq!(inside.parent(), Some(folder.at()));
        assert_eq!(
            inside.file_name().and_then(|name| name.to_str()),
            Some("2026-09-16-120000.png")
        );
    }

    /// **A path that is not a full path is refused.** Where it lands depends on
    /// which folder a process happens to be running in, and nobody chose that.
    #[test]
    fn a_path_that_is_not_a_full_path_is_refused() {
        for relative in ["Pictures", "./Pictures", "../Pictures"] {
            assert_eq!(
                Folder::chosen(Path::new(relative)),
                Err(NotTaken::NotAFolderForPictures),
                "{relative}"
            );
        }
    }

    /// **The root of a filesystem is refused.** A person's pictures are inside
    /// their own files, and a machine writing into the root of a disk because a
    /// picker was left at the top is doing something nobody meant.
    #[test]
    fn the_root_of_a_filesystem_is_refused() {
        let root = a_full_path()
            .ancestors()
            .last()
            .map(Path::to_path_buf)
            .unwrap();
        assert_eq!(root.parent(), None, "the fixture is not a root: {root:?}");
        assert_eq!(
            Folder::chosen(&root),
            Err(NotTaken::NotAFolderForPictures),
            "{root:?}"
        );
    }
}
