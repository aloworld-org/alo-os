//! One look at a thing on the disk: its bytes, and how many names it has.
//!
//! The walk read a size for every file as it went. This is the second look,
//! after the walk, at the file itself rather than at the folder's entry for
//! it — and it is the only place the two facts a tree of sizes cannot do
//! without are read: **how many names** the file has, which is what makes a
//! hard link countable once rather than once per name, and **which file it
//! is**, so that two names can be told to be the same file.
//!
//! Read without following anything. If the path has become a link since the
//! walk saw it, the answer is the link's own bytes and says so; nothing here
//! opens what a link points at.

#[cfg(target_os = "linux")]
use std::path::Path;

/// What one look at a path found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Looked {
    /// Its own bytes: a file's contents, or a link's own length.
    pub(crate) bytes: u64,
    /// How many names it has on the disk.
    pub(crate) names: u64,
    /// The filesystem it is on, as a number two files can be compared by.
    pub(crate) on: u64,
    /// Which file on that filesystem, as a number two names can be compared
    /// by.
    pub(crate) inode: u64,
    /// Whether it is a link now, whatever the walk saw.
    pub(crate) is_link: bool,
}

/// One look at this path, as this machine's filesystem answers it.
///
/// # Errors
/// What the machine said, as a sentence, when it would not say: the file is
/// gone since the walk, or its folder may no longer be read.
#[cfg(target_os = "linux")]
pub(crate) fn looked_at(at: &Path) -> Result<Looked, String> {
    use std::os::unix::fs::MetadataExt;
    let about = std::fs::symlink_metadata(at).map_err(|why| why.to_string())?;
    Ok(Looked {
        bytes: about.len(),
        names: about.nlink(),
        on: about.dev(),
        inode: about.ino(),
        is_link: about.file_type().is_symlink(),
    })
}
