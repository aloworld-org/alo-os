//! Which disk the person chose, as the restart carried it here.
//!
//! The installer that runs on the machine's previous system asks a person to
//! type the name of the disk alo OS goes onto, and writes that choice into the
//! boot entry it stages: one word on this environment's kernel command line,
//! `alo.installing.to=` and the disk's own name. That is the only thing this
//! environment is told, and it is read from the one place the firmware and the
//! loader put it.
//!
//! # Never a guess
//!
//! No word is no choice, and an empty word is no choice either — which is what a
//! loader writes when the file holding the choice was never staged. Two words
//! are two choices, and nothing here decides which one was meant. A word that
//! does not name a disk is not a disk. Each of those is a refusal before
//! anything is looked at, let alone written.

use crate::disk::{DiskName, NotADisk, NotAPartition, PartitionName};

/// The word on the kernel command line that carries the choice.
pub const THE_CHOICE: &str = "alo.installing.to=";

/// The word that says this install replaces what is already on that disk.
///
/// The ordinary road never sets it, and the loader leaves it empty when the
/// file that would set it was not staged — so *not replacing* is what a
/// machine does when nothing said otherwise, in every direction.
pub const THE_REPLACING: &str = "alo.installing.replacing=";

/// The one value that means yes, in the one place a person never types.
///
/// Not a translated word and not a number: it travels from this repository's
/// own installer to this repository's own environment, and anything else at
/// all — including an empty word — is *no*.
pub const REPLACING: &str = "windows";

/// The word naming the partition alo OS is installed **into**, keeping the rest
/// of the disk.
///
/// The installer plan's task 4. Read **beside** [`THE_CHOICE`] rather than
/// instead of it: the disk is still what the environment waits for, looks at and
/// says, because that is what the person chose and what every sentence is about.
/// This says where on it alo OS goes.
pub const THE_PARTITION: &str = "alo.installing.into=";

/// The word naming the EFI partition already on that disk — Windows' own.
///
/// A disk has one, the firmware looks only there, and alo OS's loader goes into
/// a directory of its own beside Windows'. **Named rather than searched for**,
/// because the Windows installer already read it and this environment does not
/// go hunting for partitions on a disk it is keeping.
pub const THE_EFI: &str = "alo.installing.efi=";

/// What was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Told {
    /// The one disk to install onto.
    disk: DiskName,
    /// Whether the install replaces what is already on that disk.
    replacing: bool,
    /// Where on that disk alo OS goes, when it is not taking the whole of it.
    ///
    /// [`None`] is the road that takes a disk whole. [`Some`] is alo OS beside
    /// what is already there, and carries **both** partitions, because a root
    /// with no EFI partition to put a loader in is an install nothing can start.
    beside: Option<Beside>,
}

/// The two partitions the road that keeps Windows is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beside {
    /// The partition alo OS is installed into.
    root: PartitionName,
    /// The EFI partition already on the disk.
    efi: PartitionName,
}

impl Beside {
    /// The partition alo OS is installed into.
    #[must_use]
    pub const fn root(&self) -> &PartitionName {
        &self.root
    }

    /// The EFI partition already on the disk.
    #[must_use]
    pub const fn efi(&self) -> &PartitionName {
        &self.efi
    }
}

