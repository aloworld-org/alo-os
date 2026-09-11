//! The person's settings file on the disk, replaced whole or not at all.
//!
//! `alo_remembering::kept` is the same shape one file over, and the differences
//! between them are all one fact: **that file is the machine's and this one is
//! the person's.**
//!
//! # Whole or nothing
//!
//! A sibling (`settings.toml.new`) is written, synced and renamed over the real
//! file. A machine that loses power in the middle keeps the settings it had —
//! `crate::written::read` refuses a torn file whole, and somebody who picked a
//! model this morning would otherwise find their machine answering nothing this
//! afternoon with nothing able to say why.
//!
//! That is also what makes `crate::Choosing`'s *a write that fails leaves the
//! file as it was* a property rather than a hope: every way this can fail fails
//! before the rename, and the rename is the only step that changes what a
//! reader sees.
//!
//! # The folder **is** made here, and `alo-remembering` deliberately does not
//!
//! `/var/lib/alo` belongs to the image, so a missing one means the machine is
//! not an alo OS machine and making one would turn a typo into a second list
//! nobody reads. `$XDG_CONFIG_HOME/alo` is the opposite case: it belongs to the
//! person, nothing has ever made it, and *the first choice somebody makes* is
//! precisely the moment it does not exist. A first choice that could not be
//! saved until somebody had made a directory by hand would be the machine
//! refusing the ordinary morning `crate::Choosing` exists for.
//!
//! Only the folder the file sits in is made, by the ordinary recursive route,
//! so a `$HOME` that is not there is still a refusal rather than a home
//! directory invented under it.
//!
//! # The mode is the owner's alone, where a host has modes
//!
//! `0600` for the file and `0700` for the folder on a Unix machine. A person's
//! settings say which provider answers their questions and in which region,
//! which is nobody else's business on a machine with several logins. Nothing
//! here *reads* a mode, which is the other difference from
//! `alo_remembering::keeping`: whoever may rewrite that file says what an agent
//! may reach, and this file only says what its own owner prefers.
//!
//! This crate is built on Windows too, where the mode has no meaning and the
//! ordinary inherited permissions are what a file gets. `docs/quirks.md` is
//! where that belongs and `crate::place` already records the other half of it.

use std::io::Write;
use std::path::{Path, PathBuf};

/// The mode a settings file is created with: the owner and nobody else.
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
/// that could not be made and a rename that did not happen. The file at `at` is
/// untouched in every one of them.
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

/// The folder a person's settings sit in, made if it is not there.
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

/// The sibling the new settings are written into before they replace the real
/// ones.
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

/// The path the settings are staged at before they replace the real ones.
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

    /// **What was written is what is read back**, and no staging file outlived
    /// the rename.
    #[test]
    fn what_was_written_is_there_afterwards() {
        let folder = a_folder_of_our_own("keeping-written");
        let at = folder.join("settings.toml");

        kept(&at, "format = 2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 2\n");
        assert!(
            !a_sibling_of(&at).exists(),
            "the staging file outlived the rename"
        );
    }

    /// **The folder a person's settings sit in is made**, because the first
    /// choice somebody makes on a machine is exactly the moment it is not
    /// there.
    #[test]
    fn the_folder_a_first_choice_needs_is_made_rather_than_refused() {
        let folder = a_folder_of_our_own("keeping-first-choice");
        let at = folder.join("alo").join("settings.toml");

        kept(&at, "format = 2\n").unwrap();

        assert!(at.exists());
    }

    /// **Writing again replaces the file whole**, so a provider removed from
    /// somebody's settings is not left behind in the file it was removed from.
    #[test]
    fn writing_again_replaces_rather_than_adds() {
        let folder = a_folder_of_our_own("keeping-replaced");
        let at = folder.join("settings.toml");

        kept(&at, "format = 2\n\n[reading]\nlanguages = [\"de\"]\n").unwrap();
        kept(&at, "format = 2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 2\n");
    }

    /// **A staging file left behind by a write that died does not wedge the
    /// next one.**
    #[test]
    fn a_stale_staging_file_does_not_stop_the_next_write() {
        let folder = a_folder_of_our_own("keeping-stale");
        let at = folder.join("settings.toml");
        std::fs::write(a_sibling_of(&at), "half a file").unwrap();

        kept(&at, "format = 2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 2\n");
    }

    /// **A path that cannot be written leaves what was there as it was.** The
    /// folder in the way is a file, so nothing can be made under it — and the
    /// settings beside it are untouched, which is the promise the whole-or-
    /// nothing shape exists to make.
    #[test]
    fn a_write_that_cannot_happen_leaves_the_settings_alone() {
        let folder = a_folder_of_our_own("keeping-in-the-way");
        let at = folder.join("settings.toml");
        kept(&at, "format = 2\n").unwrap();

        let blocked = folder
            .join("settings.toml")
            .join("alo")
            .join("settings.toml");
        assert!(kept(&blocked, "format = 2\n\n[reading]\nlanguages = []\n").is_err());

        assert_eq!(std::fs::read_to_string(&at).unwrap(), "format = 2\n");
    }

    /// **A person's settings go down readable by their owner and nobody else**,
    /// which is what a file naming a provider and a region deserves on a
    /// machine with several logins.
    #[cfg(unix)]
    #[test]
    fn settings_go_down_readable_by_their_owner_alone() {
        use std::os::unix::fs::PermissionsExt;

        let folder = a_folder_of_our_own("keeping-mode");
        let at = folder.join("alo").join("settings.toml");
        kept(&at, "format = 2\n").unwrap();

        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the settings went down mode {mode:o}");
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
