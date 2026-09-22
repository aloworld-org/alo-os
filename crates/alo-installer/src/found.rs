//! What the installer found out about the computer, and how each finding is said.
//!
//! Every check is said, whatever it found — a person is told their computer
//! has no TPM as plainly as that it has one, and *could not be found out* is
//! said as that rather than skipped. Nothing here decides anything; that is
//! `crate::deciding`.

use alo_strings::{Filling, Said, Strings, Word};

use crate::bitlocker::BitLocker;
use crate::disks::{Disks, Standing};
use crate::fast_startup::FastStartup;
use crate::memory::MADE_FOR;
use crate::security_chip::SecurityChip;
use crate::sizes;
use crate::starting::Starting;
use crate::windows_volume::WindowsVolume;
use crate::words;

/// Everything the checks found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// UEFI, and Secure Boot.
    pub starting: Starting,
    /// The TPM.
    pub chip: SecurityChip,
    /// BitLocker on the Windows volume.
    pub bitlocker: BitLocker,
    /// Installed memory, in bytes.
    pub memory: Option<u64>,
    /// The Windows volume.
    pub windows: Option<WindowsVolume>,
    /// Every disk.
    pub disks: Option<Disks>,
    /// Whether the firmware already lists an entry named alo OS; [`None`] when
    /// the list could not be read.
    pub an_entry_is_named_alo_os: Option<bool>,
    /// Whether Windows' Fast Startup is on.
    pub fast_startup: FastStartup,
}

impl Found {
    /// Every finding, as sentences, in the order a person reads them.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let say = |word: Word, filling: Filling| strings.say(&word.key(), &filling);
        let mut said = Vec::new();

        said.push(match self.starting.uefi {
            Some(true) => say(words::FOUND_UEFI, Filling::nothing()),
            Some(false) => say(words::FOUND_NOT_UEFI, Filling::nothing()),
            None => say(words::FOUND_STARTING_NOT_READ, Filling::nothing()),
        });
        if self.starting.uefi != Some(false) {
            said.push(match self.starting.secure_boot {
                Some(true) => say(words::FOUND_SECURE_BOOT_ON, Filling::nothing()),
                Some(false) => say(words::FOUND_SECURE_BOOT_OFF, Filling::nothing()),
                None => say(words::FOUND_SECURE_BOOT_NOT_READ, Filling::nothing()),
            });
        }
        said.push(say(
            match self.chip {
                SecurityChip::Ready => words::FOUND_TPM_READY,
                SecurityChip::NotReady => words::FOUND_TPM_NOT_READY,
                SecurityChip::Absent => words::FOUND_NO_TPM,
                SecurityChip::NotRead => words::FOUND_TPM_NOT_READ,
            },
            Filling::nothing(),
        ));

        // Both of these name the drive Windows is on, and a drive letter this
        // installer did not read is not one it says; with no volume read, the
        // refusal that follows says the disks could not be read.
        if let Some(windows) = self.windows {
            said.push(say(
                match self.bitlocker {
                    BitLocker::On => words::FOUND_BITLOCKER_ON,
                    BitLocker::Off => words::FOUND_BITLOCKER_OFF,
                    BitLocker::Changing => words::FOUND_BITLOCKER_CHANGING,
                    BitLocker::NotRead => words::FOUND_BITLOCKER_NOT_READ,
                },
                Filling::of("volume", windows.letter.drive()),
            ));
            said.push(say(
                words::FOUND_SPACE,
                Filling::of("volume", windows.letter.drive())
                    .and("free", sizes::had(windows.free))
                    .and("size", sizes::had(windows.size)),
            ));
        }

        // Said whichever it is, like every other finding; what is done about
        // it is the person's answer to a question, and only when it is on
        // (ADR 0064 term 9, `crate::asking`).
        said.push(say(
            match self.fast_startup {
                FastStartup::On => words::FOUND_FAST_STARTUP_ON,
                FastStartup::Off => words::FOUND_FAST_STARTUP_OFF,
                FastStartup::NotRead => words::FOUND_FAST_STARTUP_NOT_READ,
            },
            Filling::nothing(),
        ));

        said.push(match self.memory {
            Some(bytes) if bytes >= MADE_FOR => say(
                words::FOUND_MEMORY,
                Filling::of("memory", rounded_memory(bytes)),
            ),
            Some(bytes) => say(
                words::FOUND_LITTLE_MEMORY,
                Filling::of("memory", rounded_memory(bytes)),
            ),
            None => say(words::FOUND_MEMORY_NOT_READ, Filling::nothing()),
        });

        if let (Some(disks), Some(windows)) = (&self.disks, self.windows) {
            for disk in disks.every() {
                let filling = Filling::of("disk", disks.shown_name(disk))
                    .and("size", sizes::had(disk.size()))
                    .and("least", sizes::needed(sizes::THE_LEAST_DISK));
                let word = match disk.standing(windows.disk) {
                    Standing::HoldsWindows => words::FOUND_WINDOWS_DISK,
                    Standing::ForAloOs(_) => words::FOUND_A_DISK_FOR_ALO_OS,
                    Standing::InUse => words::FOUND_A_DISK_IN_USE,
                    Standing::TooSmall => words::FOUND_A_DISK_TOO_SMALL,
                    Standing::NotUsable => words::FOUND_A_DISK_NOT_USABLE,
                };
                said.push(say(word, filling));
            }
        }
        said
    }
}

/// Memory as a person knows it: a module of 16 GB reports a little less once
/// the firmware has kept its share, and a computer sold with 16 GB is said to
/// have 16, so the number is rounded to the nearest gigabyte.
fn rounded_memory(bytes: u64) -> String {
    ((bytes + sizes::GIB / 2) / sizes::GIB).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 16 GB module, as Windows reports it, is 16 GB.
    #[test]
    fn memory_is_said_as_it_was_sold() {
        assert_eq!(rounded_memory(17_062_334_464), "16");
        assert_eq!(rounded_memory(16 * sizes::GIB), "16");
    }
}