/// Why the command line did not name one disk.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotTold {
    /// Nothing was chosen.
    #[error("no disk was chosen")]
    NothingChosen,
    /// More than one disk was named.
    #[error("more than one disk was chosen")]
    MoreThanOne,
    /// What was named is not a disk's name.
    #[error("what was chosen is not a disk: {0}")]
    NotADisk(NotADisk),
    /// What was named is a partition, and its name is kept to be said.
    #[error("what was chosen is part of a disk: {0}")]
    APartition(String),
    /// A partition was named to install into, and no EFI partition beside it.
    ///
    /// Both or neither. A root with nowhere to put a loader is an install that
    /// finishes and starts nothing, and this environment finds that out before
    /// it writes rather than after.
    #[error("a partition to install into was named, and no EFI partition with it")]
    NoEfiPartition,
    /// An EFI partition was named and no partition to install into.
    #[error("an EFI partition was named, and no partition to install into with it")]
    NoPartitionToInstallInto,
    /// One of the two names is not a partition's.
    #[error("what was named is not a partition: {0}")]
    NotAPartition(NotAPartition),
    /// The line says both to replace what is there and to keep it.
    ///
    /// The two roads are opposite answers to the one question the person was
    /// asked, and nothing here picks the safer of them: a line that says both is
    /// a line something went wrong writing, and what went wrong might as easily
    /// have been the half that says *replace*.
    #[error("the line says both to replace what is there and to install beside it")]
    BothRoads,
}

impl Told {
    /// What a kernel command line says was chosen.
    ///
    /// # Errors
    /// [`NotTold`], for each of the ways it does not name exactly one disk.
    pub fn from_the_command_line(line: &str) -> Result<Self, NotTold> {
        let mut chosen = line
            .split_ascii_whitespace()
            .filter_map(|word| word.strip_prefix(THE_CHOICE));
        let Some(first) = chosen.next() else {
            return Err(NotTold::NothingChosen);
        };
        if chosen.next().is_some() {
            return Err(NotTold::MoreThanOne);
        }
        if first.is_empty() {
            return Err(NotTold::NothingChosen);
        }
        let mut replacing = line
            .split_ascii_whitespace()
            .filter_map(|word| word.strip_prefix(THE_REPLACING));
        let first_replacing = replacing.next().unwrap_or_default();
        if replacing.next().is_some() {
            return Err(NotTold::MoreThanOne);
        }
        // Exactly the one value, or no. A word that is not it is not a reason
        // to destroy an operating system, and a road this dangerous never
        // reads *almost yes* as yes.
        let replacing = first_replacing == REPLACING;
        let beside = Self::beside_from(line)?;
        // **Replacing what is there and keeping it are opposites**, and this is
        // where they are kept from arriving together. Further on, the sequence
        // reads them as two roads with nothing between them, and it may do that
        // only because a line claiming both never gets this far.
        if replacing && beside.is_some() {
            return Err(NotTold::BothRoads);
        }
        match DiskName::named(first) {
            Ok(disk) => Ok(Self {
                disk,
                replacing,
                beside,
            }),
            Err(NotADisk::APartition) => Err(NotTold::APartition(first.to_owned())),
            Err(why) => Err(NotTold::NotADisk(why)),
        }
    }

    /// The two partitions, where the line names them.
    ///
    /// **Both or neither.** One without the other is not a road half described,
    /// it is a road that cannot be walked: a root with no EFI partition installs
    /// alo OS somewhere nothing will start it from, and an EFI partition with no
    /// root has nothing to install. Either half alone is refused here, before
    /// the disk is even looked at.
    fn beside_from(line: &str) -> Result<Option<Beside>, NotTold> {
        let one = |word: &str| -> Result<Option<&str>, NotTold> {
            let mut found = line
                .split_ascii_whitespace()
                .filter_map(|it| it.strip_prefix(word))
                .filter(|it| !it.is_empty());
            let first = found.next();
            if found.next().is_some() {
                return Err(NotTold::MoreThanOne);
            }
            Ok(first)
        };
        match (one(THE_PARTITION)?, one(THE_EFI)?) {
            (None, None) => Ok(None),
            (Some(_), None) => Err(NotTold::NoEfiPartition),
            (None, Some(_)) => Err(NotTold::NoPartitionToInstallInto),
            (Some(root), Some(efi)) => Ok(Some(Beside {
                root: PartitionName::named(root).map_err(NotTold::NotAPartition)?,
                efi: PartitionName::named(efi).map_err(NotTold::NotAPartition)?,
            })),
        }
    }

    /// Where on the disk alo OS goes, when it is not taking the whole of it.
    #[must_use]
    pub const fn beside(&self) -> Option<&Beside> {
        self.beside.as_ref()
    }

