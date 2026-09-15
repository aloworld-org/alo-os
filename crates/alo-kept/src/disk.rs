//! Text written whole to a file in a person's folder, or not at all.
//!
//! The mechanics are `alo_choosing`'s `keeping.rs`, for the same file one door
//! over, with one step added: **the sibling is read back off the disk and asked
//! whether it is still the value before it is renamed over the old file.** A
//! write the disk mangled, or a filesystem that answered a sync without
//! keeping the bytes, is then a refusal that leaves the person's file as it
//! was, rather than a file that reads as something else at the next sign-in.
//!
//! # Whole or nothing
//!
//! A sibling (`<file>.new`) is written, synced, read back and renamed over the
//! real file. Every way this can fail fails before the rename, and the rename
//! is the only step that changes what a reader sees. A sibling left behind by a
//! refusal is removed, and one left behind by a write that died is cleared by
//! the next.
//!
//! # The folder is made, and is the owner's alone
//!
//! `$XDG_CONFIG_HOME/alo` belongs to the person and nothing has ever made it,
//! so it is made here. `0600` for the file and `0700` for the folder on a Unix
//! machine: what somebody's desktop looks like is nobody else's business on a
//! machine with several logins. On a host with no modes the ordinary inherited
//! permissions are what a file gets.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::unwritten::Unwritten;

/// The mode a kept file is created with: the owner and nobody else.
#[cfg(unix)]
const OURS_ALONE: u32 = 0o600;

/// The mode the folder around it is created with.
#[cfg(unix)]
const OUR_FOLDER: u32 = 0o700;

/// This text, written whole to `at`, once `holds` has said the bytes the disk
/// handed back are still what was meant.
///
/// # Errors
///
/// [`Unwritten::Disk`] at the first step the disk refused, or whatever `holds`
/// refused the bytes with. The file at `at` is untouched in every one of them.
pub(crate) fn kept(
    at: &Path,
    text: &str,
    holds: impl FnOnce(&[u8]) -> Result<(), Unwritten>,
) -> Result<(), Unwritten> {
    if let Some(folder) = at.parent() {
        made(folder).map_err(on_the_disk)?;
    }
    let fresh = a_sibling_of(at);
    // A sibling left behind by a write that died is cleared, so `create_new`
    // below can insist the file being written is this one's.
    if let Err(why) = std::fs::remove_file(&fresh)
        && why.kind() != std::io::ErrorKind::NotFound
    {
        return Err(on_the_disk(why));
    }
    let answered = staged_and_renamed(at, &fresh, text, holds);
    if answered.is_err() {
        // Best effort: a sibling nobody reads changes nothing a person sees,
        // and the next write clears it anyway. The refusal that matters is the
        // one already in hand.
        let _cleared = std::fs::remove_file(&fresh);
    }
    answered
}

/// The sibling written, synced, read back, asked about, and renamed.
fn staged_and_renamed(
    at: &Path,
    fresh: &Path,
    text: &str,
    holds: impl FnOnce(&[u8]) -> Result<(), Unwritten>,
) -> Result<(), Unwritten> {
    let mut file = staged(fresh).map_err(on_the_disk)?;
    file.write_all(text.as_bytes()).map_err(on_the_disk)?;
    file.sync_all().map_err(on_the_disk)?;
    drop(file);
    let back = std::fs::read(fresh).map_err(on_the_disk)?;
    holds(&back)?;
    std::fs::rename(fresh, at).map_err(on_the_disk)
}

/// What the disk said, as a refusal.
fn on_the_disk(why: std::io::Error) -> Unwritten {
    Unwritten::Disk(why.kind())
}

/// The folder a person's file sits in, made if it is not there.
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

/// The sibling the new text is written into before it replaces the real file.
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

/// The path the text is staged at before it replaces the real file.
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
    use crate::testing::a_folder_of_our_own;

    /// Bytes that are always what was meant.
    fn fine(_: &[u8]) -> Result<(), Unwritten> {
        Ok(())
    }

    /// **What was written is there afterwards**, and no sibling outlived the
    /// rename.
    #[test]
    fn what_was_written_is_there_afterwards() {
        let folder = a_folder_of_our_own("disk-written");
        let at = folder.join("example.toml");

        kept(&at, "format = 1\n", fine).unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
        assert!(!a_sibling_of(&at).exists());
    }

    /// **The folder a first change needs is made.**
    #[test]
    fn the_folder_a_first_change_needs_is_made() {
        let folder = a_folder_of_our_own("disk-first-change");
        let at = folder.join("alo").join("example.toml");

        kept(&at, "format = 1\n", fine).unwrap();

        assert!(at.exists());
    }

    /// **The bytes handed to the check are the bytes on the disk.**
    #[test]
    fn what_is_asked_about_is_what_the_disk_handed_back() {
        let folder = a_folder_of_our_own("disk-asked");
        let at = folder.join("example.toml");

        kept(&at, "format = 1\n", |back| {
            assert_eq!(back, b"format = 1\n");
            Ok(())
        })
        .unwrap();
    }

    /// **Bytes the check refuses are never renamed into place**: the old file
    /// is as it was, and the sibling is gone.
    #[test]
    fn bytes_that_do_not_hold_leave_the_file_as_it_was() {
        let folder = a_folder_of_our_own("disk-refused");
        let at = folder.join("example.toml");
        kept(&at, "format = 1\n", fine).unwrap();

        let refused = kept(&at, "format = 1\n\nedge = \"left\"\n", |_| {
            Err(Unwritten::ReadBackAsSomethingElse)
        });

        assert_eq!(refused, Err(Unwritten::ReadBackAsSomethingElse));
        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
        assert!(!a_sibling_of(&at).exists());
    }

    /// **A sibling left by a write that died does not wedge the next one.**
    #[test]
    fn a_stale_sibling_does_not_stop_the_next_write() {
        let folder = a_folder_of_our_own("disk-stale");
        let at = folder.join("example.toml");
        std::fs::write(a_sibling_of(&at), "half a file").unwrap();

        kept(&at, "format = 1\n", fine).unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    }

    /// **A path that cannot be written leaves what was there alone.** The
    /// folder in the way is a file.
    #[test]
    fn a_write_that_cannot_happen_leaves_the_file_alone() {
        let folder = a_folder_of_our_own("disk-in-the-way");
        let at = folder.join("example.toml");
        kept(&at, "format = 1\n", fine).unwrap();

        let blocked = at.join("alo").join("example.toml");
        assert!(matches!(
            kept(&blocked, "format = 1\n", fine),
            Err(Unwritten::Disk(_))
        ));

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 1\n");
    }

    /// **A person's file goes down readable by its owner and nobody else.**
    #[cfg(unix)]
    #[test]
    fn a_kept_file_is_its_owners_alone() {
        use std::os::unix::fs::PermissionsExt;

        let folder = a_folder_of_our_own("disk-mode");
        let at = folder.join("alo").join("example.toml");
        kept(&at, "format = 1\n", fine).unwrap();

        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the file went down {mode:o}");
        let folder_mode = std::fs::metadata(at.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            folder_mode, OUR_FOLDER,
            "the folder went down {folder_mode:o}"
        );
    }
}
