//! Where Windows is, how big it is, and how far Windows says it can shrink.
//!
//! Every number comes from Windows' storage cmdlets: the partition holding
//! `%SystemDrive%`, the smallest size `Get-PartitionSupportedSize` says that
//! partition can be shrunk to (which accounts for files Windows cannot move),
//! and the volume's free space.

use serde::Deserialize;

use crate::identities::{DiskNumber, Letter, PartitionNumber};
use crate::sizes::{THE_AREA, THE_LEAST_DISK, WINDOWS_KEEPS_FREE, aligned};

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

/// The shrink that puts alo OS on the disk Windows is on, beside it.
///
/// Four numbers rather than two, because this road creates two partitions where
/// [`Shrink`] creates one: the installer's own area, and the space alo OS is
/// installed into. Every one of them is a byte offset or a byte count on the
/// Windows disk, so nothing downstream has to work out where the second one
/// goes from the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BesideWindows {
    /// The size the Windows partition is shrunk to.
    pub to: u64,
    /// Where the freed space, and so the installer's area, begins.
    pub area_begins: u64,
    /// Where the space alo OS is installed into begins, after the area.
    pub alo_os_begins: u64,
    /// How much space alo OS is given, which is never less than
    /// [`THE_LEAST_DISK`].
    pub alo_os: u64,
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

    /// The shrink that makes room for the area **and for alo OS itself** on the
    /// disk Windows is on, or why there is not room for it.
    ///
    /// This is the installer plan's task 4: a computer with one disk had no road
    /// at all, because [`crate::deciding`] refuses `NoDiskForAloOs` when no
    /// *other* disk can hold it, and the only shrink there was gave up
    /// [`THE_AREA`] — a gibibyte for the installer's own boot environment, not
    /// room for an operating system.
    ///
    /// **Windows still keeps [`WINDOWS_KEEPS_FREE`].** That constant's own
    /// reason is the reason this road obeys it: *a Windows that cannot update
    /// because alo OS took its space is a Windows made worse by installing alo
    /// OS.* So what is asked of the volume is all three together, and a volume
    /// that cannot give up all three is refused rather than squeezed.
    ///
    /// **alo OS is given what was freed and not a byte more.** The freed region
    /// is [`THE_AREA`] plus [`THE_LEAST_DISK`], rounded by alignment, and alo OS
    /// takes everything after the area — so it always gets at least the least
    /// disk, and Windows keeps whatever it had above what was asked. An
    /// installer that took every free byte would be making a decision about
    /// somebody else's computer that nobody asked it to make.
    ///
    /// # Errors
    /// [`NotEnoughSpace`] when the free space is short of all three together, or
    /// Windows says the partition cannot shrink that far. Its `needed` is what
    /// this road asks, which is larger than [`Self::shrink`]'s — a person told
    /// they are short is told the number for the road they cannot walk.
    pub fn beside_windows(&self) -> Result<BesideWindows, NotEnoughSpace> {
        let for_both = THE_AREA + THE_LEAST_DISK;
        let needed = for_both + WINDOWS_KEEPS_FREE;
        let short = NotEnoughSpace {
            needed,
            free: self.free,
        };
        if self.free < needed {
            return Err(short);
        }
        // Rounded down to a mebibyte for `Self::shrink`'s reason: the area
        // begins where Windows would put a partition.
        let to = aligned(self.size.checked_sub(for_both).ok_or(short)?);
        if to < self.smallest {
            return Err(short);
        }
        let area_begins = self.offset.checked_add(to).ok_or(short)?;
        let alo_os_begins = area_begins.checked_add(THE_AREA).ok_or(short)?;
        // What the shrink actually freed, less the area. Never below
        // `THE_LEAST_DISK`, because `to` gave up both and alignment only ever
        // rounds `to` down, which frees more rather than less.
        let alo_os = self
            .size
            .checked_sub(to)
            .and_then(|freed| freed.checked_sub(THE_AREA))
            .ok_or(short)?;
        if alo_os < THE_LEAST_DISK {
            return Err(short);
        }
        Ok(BesideWindows {
            to,
            area_begins,
            alo_os_begins,
            alo_os,
        })
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

    /// **The same-disk road frees the area and alo OS's own space, in that
    /// order, and leaves Windows what it keeps free.**
    ///
    /// The numbers are checked against each other rather than against constants
    /// written twice: alo OS begins exactly one area after the area does, and
    /// what it is given is exactly what the shrink freed less that area. A test
    /// that restated `THE_AREA + THE_LEAST_DISK` would pass on an
    /// implementation that computed the offsets from the wrong one of them.
    #[test]
    fn beside_windows_frees_the_area_then_alo_os() {
        let volume = WindowsVolume::read(Some(PRINTED)).unwrap();
        let beside = volume.beside_windows().unwrap();

        assert_eq!(beside.to % MIB, 0, "Windows is not left mebibyte-aligned");
        assert_eq!(beside.area_begins, volume.offset + beside.to);
        assert_eq!(
            beside.alo_os_begins,
            beside.area_begins + THE_AREA,
            "alo OS does not begin one area after the area"
        );
        assert_eq!(
            beside.alo_os,
            volume.size - beside.to - THE_AREA,
            "alo OS is not given what the shrink freed less the area"
        );
        assert!(
            beside.alo_os >= THE_LEAST_DISK,
            "alo OS was given {} bytes, less than the least disk",
            beside.alo_os
        );

        // **It gives up more than the other road**, which is the whole point:
        // the old shrink freed a gibibyte for a boot environment and nothing
        // for an operating system.
        let only_the_area = volume.shrink().unwrap();
        assert!(
            beside.to < only_the_area.to,
            "the same-disk road shrank Windows no further than the area-only road"
        );

        // **Windows keeps what it is promised.** What is taken from the volume
        // is the freed region, so what is left free is the rest.
        let taken = volume.size - beside.to;
        assert!(
            volume.free - taken >= WINDOWS_KEEPS_FREE,
            "Windows is left {} bytes free, below what it keeps",
            volume.free - taken
        );
    }

    /// **The machine this road was built for: one 111.8 GB disk with Windows
    /// on it.**
    ///
    /// The testing NUC, as that machine measured itself by hand, elevated and
    /// read-only, at 2026-10-06 03:44:39Z with `Get-PartitionSupportedSize`.
    /// Those are its numbers and nobody in this repository has touched that
    /// machine; they are here because **a road whose only purpose is that
    /// computer should fail in this file if it would refuse that computer**.
    ///
    /// # A fixture, not a prediction
    ///
    /// **This does not promise that the NUC will be offered the road.** It
    /// cannot: `Get-PartitionSupportedSize` computes `SizeMin` from where
    /// unmovable files sit *at the moment it is asked* — the pagefile,
    /// hibernation, System Volume Information, the `$MFT`, shadow copies — so it
    /// moves after a Windows Update, a restore point or a defragment. **The only
    /// check that counts is the live one**: `program.rs` runs that cmdlet on the
    /// night and `smallest` comes straight from it, so nothing here is baked
    /// into the installer.
    ///
    /// What it does establish is narrower and worth having: **the code accepts a
    /// shape a real disk really had**, rather than only the shapes somebody in
    /// this repository would have thought to invent.
    ///
    /// Two drafts of this comment got that wrong in opposite directions. The
    /// first invented a `SizeMin`; the second called `size` and `SizeMin` stable
    /// properties of the partition and put the volatile one on the wrong side of
    /// the line. **Only `size` is stable.**
    ///
    /// `free` is rounded **down** from a figure printed to one decimal place and
    /// is not a measurement at all — an earlier draft carried
    /// `69 * GIB + 716 * MIB`, a byte count reconstructed from the reported
    /// *69.7 GB* and then quoted back as though it came off the disk.
    /// `crate::sizes` says which way to round, and rounding down makes this ask
    /// **less** of the volume than the machine had: passing at 69 means passing
    /// at 69.7.
    #[test]
    fn a_real_one_disk_windows_is_offered_the_road() {
        // One shape a real Windows on one disk had: measured by the testing PC,
        // elevated and read-only, 2026-10-06 03:44:39Z.
        let nuc = WindowsVolume {
            // Measured. The only one of these that does not move.
            size: 118_873_915_392,
            // Measured, and volatile: SizeMin at that moment, 42.13 GiB.
            smallest: 45_234_069_504,
            // Measured: C: is disk 0, partition 3, at this offset.
            offset: 227_540_992,
            // Rounded DOWN from "69.7 GB". Not measured, and it moves.
            free: 69 * GIB,
            ..WindowsVolume::read(Some(PRINTED)).unwrap()
        };

        assert!(
            nuc.beside_windows().is_ok(),
            "the one-disk machine this task exists for was refused: {:?}",
            nuc.beside_windows()
        );
        let beside = nuc.beside_windows().unwrap();
        assert!(beside.alo_os >= THE_LEAST_DISK);
        assert!(nuc.free - (nuc.size - beside.to) >= WINDOWS_KEEPS_FREE);
    }

    /// **A volume with room for the area but not for alo OS is refused**, and
    /// the number it is told is the larger one.
    ///
    /// This is the case that separates the two roads. 20 GB free is enough for
    /// a gibibyte of boot environment on top of what Windows keeps, and nowhere
    /// near enough for an operating system — so `shrink` must still succeed
    /// where `beside_windows` refuses, or a machine that could have taken the
    /// second-disk road would be turned away by this change.
    #[test]
    fn room_for_the_area_but_not_for_alo_os_keeps_the_other_road() {
        let volume = WindowsVolume::read(Some(PRINTED)).unwrap();
        let narrow = WindowsVolume {
            free: 20 * GIB,
            ..volume
        };

        assert!(
            narrow.shrink().is_ok(),
            "the area-only road was broken by the same-disk road being added"
        );
        assert_eq!(
            narrow.beside_windows(),
            Err(NotEnoughSpace {
                needed: THE_AREA + THE_LEAST_DISK + WINDOWS_KEEPS_FREE,
                free: 20 * GIB,
            })
        );
    }

    /// **A partition Windows will not shrink far enough is refused even with
    /// free space to spare.**
    ///
    /// Free space and shrinkability are different questions — a volume can
    /// report 120 GB free and still refuse to move, which is what `SizeMin` is
    /// for — and this road has to ask both.
    #[test]
    fn a_volume_that_will_not_move_is_refused_with_space_to_spare() {
        let volume = WindowsVolume::read(Some(PRINTED)).unwrap();
        let immovable = WindowsVolume {
            smallest: volume.size - MIB,
            ..volume
        };
        assert!(immovable.free > THE_AREA + THE_LEAST_DISK + WINDOWS_KEEPS_FREE);
        assert!(immovable.beside_windows().is_err());
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
