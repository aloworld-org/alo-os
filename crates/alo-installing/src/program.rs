//! The programs this environment runs, and nothing else.
//!
//! Law 2 binds an agent and there is no agent here — this environment runs
//! before alo OS exists on the disk. The shape is kept anyway, because it is
//! the shape that makes a privileged program reviewable: every program is a
//! variant with typed arguments made from values that were already checked, at
//! a fixed path, and there is no variant that takes a command. A reader who
//! wants to know everything this environment can do to a machine reads this
//! file.
//!
//! | | |
//! |---|---|
//! | [`Program::ListingTheDisks`] | reads what the disks hold; writes nothing |
//! | [`Program::WaitingForTheNetwork`] | waits for a wired connection; writes nothing |
//! | [`Program::Verifying`] | checks the download is the owner's; writes nothing |
//! | [`Program::MakingTheRoot`] | makes a file system on **one partition** |
//! | [`Program::Mounting`] | mounts a partition, so a root can be handed over |
//! | [`Program::Writing`] | puts alo OS on the chosen disk, or in one partition |
//! | [`Program::Restarting`] | restarts the machine after a successful install |
//!
//! The table is the list; it was headed *the five programs* while there were
//! five, and a count in a sentence is a thing that goes stale the moment
//! somebody adds a variant. The ones below the gap are for putting the
//! firmware's own list right afterwards.
//!
//! # Two of these are only for the road that keeps what is already there
//!
//! [`Program::MakingTheRoot`] and [`Program::Mounting`] exist because
//! `bootc install to-disk` takes a whole disk and erases it. Putting alo OS
//! **beside** Windows means the file system is made and mounted here, and the
//! writer is handed a root rather than a disk. Both tools were already in the
//! initramfs — `bootc` runs them itself on the other road — so nothing new is
//! carried for this.

use std::path::PathBuf;

use crate::disk::PartitionName;
use crate::verifying::Verifying;
use crate::writing::Writing;

/// How long the environment waits for a connection before saying it has none.
///
/// A wired connection with an address server answers in seconds; a minute is
/// for a slow one, and not for a cable that is not plugged in.
const THE_NETWORK_WITHIN_SECONDS: &str = "60";

/// What the root file system is called, on the road that makes it here.
///
/// The same name `bootc` gives a root it makes itself, so a machine installed
/// beside Windows and a machine given a whole disk read the same to anything
/// looking at the disk afterwards — a recovery tool, a person with a live USB,
/// or the next version of this installer.
const THE_ROOTS_LABEL: &str = "root";

/// One program this environment runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    /// Ask the machine what its disks hold.
    ListingTheDisks,
    /// Wait until the network manager says the machine is connected.
    WaitingForTheNetwork,
    /// Check that the pinned release is signed by the pinned key.
    Verifying(Verifying),
    /// Make the file system alo OS lives in, on one partition.
    ///
    /// Only on the road that keeps what is already on the disk. On the other
    /// road `bootc` makes it, which is what `--filesystem` tells it to do.
    ///
    /// **One partition, named as a partition.** The type refuses a whole disk,
    /// because a file system written over a disk that holds a partition table
    /// is the Windows this road exists to keep.
    MakingTheRoot {
        /// The partition the Windows installer already made for alo OS.
        partition: PartitionName,
    },
    /// Mount a partition somewhere, so the writer can be handed a root.
    ///
    /// Used twice and in this order: the root first, then the EFI partition
    /// **under** it, because the writer looks for the loader's home beneath the
    /// root it is given.
    Mounting {
        /// The partition to mount.
        partition: PartitionName,
        /// Where it goes.
        at: &'static str,
    },
    /// Write the pinned release onto the chosen disk.
    Writing(Writing),
    /// Restart the machine.
    Restarting,

    /// Ask the firmware which systems it can start.
    ListingTheStartEntries,
    /// Make the entry for the installed alo OS, named as a person reads it.
    ///
    /// The firmware's own tool cannot rename an entry, so the way to a named
    /// one is to make it again and take the old one away
    /// (`crate::tidying`). The file it starts is
    /// [`crate::entries::THE_LOADER_THE_BASE_INSTALLS`] and the name is
    /// [`crate::entries::THE_ENTRYS_NAME`]; neither is a value a caller passes.
    NamingTheEntry {
        /// The disk alo OS was installed onto, by its own name.
        disk: PathBuf,
        /// The partition its loader is on, read from the entry the install
        /// left behind.
        partition: u32,
    },
    /// Take one entry away, by its number.
    RemovingTheEntry {
        /// The number, as in `000B`.
        number: String,
    },
    /// Put the firmware's entries in this order.
    OrderingTheEntries {
        /// Every number, in the order they are to be started in.
        order: Vec<String>,
    },
    /// Take the installer's staging area off the disk it was made on.
    RemovingTheArea {
        /// The disk it is on.
        disk: String,
        /// Its number within that disk's own table.
        partition: u32,
    },
}

/// What a program did.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ran {
    /// Whether it said it succeeded.
    pub succeeded: bool,
    /// What it printed to its standard output, where that was kept.
    pub printed: String,
    /// What it printed to its standard error, where that was kept.
    pub complained: String,
}

