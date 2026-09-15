//! Every disk Windows sees, and which of them alo OS may be installed onto.
//!
//! The environment this installer stages replaces **one whole disk**, and
//! refuses a disk that holds it, holds a partition Windows makes, or is in use
//! (`alo_installing::Disks`). This file asks the same questions before the
//! restart, of what Windows reports, so a person is never offered a disk the
//! environment would refuse after it — and adds the ones only this side can
//! answer: the disk Windows is on is never offered, nor one too small, nor one
//! whose name cannot be carried across the restart (`crate::naming`).
//!
//! # A disk with anything on it is not offered
//!
//! Any partition at all — of a type Windows makes, which is also what a camera
//! card formatted on Windows carries, or of any other system — means a disk
//! somebody might miss. This road is for a disk with nothing on it, and
//! the sentence the person reads says so.

use alo_installing::{DiskName, THIS_INSTALLER};
use serde::Deserialize;

use crate::identities::DiskNumber;
use crate::naming;
use crate::sizes::THE_LEAST_DISK;

/// Every disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disks(Vec<Disk>);

/// One disk.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Disk {
    /// Windows' number for it.
    number: u32,
    /// Its maker's name for it.
    friendly_name: String,
    /// Its serial number.
    serial_number: String,
    /// How it is connected.
    bus_type: String,
    /// Its unique identifier.
    unique_id: String,
    /// Its size in bytes.
    size: u64,
    /// `GPT`, `MBR`, or `RAW` for a disk with no partition table.
    partition_style: String,
    /// Whether Windows will write it.
    is_read_only: bool,
    /// Its partitions.
    #[serde(default)]
    partitions: Vec<Partition>,
}

/// One partition.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Partition {
    /// Its volume's label, where it has one.
    #[serde(default)]
    label: String,
}

/// What a disk is to this installer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    /// The disk Windows is on.
    HoldsWindows,
    /// alo OS may be installed onto it, and this is the name it goes by after
    /// the restart.
    ForAloOs(DiskName),
    /// It holds partitions: files, or a system.
    InUse,
    /// Smaller than alo OS needs.
    TooSmall,
    /// Read-only, or no name can be carried across the restart for it.
    NotUsable,
}

impl Disks {
    /// What the script printed, or [`None`] when it is not the script's answer.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Option<Self> {
        let disks: Vec<Disk> = serde_json::from_str(printed?).ok()?;
        (!disks.is_empty()).then_some(Self(disks))
    }

    /// Every disk, in Windows' order.
    pub fn every(&self) -> impl Iterator<Item = &Disk> {
        self.0.iter()
    }

    /// The disk with this number.
    #[must_use]
    pub fn numbered(&self, number: DiskNumber) -> Option<&Disk> {
        self.0.iter().find(|disk| disk.number() == number)
    }

    /// Whether anything carries the installer's label — an earlier start of
    /// this installer that did not finish, or did and was never removed.
    #[must_use]
    pub fn hold_an_installer_area(&self) -> bool {
        self.0.iter().any(|disk| {
            disk.partitions
                .iter()
                .any(|partition| partition.label.trim().eq_ignore_ascii_case(THIS_INSTALLER))
        })
    }

    /// The name a person is shown for this disk, and types to agree.
    ///
    /// Its maker's name, trimmed, and followed by Windows' number for it when
    /// another disk has the same maker's name — two identical disks in one
    /// computer are ordinary, and a name that fits both is not a name.
    #[must_use]
    pub fn shown_name(&self, disk: &Disk) -> String {
        let name = disk.maker_name();
        let shared = self
            .0
            .iter()
            .filter(|other| other.maker_name().eq_ignore_ascii_case(&name))
            .count()
            > 1;
        if shared || name.is_empty() {
            format!("{name} {}", disk.number).trim().to_owned()
        } else {
            name
        }
    }
}

impl Disk {
    /// Windows' number for it.
    #[must_use]
    pub fn number(&self) -> DiskNumber {
        DiskNumber(self.number)
    }

    /// Its size in bytes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Whether it is laid out with a GPT, which is how a UEFI computer's disk is.
    #[must_use]
    pub fn is_gpt(&self) -> bool {
        self.partition_style.trim().eq_ignore_ascii_case("GPT")
    }

