//! *Remove alo OS*, from inside Windows.
//!
//! The installer plan's task 4: *remove alo OS exists as a documented, tested
//! road back — the partition freed, the boot entry gone, Windows as it was —
//! because a person who can install from a download must be able to uninstall
//! from one.* This is that road, and it is the same program the person already
//! ran, started with [`THE_REMOVALS_WORD`].
//!
//! # What it removes, and what it cannot touch
//!
//! This installer only ever puts alo OS on **a disk of its own** with nothing
//! on it (`crate::disks`), so removing alo OS is: the firmware's entry for it,
//! and then that disk — every partition of it, so the space is free again. The
//! disk it erases is found by measurement rather than by memory: the one disk
//! that is not Windows', that carries a partition of the type the image gives
//! alo OS's own, and that carries no partition of a type the image does not
//! make (`crate::disks::THE_IMAGES_PARTITION_TYPES`). It is the types and not
//! the labels because Windows reports no label at all for a filesystem it
//! cannot read, and alo OS's own it cannot. Nothing that fails that test is
//! erased, and the sentence says the disk's name before anything is done.
//!
//! **Windows is never touched**: not its partition, not its start partition,
//! not its own entry. The only thing this program removes from the Windows
//! disk is the copy of itself the install left there, and its shortcuts.
//!
//! # The order is chosen so that a failure destroys nothing
//!
//! The entry goes first. It is the reversible half — a computer whose firmware
//! no longer lists alo OS still has alo OS on its disk, and the installer can
//! be run again — so if it will not go, nothing has been erased and the person
//! is told. The disk is erased only after the entry is gone, and the typed
//! consent comes before either.
//!
//! # The consent is the disk's name, typed
//!
//! ADR 0023 §1: a typed consent, never a checkbox, for anything destructive.
//! Removing alo OS erases a whole disk, so the person agrees the way they
//! agreed to the install — by typing that disk's name (`crate::consent`).

use alo_strings::{Filling, Strings, Word};

use crate::disks::Disks;
use crate::entries;
use crate::machine::TheMachine;
use crate::program::Program;
use crate::windows_volume::WindowsVolume;
use crate::words;

/// The argument that makes this program the removal rather than the installer.
///
/// Not a translated word: it is written into the shortcut the installer makes,
/// read by this program alone, and never typed by a person.
pub const THE_REMOVALS_WORD: &str = "remove-alo-os";

/// How the removal ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Removed {
    /// alo OS is gone: the firmware no longer lists it, and its disk is empty.
    Gone {
        /// The disk that was erased, as the person was shown it.
        disk: String,
        /// Whether the copy the install left in Windows went too.
        the_copy_went: bool,
    },
    /// The person did not agree, and nothing was changed.
    NotAgreed,
    /// The firmware lists no entry named alo OS, so this computer does not
    /// start alo OS and nothing here is the thing to remove.
    NotThere,
    /// No disk of this computer is alo OS's own, so there is nothing to erase.
    NotFound,
    /// Something this needs to read did not answer.
    NotRead,
    /// The entry would not go, and **nothing was erased**.
    EntryNotRemoved,
    /// The entry is gone and the disk would not be erased.
    DiskNotCleared,
}

/// Offer to remove alo OS, and do it if the person types the disk's name.
pub fn remove_alo_os(machine: &mut impl TheMachine, strings: &Strings) -> Removed {
    say(
        machine,
        strings,
        words::REMOVE_STARTING,
        &Filling::nothing(),
    );

    let Some(printed) = read(machine, &Program::ListingTheStartEntries) else {
        say(
            machine,
            strings,
            words::REMOVE_NOT_READ,
            &Filling::nothing(),
        );
        return Removed::NotRead;
    };
    let Some(entry) = entries::the_one_named_alo_os(&printed) else {
        say(
            machine,
            strings,
            words::REMOVE_NOT_THERE,
            &Filling::nothing(),
        );
        return Removed::NotThere;
    };

    let windows = WindowsVolume::read(read(machine, &Program::ReadingTheWindowsVolume).as_deref());
    let disks = Disks::read(read(machine, &Program::ListingTheDisks).as_deref());
    let (Some(windows), Some(disks)) = (windows, disks) else {
        say(
            machine,
            strings,
            words::REMOVE_NOT_READ,
            &Filling::nothing(),
        );
        return Removed::NotRead;
    };
    let Some(disk) = disks.the_one_alo_os_is_on(windows.disk) else {
        say(
            machine,
            strings,
            words::REMOVE_NOT_FOUND,
            &Filling::nothing(),
        );
        return Removed::NotFound;
    };
    let shown = disks.shown_name(disk);
    let number = disk.number();

    say(
        machine,
        strings,
        words::REMOVE_WILL_ERASE,
        &Filling::of("disk", shown.clone()),
    );
    let typed = machine.ask(&strings.say(
        &words::REMOVE_TYPE_THE_DISKS_NAME.key(),
        &Filling::nothing(),
    ));
    if !names_the_disk(&typed, &shown) {
        say(
            machine,
            strings,
            words::REMOVE_NOT_AGREED,
            &Filling::nothing(),
        );
        return Removed::NotAgreed;
    }

    // The entry first: while it is there, nothing has been erased and the
    // installer can simply be run again.
    if read(machine, &Program::RemovingTheEntry { entry }).is_none() {
        say(
            machine,
            strings,
            words::REMOVE_ENTRY_NOT_REMOVED,
            &Filling::nothing(),
        );
        return Removed::EntryNotRemoved;
    }
    // A one-time next start that points at what is being removed would leave
    // the computer starting nothing on purpose.
    let _forgotten = read(machine, &Program::ForgettingTheNextStart);

    say(
        machine,
        strings,
        words::REMOVE_ERASING,
        &Filling::of("disk", shown.clone()),
    );
    let cleared = read(machine, &Program::ClearingTheDiskAloOsIsOn { disk: number })
        .is_some_and(|printed| printed.to_uppercase().contains(THE_DISK_IS_EMPTY));
    if !cleared {
        say(
            machine,
            strings,
            words::REMOVE_DISK_NOT_CLEARED,
            &Filling::nothing(),
        );
        return Removed::DiskNotCleared;
    }

    // The copy of this program, and its shortcuts. The copy cannot always go:
    // when this *is* the copy, Windows will not delete a running program, and
    // that is said rather than hidden.
    let the_copy_went = read(machine, &Program::RemovingWhatWasLeft).is_some();
    say(
        machine,
        strings,
        if the_copy_went {
            words::REMOVE_GONE
        } else {
            words::REMOVE_GONE_BUT_THE_COPY_STAYS
        },
        &Filling::of("disk", shown.clone()),
    );
    Removed::Gone {
        disk: shown,
        the_copy_went,
    }
}

/// What Windows calls a disk with no partition table at all: what the disk
/// alo OS was on reads back as once it has been erased.
const THE_DISK_IS_EMPTY: &str = "RAW";

/// Whether what the person typed is that disk's name — read forgivingly in
/// form and never in substance, exactly as `crate::consent` reads it.
fn names_the_disk(typed: &str, shown: &str) -> bool {
    let normalised = |written: &str| {
        written
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let typed = normalised(typed);
    !typed.is_empty() && typed == normalised(shown)
}

/// What a program printed, when it ran and succeeded.
fn read(machine: &mut impl TheMachine, program: &Program) -> Option<String> {
    machine
        .run(program)
        .ok()
        .filter(|ran| ran.succeeded)
        .map(|ran| ran.printed)
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}
