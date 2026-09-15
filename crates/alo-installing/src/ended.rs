//! How an installation ended, and the sentence each ending is said in.
//!
//! Three endings. **Installed**, and the machine restarts. **Refused**, before
//! anything was written, for one of eleven reasons each with its own sentence —
//! and every one of those sentences says that nothing was changed, because it
//! is true and it is the first thing a person needs. **Not installed**, after
//! the disk began to be written, which is the one ending that cannot say that
//! and does not.

use alo_strings::{Filling, Word};

use crate::disk::DiskName;
use crate::words;

/// How it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ended {
    /// alo OS is on the disk.
    Installed(DiskName),
    /// Nothing was written, for this reason.
    Refused(Refusal),
    /// The disk began to be written and the write did not finish.
    NotInstalled(DiskName),
}

/// Why nothing was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The environment's own pin or key is missing or unreadable.
    Damaged,
    /// No disk was chosen.
    NoDiskChosen,
    /// What was chosen could not be read as one disk.
    ChoiceNotUnderstood,
    /// What was chosen is part of a disk; its name as it was given.
    NotAWholeDisk(String),
    /// The chosen disk never appeared.
    DiskNotConnected(DiskName),
    /// What the disks hold could not be found out.
    DisksNotRead,
    /// The chosen disk holds this installer.
    HoldsThisInstaller(DiskName),
    /// The chosen disk holds another operating system.
    HoldsAnotherSystem(DiskName),
    /// The chosen disk is read-only or in use.
    CannotBeWritten(DiskName),
    /// The download could not be reached.
    NotReachable,
    /// The download is not the owner's.
    NotGenuine,
}

impl Ended {
    /// Whether anything was written to any disk.
    #[must_use]
    pub fn wrote(&self) -> bool {
        !matches!(self, Self::Refused(_))
    }

    /// The sentence this ending is said in, and what fills it.
    #[must_use]
    pub fn said_as(&self) -> (Word, Filling) {
        match self {
            Self::Installed(_) => (words::INSTALLED, Filling::nothing()),
            Self::NotInstalled(disk) => (words::NOT_INSTALLED, the_disk(disk.as_str())),
            Self::Refused(refusal) => refusal.said_as(),
        }
    }
}

impl Refusal {
    /// The sentence this refusal is said in, and what fills it.
    #[must_use]
    pub fn said_as(&self) -> (Word, Filling) {
        match self {
            Self::Damaged => (words::DAMAGED, Filling::nothing()),
            Self::NoDiskChosen => (words::NO_DISK_CHOSEN, Filling::nothing()),
            Self::ChoiceNotUnderstood => (words::CHOICE_NOT_UNDERSTOOD, Filling::nothing()),
            Self::NotAWholeDisk(named) => (words::NOT_A_WHOLE_DISK, the_disk(named)),
            Self::DiskNotConnected(disk) => (words::DISK_NOT_CONNECTED, the_disk(disk.as_str())),
            Self::DisksNotRead => (words::DISKS_NOT_READ, Filling::nothing()),
            Self::HoldsThisInstaller(disk) => {
                (words::HOLDS_THIS_INSTALLER, the_disk(disk.as_str()))
            }
            Self::HoldsAnotherSystem(disk) => {
                (words::HOLDS_ANOTHER_SYSTEM, the_disk(disk.as_str()))
            }
            Self::CannotBeWritten(disk) => (words::CANNOT_BE_WRITTEN, the_disk(disk.as_str())),
            Self::NotReachable => (words::NOT_REACHABLE, Filling::nothing()),
            Self::NotGenuine => (words::NOT_GENUINE, Filling::nothing()),
        }
    }
}

/// The one gap any of these sentences has.
pub(crate) fn the_disk(named: &str) -> Filling {
    Filling::of("disk", named)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_strings::Strings;

    use super::*;
    use crate::words::installing_words;

    /// Every ending is said in a whole sentence, with nothing left unfilled.
    #[test]
    fn every_ending_is_said_whole() {
        let strings = Strings::of(installing_words().unwrap());
        let disk = DiskName::named("virtio-alo-target").unwrap();
        for ended in [
            Ended::Installed(disk.clone()),
            Ended::NotInstalled(disk.clone()),
            Ended::Refused(Refusal::Damaged),
            Ended::Refused(Refusal::NoDiskChosen),
            Ended::Refused(Refusal::ChoiceNotUnderstood),
            Ended::Refused(Refusal::NotAWholeDisk("virtio-alo-target-part1".to_owned())),
            Ended::Refused(Refusal::DiskNotConnected(disk.clone())),
            Ended::Refused(Refusal::DisksNotRead),
            Ended::Refused(Refusal::HoldsThisInstaller(disk.clone())),
            Ended::Refused(Refusal::HoldsAnotherSystem(disk.clone())),
            Ended::Refused(Refusal::CannotBeWritten(disk.clone())),
            Ended::Refused(Refusal::NotReachable),
            Ended::Refused(Refusal::NotGenuine),
        ] {
            let (word, filling) = ended.said_as();
            let said = strings.say(&word.key(), &filling);
            assert!(said.unfilled().is_empty(), "{ended:?}: {}", said.text());
            assert!(!said.is_a_bug(), "{ended:?}");
            assert_eq!(ended.wrote(), !matches!(ended, Ended::Refused(_)));
        }
    }
}
