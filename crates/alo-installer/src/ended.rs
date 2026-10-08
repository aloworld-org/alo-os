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
    /// **Replacing only.** The disk is protected by BitLocker and this installer
    /// cannot confirm both that it is unlocked and that its recovery key is kept
    /// somewhere other than on itself. Erasing it would destroy the only copy.
    ///
    /// Two conditions rather than one, because a volume can be unlocked and
    /// still hold the only copy of the key that unlocks it. Keeping Windows
    /// beside alo OS never reaches this: nothing is erased on that road.
    BitLockerNotConfirmed(String),
    /// **Replacing only.** One disk, and nothing else this machine could start
    /// from if alo OS did not suit the person.
    ///
    /// A refusal and not a warning. After replacing there is no Windows to run a
    /// program from, so a person who changes their mind needs something to start
    /// from that is not this disk — and this installer does not offer a road
    /// whose only exit is somebody else's spare computer.
    NoWayBackAtAll,
    /// **Replacing only.** The second answer was not the disk's name and the
    /// word that names what is lost.
    ///
    /// Either half being wrong is this, and the installer never says which:
    /// guessing which half was meant is how a person is walked towards the
    /// answer that erases their disk. It is never asked again in the same run.
    NotTheWord {
        /// The disk that was not erased, by its own name.
        disk: String,
        /// The word that was not typed, in the reader's language.
        word: String,
    },
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
    /// The Windows volume is smaller, by this much.
    ///
    /// **The amount is carried rather than assumed.** It used to be the
    /// installer's area, a constant gibibyte, because that was the only amount
    /// any road had ever taken. The road that keeps Windows takes tens of
    /// gigabytes, and a sentence that told a person their Windows was a
    /// gigabyte smaller when it was sixty would be this installer saying
    /// something false about somebody's own computer - the one thing it may
    /// never do.
    ///
    /// Read from the size Windows had less the size it was shrunk to, so a
    /// shrink that gave up more than it was asked for is still said truthfully.
    Smaller {
        /// The drive Windows is on.
        volume: Letter,
        /// How many bytes smaller it is.
        by: u64,
    },
    /// The area is still on this disk, by its shown name.
    TheArea(String),
    /// alo OS's own space is still on this disk, by its shown name.
    ///
    /// Only the road that keeps Windows can leave one. It is empty - nothing
    /// of the person's was ever in it - which is why it is a different sentence
    /// from [`Self::TheArea`] rather than the same one: a person reading what
    /// is left on their disk is owed the difference between *the installer left
    /// something of its own behind* and *the space it made for alo OS is still
    /// there, and empty*.
    TheSpace(String),
    /// The entry is still among the systems the computer can start.
    TheEntry,
    /// The next start is the installer's.
    TheNextStart,
    /// Fast Startup was turned off at the person's word and not put back.
    FastStartupOff,
    /// The copy of this program left in place could not be taken away again.
    TheWayBack,
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
            Self::BitLockerNotConfirmed(disk) => (
                words::BITLOCKER_NOT_CONFIRMED,
                Filling::of("disk", disk.as_str()),
            ),
            Self::NoWayBackAtAll => (words::NO_WAY_BACK_AT_ALL, Filling::nothing()),
            Self::NotTheWord { disk, word } => (
                words::NOT_THE_WORD,
                Filling::of("disk", disk.as_str()).and("word", word.as_str()),
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
            Self::Smaller { volume, by } => (
                words::REMAINS_SMALLER,
                Filling::of("volume", volume.drive()).and("area", sizes::taken(*by)),
            ),
            Self::TheArea(disk) => (words::REMAINS_THE_AREA, Filling::of("disk", disk.as_str())),
            Self::TheSpace(disk) => (words::REMAINS_THE_SPACE, Filling::of("disk", disk.as_str())),
            Self::TheEntry => (words::REMAINS_THE_ENTRY, Filling::nothing()),
            Self::TheNextStart => (words::REMAINS_THE_NEXT_START, Filling::nothing()),
            Self::FastStartupOff => (words::REMAINS_FAST_STARTUP_OFF, Filling::nothing()),
            Self::TheWayBack => (words::REMAINS_THE_WAY_BACK, Filling::nothing()),
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
            Refusal::BitLockerNotConfirmed("Msft Virtual Disk".to_owned()),
            Refusal::NoWayBackAtAll,
            Refusal::NotTheWord {
                disk: "Msft Virtual Disk".to_owned(),
                word: "erase".to_owned(),
            },
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
            Remains::Smaller {
                volume: c,
                by: sizes::THE_AREA,
            },
            Remains::Smaller {
                // The amount the keep-Windows road takes, which is what
                // used to be impossible to say.
                volume: c,
                by: 64 * sizes::GIB,
            },
            Remains::TheArea("Msft Virtual Disk 0".to_owned()),
            Remains::TheSpace("Msft Virtual Disk 0".to_owned()),
            Remains::TheEntry,
            Remains::TheNextStart,
        ] {
            let (word, filling) = remains.said_as();
            let said = strings.say(&word.key(), &filling);
            assert!(said.unfilled().is_empty(), "{remains:?}: {}", said.text());
        }
    }
}
