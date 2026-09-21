//! The volume encryption is enrolled on, named the way it stays named.
//!
//! A kernel's `/dev/sda` is whichever disk answered first on this start, and it
//! can be a different disk on the next one. A name under `/dev/disk/by-id/` is
//! the disk's own — its bus, its maker, its model and its serial number — and it
//! is the same on every start and from every program that asks. The installer
//! already writes a disk down that way (`alo_installing::DiskName`), and this is
//! the same fact on the other side of the install, written again rather than
//! borrowed for the reason [`crate::TheChip`] is: this crate depends on nothing,
//! and the absence is the argument.
//!
//! # There is no free string here
//!
//! [`TheDisk::named`] takes the characters udev writes and nothing else: no
//! slash, no leading dot, no space, nothing empty, and nothing that could leave
//! `/dev/disk/by-id`. A partition is refused as a partition rather than as
//! nonsense, because a partition is named by its disk and a number here —
//! [`TheVolume::the_partition_of`] — and the number is a number rather than
//! something typed on the end of a name.
//!
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
//! asks for the sequence as closed types *with no free string that becomes a
//! device or an argument*. This file is where that is true or not.

use std::fmt;

/// Where the names of disks by their own identity are.
pub const BY_ID: &str = "/dev/disk/by-id";

/// What udev appends to a disk's name to name one of its partitions.
const A_PARTITION: &str = "-part";

/// The longest name accepted. udev's names are far shorter; a file name on
/// Linux is at most this.
const LONGEST: usize = 255;

/// The highest partition number a GPT disk has.
///
/// A number outside this is a program that has lost count rather than a disk
/// anybody has.
pub const THE_LAST_PARTITION: u8 = 128;

/// The name the opened volume answers to.
///
/// One name, fixed, because a machine has one encrypted disk and a name a
/// caller chose would be the free string this file exists to refuse.
pub const IT_OPENS_AS: &str = "alo";

/// Where the opened volume appears once it is open.
pub const WHERE_IT_OPENS: &str = "/dev/mapper/alo";

/// A disk, by its own identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheDisk(String);

/// Why a name or a number was not a volume's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotADisk {
    /// It is not a name udev would have written.
    NotAName,
    /// It names a partition of a disk, and a partition is named by its disk
    /// and a number here.
    APartition,
    /// There is no partition of a disk with that number.
    NotAPartitionNumber,
}

impl fmt::Display for NotADisk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAName => f.write_str("the name is not one a disk is known by"),
            Self::APartition => f.write_str("the name is a partition's, not a whole disk's"),
            Self::NotAPartitionNumber => write!(
                f,
                "a disk's partitions are numbered 1 to {THE_LAST_PARTITION}"
            ),
        }
    }
}

impl std::error::Error for NotADisk {}

impl TheDisk {
    /// The disk this name is.
    ///
    /// # Errors
    /// [`NotADisk::NotAName`] for anything that is not a single component of
    /// udev's characters, and [`NotADisk::APartition`] for a partition's name.
    pub fn named(name: &str) -> Result<Self, NotADisk> {
        let acceptable = !name.is_empty()
            && name.len() <= LONGEST
            && !name.starts_with('.')
            && !name.starts_with('-')
            && name.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'+')
            });
        if !acceptable {
            return Err(NotADisk::NotAName);
        }
        if names_a_partition(name) {
            return Err(NotADisk::APartition);
        }
        Ok(Self(name.to_owned()))
    }

    /// The name, as a person is shown it.
    #[must_use]
    pub fn as_it_is_named(&self) -> &str {
        &self.0
    }
}

/// The volume this machine's encryption is enrolled on: one partition of one
/// disk, by the disk's own identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheVolume {
    /// Where the block device is, written out once so that every run of every
    /// rented tool is given the same bytes.
    at: String,
}

impl TheVolume {
    /// The partition of a disk that holds the encrypted volume.
    ///
    /// # Errors
    /// [`NotADisk::NotAPartitionNumber`] where there is no such partition.
    pub fn the_partition_of(disk: &TheDisk, partition: u8) -> Result<Self, NotADisk> {
        if partition == 0 || partition > THE_LAST_PARTITION {
            return Err(NotADisk::NotAPartitionNumber);
        }
        Ok(Self {
            // Written rather than joined, so the path is the same on every host.
            at: format!("{BY_ID}/{}{A_PARTITION}{partition}", disk.0),
        })
    }