    /// Its maker's name, with its white space made single.
    fn maker_name(&self) -> String {
        self.friendly_name
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// What it is to this installer, given which disk Windows is on.
    ///
    /// In the order a person most needs to hear: that it holds Windows before
    /// that it is too small, that it holds files before that it cannot be named.
    #[must_use]
    pub fn standing(&self, windows_is_on: DiskNumber) -> Standing {
        if self.number() == windows_is_on {
            return Standing::HoldsWindows;
        }
        // Any partition at all, not only the types the environment refuses
        // (`alo_installing::ANOTHER_SYSTEMS`): a disk holding a Linux, or a
        // camera's pictures, is a disk somebody would miss as surely as one
        // holding Windows.
        if !self.partitions.is_empty() {
            return Standing::InUse;
        }
        if self.size < THE_LEAST_DISK {
            return Standing::TooSmall;
        }
        if self.is_read_only {
            return Standing::NotUsable;
        }
        match naming::after_the_restart(
            &self.bus_type,
            &self.friendly_name,
            &self.serial_number,
            &self.unique_id,
        ) {
            Some(name) => Standing::ForAloOs(name),
            None => Standing::NotUsable,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::sizes::GIB;

    /// A Hyper-V machine: Windows on disk 0, an empty 32 GB disk 1, a
    /// formatted 64 GB disk 2, a small disk 3 and a USB stick 4.
    pub(crate) const PRINTED: &str = r#"[
      {"Number":0,"FriendlyName":"Msft Virtual Disk","SerialNumber":"","BusType":"SAS","UniqueId":"600224801B4C5D6E7F8091A2B3C4D5E6","Size":136365211648,"PartitionStyle":"GPT","IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Label":""},
                     {"PartitionNumber":2,"GptType":"{e3c9e316-0b5c-4db8-817d-f92df00215ae}","Label":""},
                     {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":""}]},
      {"Number":1,"FriendlyName":"Msft Virtual Disk","SerialNumber":"","BusType":"SAS","UniqueId":"60022480AAAABBBBCCCCDDDDEEEEFFFF","Size":34359738368,"PartitionStyle":"RAW","IsReadOnly":false,"Partitions":[]},
      {"Number":2,"FriendlyName":"Samsung SSD 870 EVO","SerialNumber":"S5Y1NJ0R123456","BusType":"SATA","UniqueId":"","Size":68719476736,"PartitionStyle":"GPT","IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Photos"}]},
      {"Number":3,"FriendlyName":"Small Disk","SerialNumber":"X1","BusType":"SATA","UniqueId":"","Size":8589934592,"PartitionStyle":"RAW","IsReadOnly":false,"Partitions":[]},
      {"Number":4,"FriendlyName":"SanDisk Ultra","SerialNumber":"4C530001","BusType":"USB","UniqueId":"","Size":64023257088,"PartitionStyle":"RAW","IsReadOnly":false,"Partitions":[]}
    ]"#;

    /// **Only the empty disk that can be named is for alo OS**, and every other
    /// disk is what it is.
    #[test]
    fn only_the_empty_nameable_disk_is_for_alo_os() {
        let disks = Disks::read(Some(PRINTED)).unwrap();
        let standing: Vec<Standing> = disks
            .every()
            .map(|disk| disk.standing(DiskNumber(0)))
            .collect();
        assert_eq!(standing.first(), Some(&Standing::HoldsWindows));
        assert!(
            matches!(standing.get(1), Some(Standing::ForAloOs(name)) if name.as_str() == "wwn-0x60022480aaaabbbbccccddddeeeeffff")
        );
        assert_eq!(standing.get(2), Some(&Standing::InUse));
        assert_eq!(standing.get(3), Some(&Standing::TooSmall));
        assert_eq!(standing.get(4), Some(&Standing::NotUsable));
        assert!(!disks.hold_an_installer_area());
    }

    /// **A read-only disk, or one holding any partition at all, is refused**,
    /// and so is the disk Windows is on however empty it looks.
    #[test]
    fn a_read_only_or_partitioned_disk_is_refused() {
        let disks = Disks::read(Some(PRINTED)).unwrap();
        let empty = disks.numbered(DiskNumber(1)).unwrap().clone();
        let read_only = Disk {
            is_read_only: true,
            ..empty.clone()
        };
        assert_eq!(read_only.standing(DiskNumber(0)), Standing::NotUsable);
        let holding_a_linux = Disk {
            partitions: vec![Partition {
                label: "home".to_owned(),
            }],
            ..empty.clone()
        };
        assert_eq!(holding_a_linux.standing(DiskNumber(0)), Standing::InUse);
        assert_eq!(empty.standing(DiskNumber(1)), Standing::HoldsWindows);
        assert!(empty.size() >= 32 * GIB);
    }

    /// **An area an earlier start left is found by its label.**
    #[test]
    fn an_area_an_earlier_start_left_is_found() {
        let left = PRINTED.replace(r#""Label":"Photos""#, r#""Label":"alo-install""#);
        assert!(Disks::read(Some(&left)).unwrap().hold_an_installer_area());
    }

    /// **Two disks with one maker's name are told apart by number**, and a
    /// unique name is shown alone.
    #[test]
    fn two_disks_with_one_name_are_told_apart() {
        let disks = Disks::read(Some(PRINTED)).unwrap();
        assert_eq!(
            disks.shown_name(disks.numbered(DiskNumber(1)).unwrap()),
            "Msft Virtual Disk 1"
        );
        assert_eq!(
            disks.shown_name(disks.numbered(DiskNumber(2)).unwrap()),
            "Samsung SSD 870 EVO"
        );
    }

    /// No answer, or an empty one, is not a list of disks.
    #[test]
    fn no_answer_is_no_disks() {
        assert_eq!(Disks::read(None), None);
        assert_eq!(Disks::read(Some("[]")), None);
        assert_eq!(Disks::read(Some("{}")), None);
    }
}
