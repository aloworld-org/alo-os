//! Whether the installer offers to go on, from what it found — and if not, why.
//!
//! One function, and the order of its refusals is the order a person most needs
//! to hear them in: that this computer cannot start alo OS at all before that
//! its disk is full. Every refusal here happens before anything was changed,
//! because nothing has been.
//!
//! # Secure Boot
//!
//! On is refused, and so is *could not be found out* (ADR 0033 §4), and a person
//! is never told to change the setting. The environment's loaders are the base's
//! signed ones.
//!
//! **What the refusal waits on is the shim review, which is external and takes
//! months.** That is ADR 0033 §4's own pacing item, and its words are *refused,
//! not disabled — except by the owner, for certification, on the record*: until
//! the review lands the shipped installer refuses to proceed with Secure Boot
//! enabled, the owner disables it in firmware themselves for a certification run,
//! and the evidence ledger records that the machine was certified that way — so
//! *boots on one certified machine* is not read as *with Secure Boot on* until it
//! is.
//!
//! **Corrected 2026-09-28.** This said the refusal stood until the installer
//! plan's task 9 had shown the loaders starting under Secure Boot. Task 9
//! finished on 2026-09-16, and so did 12, 13 and 14 — task 13's status says
//! outright that *with Secure Boot on, the install onto the second disk finishes,
//! and the second disk boots*. So the comment named a condition that has been met
//! for a rule whose real condition has not, and a reader who checked it would
//! conclude the refusal was stale and should be lifted. One did, and stopped only
//! because the ADR said otherwise.

use alo_installing::DiskName;

use crate::bitlocker::BitLocker;
use crate::disks::Standing;
use crate::ended::Refusal;
use crate::found::Found;
use crate::identities::{DiskNumber, PartitionNumber};
use crate::windows_volume::{BesideWindows, Shrink, WindowsVolume};