    /// Whether this install replaces what is already on the disk.
    ///
    /// The one road on which the environment may write over another system's
    /// partitions (the installer plan's task 7). Everything else refuses a disk
    /// that holds one.
    #[must_use]
    pub fn replaces_what_is_there(&self) -> bool {
        self.replacing
    }

    /// The disk.
    #[must_use]
    pub fn disk(&self) -> &DiskName {
        &self.disk
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The command line the staged loader writes, with a choice in it.
    const STAGED: &str = "BOOT_IMAGE=/EFI/alo-installing/vmlinuz rd.systemd.unit=alo-installing.target \
                          rd.neednet=1 ip=dhcp console=ttyS0,115200 console=tty0";

    /// One disk named is that disk, wherever on the line it is.
    #[test]
    fn one_disk_named_is_that_disk() {
        let told =
            Told::from_the_command_line(&format!("{STAGED} alo.installing.to=virtio-alo-target"))
                .unwrap();
        assert_eq!(told.disk().as_str(), "virtio-alo-target");

        let told = Told::from_the_command_line(&format!(
            "alo.installing.to=nvme-Samsung_SSD_980 {STAGED}\n"
        ))
        .unwrap();
        assert_eq!(told.disk().as_str(), "nvme-Samsung_SSD_980");
    }

    /// **Nothing chosen is refused**, whether the word is missing or empty.
    #[test]
    fn nothing_chosen_is_refused() {
        assert_eq!(
            Told::from_the_command_line(STAGED),
            Err(NotTold::NothingChosen)
        );
        assert_eq!(
            Told::from_the_command_line(&format!("{STAGED} alo.installing.to=")),
            Err(NotTold::NothingChosen)
        );
        assert_eq!(Told::from_the_command_line(""), Err(NotTold::NothingChosen));
    }

    /// **Two choices are refused**, even when they are the same disk twice —
    /// a line that says it twice is a line something went wrong writing.
    #[test]
    fn two_choices_are_refused() {
        assert_eq!(
            Told::from_the_command_line(&format!(
                "{STAGED} alo.installing.to=virtio-a alo.installing.to=virtio-b"
            )),
            Err(NotTold::MoreThanOne)
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=virtio-a alo.installing.to=virtio-a"),
            Err(NotTold::MoreThanOne)
        );
    }

    /// **A choice that is not a disk, or is part of one, is refused as that.**
    #[test]
    fn a_choice_that_is_not_a_whole_disk_is_refused() {
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=../../sda"),
            Err(NotTold::NotADisk(NotADisk::NotAName))
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=/dev/sda"),
            Err(NotTold::NotADisk(NotADisk::NotAName))
        );
        assert_eq!(
            Told::from_the_command_line("alo.installing.to=virtio-alo-target-part2"),
            Err(NotTold::APartition("virtio-alo-target-part2".to_owned()))
        );
    }

    /// A word that only resembles the choice is not the choice.
    #[test]
    fn a_word_that_only_resembles_the_choice_is_not_one() {
        assert_eq!(
            Told::from_the_command_line("xalo.installing.to=virtio-a alo.installing.too=virtio-b"),
            Err(NotTold::NothingChosen)
        );
    }

    /// The line a person's *keep Windows* produces: the disk, and the two
    /// partitions on it.
    const KEEPING: &str = "alo.installing.to=virtio-alo-windows \
                           alo.installing.into=virtio-alo-windows-part6 \
                           alo.installing.efi=virtio-alo-windows-part1";

    /// **Both partitions named is the road that keeps what is there**, and the
    /// disk is still the disk, because everything the environment says, waits
    /// for and ends with is about the disk the person chose.
    #[test]
    fn both_partitions_named_is_the_road_that_keeps_what_is_there() {
        let told = Told::from_the_command_line(&format!("{STAGED} {KEEPING}")).unwrap();
        assert_eq!(told.disk().as_str(), "virtio-alo-windows");
        assert!(!told.replaces_what_is_there());
        let beside = told.beside().unwrap();
        assert_eq!(beside.root().as_str(), "virtio-alo-windows-part6");
        assert_eq!(beside.efi().as_str(), "virtio-alo-windows-part1");
    }

