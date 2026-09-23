//! Which system the machine starts at when nobody chooses — read and written
//! in the loader's own environment block, and nowhere else.
//!
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)'s
//! third term: *the last choice is the loader's own saved default, and nobody
//! keeps a copy. Two copies drift, and a menu that preselects one thing while a
//! setting says another is the bug this term exists to prevent.* This file is
//! that term. It holds one path and one name, and nothing else in alo OS keeps
//! a second answer to *which system does this machine start*.
//!
//! # Why only Windows has a name here
//!
//! The Windows entry is **ours**: [`crate::Menu`] generates it and gives it an
//! identifier we choose, so it is a word this file can compare. alo OS's own
//! entries are the base's, made from the kernel each of them starts, and their
//! identifiers change with every update — so a copy of one kept here would be
//! wrong by the next update and would be a second copy while it lasted.
//!
//! So the reading is: **the saved entry is the Windows one, or the machine
//! starts alo OS.** That is true of a block the loader itself wrote when a
//! person chose alo OS at the menu, of a block where the value was never set,
//! and of a block that came from an update carrying a kernel nobody has seen
//! before. It is the whole of it, and there is nothing else to keep in step.

use crate::saved::{EnvironmentBlock, NotAnEnvironmentBlock};
use crate::systems::{System, THE_WINDOWS_ENTRY};

/// The one file the last choice is kept in, as alo OS reaches it.
///
/// On the **EFI system partition**, which
/// [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
/// term 1 decided it: *the one filesystem both systems can read and write, and
/// the one the loader already reads at start.* A machine that starts Windows
/// tomorrow is told so from Windows as often as from here, and a default kept
/// under `/boot` could only ever be changed from one side — the other half of
/// ADR 0066 would be a promise this repository could not keep.
///
/// It is **not** beside [`crate::THE_MENU`], and that is the point rather than
/// an oversight: the menu is alo OS's own file and belongs on alo OS's own
/// filesystem, while the answer to *which system* belongs to both systems.
///
/// # What was measured, on the base this image is built from
///
/// The base keeps no default here. `quay.io/fedora/fedora-bootc:42` at the
/// pinned digest was installed to a disk by its own installer, and its
/// `/boot/grub2/grubenv` came out a **regular file** — not the symlink onto
/// the ESP that older Fedora layouts had — holding nothing at all, while the
/// ESP had no environment block on it. So there was no file of the base's to
/// adopt and none left behind keeping a second answer. `docs/quirks.md` has
/// the measurement, including that the loader can save into this one.
pub const THE_ENVIRONMENT_BLOCK: &str = "/boot/efi/EFI/fedora/grubenv";

/// The same one file, named from inside the partition it is on.
///
/// alo OS mounts that partition at `/boot/efi` and so names the file from `/`;
/// the loader is already running on it and names it from the partition's own
/// root; the Windows program mounts it at a letter of its own choosing. Three
/// names for one file, which is what ADR 0066 term 1 asks for and also the way
/// it could quietly become two — so
/// `the_file_alo_os_opens_is_the_file_the_loader_saves_into` in this crate's
/// tests holds the two of them in this repository together.
pub const THE_BLOCK_ON_THE_ESP: &str = "/EFI/fedora/grubenv";

/// The one name it is kept under, which is the loader's own.
pub const SAVED_ENTRY: &str = "saved_entry";

/// Which system this machine starts when nobody chooses at the menu.
///
/// Not a value of its own: every method reads or writes
/// [`SAVED_ENTRY`] in an [`EnvironmentBlock`], which is the only place the
/// answer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheStartingChoice;

impl TheStartingChoice {
    /// Which system this block says the machine starts.
    #[must_use]
    pub fn read(block: &EnvironmentBlock) -> System {
        match block.kept_as(SAVED_ENTRY) {
            Some(THE_WINDOWS_ENTRY) => System::Windows,
            _ => System::AloOs,
        }
    }

    /// Say in this block that the machine starts `system`.
    ///
    /// alo OS is written as **nothing kept** under the name rather than as an
    /// identifier of its own, for the reason this file's header gives: the
    /// identifiers of alo OS's own entries are the base's and change with every
    /// update. An empty value is what the loader reads as *no saved choice*,
    /// and it starts the first entry it has, which is alo OS's newest.
    ///
    /// # Errors
    /// [`NotAnEnvironmentBlock`], from the block itself. Neither value this
    /// writes can cause one.
    pub fn write(
        block: &mut EnvironmentBlock,
        system: System,
    ) -> Result<(), NotAnEnvironmentBlock> {
        let value = match system {
            System::AloOs => "",
            System::Windows => THE_WINDOWS_ENTRY,
        };
        block.keep(SAVED_ENTRY, value)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Both read back from what was written**, through the loader's own file
    /// and nothing beside it.
    #[test]
    fn both_read_back_from_the_one_place_they_are_kept() {
        for system in System::BOTH {
            let mut block = EnvironmentBlock::empty();
            TheStartingChoice::write(&mut block, system).unwrap();
            let written = block.written().unwrap();
            let read = EnvironmentBlock::read(&written).unwrap();
            assert_eq!(TheStartingChoice::read(&read), system);
        }
    }

    /// **A block that says nothing about it says alo OS.** A machine whose
    /// loader has never saved a choice starts the system it was installed to
    /// start, rather than nothing.
    #[test]
    fn a_block_that_says_nothing_starts_alo_os() {
        assert_eq!(
            TheStartingChoice::read(&EnvironmentBlock::empty()),
            System::AloOs
        );
    }

    /// **An entry nobody here recognises is alo OS, not a refusal.** The loader
    /// writes its own identifier for alo OS's entries when a person chooses one
    /// at the menu, and those identifiers carry the kernel they start — so the
    /// value after an update is one this crate has never seen, and reading it
    /// as anything but alo OS would preselect Windows on a machine whose person
    /// chose alo OS.
    #[test]
    fn an_entry_this_crate_has_never_seen_is_alo_os() {
        for saved in [
            "",
            "b0d4e8f-6.17.4-200.fc42.x86_64",
            "ostree-2-fedora.conf",
            "alo-windows-but-not-quite",
        ] {
            let mut block = EnvironmentBlock::empty();
            block.keep(SAVED_ENTRY, saved).unwrap();
            assert_eq!(
                TheStartingChoice::read(&block),
                System::AloOs,
                "{saved:?} was read as Windows"
            );
        }
    }

    /// **Writing one does not disturb anything else in the block**, which is
    /// the difference between a setting and a file this crate owns.
    #[test]
    fn writing_the_choice_leaves_everything_else_alone() {
        let mut block = EnvironmentBlock::empty();
        block.keep("boot_success", "1").unwrap();
        block.keep("boot_indeterminate", "0").unwrap();
        TheStartingChoice::write(&mut block, System::Windows).unwrap();
        assert_eq!(block.kept_as("boot_success"), Some("1"));
        assert_eq!(block.kept_as("boot_indeterminate"), Some("0"));
        assert_eq!(block.how_many(), 3);
    }

    /// **Writing it twice writes one setting**, so the block does not grow a
    /// second answer each time a person changes their mind.
    #[test]
    fn writing_it_twice_keeps_one_answer() {
        let mut block = EnvironmentBlock::empty();
        for system in [System::Windows, System::AloOs, System::Windows] {
            TheStartingChoice::write(&mut block, system).unwrap();
        }
        assert_eq!(block.how_many(), 1);
        assert_eq!(TheStartingChoice::read(&block), System::Windows);
    }
}