/// What the installer offers to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The Windows volume that is shrunk.
    pub windows: WindowsVolume,
    /// The shrink.
    pub shrink: Shrink,
    /// The disk Windows is on, as a person is shown it.
    pub windows_disk: String,
    /// Every disk alo OS may be installed onto **beside** Windows.
    pub disks_for_alo_os: Vec<ForAloOs>,
    /// The disk Windows is on, as a target for the road that replaces it, or
    /// [`None`] when this installer cannot name it across the restart.
    ///
    /// Never one of [`Self::disks_for_alo_os`]: those are disks alo OS goes
    /// beside Windows on, and this is the one it goes *instead* of Windows on.
    /// Offering a road whose disk cannot be named would be offering a road that
    /// cannot be walked, so `None` is how the question is not asked at all.
    pub the_windows_disk: Option<ForAloOs>,
    /// The shrink that puts alo OS **on the disk Windows is on, beside it**, or
    /// [`None`] when that volume cannot give up the area, alo OS's own space and
    /// what Windows keeps free, all three.
    ///
    /// This is the third road and the installer plan's task 4. The other two
    /// need a second disk ([`Self::disks_for_alo_os`]) or give up Windows
    /// entirely ([`Self::the_windows_disk`]); a computer with one disk had
    /// neither, and was refused outright.
    ///
    /// **Present does not mean chosen.** It is offered beside the others, and a
    /// machine with a second disk will usually have both — which road is walked
    /// is a person's answer, not this function's.
    pub beside_windows: Option<BesideWindows>,
    /// The partition on the Windows disk that the firmware starts from,
    /// where that disk has one.
    ///
    /// **Only the road that keeps Windows needs it.** alo OS's loader goes
    /// into a directory of its own inside that partition rather than into one
    /// of alo OS's own, because there is only one start-up area on a disk and
    /// Windows already has it. The boot environment is told which partition it
    /// is, by number, rather than left to go looking on a disk it is keeping.
    ///
    /// A disk with none is a machine that does not start the way this
    /// installer believes it does, and [`Self::beside_windows`] is [`None`]
    /// there: a road whose start-up area cannot be named is a road that cannot
    /// be walked, so it is not offered.
    pub the_start_up_area: Option<PartitionNumber>,
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
    /// What Windows gives up, where this is the disk Windows is on and alo OS
    /// goes **beside** it rather than onto an empty disk.
    ///
    /// [`None`] is the older road and still the common one: an empty disk alo
    /// OS takes whole, where nothing of anybody's is moved. [`Some`] is task 4
    /// of the installer plan — the disk is repartitioned, Windows is shrunk to
    /// [`BesideWindows::to`], and **nothing is erased**.
    ///
    /// It is carried on the disk rather than beside it so that **the thing a
    /// person typed the name of is the thing that says which road this is.** A
    /// road chosen by one answer and a target chosen by another are two answers
    /// that can disagree, and `crate::consent`'s whole shape is that there is
    /// no second question.
    pub beside: Option<BesideWindows>,
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
    let mut disks_for_alo_os: Vec<ForAloOs> = disks
        .every()
        .filter_map(|disk| match disk.standing(windows.disk) {
            Standing::ForAloOs(after_the_restart) => Some(ForAloOs {
                number: disk.number(),
                shown: disks.shown_name(disk),
                after_the_restart,
                // An empty disk alo OS takes whole: nothing is shrunk, because
                // there is nothing on it to keep.
                beside: None,
            }),
            Standing::HoldsWindows | Standing::InUse | Standing::TooSmall | Standing::NotUsable => {
                None
            }
        })
        .collect();
    // **The road a one-disk computer has**, which is the installer plan's task
    // 4 and the promise `the-windows-installer-program.md` records as owed:
    // *one-disk computers are refused until task 4*. Asked of the volume whether
    // or not another disk exists, because a person with a second disk may still
    // want to keep it for something else — both roads are *offered* here and
    // chosen later, which is how `the_windows_disk` is already handled.
    //
    // **And gated on the start-up area being nameable**, which is the
    // difference between a road that is offered and a road that can be
    // walked. This is the fault family `docs/misreadings/` keeps: a check
    // whose inputs are less specific than its question. *Can this volume give
    // up the space* is not the whole question; *and can the loader be put
    // somewhere the firmware will look* is the rest of it.
    let the_start_up_area = windows_disk.the_start_up_area();
    let beside_windows = the_start_up_area.and_then(|_| windows.beside_windows().ok());

    // **The disk Windows is on is now a disk alo OS may go on**, and this is
    // the last term of the owner's ruling of 2026-10-06: *the refusal that
    // keeps the Windows disk out is flipped last, when the whole road exists
    // and not before.*
    //
    // **What makes the road whole**, each half walked or tested rather than
    // argued:
    //
    // - the boot environment installs into a partition rather than over a whole
    //   disk, and refuses any partition not labelled `ALO-ROOT`
    //   (`alo_installing::may_keep_what_is_there`);
    // - the Windows half shrinks the volume by the area and alo OS's space
    //   together, makes the space, labels it, and stages a command line naming
    //   the disk, that partition and Windows' own start-up area
    //   (`crate::staging`);
    // - and the person is asked which road they want and told what it costs
    //   before they consent to it, including that **Windows does not get this
    //   space back** while alo OS is there (`crate::where_alo_os_goes`).
    //
    // **Why the refusal was worth this much care.** On 2026-10-05 it was
    // flipped on its own, and the run reached the prompt and died at
    // `NotADisksName`: a person sent from a clear refusal to a question with no
    // answer. A refusal is a worse thing to remove than it looks, because what
    // replaces it is whatever happens next — and for five days what replaced
    // this one was nothing.
    //
    // **And the start-up area is why this is one line rather than a condition.**
    // `beside_windows` is already `None` unless the volume can give up the space
    // *and* the firmware's own partition can be named, so a disk that cannot
    // walk the road is never pushed here. The road opens exactly where it can
    // be walked.
    if let Some(beside) = beside_windows {
        // `after_the_restart` is the name the boot environment waits for, and a
        // disk it cannot name is a disk that would be agreed to and then not
        // found. `None` here leaves the road closed on this machine rather than
        // offering a disk nothing can point at.
        if let Some(after_the_restart) = windows_disk.after_the_restart() {
            disks_for_alo_os.push(ForAloOs {
                number: windows_disk.number(),
                shown: disks.shown_name(windows_disk),
                after_the_restart,
                beside: Some(beside),
            });
        }
    }

    // **Still refused where there is no road at all.** Nothing above lowers
    // this: a machine with no empty disk big enough *and* no beside road — no
    // space, or no start-up area to name — reaches here with an empty list and
    // is told so, which is the sentence it has always been told.
    if disks_for_alo_os.is_empty() {
        return Err(Refusal::NoDiskForAloOs);
    }
    let the_windows_disk = windows_disk
        .after_the_restart()
        .map(|after_the_restart| ForAloOs {
            number: windows_disk.number(),
            shown: disks.shown_name(windows_disk),
            after_the_restart,
            // The road that replaces Windows takes the disk whole; nothing is
            // shrunk and nothing is kept, which is what makes it the other road.
            beside: None,
        });
    Ok(Offer {
        windows,
        shrink,
        windows_disk: disks.shown_name(windows_disk),
        disks_for_alo_os,
        the_windows_disk,
        beside_windows,
        the_start_up_area,
    })
}