    /// **No partition named is the road that takes the disk whole**, which is
    /// what every line written before this road existed says.
    #[test]
    fn no_partition_named_is_the_road_that_takes_the_disk_whole() {
        let told =
            Told::from_the_command_line(&format!("{STAGED} alo.installing.to=virtio-alo-target"))
                .unwrap();
        assert_eq!(told.beside(), None);
    }

    /// **One of the two is refused as that**, and an empty word counts as not
    /// named — which is what a loader writes when the file holding it was never
    /// staged.
    ///
    /// A root with nowhere to put a loader is an install that finishes and
    /// starts nothing; a start-up area with no root is an install with nothing
    /// to do. Either is found out here, before a disk is looked at.
    #[test]
    fn one_of_the_two_partitions_alone_is_refused() {
        for (line, why) in [
            (
                "alo.installing.to=virtio-a alo.installing.into=virtio-a-part6",
                NotTold::NoEfiPartition,
            ),
            (
                "alo.installing.to=virtio-a alo.installing.efi=virtio-a-part1",
                NotTold::NoPartitionToInstallInto,
            ),
            (
                "alo.installing.to=virtio-a alo.installing.into=virtio-a-part6 alo.installing.efi=",
                NotTold::NoEfiPartition,
            ),
            (
                "alo.installing.to=virtio-a alo.installing.into= alo.installing.efi=virtio-a-part1",
                NotTold::NoPartitionToInstallInto,
            ),
        ] {
            assert_eq!(Told::from_the_command_line(line), Err(why), "{line}");
        }
    }

    /// **A name that is not a partition's is refused as that**, including a
    /// whole disk's name, which is the one that would have had a file system
    /// written over somebody's partition table.
    #[test]
    fn a_name_that_is_not_a_partitions_is_refused() {
        assert_eq!(
            Told::from_the_command_line(
                "alo.installing.to=virtio-a alo.installing.into=virtio-a alo.installing.efi=virtio-a-part1"
            ),
            Err(NotTold::NotAPartition(NotAPartition::AWholeDisk))
        );
        assert_eq!(
            Told::from_the_command_line(
                "alo.installing.to=virtio-a alo.installing.into=virtio-a-part6 alo.installing.efi=/dev/vda1"
            ),
            Err(NotTold::NotAPartition(NotAPartition::NotAName))
        );
    }

    /// **Two of either partition is refused**, as two disks are.
    #[test]
    fn two_of_either_partition_is_refused() {
        for line in [
            "alo.installing.to=virtio-a alo.installing.into=virtio-a-part6 \
             alo.installing.into=virtio-a-part7 alo.installing.efi=virtio-a-part1",
            "alo.installing.to=virtio-a alo.installing.into=virtio-a-part6 \
             alo.installing.efi=virtio-a-part1 alo.installing.efi=virtio-a-part2",
        ] {
            assert_eq!(
                Told::from_the_command_line(line),
                Err(NotTold::MoreThanOne),
                "{line}"
            );
        }
    }

    /// **A line saying both roads is refused**, rather than one of them being
    /// taken as the one that was meant.
    #[test]
    fn a_line_saying_both_roads_is_refused() {
        assert_eq!(
            Told::from_the_command_line(&format!("{KEEPING} {THE_REPLACING}{REPLACING}")),
            Err(NotTold::BothRoads)
        );
        // And the word that is not the one value does not make a road, so it
        // does not make a contradiction either.
        let told =
            Told::from_the_command_line(&format!("{KEEPING} {THE_REPLACING}yes-please")).unwrap();
        assert!(!told.replaces_what_is_there());
        assert!(told.beside().is_some());
    }
}
