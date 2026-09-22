//! How a run of the installer ended, and the sentence each ending is said in.
//!
//! Three endings. **Staged**: everything is ready and the computer restarts
//! into the environment. **Refused**: nothing was changed — either nothing was
//! ever changed, or every change was put back — for one of the reasons below,
//! each with its own sentence saying so. **Not put back**: preparing failed and
//! undoing it failed too, which is the one ending that cannot say nothing was
//! changed, and instead says exactly what remains.

use alo_strings::{Filling, Word};

use crate::identities::Letter;
use crate::sizes;
use crate::words;

/// How it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// Everything is ready; `restarted` is whether Windows carried out the restart.
    Staged {
        /// Whether the restart was started.
        restarted: bool,
    },
    /// Nothing was changed, for this reason.
    Refused(Refusal),
    /// Preparing did not finish and these could not be put back.
    NotPutBack(Vec<Remains>),
}

/// Why nothing was changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Not started with an administrator's rights.
    NotAnAdministrator,
    /// Something that came with the installer is missing or unreadable.
    Incomplete,
    /// The download is not what this release was built with.
    NotGenuine,
    /// The computer starts the BIOS way.
    NotUefi,
    /// How it starts could not be read.
    StartingNotRead,
    /// Secure Boot is on.
    SecureBootOn,
    /// Whether Secure Boot is on could not be read.
    SecureBootNotRead,
    /// The disks, or where Windows is, could not be read.
    DisksNotRead,
    /// The firmware's list of systems could not be read.
    EntriesNotRead,
    /// The disk Windows is on is not GPT; its shown name.
    WindowsDiskNotSupported(String),
    /// BitLocker is converting the Windows volume.
    BitLockerChanging(Letter),
    /// Not enough free space on the Windows volume.
    NotEnoughSpace {
        /// The Windows volume.
        volume: Letter,
        /// The free space needed, in bytes.
        needed: u64,
        /// The free space it has, in bytes.
        free: u64,
    },
    /// An earlier start left its area or its entry.
    AlreadyStarted,
    /// No empty disk alo OS can be installed onto.
    NoDiskForAloOs,
    /// The person typed nothing.
    NotAgreed,
    /// The person typed something that is not an offered disk's name.
    NotADisksName,
    /// Preparing failed, and every change was put back.
    PutBack,
}

/// One change that could not be put back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remains {
    /// The Windows volume is smaller by the area.
    Smaller(Letter),
    /// The area is still on this disk, by its shown name.
    TheArea(String),
    /// The entry is still among the systems the computer can start.
    TheEntry,
    /// The next start is the installer's.
    TheNextStart,
    /// Fast Startup was turned off at the person's word and not put back.
    FastStartupOff,
}

impl Refusal {
    /// The sentence this refusal is said in, and what fills it.
    #[must_use]
    pub fn said_as(&self) -> (Word, Filling) {
        match self {
            Self::NotAnAdministrator => (words::NOT_AN_ADMINISTRATOR, Filling::nothing()),
            Self::Incomplete => (words::INCOMPLETE, Filling::nothing()),
            Self::NotGenuine => (words::NOT_GENUINE, Filling::nothing()),
            Self::NotUefi => (words::NOT_UEFI, Filling::nothing()),
            Self::StartingNotRead => (words::STARTING_NOT_READ, Filling::nothing()),
            Self::SecureBootOn => (words::SECURE_BOOT_ON, Filling::nothing()),
            Self::SecureBootNotRead => (words::SECURE_BOOT_NOT_READ, Filling::nothing()),
            Self::DisksNotRead => (words::DISKS_NOT_READ, Filling::nothing()),
            Self::EntriesNotRead => (words::ENTRIES_NOT_READ, Filling::nothing()),
            Self::WindowsDiskNotSupported(disk) => (
                words::WINDOWS_DISK_NOT_SUPPORTED,
                Filling::of("disk", disk.as_str()),
            ),
            Self::BitLockerChanging(volume) => (
                words::BITLOCKER_CHANGING,
                Filling::of("volume", volume.drive()),
            ),
            Self::NotEnoughSpace {
                volume,
                needed,
                free,
            } => (
                words::NOT_ENOUGH_SPACE,
                Filling::of("volume", volume.drive())
                    .and("needed", sizes::needed(*needed))
                    .and("free", sizes::had(*free)),
            ),
            Self::AlreadyStarted => (words::ALREADY_STARTED, Filling::nothing()),
            Self::NoDiskForAloOs => (
                words::NO_DISK_FOR_ALO_OS,
                Filling::of("least", sizes::needed(sizes::THE_LEAST_DISK)),
            ),
            Self::NotAgreed => (words::NOT_AGREED, Filling::nothing()),
            Self::NotADisksName => (words::NOT_A_DISKS_NAME, Filling::nothing()),
            Self::PutBack => (words::PUT_BACK, Filling::nothing()),
        }
    }
}

