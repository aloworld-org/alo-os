//! The one disk a person chose, named the way it stays named.
//!
//! A kernel's `/dev/sda` is whichever disk answered first on this start, and it
//! can be a different disk on the next one. A name under `/dev/disk/by-id/` is
//! the disk's own — its bus, its maker, its model and its serial number — and it
//! is the same on every start and from every program that asks. The installer
//! that runs before the restart writes that name down; this environment looks
//! for exactly that name and nothing resembling it.
//!
//! # Read strictly
//!
//! A name is one path component made of the characters udev writes into those
//! names. Anything else — a slash, a leading dot, a space, an empty string — is
//! not a disk name with something wrong with it, it is not a disk name, and a
//! program that tried to make sense of it would be a program writing to a disk
//! nobody named.
//!
//! A name ending in `-partN` is the name of a partition, and is refused with a
//! reason of its own: this environment replaces whole disks, and *part of a
//! disk* is a sentence a person can act on where *not understood* is not.

use std::path::PathBuf;

/// Where the names of disks by their own identity are.
pub const BY_ID: &str = "/dev/disk/by-id";

/// The longest name accepted. udev's names are far shorter; a file name on
/// Linux is at most this.
const LONGEST: usize = 255;

/// What udev appends to a disk's name to name one of its partitions.
const A_PARTITION: &str = "-part";

/// A disk, by its own identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskName(String);

/// Why a name was not accepted as a disk's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotADisk {
    /// It is not a name udev would have written.
    #[error("the name is not one a disk is known by")]
    NotAName,
    /// It names a partition of a disk.
    #[error("the name is a partition's, not a whole disk's")]
    APartition,
}

impl DiskName {
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
            && name.bytes().all(|b| {
                b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b':' | b'+' | b'@')
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
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Where the disk appears once the machine has found it.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        // Written rather than joined, so the path is the same on every host.
        PathBuf::from(format!("{BY_ID}/{}", self.0))
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
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The names udev really writes are accepted, and found where udev puts
    /// them.
    #[test]
    fn a_disks_own_name_is_accepted() {
        for name in [
            "virtio-alo-target",
            "nvme-Samsung_SSD_980_1TB_S64ANS0T123456X",
            "nvme-eui.002538b111b2c3d4",
            "ata-WDC_WD10EZEX-08WN4A0_WD-WCC6Y1234567",
            "wwn-0x5000c500a1b2c3d4",
            "scsi-360022480a1b2c3d4e5f60718293a4b5c",
        ] {
            let disk = DiskName::named(name).unwrap();
            assert_eq!(disk.as_str(), name);
            assert_eq!(
                disk.path(),
                PathBuf::from(format!("/dev/disk/by-id/{name}"))
            );
        }
    }

    /// **Anything that is not one component of udev's characters is refused**,
    /// and in particular anything that would leave `/dev/disk/by-id`.
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
            "virtio-alo\ttarget",
            "nvme-Samsung\\x20SSD",
            "disk$(reboot)",
            "disk;rm",
            "diskö",
        ] {
            assert_eq!(DiskName::named(name), Err(NotADisk::NotAName), "{name:?}");
        }
        assert_eq!(
            DiskName::named(&"a".repeat(LONGEST + 1)),
            Err(NotADisk::NotAName)
        );
    }

    /// **A partition's name is refused as a partition**, not as nonsense.
    #[test]
    fn a_partitions_name_is_refused_as_a_partition() {
        for name in ["virtio-alo-target-part1", "nvme-Samsung_SSD_980-part12"] {
            assert_eq!(DiskName::named(name), Err(NotADisk::APartition), "{name}");
        }
        // Words that merely contain the suffix are disks.
        for name in ["virtio-part", "virtio-partner", "ata-part-x1"] {
            assert!(DiskName::named(name).is_ok(), "{name}");
        }
    }
}
