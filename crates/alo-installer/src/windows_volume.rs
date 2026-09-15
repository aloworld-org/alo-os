//! Where Windows is, how big it is, and how far Windows says it can shrink.
//!
//! Every number comes from Windows' storage cmdlets: the partition holding
//! `%SystemDrive%`, the smallest size `Get-PartitionSupportedSize` says that
//! partition can be shrunk to (which accounts for files Windows cannot move),
//! and the volume's free space.

use serde::Deserialize;

use crate::identities::{DiskNumber, Letter, PartitionNumber};
use crate::sizes::{THE_AREA, WINDOWS_KEEPS_FREE, aligned};

/// The Windows volume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsVolume {
    /// Its drive letter.
    pub letter: Letter,
    /// The disk it is on.
    pub disk: DiskNumber,
    /// Its partition's number.
    pub partition: PartitionNumber,
    /// Where its partition begins, in bytes.
    pub offset: u64,
    /// Its partition's size, in bytes.
    pub size: u64,
    /// The smallest size Windows can shrink it to, in bytes.
    pub smallest: u64,
    /// Its free space, in bytes.
    pub free: u64,
}

/// The shrink this installer would make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shrink {
    /// The size the Windows partition is shrunk to.
    pub to: u64,
    /// Where the freed space, and so the area, begins.
    pub area_begins: u64,
}

/// Why the Windows volume cannot give up the area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotEnoughSpace {
    /// The free space it would need.
    pub needed: u64,
    /// The free space it has.
    pub free: u64,
}

/// What the script prints.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Printed {
    /// The letter.
    drive_letter: String,
    /// The disk.
    disk_number: u32,
    /// The partition.
    partition_number: u32,
    /// Where it begins.
    offset: u64,
    /// Its size.
    size: u64,
    /// How small it can be.
    size_min: u64,
    /// Free space.
    size_remaining: u64,
}

impl WindowsVolume {
    /// What the script printed, or [`None`] when it is not the script's answer.
    #[must_use]
    pub fn read(printed: Option<&str>) -> Option<Self> {
        let printed = serde_json::from_str::<Printed>(printed?).ok()?;
        let letter = Letter::of(&printed.drive_letter)?;
        let sane = printed.size > 0
            && printed.size_min <= printed.size
            && printed.size_remaining <= printed.size;
        sane.then_some(Self {
            letter,
            disk: DiskNumber(printed.disk_number),
            partition: PartitionNumber(printed.partition_number),
            offset: printed.offset,
            size: printed.size,
            smallest: printed.size_min,
            free: printed.size_remaining,
        })
    }

    /// The shrink that gives up [`THE_AREA`] and leaves Windows
    /// [`WINDOWS_KEEPS_FREE`], or why there is not room for it.
    ///
    /// # Errors
    /// [`NotEnoughSpace`] when the free space is short of both together, or
    /// Windows says the partition cannot shrink that far.
    pub fn shrink(&self) -> Result<Shrink, NotEnoughSpace> {
        let needed = THE_AREA + WINDOWS_KEEPS_FREE;
        let short = NotEnoughSpace {
            needed,
            free: self.free,
        };
        if self.free < needed {
            return Err(short);
        }
        // Rounded down to a mebibyte so the area begins where Windows would
        // put a partition, and never less than the area is given up.
        let to = aligned(self.size.checked_sub(THE_AREA).ok_or(short)?);
        if to < self.smallest {
            return Err(short);
        }
        let area_begins = self.offset.checked_add(to).ok_or(short)?;
        Ok(Shrink { to, area_begins })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::sizes::{GIB, MIB};

    /// A 237 GB Windows with 120 GB free, as a laptop's might be.
    const PRINTED: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,"Offset":290455552,"Size":254633566208,"SizeMin":61234567168,"SizeRemaining":128849018880}"#;

    /// **The shrink gives up exactly the area, aligned, right after Windows.**
    #[test]
    fn the_shrink_gives_up_the_area_right_after_windows() {
        let volume = WindowsVolume::read(Some(PRINTED)).unwrap();
        assert_eq!(volume.letter.drive(), "C:");
        let shrink = volume.shrink().unwrap();
        assert_eq!(shrink.to % MIB, 0);
        assert!(volume.size - shrink.to >= THE_AREA);
        assert!(volume.size - shrink.to < THE_AREA + MIB);
        assert_eq!(shrink.area_begins, volume.offset + shrink.to);
    }

    /// **Too little free space, or a partition Windows will not shrink that
    /// far, is refused.**
    #[test]
    fn too_little_space_is_refused() {
        let volume = WindowsVolume::read(Some(PRINTED)).unwrap();
        let tight = WindowsVolume {
            free: 16 * GIB,
            ..volume
        };
        assert_eq!(
            tight.shrink(),
            Err(NotEnoughSpace {
                needed: 17 * GIB,
                free: 16 * GIB
            })
        );
        let immovable = WindowsVolume {
            smallest: volume.size - MIB,
            ..volume
        };
        assert!(immovable.shrink().is_err());
    }

    /// **An answer that is not the script's, or does not add up, is no answer.**
    #[test]
    fn an_answer_that_does_not_add_up_is_no_answer() {
        assert_eq!(WindowsVolume::read(None), None);
        assert_eq!(
            WindowsVolume::read(Some(&PRINTED.replace(r#""C""#, r#""CC""#))),
            None
        );
        assert_eq!(
            WindowsVolume::read(Some(&PRINTED.replace("128849018880", "999999999999999"))),
            None
        );
    }
}
