//! Which disk the person chose, as the restart carried it here.
//!
//! The installer that runs on the machine's previous system asks a person to
//! type the name of the disk alo OS goes onto, and writes that choice into the
//! boot entry it stages: one word on this environment's kernel command line,
//! `alo.installing.to=` and the disk's own name. That is the only thing this
//! environment is told, and it is read from the one place the firmware and the
//! loader put it.
//!
//! # Never a guess
//!
//! No word is no choice, and an empty word is no choice either — which is what a
//! loader writes when the file holding the choice was never staged. Two words
//! are two choices, and nothing here decides which one was meant. A word that
//! does not name a disk is not a disk. Each of those is a refusal before
//! anything is looked at, let alone written.

use crate::disk::{DiskName, NotADisk};

/// The word on the kernel command line that carries the choice.
pub const THE_CHOICE: &str = "alo.installing.to=";

/// What was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// The one disk to install onto.
    disk: DiskName,
}

/// Why the command line did not name one disk.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotTold {
    /// Nothing was chosen.
    #[error("no disk was chosen")]
    NothingChosen,
    /// More than one disk was named.
    #[error("more than one disk was chosen")]
    MoreThanOne,
    /// What was named is not a disk's name.
    #[error("what was chosen is not a disk: {0}")]
    NotADisk(NotADisk),
    /// What was named is a partition, and its name is kept to be said.
    #[error("what was chosen is part of a disk: {0}")]
    APartition(String),
}

impl Told {
    /// What a kernel command line says was chosen.
    ///
    /// # Errors
    /// [`NotTold`], for each of the ways it does not name exactly one disk.
    pub fn from_the_command_line(line: &str) -> Result<Self, NotTold> {
        let mut chosen = line
            .split_ascii_whitespace()
            .filter_map(|word| word.strip_prefix(THE_CHOICE));
        let Some(first) = chosen.next() else {
            return Err(NotTold::NothingChosen);
        };
        if chosen.next().is_some() {
            return Err(NotTold::MoreThanOne);
        }
        if first.is_empty() {
            return Err(NotTold::NothingChosen);
        }
        match DiskName::named(first) {
            Ok(disk) => Ok(Self { disk }),
            Err(NotADisk::APartition) => Err(NotTold::APartition(first.to_owned())),
            Err(why) => Err(NotTold::NotADisk(why)),
        }
    }

    /// The disk.
    #[must_use]
    pub fn disk(&self) -> &DiskName {
        &self.disk
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The command line the staged loader writes, with a choice in it.
    const STAGED: &str = "BOOT_IMAGE=/EFI/alo-installing/vmlinuz rd.systemd.unit=alo-installing.target \
                          rd.neednet=1 ip=dhcp console=ttyS0,115200 console=tty0";

    /// One disk named is that disk, wherever on the line it is.
    #[test]
    fn one_disk_named_is_that_disk() {
        let told =
            Told::from_the_command_line(&format!("{STAGED} alo.installing.to=virtio-alo-target"))
                .unwrap();
        assert_eq!(told.disk().as_str(), "virtio-alo-target");

        let told = Told::from_the_command_line(&format!(
            "alo.installing.to=nvme-Samsung_SSD_980 {STAGED}\n"
        ))
        .unwrap();
        assert_eq!(told.disk().as_str(), "nvme-Samsung_SSD_980");
    }

    /// **Nothing chosen is refused**, whether the word is missing or empty.
    #[test]
    fn nothing_chosen_is_refused() {
        assert_eq!(
            Told::from_the_command_line(STAGED),
            Err(NotTold::NothingChosen)
        );
        assert_eq!(
            Told::from_the_command_line(&format!("{STAGED} alo.installing.to=")),
            Err(NotTold::NothingChosen)
        );
        assert_eq!(Told::from_the_command_line(""), Err(NotTold::NothingChosen));
    }

    /// **Two choices are refused**, even when they are the same disk twice —
    /// a line that says it twice is a line something went wrong writing.
    #[test]
    fn two_choices_are_refused() {
        assert_eq!(
            Told::from_the_command_line(&format!(
                "{STAGED} alo.installing.to=virtio-a alo.installing.to=virtio-b"
            )),
            Err(NotTold::MoreThanOne)
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=virtio-a alo.installing.to=virtio-a"),
            Err(NotTold::MoreThanOne)
        );
    }

    /// **A choice that is not a disk, or is part of one, is refused as that.**
    #[test]
    fn a_choice_that_is_not_a_whole_disk_is_refused() {
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=../../sda"),
            Err(NotTold::NotADisk(NotADisk::NotAName))
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=/dev/sda"),
            Err(NotTold::NotADisk(NotADisk::NotAName))
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=virtio-alo-target-part2"),
            Err(NotTold::APartition("virtio-alo-target-part2".to_owned()))
        );
    }

    /// A word that only resembles the choice is not the choice.
    #[test]
    fn a_word_that_only_resembles_the_choice_is_not_one() {
        assert_eq!(
            Told::from_the_command_line("xalo.installing.to=virtio-a alo.installing.too=virtio-b"),
            Err(NotTold::NothingChosen)
        );
    }
}
