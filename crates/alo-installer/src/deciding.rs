//! Whether the installer offers to go on, from what it found — and if not, why.
//!
//! One function, and the order of its refusals is the order a person most needs
//! to hear them in: that this computer cannot start alo OS at all before that
//! its disk is full. Every refusal here happens before anything was changed,
//! because nothing has been.
//!
//! # Secure Boot
//!
//! On is refused, and so is *could not be found out* (ADR 0033 §4). The
//! environment's loaders are the base's signed ones, but the installer plan's
//! task 9 has not yet shown them starting under Secure Boot, and a person is
//! never told to change the setting — so until it has, the only honest answer
//! is to stop, say why, and change nothing.

use alo_installing::DiskName;

use crate::bitlocker::BitLocker;
use crate::disks::Standing;
use crate::ended::Refusal;
use crate::found::Found;
use crate::identities::DiskNumber;
use crate::windows_volume::{Shrink, WindowsVolume};

/// What the installer offers to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The Windows volume that is shrunk.
    pub windows: WindowsVolume,
    /// The shrink.
    pub shrink: Shrink,
    /// The disk Windows is on, as a person is shown it.
    pub windows_disk: String,
    /// Every disk alo OS may be installed onto.
    pub disks_for_alo_os: Vec<ForAloOs>,
}

/// One disk alo OS may be installed onto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForAloOs {
    /// Windows' number for it.
    pub number: DiskNumber,
    /// Its name as a person is shown it, and types to agree.
    pub shown: String,
    /// Its name after the restart.
    pub after_the_restart: DiskName,
}

/// The offer, or the first reason there is none.
///
/// # Errors
/// The [`Refusal`] a person needs to hear first.
pub fn decide(found: &Found) -> Result<Offer, Refusal> {
    match found.starting.uefi {
        Some(true) => {}
        Some(false) => return Err(Refusal::NotUefi),
        None => return Err(Refusal::StartingNotRead),
    }
    match found.starting.secure_boot {
        Some(false) => {}
        Some(true) => return Err(Refusal::SecureBootOn),
        None => return Err(Refusal::SecureBootNotRead),
    }
    let (Some(windows), Some(disks)) = (found.windows, &found.disks) else {
        return Err(Refusal::DisksNotRead);
    };
    let windows_disk = disks.numbered(windows.disk).ok_or(Refusal::DisksNotRead)?;
    if !windows_disk.is_gpt() {
        return Err(Refusal::WindowsDiskNotSupported(
            disks.shown_name(windows_disk),
        ));
    }
    let an_entry_is_named_alo_os = found
        .an_entry_is_named_alo_os
        .ok_or(Refusal::EntriesNotRead)?;
    if an_entry_is_named_alo_os || disks.hold_an_installer_area() {
        return Err(Refusal::AlreadyStarted);
    }
    if found.bitlocker == BitLocker::Changing {
        return Err(Refusal::BitLockerChanging(windows.letter));
    }
    let shrink = windows.shrink().map_err(|short| Refusal::NotEnoughSpace {
        volume: windows.letter,
        needed: short.needed,
        free: short.free,
    })?;
    let disks_for_alo_os: Vec<ForAloOs> = disks
        .every()
        .filter_map(|disk| match disk.standing(windows.disk) {
            Standing::ForAloOs(after_the_restart) => Some(ForAloOs {
                number: disk.number(),
                shown: disks.shown_name(disk),
                after_the_restart,
            }),
            Standing::HoldsWindows | Standing::InUse | Standing::TooSmall | Standing::NotUsable => {
                None
            }
        })
        .collect();
    if disks_for_alo_os.is_empty() {
        return Err(Refusal::NoDiskForAloOs);
    }
    Ok(Offer {
        windows,
        shrink,
        windows_disk: disks.shown_name(windows_disk),
        disks_for_alo_os,
    })
}