/// Where every program this environment runs is, in the order of the table
/// above.
///
/// `tests/what_the_environment_carries.rs` holds `image/installing/alo-installing.conf`
/// to carrying every one of them: a program the environment runs and the
/// initramfs does not hold is a refusal nobody sees until a machine restarts
/// into it.
pub const EVERY_PROGRAM: [&str; 9] = [
    "/usr/bin/lsblk",
    "/usr/bin/nm-online",
    "/usr/bin/cosign",
    "/usr/bin/bootc",
    "/usr/bin/systemctl",
    "/usr/sbin/efibootmgr",
    "/usr/sbin/sfdisk",
    // Both already in the initramfs, because `bootc` runs them itself on the
    // road that takes a whole disk. The road that keeps what is there runs them
    // directly instead, and nothing new has to be carried for it.
    "/usr/sbin/mkfs.btrfs",
    "/usr/bin/mount",
];

impl Program {
    /// Where the program is, in the environment the recipe builds.
    ///
    /// Whole paths, so that nothing on a search path decides which program
    /// runs; every one of them is in [`EVERY_PROGRAM`].
    #[must_use]
    pub fn path(&self) -> &'static str {
        match self {
            Self::ListingTheDisks => "/usr/bin/lsblk",
            Self::WaitingForTheNetwork => "/usr/bin/nm-online",
            Self::Verifying(_) => "/usr/bin/cosign",
            Self::MakingTheRoot { .. } => "/usr/sbin/mkfs.btrfs",
            Self::Mounting { .. } => "/usr/bin/mount",
            Self::Writing(_) => "/usr/bin/bootc",
            Self::Restarting => "/usr/bin/systemctl",
            Self::ListingTheStartEntries
            | Self::NamingTheEntry { .. }
            | Self::RemovingTheEntry { .. }
            | Self::OrderingTheEntries { .. } => "/usr/sbin/efibootmgr",
            Self::RemovingTheArea { .. } => "/usr/sbin/sfdisk",
        }
    }

    /// Its arguments, each one a whole argument and never a line for a shell.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        match self {
            Self::ListingTheDisks => [
                "--json",
                "--paths",
                "--output",
                "NAME,TYPE,RO,MOUNTPOINTS,PARTTYPE,LABEL",
            ]
            .map(str::to_owned)
            .to_vec(),
            Self::WaitingForTheNetwork => ["--timeout", THE_NETWORK_WITHIN_SECONDS, "--quiet"]
                .map(str::to_owned)
                .to_vec(),
            Self::Verifying(verifying) => verifying.arguments(),
            // `--force` because the partition the Windows installer just made
            // is empty and the maker otherwise asks whether it may use a disk
            // that looks used — a question the person already answered, and one
            // nothing in this environment can answer for them. The *label* is
            // the same one `bootc` gives a root it makes, so a machine written
            // either way reads the same.
            Self::MakingTheRoot { partition } => vec![
                "--force".to_owned(),
                "--label".to_owned(),
                THE_ROOTS_LABEL.to_owned(),
                partition.path().display().to_string(),
            ],
            Self::Mounting { partition, at } => {
                vec![partition.path().display().to_string(), (*at).to_owned()]
            }
            Self::Writing(writing) => writing.arguments(),
            Self::Restarting => vec!["reboot".to_owned()],
            // `--quiet` is never passed: what the tool prints is the entry it
            // made, and `crate::tidying` reads the number out of it.
            Self::ListingTheStartEntries => Vec::new(),
            Self::NamingTheEntry { disk, partition } => vec![
                "--create".to_owned(),
                "--disk".to_owned(),
                disk.display().to_string(),
                "--part".to_owned(),
                partition.to_string(),
                "--loader".to_owned(),
                crate::entries::THE_LOADER_THE_BASE_INSTALLS.to_owned(),
                "--label".to_owned(),
                crate::entries::THE_ENTRYS_NAME.to_owned(),
            ],
            Self::RemovingTheEntry { number } => vec![
                "--bootnum".to_owned(),
                number.clone(),
                "--delete-bootnum".to_owned(),
            ],
            Self::OrderingTheEntries { order } => {
                vec!["--bootorder".to_owned(), order.join(",")]
            }
            Self::RemovingTheArea { disk, partition } => {
                vec!["--delete".to_owned(), disk.clone(), partition.to_string()]
            }
        }
    }

    /// Whether what it prints is an answer this environment reads.
    ///
    /// The writer's output is not read — it is long, it names the machinery,
    /// and it goes to the machine's log rather than to the person's screen.
    #[must_use]
    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Self::ListingTheDisks
                | Self::Verifying(_)
                | Self::ListingTheStartEntries
                | Self::NamingTheEntry { .. }
        )
    }

    /// Whether it writes to a disk.
    ///
    /// One of the five, and the one a test counts: a refusal is a sequence in
    /// which this never ran.
    #[must_use]
    pub fn writes(&self) -> bool {
        matches!(self, Self::Writing(_) | Self::RemovingTheArea { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only the writer writes, and only what is read is kept.
    #[test]
    fn only_the_writer_writes() {
        assert!(!Program::ListingTheDisks.writes());
        assert!(!Program::WaitingForTheNetwork.writes());
        assert!(!Program::WaitingForTheNetwork.is_read());
        assert!(!Program::Restarting.writes());
        assert!(Program::ListingTheDisks.is_read());
        assert!(!Program::Restarting.is_read());
        assert_eq!(Program::Restarting.arguments(), ["reboot"]);
    }

    /// Every program is at a whole path, so nothing on a search path chooses.
    #[test]
    fn every_program_is_at_a_whole_path() {
        for program in [
            Program::ListingTheDisks,
            Program::WaitingForTheNetwork,
            Program::Restarting,
        ] {
            assert!(program.path().starts_with("/usr/bin/"));
            assert!(EVERY_PROGRAM.contains(&program.path()), "{program:?}");
        }
    }
}