    /// Where the block device is.
    #[must_use]
    pub fn where_it_is(&self) -> &str {
        &self.at
    }
}

/// Whether a name ends in `-part` and a number, which is how udev names a
/// partition beside its disk.
fn names_a_partition(name: &str) -> bool {
    name.rfind(A_PARTITION).is_some_and(|at| {
        name.get(at + A_PARTITION.len()..)
            .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names udev really writes are accepted.
    #[test]
    fn a_disks_own_name_is_accepted() {
        for name in [
            "virtio-alo-target",
            "nvme-Samsung_SSD_980_1TB_S64ANS0T123456X",
            "nvme-eui.002538b111b2c3d4",
            "ata-WDC_WD10EZEX-08WN4A0_WD-WCC6Y1234567",
            "wwn-0x5000c500a1b2c3d4",
        ] {
            let Ok(disk) = TheDisk::named(name) else {
                unreachable!("{name} is a disk's own name")
            };
            assert_eq!(disk.as_it_is_named(), name);
        }
    }

    /// **Anything that is not one component of udev's characters is refused**,
    /// and in particular anything that would leave `/dev/disk/by-id` or carry a
    /// shell into an argument.
    #[test]
    fn a_name_that_is_not_a_disks_is_refused() {
        for name in [
            "",
            ".",
            "..",
            "../../sda",
            "/dev/sda",
            "sda/../../vda",
            "virtio alo",
            "-virtio",
            ".hidden",
            "disk$(reboot)",
            "disk;rm",
            "disk|wc",
            "disk\nwc",
            "diskö",
            "disk@host",
        ] {
            assert_eq!(TheDisk::named(name), Err(NotADisk::NotAName), "{name:?}");
        }
        assert_eq!(
            TheDisk::named(&"a".repeat(LONGEST + 1)),
            Err(NotADisk::NotAName)
        );
    }

    /// **A partition's name is refused as a partition**, because a partition is
    /// a disk and a number here.
    #[test]
    fn a_partitions_name_is_refused_as_a_partition() {
        for name in ["virtio-alo-target-part1", "nvme-Samsung_SSD_980-part12"] {
            assert_eq!(TheDisk::named(name), Err(NotADisk::APartition), "{name}");
        }
        for name in ["virtio-part", "virtio-partner", "ata-part-x1"] {
            assert!(TheDisk::named(name).is_ok(), "{name}");
        }
    }

    /// The volume is the disk's own name and a number, under `by-id`.
    #[test]
    fn a_volume_is_a_partition_of_a_disk_named_the_way_it_stays_named() {
        let Ok(disk) = TheDisk::named("virtio-alo-target") else {
            unreachable!("that is a disk's own name")
        };
        let Ok(volume) = TheVolume::the_partition_of(&disk, 4) else {
            unreachable!("4 is a partition")
        };
        assert_eq!(
            volume.where_it_is(),
            "/dev/disk/by-id/virtio-alo-target-part4"
        );
    }

    /// **No partition is numbered nought, and none past the last one.**
    #[test]
    fn a_number_that_is_not_a_partitions_is_refused() {
        let Ok(disk) = TheDisk::named("virtio-alo-target") else {
            unreachable!("that is a disk's own name")
        };
        for number in [0, THE_LAST_PARTITION + 1, u8::MAX] {
            assert_eq!(
                TheVolume::the_partition_of(&disk, number),
                Err(NotADisk::NotAPartitionNumber),
                "{number}"
            );
        }
        assert!(TheVolume::the_partition_of(&disk, THE_LAST_PARTITION).is_ok());
    }

    /// The opened volume has one name, and it is where it is said to be.
    #[test]
    fn the_opened_volume_has_one_name() {
        assert_eq!(IT_OPENS_AS, "alo");
        assert_eq!(WHERE_IT_OPENS, format!("/dev/mapper/{IT_OPENS_AS}"));
    }

    /// Each refusal says what is wrong and names nothing that was read.
    #[test]
    fn each_refusal_says_what_is_wrong() {
        for refusal in [
            NotADisk::NotAName,
            NotADisk::APartition,
            NotADisk::NotAPartitionNumber,
        ] {
            assert!(!refusal.to_string().is_empty(), "{refusal:?}");
        }
    }
}
