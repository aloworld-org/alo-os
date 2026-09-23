//! The loader's two files where a machine keeps them, read and written as
//! files.
//!
//! [`crate::TheLoader`] over the two of them: the menu this crate generated at
//! install, under `/boot` on alo OS's own filesystem
//! ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! term 3), and the environment block the last choice is kept in, on the EFI
//! system partition
//! ([ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
//! term 1, because that is the one filesystem Windows can write as well). Both
//! are reached only from the privileged side of the broker's door: a person is
//! not root, and neither file is theirs to open.
//!
//! # The partition has to be mounted, and the base does not mount it
//!
//! `/boot/efi` is a directory the base ships and leaves empty. Measured on the
//! pinned base, installed by its own installer and then booted: no vfat is
//! mounted anywhere, there is no `/etc/fstab` at all, and nothing generates a
//! mount for the ESP. So [`crate::THE_ENVIRONMENT_BLOCK`] is the right path
//! once alo OS mounts that partition there, and a file that is not there until
//! it does — which is already a refusal in this file, named and tested, rather
//! than a file invented in its place. `docs/quirks.md` records the measurement
//! and who owes the mount.
//!
//! # The block is written in place, and never replaced
//!
//! The loader saves the last choice itself, from inside the loader, with no
//! filesystem driver that could grow a file or follow it to a new one. So the
//! block is opened for writing and its bytes overwritten where they are — the
//! same file, the same length, the same blocks on the disk. It is not
//! truncated, not created, and never written to a new file and renamed over:
//! each of those hands the loader a file it may no longer be able to save
//! into, which is the last choice silently stopping being kept.
//!
//! [`crate::EnvironmentBlock::written`] is what makes that safe to say — it
//! refuses to produce anything but the length the block was read at.
//!
//! # A machine with no menu of ours has one system, and that is not an error
//!
//! A machine alo OS **replaced** Windows on has no generated menu, because
//! there is nothing to choose between. So a menu that is not there reads as
//! *this machine offers alo OS*, and every other reason it could not be read is
//! a refusal: the difference between *there is no Windows here* and *this could
//! not be read* matters to the person in front of it.
//!
//! # Where this has run
//!
//! Against files in a directory, in this file's own tests. **Not on a machine
//! with a `/boot` behind it**: `docs/booting.md` says what is owed and to whom.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::chosen::THE_ENVIRONMENT_BLOCK;
use crate::loader::{NotRead, NotWritten, TheLoader};
use crate::menu::{Menu, THE_MENU};
use crate::systems::System;

/// The loader's files on the machine this is running on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheLoadersFiles {
    /// The generated menu.
    menu: PathBuf,
    /// The file the last choice is kept in.
    block: PathBuf,
}

impl Default for TheLoadersFiles {
    fn default() -> Self {
        Self::of_this_machine()
    }
}

impl TheLoadersFiles {
    /// This machine's own.
    #[must_use]
    pub fn of_this_machine() -> Self {
        Self::at(Path::new(THE_MENU), Path::new(THE_ENVIRONMENT_BLOCK))
    }

    /// A loader whose two files are somewhere else — how a test reaches one.
    #[must_use]
    pub fn at(menu: &Path, block: &Path) -> Self {
        Self {
            menu: menu.to_owned(),
            block: block.to_owned(),
        }
    }

    /// Where the menu is.
    #[must_use]
    pub fn menu(&self) -> &Path {
        &self.menu
    }

    /// Where the last choice is kept.
    #[must_use]
    pub fn block(&self) -> &Path {
        &self.block
    }
}

