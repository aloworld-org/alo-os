//! What the install leaves behind, put right — after alo OS is on the disk and
//! before the computer restarts.
//!
//! Measured on the installer's own road, 2026-09-22 (`docs/quirks.md`): once
//! `bootc install` has finished, the firmware's first entry is the one its
//! boot-loader updater made, named **Fedora**, and the installer's staging
//! entry and its area are still there. A person who has just installed alo OS
//! beside their Windows should see *alo OS* in their firmware's own menu, with
//! Windows directly behind it
//! ([ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! term 1), and no leftover gigabyte on the disk Windows is on.
//!
//! # Three acts, and none of them undoes the install
//!
//! 1. **Name the entry.** The firmware's own tool cannot rename one, so the
//!    entry is made again with the same file on the same partition and the
//!    right name, and the old one is removed.
//! 2. **Order them.** alo OS first, Windows directly behind it, everything
//!    else after them in the order the firmware already had.
//! 3. **Take the staging area away.** The partition the installer made on the
//!    Windows disk, found by the label the installer gave it and by nothing
//!    else, and removed from the partition table. Windows is not grown back
//!    into the space: growing a filesystem nobody asked us to touch is a
//!    bigger act than leaving a gigabyte free, and *remove alo OS* is where
//!    that question belongs.
//!
//! **alo OS is installed before any of this runs, and stays installed if all
//! of it fails.** Each act says what it did; a failure is noted where a
//! technician reads it and said in the person's own words as something left
//! behind, and the machine still restarts into alo OS.

use std::path::PathBuf;

use alo_strings::{Filling, Strings};

use crate::disk::DiskName;
use crate::disks::Disks;
use crate::entries::{Entries, THE_ENTRYS_NAME};
use crate::machine::{STILL_EVERY, TheMachine};
use crate::program::{Program, Ran};
use crate::words;

/// What tidying up left the machine as.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tidied {
    /// Whether the entry for alo OS is named alo OS.
    pub named: bool,
    /// Whether alo OS starts first, with Windows directly behind it.
    pub ordered: bool,
    /// Whether the installer's area is gone.
    pub area_removed: bool,
}

impl Tidied {
    /// Whether everything that had to be done was done.
    #[must_use]
    pub const fn whole(&self) -> bool {
        self.named && self.ordered && self.area_removed
    }
}

/// Put right what the install leaves behind, and say each part of it.
pub fn tidy_up(
    machine: &mut impl TheMachine,
    strings: &Strings,
    installed_on: &DiskName,
) -> Tidied {
    say(machine, strings, words::TIDYING);
    let mut tidied = Tidied::default();

    let listed = ran(machine, &Program::ListingTheStartEntries, strings);
    let Some(listed) = listed else {
        say(machine, strings, words::TIDY_ENTRIES_NOT_READ);
        return tidied;
    };
    let entries = Entries::read(&listed.printed);
    let Some(installed) = entries.the_installed_system().cloned() else {
        say(machine, strings, words::TIDY_ENTRIES_NOT_READ);
        return tidied;
    };

    // 1. The name. An entry that already carries it is left exactly as it is.
    let mut alo_os = installed.number.clone();
    if installed.named == THE_ENTRYS_NAME {
        tidied.named = true;
    } else if let Some(partition) = installed.partition {
        let disk: PathBuf = installed_on.path();
        let made = ran(
            machine,
            &Program::NamingTheEntry { disk, partition },
            strings,
        );
        if let Some(made) = made {
            // The tool prints the entry it made, and the number in that line
            // is the entry that now has to be ordered and the one the old must
            // not be confused with.
            if let Some(new) = crate::entries::Entry::read(made.printed.trim()) {
                alo_os = new.number;
            }
            tidied.named = ran(
                machine,
                &Program::RemovingTheEntry {
                    number: installed.number.clone(),
                },
                strings,
            )
            .is_some();
        }
    }

    // 2. The order.
    let windows = entries.windows().map(|entry| entry.number.clone());
    let order = entries.with_alo_os_first(&alo_os, windows.as_deref());
    tidied.ordered = ran(machine, &Program::OrderingTheEntries { order }, strings).is_some();

    // 3. The area. A machine with none is a machine with nothing to remove,
    //    which is the whole of this act being done.
    let area = ran(machine, &Program::ListingTheDisks, strings)
        .and_then(|listed| Disks::read(&listed.printed).ok())
        .and_then(|disks| disks.the_staging_area());
    tidied.area_removed = match area {
        None => true,
        Some((disk, partition)) => ran(
            machine,
            &Program::RemovingTheArea { disk, partition },
            strings,
        )
        .is_some(),
    };

    say(
        machine,
        strings,
        if tidied.whole() {
            words::TIDIED
        } else {
            words::TIDY_NOT_WHOLE
        },
    );
    tidied
}

/// What a program did, when it ran and succeeded; noted either way.
fn ran(machine: &mut impl TheMachine, program: &Program, strings: &Strings) -> Option<Ran> {
    let still = strings.say(&words::TIDYING.key(), &Filling::nothing());
    match machine.run(program, &still, STILL_EVERY) {
        Ok(ran) if ran.succeeded => Some(ran),
        Ok(ran) => {
            machine.note(&format!("{program:?}: {}", ran.complained.trim()));
            None
        }
        Err(why) => {
            machine.note(&format!("{program:?}: {why}"));
            None
        }
    }
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: alo_strings::Word) {
    machine.say(&strings.say(&word.key(), &Filling::nothing()));
}
