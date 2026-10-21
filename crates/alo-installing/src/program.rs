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
//! | [`Program::ScanningForNetworks`] | asks what wireless networks are in range; writes nothing |
//! | [`Program::WaitingForTheNetwork`] | waits for a wired connection; writes nothing |
//! | [`Program::AskingAboutTheNetworkCards`] | asks the kernel what cards there are and
//!   whether the wire is live, after the wait above failed; writes nothing |
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
    /// Make a connection profile for this network, **held in memory only**.
    ///
    /// # Why two programs and not one
    ///
    /// `nmcli device wifi connect …` is one call and has no way to say *do not
    /// write this down*. `connection add --temporary` then `connection up` is
    /// two, and the first of them carries the flag. **One call that keeps a
    /// secret is worth less than two that do not.**
    ///
    /// ADR 0128 decided the environment asks for the network and that no
    /// pre-shared key is written to the staging partition, and `S3` says it to
    /// a person in as many words: *your Wi-Fi password is never written to this
    /// computer's disk*. A profile made without [`crate::TEMPORARY`] lands in
    /// `/etc/NetworkManager/system-connections/` — a directory the image has —
    /// so the flag is what makes the promise true by construction rather than
    /// by where this happens to be running.
    AddingANetwork {
        /// What the network calls itself.
        ssid: String,
        /// Its pre-shared key, and empty for an open network.
        ///
        /// **The one field in this enum that holds a secret.** It reaches the
        /// program's arguments and nothing else: `sequence::noted` logs a
        /// program's *path* and what it complained of, never its arguments,
        /// and a test in `joining_a_network` holds that.
        password: String,
    },
    /// Bring up the profile just made.
    BringingItUp {
        /// The profile's name, which is the network's own.
        ssid: String,
    },
    /// Ask the network manager which wireless networks it can see.
    ///
    /// **Reading, and it changes nothing.** A scan asks the card to listen; it
    /// joins nothing and writes nothing, which is why it is here rather than
    /// behind the one approval `joining_a_network` takes.
    ScanningForNetworks,
    /// Wait until the network manager says the machine is connected.
    WaitingForTheNetwork,
    /// Ask the kernel what network cards there are and what state they are in.
    ///
    /// Run **only after the wait above has failed**, so that a refusal can name
    /// which of three things went wrong rather than offering one sentence that
    /// is impossible to act on for one of them
    /// (`crate::why_there_is_no_network`).
    ///
    /// The kernel's own answer rather than the network manager's, because
    /// `nmcli` prints a device's state as a translated string and a person
    /// chooses their language at first boot. `flags`, `operstate` and
    /// `link_type` are iproute2's constants.
    AskingAboutTheNetworkCards,
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
    /// What it exited with, where the machine could say.
    ///
    /// **Kept because a bool cannot tell five failures apart.**
    /// `joining_a_network::why_not_joined` was written against nmcli's own
    /// stable codes -- 3 is a timeout, 10 is something not found -- and for as
    /// long as this struct carried only `succeeded`, two of its five answers
    /// were unreachable and nothing said so. A person told *no networks found*
    /// when the truth is *it never answered* cannot act on it.
    ///
    /// [`None`] where the program was killed by a signal rather than
    /// exiting, which is a real thing on Unix and not the same as exiting
    /// non-zero.
    pub exit_code: Option<i32>,
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
pub const EVERY_PROGRAM: [&str; 11] = [
    "/usr/bin/lsblk",
    // The network manager's own client, for the scan the network screen shows.
    // It is in the initramfs because NetworkManager is, and it is named here
    // for the reason the whole list exists: relying on another module's
    // transitive choice is what this list exists to stop.
    "/usr/bin/nmcli",
    "/usr/bin/nm-online",
    // **Named rather than inherited.** It is already in the initramfs —
    // dracut's own `network` module brings it, and the shipped candidate of
    // 2026-10-20 has it at this path — but nothing asked for it, and relying on
    // another module's transitive choice is what this list exists to stop.
    "/usr/bin/ip",
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
            Self::ScanningForNetworks | Self::AddingANetwork { .. } | Self::BringingItUp { .. } => {
                "/usr/bin/nmcli"
            }
            Self::WaitingForTheNetwork => "/usr/bin/nm-online",
            Self::AskingAboutTheNetworkCards => "/usr/bin/ip",
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
            // **`--bytes`, and three columns a person reads rather than a
            // program.** `SIZE` and `MODEL` are how a disk is *named* to
            // somebody choosing one — `docs/features.md`: *it says what is on
            // each disk, in words, before anybody chooses one* — and `FSUSED`
            // is how much of it is theirs. Without them the only honest thing
            // to show is `/dev/nvme0n1`, which is the fault that line exists to
            // end: two disks of the same size from the same maker are ordinary
            // to own, and a device path tells nobody which is which.
            //
            // `--bytes` because a size is arithmetic here, not a label:
            // `931.5G` cannot be compared against the least disk alo OS needs,
            // and rounding it for a person is this product's own job and not
            // `lsblk`'s.
            Self::ListingTheDisks => [
                "--json",
                "--paths",
                "--bytes",
                "--output",
                "NAME,TYPE,RO,MOUNTPOINTS,PARTTYPE,LABEL,SIZE,MODEL,FSUSED",
            ]
            .map(str::to_owned)
            .to_vec(),
            // **Three fields and nothing else**, because `every_network` parses
            // exactly these three and in this order: what it calls itself, how
            // strong it is, and what security it wants. `--terse` is what makes
            // the answer a shape rather than a table drawn for a person — and
            // it is also what escapes a colon inside a name, which is the one
            // thing about this format that needed a test.
            //
            // `--rescan yes` because a cached answer is the wrong answer here.
            // A person standing in front of this screen has just switched the
            // machine on, and the list they are shown has to be what is in
            // range now rather than whatever the card last happened to see.
            // **The password is the last argument and the profile is
            // temporary.** `wifi-sec` is given only where there is a key: an
            // open network with `wpa-psk` and an empty `psk` is refused by
            // the network manager, and refused in a way a person would read as
            // their password being wrong.
            //
            // `ifname '*'` because the environment does not know which card
            // it has and does not need to: one wireless card is the ordinary
            // case and the manager picks it.
            Self::AddingANetwork { ssid, password } => {
                let mut arguments = vec![
                    "connection".to_owned(),
                    "add".to_owned(),
                    crate::joining_a_network::TEMPORARY.to_owned(),
                    "type".to_owned(),
                    "wifi".to_owned(),
                    "con-name".to_owned(),
                    ssid.clone(),
                    "ifname".to_owned(),
                    "*".to_owned(),
                    "ssid".to_owned(),
                    ssid.clone(),
                ];
                if !password.is_empty() {
                    arguments.extend(
                        [
                            "wifi-sec.key-mgmt",
                            "wpa-psk",
                            "wifi-sec.psk",
                            password.as_str(),
                        ]
                        .map(str::to_owned),
                    );
                }
                arguments
            }
            Self::BringingItUp { ssid } => ["connection", "up", ssid.as_str()]
                .map(str::to_owned)
                .to_vec(),
            Self::ScanningForNetworks => [
                "--terse",
                "--fields",
                "SSID,SIGNAL,SECURITY",
                "device",
                "wifi",
                "list",
                "--rescan",
                "yes",
            ]
            .map(str::to_owned)
            .to_vec(),
            Self::WaitingForTheNetwork => ["--timeout", THE_NETWORK_WITHIN_SECONDS, "--quiet"]
                .map(str::to_owned)
                .to_vec(),
            // `-json` so the answer is a shape rather than a layout, and
            // `link` rather than `addr` because the question is whether there
            // is a card and whether the wire is live, not what address it has.
            Self::AskingAboutTheNetworkCards => {
                ["-json", "link", "show"].map(str::to_owned).to_vec()
            }
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
    ///
    /// **Written out, and not as `matches!`.** See [`Program::writes`]: a
    /// predicate over this enum that a test trusts has to be one the compiler
    /// makes the next person visit.
    #[must_use]
    pub fn is_read(&self) -> bool {
        match self {
            Self::ListingTheDisks
            | Self::ScanningForNetworks
            | Self::AddingANetwork { .. }
            | Self::BringingItUp { .. }
            | Self::Verifying(_)
            | Self::AskingAboutTheNetworkCards
            | Self::ListingTheStartEntries
            | Self::NamingTheEntry { .. } => true,
            Self::WaitingForTheNetwork
            | Self::MakingTheRoot { .. }
            | Self::Mounting { .. }
            | Self::Writing(_)
            | Self::Restarting
            | Self::RemovingTheEntry { .. }
            | Self::OrderingTheEntries { .. }
            | Self::RemovingTheArea { .. } => false,
        }
    }

    /// Whether it writes to a disk.
    ///
    /// What a test counts: a refusal is a sequence in which none of these ran.
    /// So the question it answers has to be *does a disk come out different*,
    /// not *is this the install* — and by that question the maker of the file
    /// system is the most destructive program here. It is the one that turns the
    /// space the installer made into a file system, and if it were ever pointed
    /// at a partition Windows is using, that Windows would be gone. Mounting is
    /// not here: it changes nothing on a disk, and a mount that failed is caught
    /// by being checked rather than by being counted.
    ///
    /// **Written out, with no wildcard, and deliberately long.** This was a
    /// `matches!` of two variants, which is a non-exhaustive match — so adding
    /// the maker of file systems could not break it, and for the length of that
    /// change every refusal test on the road that keeps Windows would have
    /// passed with a file system already made. The next program added must not
    /// compile until somebody has decided this answer for it, which is what a
    /// match the compiler re-reads buys and a one-line predicate does not.
    /// `docs/misreadings/two-predicates-in-one-file-and-only-one-was-armed.md`
    /// is the whole of it.
    #[must_use]
    pub fn writes(&self) -> bool {
        match self {
            Self::Writing(_) | Self::MakingTheRoot { .. } | Self::RemovingTheArea { .. } => true,
            Self::ListingTheDisks
            | Self::ScanningForNetworks
            // **Neither writes to a disk**, which is what this predicate asks:
            // a temporary profile is held in the network manager's memory for
            // the life of the daemon and writes no keyfile at all. That is the
            // whole point of the flag, and if it ever stopped being true this
            // answer would have to change with it.
            | Self::AddingANetwork { .. }
            | Self::BringingItUp { .. }
            | Self::WaitingForTheNetwork
            | Self::AskingAboutTheNetworkCards
            | Self::Verifying(_)
            | Self::Mounting { .. }
            | Self::Restarting
            | Self::ListingTheStartEntries
            | Self::NamingTheEntry { .. }
            | Self::RemovingTheEntry { .. }
            | Self::OrderingTheEntries { .. } => false,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Only what writes says it writes, and only what is read is kept.
    #[test]
    fn only_what_writes_says_it_writes() {
        assert!(!Program::ListingTheDisks.writes());
        assert!(!Program::WaitingForTheNetwork.writes());
        assert!(!Program::WaitingForTheNetwork.is_read());
        assert!(!Program::Restarting.writes());
        assert!(Program::ListingTheDisks.is_read());
        assert!(!Program::Restarting.is_read());
        assert_eq!(Program::Restarting.arguments(), ["reboot"]);

        // **Making the file system writes.** Everything a refusal promises
        // rests on this answer, so it is asserted rather than assumed.
        let space = PartitionName::named("virtio-alo-windows-part6").expect("a partition");
        assert!(
            Program::MakingTheRoot {
                partition: space.clone()
            }
            .writes()
        );
        // And mounting does not: nothing on a disk comes out different.
        assert!(
            !Program::Mounting {
                partition: space,
                at: "/run/alo-os-root",
            }
            .writes()
        );
    }

    /// The maker is given the partition, with the label a `bootc` root carries
    /// and the answer to the question it would otherwise ask.
    #[test]
    fn the_maker_is_given_the_partition_and_never_a_disk() {
        let partition = PartitionName::named("virtio-alo-windows-part6").expect("a partition");
        let program = Program::MakingTheRoot { partition };
        assert_eq!(program.path(), "/usr/sbin/mkfs.btrfs");
        assert_eq!(
            program.arguments(),
            [
                "--force",
                "--label",
                "root",
                "/dev/disk/by-id/virtio-alo-windows-part6",
            ]
        );
        assert!(!program.is_read(), "what the maker prints is not an answer");
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