impl TheLoader for TheLoadersFiles {
    fn offering(&self) -> Result<Vec<System>, NotRead> {
        let written = match fs::read_to_string(&self.menu) {
            Ok(written) => written,
            // A machine alo OS replaced Windows on has no menu of ours, and
            // that is an answer rather than a failure.
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                return Ok(vec![System::AloOs]);
            }
            Err(why) => {
                return Err(NotRead(format!(
                    "{} could not be read, so this machine was not asked what it offers: {why}",
                    self.menu.display()
                )));
            }
        };
        let mut offering = vec![System::AloOs];
        if Menu::offers_windows(&written) {
            offering.push(System::Windows);
        }
        Ok(offering)
    }

    fn saved(&self) -> Result<Vec<u8>, NotRead> {
        fs::read(&self.block).map_err(|why| {
            NotRead(format!(
                "{} could not be read, so this machine was not asked which system it starts: \
                 {why}",
                self.block.display()
            ))
        })
    }

    fn save(&self, bytes: &[u8]) -> Result<(), NotWritten> {
        let refused = |why: std::io::Error| {
            NotWritten(format!(
                "{} could not be written, so which system this machine starts was not changed: \
                 {why}",
                self.block.display()
            ))
        };
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&self.block)
            .map_err(refused)?;
        file.write_all(bytes).map_err(refused)?;
        file.flush().map_err(refused)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::menu::THE_COUNTDOWN;
    use crate::saved::{EnvironmentBlock, LENGTH};

    /// A directory of a loader's files, made for one test.
    fn a_directory(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-starting-loader-{}-{named}",
            std::process::id()
        ));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).unwrap();
        at
    }

    /// A loader with a menu offering both systems and an empty block.
    fn a_loader(named: &str) -> TheLoadersFiles {
        let at = a_directory(named);
        let loader = TheLoadersFiles::at(&at.join("menu.cfg"), &at.join("block"));
        fs::write(
            loader.menu(),
            Menu::offering("Windows", THE_COUNTDOWN).unwrap().written(),
        )
        .unwrap();
        fs::write(loader.block(), EnvironmentBlock::empty().written().unwrap()).unwrap();
        loader
    }

    /// **A machine with the generated menu on it offers both systems.**
    #[test]
    fn a_machine_with_the_menu_offers_both_systems() {
        let loader = a_loader("both");
        assert_eq!(loader.offering().unwrap(), System::BOTH.to_vec());
    }

    /// **A machine with no menu of ours offers alo OS**, which is what a
    /// machine alo OS replaced Windows on is — and not a refusal.
    #[test]
    fn a_machine_with_no_menu_offers_alo_os() {
        let at = a_directory("replaced");
        let loader = TheLoadersFiles::at(&at.join("menu.cfg"), &at.join("block"));
        assert_eq!(loader.offering().unwrap(), vec![System::AloOs]);
    }

    /// **A menu that is there and offers no Windows offers alo OS.** The file
    /// is read for what it says rather than for being present.
    #[test]
    fn a_menu_that_offers_no_windows_offers_alo_os() {
        let loader = a_loader("no-windows");
        fs::write(loader.menu(), "# nothing to choose between\n").unwrap();
        assert_eq!(loader.offering().unwrap(), vec![System::AloOs]);
    }

    /// **The file the choice is kept in is read as its own bytes, and written
    /// back in place at exactly the same length.**
    #[test]
    fn the_block_is_written_back_in_place_at_the_same_length() {
        let loader = a_loader("in-place");
        let before = loader.saved().unwrap();
        assert_eq!(before.len(), LENGTH);

        let mut block = EnvironmentBlock::read(&before).unwrap();
        block.keep("saved_entry", "alo-windows").unwrap();
        let bytes = block.written().unwrap();
        loader.save(&bytes).unwrap();

        let after = loader.saved().unwrap();
        assert_eq!(after.len(), before.len());
        assert_eq!(after, bytes);
        assert_eq!(
            fs::metadata(loader.block()).unwrap().len(),
            u64::try_from(LENGTH).unwrap(),
            "the file the loader writes in place changed size"
        );
    }

    /// **A file that is not there is not created.** The block belongs to the
    /// loader, and a machine whose block is missing is one alo OS invents
    /// nothing for.
    #[test]
    fn a_block_that_is_not_there_is_not_created() {
        let at = a_directory("missing");
        let loader = TheLoadersFiles::at(&at.join("menu.cfg"), &at.join("block"));
        assert!(loader.saved().is_err());
        assert!(loader.save(b"anything at all").is_err());
        assert!(!loader.block().exists());
    }

    /// **This machine's own files are the loader's own**, beside each other,
    /// and neither is somewhere alo OS invented.
    #[test]
    fn this_machines_own_files_are_the_loaders_own() {
        let loader = TheLoadersFiles::of_this_machine();
        assert_eq!(loader.menu(), Path::new(THE_MENU));
        assert_eq!(loader.block(), Path::new(THE_ENVIRONMENT_BLOCK));
        assert_eq!(loader, TheLoadersFiles::default());
    }
}
