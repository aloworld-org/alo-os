//! Whether this computer may take the road that erases its disk.
//!
//! Task 7 names three refusals. One of them,
//! [`Refusal::NotTheWord`](crate::ended::Refusal::NotTheWord), is about what a
//! person typed and lives with the consent that reads it
//! ([`crate::erasing_consent`]). The other two are about the machine, and they
//! are decided here, before the question is ever asked.
//!
//! **Asked before, not after.** A person should never be told what replacing
//! costs, shown their own files, and asked to type the word, only to be refused
//! afterwards for something the installer knew from the start. Everything here
//! is read during [`crate::checking`], long before anything is said about
//! erasing.
//!
//! # The order is deliberate: the refusal a person cannot fix comes first
//!
//! A person whose Windows volume is protected can decrypt it and come back. A
//! person whose computer has one disk and nothing else to start from cannot
//! change that by preparing better. So the unfixable one is checked first, and
//! somebody in that position learns it immediately instead of decrypting a whole
//! disk and then hitting a wall.

use alo_installing::Replacing;

use crate::bitlocker::BitLocker;
use crate::deciding::Offer;
use crate::ended::Refusal;
use crate::found::Found;

/// The fewest disks a machine may have and still be offered this road.
///
/// Two: the one alo OS replaces, and anything at all to start from afterwards.
/// A recovery drive a person made is a disk, an external drive is a disk, and a
/// second internal drive is a disk — this does not try to tell them apart,
/// because *something else exists* is the whole question.
const AND_SOMETHING_TO_COME_BACK_TO: usize = 2;

/// Whether this machine may be offered the road that erases its disk.
///
/// # Errors
/// [`Refusal::NoWayBackAtAll`] when the machine has one disk and nothing else it
/// could start from, and [`Refusal::BitLockerNotConfirmed`] when the Windows
/// volume is protected.
pub fn may_replace(found: &Found, offer: &Offer) -> Result<Replacing, Refusal> {
    let how_many = found
        .disks
        .as_ref()
        .map_or(0, |disks| disks.every().count());
    if how_many < AND_SOMETHING_TO_COME_BACK_TO {
        return Err(Refusal::NoWayBackAtAll);
    }

    // **Anything but `Off` is a refusal, including `NotRead`.** Task 7 asks for
    // a refusal when the installer *cannot confirm* the volume is unlocked and
    // backed up — and it can never confirm the second half at all: nothing
    // Windows will tell this program says whether what unlocks the disk is
    // written down anywhere else. So a protected volume is refused on this road,
    // full stop, and "Windows did not say" is not a reason to erase somebody's
    // disk.
    //
    // `Changing` is already refused on every road by `crate::deciding`, so it
    // cannot arrive here; it is matched anyway rather than trusting a caller's
    // ordering to keep a destructive road safe.
    if found.bitlocker != BitLocker::Off {
        return Err(Refusal::BitLockerNotConfirmed(offer.windows_disk.clone()));
    }

    Ok(Replacing::TheSystemOnTheDisk)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::disks::Disks;
    // `disks.rs`' own description of what Windows prints, reused rather than
    // copied, so this crate holds one idea of what a machine's disks look like.
    use crate::disks::tests::PRINTED as SEVERAL_DISKS;
    use crate::fast_startup::FastStartup;
    use crate::security_chip::SecurityChip;
    use crate::starting::Starting;
    use crate::windows_volume::WindowsVolume;

    /// A machine with one disk and nothing else to start from.
    const ONE_DISK: &str = r#"[
      {"Number":0,"FriendlyName":"Samsung SSD 870 EVO","SerialNumber":"S5Y1NJ0R123456",
       "BusType":"SATA","UniqueId":"","Size":274877906944,"PartitionStyle":"GPT",
       "IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Label":""},
                     {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Windows"}]}
    ]"#;

    /// What Windows prints for the Windows volume.
    const THE_VOLUME: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,
      "Offset":123731968,"Size":274341593088,"SizeMin":34341593088,
      "SizeRemaining":128849018880}"#;

    /// A machine that was read, with this encryption and these disks.
    fn found(bitlocker: BitLocker, disks: &str) -> Found {
        Found {
            starting: Starting {
                uefi: Some(true),
                secure_boot: Some(false),
            },
            chip: SecurityChip::Ready,
            bitlocker,
            memory: Some(16 * 1024 * 1024 * 1024),
            windows: WindowsVolume::read(Some(THE_VOLUME)),
            disks: Disks::read(Some(disks)),
            an_entry_is_named_alo_os: Some(false),
            fast_startup: FastStartup::Off,
        }
    }

    /// The offer made about that machine.
    fn offer() -> Offer {
        Offer {
            windows: WindowsVolume::read(Some(THE_VOLUME)).unwrap(),
            shrink: WindowsVolume::read(Some(THE_VOLUME))
                .unwrap()
                .shrink()
                .unwrap(),
            windows_disk: "Samsung SSD 870 EVO".to_owned(),
            disks_for_alo_os: Vec::new(),
        }
    }

    /// **A machine with somewhere to come back to, and no protection, may take
    /// this road.**
    #[test]
    fn several_disks_and_no_protection_may_take_this_road() {
        assert_eq!(
            may_replace(&found(BitLocker::Off, SEVERAL_DISKS), &offer()),
            Ok(Replacing::TheSystemOnTheDisk)
        );
    }

    /// **One disk is refused, and it is refused before anything else** — the one
    /// a person cannot fix by preparing better.
    ///
    /// Checked with the volume protected too, which would also be a refusal:
    /// the answer must still be the unfixable one.
    #[test]
    fn one_disk_is_refused_first_because_it_cannot_be_fixed() {
        for bitlocker in [BitLocker::Off, BitLocker::On, BitLocker::NotRead] {
            assert_eq!(
                may_replace(&found(bitlocker, ONE_DISK), &offer()),
                Err(Refusal::NoWayBackAtAll),
                "{bitlocker:?}"
            );
        }
    }

    /// **Anything but "not encrypted" is refused, including "Windows did not
    /// say".**
    ///
    /// The installer can never confirm that what unlocks the disk is kept
    /// anywhere else, so a protected volume does not take this road — and an
    /// unknown state is not a reason to erase somebody's disk.
    #[test]
    fn anything_but_unencrypted_is_refused_and_names_the_disk() {
        for bitlocker in [BitLocker::On, BitLocker::Changing, BitLocker::NotRead] {
            assert_eq!(
                may_replace(&found(bitlocker, SEVERAL_DISKS), &offer()),
                Err(Refusal::BitLockerNotConfirmed(
                    "Samsung SSD 870 EVO".to_owned()
                )),
                "{bitlocker:?}"
            );
        }
    }

    /// **Disks that could not be read are no disks**, so the road is refused.
    ///
    /// A machine this installer could not count is not a machine it may erase.
    #[test]
    fn disks_that_could_not_be_read_are_refused() {
        assert_eq!(
            may_replace(&found(BitLocker::Off, "not json"), &offer()),
            Err(Refusal::NoWayBackAtAll)
        );
    }
}