impl Remains {
    /// The sentence this is said in, and what fills it.
    #[must_use]
    pub fn said_as(&self) -> (Word, Filling) {
        match self {
            Self::Smaller(volume) => (
                words::REMAINS_SMALLER,
                Filling::of("volume", volume.drive()).and("area", sizes::needed(sizes::THE_AREA)),
            ),
            Self::TheArea(disk) => (words::REMAINS_THE_AREA, Filling::of("disk", disk.as_str())),
            Self::TheEntry => (words::REMAINS_THE_ENTRY, Filling::nothing()),
            Self::TheNextStart => (words::REMAINS_THE_NEXT_START, Filling::nothing()),
            Self::FastStartupOff => (words::REMAINS_FAST_STARTUP_OFF, Filling::nothing()),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_strings::Strings;

    use super::*;
    use crate::words::{EVERY_REFUSAL, installer_words};

    /// **Every refusal and every remainder is said whole**, with nothing left
    /// unfilled, and every refusal is one of the sentences that says nothing
    /// was changed.
    #[test]
    fn every_ending_is_said_whole() {
        let strings = Strings::of(installer_words().unwrap());
        let c = Letter::of("C").unwrap();
        let refusals = [
            Refusal::NotAnAdministrator,
            Refusal::Incomplete,
            Refusal::NotGenuine,
            Refusal::NotUefi,
            Refusal::StartingNotRead,
            Refusal::SecureBootOn,
            Refusal::SecureBootNotRead,
            Refusal::DisksNotRead,
            Refusal::EntriesNotRead,
            Refusal::WindowsDiskNotSupported("Msft Virtual Disk".to_owned()),
            Refusal::BitLockerChanging(c),
            Refusal::NotEnoughSpace {
                volume: c,
                needed: 17 * sizes::GIB,
                free: sizes::GIB,
            },
            Refusal::AlreadyStarted,
            Refusal::NoDiskForAloOs,
            Refusal::NotAgreed,
            Refusal::NotADisksName,
            Refusal::PutBack,
        ];
        assert_eq!(refusals.len(), EVERY_REFUSAL.len());
        for refusal in refusals {
            let (word, filling) = refusal.said_as();
            assert!(EVERY_REFUSAL.contains(&word), "{refusal:?}");
            let said = strings.say(&word.key(), &filling);
            assert!(said.unfilled().is_empty(), "{refusal:?}: {}", said.text());
            assert!(said.text().contains("nothing was changed"), "{refusal:?}");
        }
        for remains in [
            Remains::Smaller(c),
            Remains::TheArea("Msft Virtual Disk 0".to_owned()),
            Remains::TheEntry,
            Remains::TheNextStart,
        ] {
            let (word, filling) = remains.said_as();
            let said = strings.say(&word.key(), &filling);
            assert!(said.unfilled().is_empty(), "{remains:?}: {}", said.text());
        }
    }
}
