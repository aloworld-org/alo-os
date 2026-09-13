//! The index file on the disk, replaced whole or not at all.
//!
//! `alo_choosing`'s `keeping.rs` is the same shape one crate over, for the
//! person's settings; this is the person's index, and the argument is the
//! same. A sibling (`<name>.new`) is written, synced and renamed over the
//! real file, so a machine that loses power in the middle keeps the index it
//! had — and `format.rs` refuses a torn file whole, so the worst case is an
//! index made again rather than one that silently lost its last thousand
//! entries.
//!
//! # The folder is made here
//!
//! `$XDG_DATA_HOME/alo/finding` belongs to the person, nothing has ever made
//! it, and the first folder somebody indexes is precisely the moment it does
//! not exist. Only the folder the file sits in is made, by the ordinary
//! recursive route, so a `$HOME` that is not there is a refusal rather than a
//! home directory invented under it.
//!
//! # The mode is the owner's alone, where a host has modes
//!
//! `0600` for the file and `0700` for the folder on a Unix machine: an index
//! holds every word of every text file under the folder, which is nobody
//! else's business on a machine with several logins. On Windows the mode has
//! no meaning and the ordinary inherited permissions are what a file gets.

use std::io::Write;
use std::path::{Path, PathBuf};

/// The mode an index file is created with: the owner and nobody else.
#[cfg(unix)]
const OURS_ALONE: u32 = 0o600;

/// The mode the folder around it is created with.
#[cfg(unix)]
const OUR_FOLDER: u32 = 0o700;

/// This text, written whole to this path.
///
/// # Errors
///
/// Whatever the disk said, at the first step that failed — including a folder
/// that could not be made and a rename that did not happen. The file at `at`
/// is untouched in every one of them.
pub(crate) fn kept(at: &Path, text: &str) -> std::io::Result<()> {
    if let Some(folder) = at.parent() {
        made(folder)?;
    }
    let fresh = a_sibling_of(at);
    // A sibling left behind by a write that died is cleared, so `create_new`
    // below can insist the file being written is this one's.
    if let Err(why) = std::fs::remove_file(&fresh)
        && why.kind() != std::io::ErrorKind::NotFound
    {
        return Err(why);
    }
    let mut file = staged(&fresh)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&fresh, at)
}

/// The folder an index sits in, made if it is not there.
#[cfg(unix)]
fn made(folder: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(OUR_FOLDER)
        .create(folder)
}

/// The same, on a host with no modes.
#[cfg(not(unix))]
fn made(folder: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(folder)
}

/// The sibling the new index is written into before it replaces the real
/// one.
#[cfg(unix)]
fn staged(fresh: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(OURS_ALONE)
        .open(fresh)
}

/// The same, on a host with no modes.
#[cfg(not(unix))]
fn staged(fresh: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(fresh)
}

/// The path the index is staged at before it replaces the real one.
fn a_sibling_of(at: &Path) -> PathBuf {
    let mut named = at.as_os_str().to_owned();
    named.push(".new");
    PathBuf::from(named)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A folder of this test's own, under this machine's temporary directory.
    fn a_folder_of_our_own(what: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("alo-finding-keeping-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        folder
    }

    /// **What was written is what is read back**, the folder it needed was
    /// made, no staging file outlived the rename, writing again replaces,
    /// and a stale staging file does not stop the next write.
    #[test]
    fn what_was_written_is_there_afterwards_and_nothing_else_is() {
        let folder = a_folder_of_our_own("kept");
        let at = folder.join("alo").join("finding").join("one.index");
        kept(&at, "{\"format\":1}\n").unwrap();
        assert_eq!(std::fs::read_to_string(&at).unwrap(), "{\"format\":1}\n");
        assert!(!a_sibling_of(&at).exists());

        std::fs::write(a_sibling_of(&at), b"left behind").unwrap();
        kept(&at, "second\n").unwrap();
        assert_eq!(std::fs::read_to_string(&at).unwrap(), "second\n");
        assert!(!a_sibling_of(&at).exists());
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **A write that cannot happen leaves the index alone.** The path's
    /// parent is a file, so no folder can be made and nothing is written.
    #[test]
    fn a_write_that_cannot_happen_leaves_the_index_alone() {
        let folder = a_folder_of_our_own("refused");
        std::fs::create_dir_all(&folder).unwrap();
        let a_file = folder.join("a-file");
        std::fs::write(&a_file, b"in the way").unwrap();
        let at = a_file.join("one.index");
        assert!(kept(&at, "anything").is_err());
        assert_eq!(std::fs::read(&a_file).unwrap(), b"in the way");
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// The index goes down readable by its owner alone.
    #[cfg(unix)]
    #[test]
    fn an_index_goes_down_readable_by_its_owner_alone() {
        use std::os::unix::fs::PermissionsExt;

        let folder = a_folder_of_our_own("mode");
        let at = folder.join("finding").join("one.index");
        kept(&at, "x").unwrap();
        assert_eq!(
            std::fs::metadata(&at).unwrap().permissions().mode() & 0o777,
            OURS_ALONE
        );
        assert_eq!(
            std::fs::metadata(at.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            OUR_FOLDER
        );
        let _ = std::fs::remove_dir_all(&folder);
    }
}
