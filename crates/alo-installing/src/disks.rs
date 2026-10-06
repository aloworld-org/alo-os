//! Whether what was chosen may be written, from what the machine says is on it.
//!
//! One question, asked for two roads. The person typed the disk's name before
//! the restart, and a name is a claim about which disk it is — so before
//! anything is written the environment asks the machine what is actually there.
//!
//! # The disk taken whole ([`Disks::may_receive`])
//!
//! Four things no consent given on another screen could have been about are
//! refused:
//!
//! - **part of a disk**, or something that is not a disk at all — a disc drive,
//!   a loop device, a memory card reader with nothing in it;
//! - **the disk this installer is running from.** The staged environment lives
//!   on a small area labelled [`THIS_INSTALLER`], and writing over it is writing
//!   over the program doing the writing;
//! - **a disk holding another operating system.** A partition of a type Windows
//!   makes — its reserved area, its recovery area, a basic data volume — means
//!   this is not the empty or spare disk a whole-disk install is for. Replacing
//!   Windows is a separate road that asks twice (the installer plan's task 7),
//!   and this environment takes it only when it was told to in so many words
//!   ([`Replacing`], `crate::Told::replaces_what_is_there`) — never by
//!   accident, and never as a default;
//! - **a disk in use or that cannot be written**: read-only, or with anything
//!   on it mounted.
//!
//! # The disk kept ([`Disks::may_keep_what_is_there`])
//!
//! The road that puts alo OS beside Windows asks none of those: there the disk
//! is *meant* to hold Windows and the installer's own staging area. What it asks
//! instead is whether the two partitions it was named are the two it may touch,
//! and the answer turns on a label the installer wrote
//! ([`THE_SPACE_THE_INSTALLER_MADE`]) rather than on a name on a command line.
//! That method says why at length, because it is the check standing between a
//! mis-staged name and somebody's Windows.
//!
//! # What is read
//!
//! `lsblk`'s JSON, with whole paths, so a name here is a name a program can open
//! and the comparison with the chosen disk is a comparison of paths rather than
//! of guesses. The order of the refusals is the order of what a person most
//! needs to hear: that it is the installer's own disk is more useful than that
//! it is in use, which it also is.

use std::path::Path;

use serde::Deserialize;

/// The label of the area the boot environment is staged on.
///
/// A FAT label, so at most eleven characters, upper case; the installer that
/// stages the environment writes it, and the environment recognises its own
/// disk by it.
pub const THIS_INSTALLER: &str = "ALO-INSTALL";

/// The label the installer on Windows puts on the space it made for alo OS.
///
/// A FAT label, so at most eleven characters and upper case, written by the
/// installer when it makes the partition and read here before anything is made
/// on it. It is the whole of the environment's permission to write that
/// partition: see [`Disks::may_keep_what_is_there`].
pub const THE_SPACE_THE_INSTALLER_MADE: &str = "ALO-ROOT";

/// The partition type a machine's start-up area is, as GPT names it.
///
/// The one place the firmware looks. On the road that keeps Windows this is
/// Windows' own, already on the disk, and alo OS's loader goes into a directory
/// of its own inside it.
pub const THE_START_UP_AREA: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";

/// The partition types Windows makes, as GPT names them.
///
/// Basic data, Microsoft reserved, Windows recovery, and the two halves of a
/// Windows dynamic disk. A basic data partition is also what a camera's card or
/// an external drive formatted on Windows carries, and a disk holding one is
/// refused too: this road is for a disk with nothing on it anybody would miss.
pub const ANOTHER_SYSTEMS: [&str; 5] = [
    "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7",
    "e3c9e316-0b5c-4db8-817d-f92df00215ae",
    "de94bba4-06d1-4d40-a16a-bfd50179d6ac",
    "5808c8aa-7e8f-42e0-85d2-e1e90434cfb3",
    "af9b60a0-1431-4f62-bc68-3311714a69ad",
];

/// Everything `lsblk --json --paths --output NAME,TYPE,RO,MOUNTPOINTS,PARTTYPE,LABEL`
/// said.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Disks {
    /// Every device at the top of the tree.
    blockdevices: Vec<Device>,
}

/// One device, and whatever is inside it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Device {
    /// Its whole path.
    name: String,
    /// `disk`, `part`, `rom`, `loop` and so on.
    #[serde(rename = "type")]
    kind: String,
    /// Whether the kernel will refuse to write it.
    ro: bool,
    /// Where it is mounted; `null` entries are not mounted.
    #[serde(default)]
    mountpoints: Vec<Option<String>>,
    /// The partition's type, where it is one.
    #[serde(default)]
    parttype: Option<String>,
    /// The filesystem's label, where there is one.
    #[serde(default)]
    label: Option<String>,
    /// Partitions and anything else stacked on it.
    #[serde(default)]
    children: Vec<Device>,
}

/// Whether this install may write over the system already on the disk.
///
/// It exists so that no caller can pass a `true` by accident: the road that
/// replaces Windows names itself here, and every other road says the other
/// word. The person's agreement to it was given twice, in the installer, on
/// the machine's previous system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replacing {
    /// The ordinary road: a disk holding another system is refused.
    Nothing,
    /// The road the person took twice: the system on this disk is replaced.
    TheSystemOnTheDisk,
}

/// Why the chosen disk may not be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unsuitable {
    /// The machine does not list it.
    #[error("the machine lists no such device")]
    NotListed,
    /// It is part of a disk, or not a disk.
    #[error("it is not a whole disk")]
    NotAWholeDisk,
    /// It holds the installer that is running.
    #[error("it holds this installer")]
    HoldsThisInstaller,
    /// It holds partitions another operating system made.
    #[error("it holds another operating system")]
    HoldsAnotherSystem,
    /// It is read-only or something on it is mounted.
    #[error("it is in use or cannot be written")]
    CannotBeWritten,
}

/// Why the two partitions named may not be used, on a disk that is kept.
///
/// Every one of these is said to the person as the same sentence, because every
/// one of them means the same thing to them — the space the installer made is
/// not there — and the thing to do about all of them is the same. Which one it
/// was goes to the log, where whoever is helping them can read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotKeeping {
    /// The machine does not list the chosen disk.
    #[error("the machine lists no such disk")]
    DiskNotListed,
    /// One of them is not a partition of the chosen disk.
    #[error("a partition named is not part of the chosen disk")]
    NotOnThatDisk,
    /// The same partition was named for both.
    #[error("the same partition was named for alo OS and for starting up")]
    OneSpaceForBoth,
    /// The partition to install into is not the one the installer made.
    #[error("the partition named does not carry the installer's own label")]
    NotTheSpaceTheInstallerMade,
    /// The partition named for starting up is not a start-up area.
    #[error("the partition named for starting up is not a start-up area")]
    NotAStartUpArea,
    /// One of them is read-only.
    #[error("a partition named cannot be written")]
    CannotBeWritten,
    /// One of them is mounted.
    #[error("a partition named is in use")]
    InUse,
}

impl Disks {
    /// What the lister printed.
    ///
    /// # Errors
    /// The parser's own error, when it is not the lister's JSON.
    pub fn read(printed: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(printed)
    }

    /// Where the installer's own staging area is, when this machine has one:
    /// the disk it is on, and the partition's number within that disk.
    ///
    /// Found by the label the installer gives it ([`THIS_INSTALLER`]) and by
    /// nothing else, which is the same thing [`Disks::may_receive`] refuses a
    /// disk for. The number is the partition's place in its disk's own list,
    /// counted from one, because that is what the partitioner takes.
    #[must_use]
    pub fn the_staging_area(&self) -> Option<(String, u32)> {
        for disk in &self.blockdevices {
            for (at, partition) in disk.children.iter().enumerate() {
                if partition.label.as_deref() == Some(THIS_INSTALLER) {
                    let number = u32::try_from(at + 1).ok()?;
                    return Some((disk.name.clone(), number));
                }
            }
        }
        None
    }

    /// Whether the device at this path may be written by a whole-disk install.
    ///
    /// **[`Replacing::TheSystemOnTheDisk`] lifts one refusal and only one**:
    /// that the disk holds another operating system. The installer's own disk,
    /// a device that is not a whole disk, and a disk in use are refused on
    /// every road, because no agreement a person gave was about those.
    ///
    /// # Errors
    /// [`Unsuitable`], naming the first thing a person needs to hear.
    pub fn may_receive(&self, device: &Path, replacing: Replacing) -> Result<(), Unsuitable> {
        let Some(disk) = self.blockdevices.iter().find(|one| one.is(device)) else {
            return Err(if self.blockdevices.iter().any(|one| one.holds(device)) {
                Unsuitable::NotAWholeDisk
            } else {
                Unsuitable::NotListed
            });
        };
        if disk.kind != "disk" {
            return Err(Unsuitable::NotAWholeDisk);
        }
        if disk.any(&|one| one.label.as_deref() == Some(THIS_INSTALLER)) {
            return Err(Unsuitable::HoldsThisInstaller);
        }
        if replacing == Replacing::Nothing
            && disk.any(&|one| {
                one.parttype.as_deref().is_some_and(|kind| {
                    ANOTHER_SYSTEMS.contains(&kind.to_ascii_lowercase().as_str())
                })
            })
        {
            return Err(Unsuitable::HoldsAnotherSystem);
        }
        if disk.ro || disk.any(&|one| one.mountpoints.iter().any(Option::is_some)) {
            return Err(Unsuitable::CannotBeWritten);
        }
        Ok(())
    }

    /// Whether these two partitions may be used, on a disk that is being kept.
    ///
    /// The road that puts alo OS **beside** Windows. Not a lifted refusal but a
    /// different question: on this road the disk is *meant* to hold another
    /// operating system and the installer's own staging area, so
    /// [`Disks::may_receive`]'s refusals are all the wrong ones, and the one
    /// refusal that matters here is not among them.
    ///
    /// **A file system is about to be made on the partition named**, which is
    /// the one irreversible thing in this environment: name Windows' own
    /// partition by mistake and the Windows this road exists to keep is gone.
    /// So the partition is not taken on its name. It is taken only when it
    /// carries the label [`THE_SPACE_THE_INSTALLER_MADE`] — the label the
    /// installer on Windows puts on the space it made and on nothing else,
    /// exactly as it labels its own staging area [`THIS_INSTALLER`]. Windows'
    /// own partitions do not carry it, a disk from another machine does not
    /// carry it, and a kernel command line that was staged wrongly cannot make
    /// one carry it.
    ///
    /// What else is held, and why each is not covered by that label:
    ///
    /// - **both partitions are on the disk the person chose.** A name is a claim
    ///   about which partition it is; this is where the claim is checked against
    ///   the disk the person typed, so a name that happens to exist on some
    ///   other disk in the machine is not reached;
    /// - **the start-up area really is one** ([`THE_START_UP_AREA`]). Nothing is
    ///   made on it — alo OS's loader goes into a directory of its own beside
    ///   Windows' — but it is mounted and written into, and a loader written
    ///   into what is actually somebody's documents is a machine that does not
    ///   start;
    /// - **neither is in use**, and neither is read-only. A partition mounted in
    ///   this environment is one something else is already holding.
    ///
    /// Each path is where the kernel put the device, as
    /// `TheMachine::wait_for` resolved it — not a by-id name, because what
    /// `lsblk` prints and what a name points at have to be compared as the same
    /// kind of thing.
    ///
    /// **After the file system is made this says no.** The maker gives the
    /// partition its own label, so a second run of this environment over a
    /// half-finished install is refused rather than quietly starting again. That
    /// is the honest consequence of trusting one label, and it is the one worth
    /// having: the way to try again is to run the installer on Windows again,
    /// which makes the space again, and the alternative — also accepting the
    /// label the maker writes — would accept any partition anybody had labelled
    /// that, on a disk this road is keeping.
    ///
    /// # Errors
    /// [`NotKeeping`], naming which of them it was, for the log. The person is
    /// told one sentence: all of these mean the space the installer made is not
    /// there, and the thing to do about every one of them is the same.
    pub fn may_keep_what_is_there(
        &self,
        disk: &Path,
        root: &Path,
        esp: &Path,
    ) -> Result<(), NotKeeping> {
        let Some(the_disk) = self.blockdevices.iter().find(|one| one.is(disk)) else {
            return Err(NotKeeping::DiskNotListed);
        };
        if root == esp {
            return Err(NotKeeping::OneSpaceForBoth);
        }
        let Some(the_root) = the_disk.partition(root) else {
            return Err(NotKeeping::NotOnThatDisk);
        };
        let Some(the_esp) = the_disk.partition(esp) else {
            return Err(NotKeeping::NotOnThatDisk);
        };
        if the_root.label.as_deref() != Some(THE_SPACE_THE_INSTALLER_MADE) {
            return Err(NotKeeping::NotTheSpaceTheInstallerMade);
        }
        if the_esp
            .parttype
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
            != Some(THE_START_UP_AREA)
        {
            return Err(NotKeeping::NotAStartUpArea);
        }
        if the_root.ro || the_esp.ro {
            return Err(NotKeeping::CannotBeWritten);
        }
        if [the_root, the_esp]
            .iter()
            .any(|one| one.mountpoints.iter().any(Option::is_some))
        {
            return Err(NotKeeping::InUse);
        }
        Ok(())
    }
}

impl Device {
    /// Whether this is the device at that path.
    fn is(&self, device: &Path) -> bool {
        Path::new(&self.name) == device
    }

    /// This disk's own partition at that path, where it has one.
    ///
    /// **A partition of this disk and not a descendant of it**: directly a
    /// child, and of the kind the lister calls a partition. Anything stacked on
    /// top of a partition — a decrypted volume, a volume group — is deeper in
    /// the tree and is not a partition anything here may make a file system on.
    fn partition(&self, device: &Path) -> Option<&Self> {
        self.children
            .iter()
            .find(|child| child.is(device) && child.kind == "part")
    }

    /// Whether the device at that path is somewhere beneath this one.
    fn holds(&self, device: &Path) -> bool {
        self.children
            .iter()
            .any(|child| child.is(device) || child.holds(device))
    }

    /// Whether this device or anything beneath it is so.
    fn any(&self, so: &dyn Fn(&Self) -> bool) -> bool {
        so(self) || self.children.iter().any(|child| child.any(so))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The machine the virtual-machine test builds: a first disk laid out the
    /// way Windows lays one out, with the installer staged beside it, and an
    /// empty second disk — exactly as `lsblk` prints it.
    const THE_TEST_MACHINE: &str = r#"{
       "blockdevices": [
          {"name": "/dev/sr0", "type": "rom", "ro": false, "mountpoints": [null], "parttype": null, "label": null},
          {"name": "/dev/vda", "type": "disk", "ro": false, "mountpoints": [null], "parttype": null, "label": null,
           "children": [
              {"name": "/dev/vda1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": null},
              {"name": "/dev/vda2", "type": "part", "ro": false, "mountpoints": [null], "parttype": "e3c9e316-0b5c-4db8-817d-f92df00215ae", "label": null},
              {"name": "/dev/vda3", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": null},
              {"name": "/dev/vda4", "type": "part", "ro": false, "mountpoints": [null], "parttype": "de94bba4-06d1-4d40-a16a-bfd50179d6ac", "label": null},
              {"name": "/dev/vda5", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": "ALO-INSTALL"}
           ]
          },
          {"name": "/dev/vdb", "type": "disk", "ro": false, "mountpoints": [null], "parttype": null, "label": null}
       ]
    }"#;

    /// The empty second disk may be written.
    #[test]
    fn an_empty_second_disk_may_be_written() {
        let disks = Disks::read(THE_TEST_MACHINE).unwrap();
        assert_eq!(
            disks.may_receive(Path::new("/dev/vdb"), Replacing::Nothing),
            Ok(())
        );
    }

    /// **The disk the installer runs from is refused as that**, even though it
    /// also holds Windows: which one it is matters more to the person.
    #[test]
    fn the_installers_own_disk_is_refused() {
        let disks = Disks::read(THE_TEST_MACHINE).unwrap();
        assert_eq!(
            disks.may_receive(Path::new("/dev/vda"), Replacing::Nothing),
            Err(Unsuitable::HoldsThisInstaller)
        );
    }

    /// **A disk holding Windows' partitions is refused**, for each kind of
    /// partition Windows makes, upper or lower case.
    #[test]
    fn a_disk_holding_another_system_is_refused() {
        for kind in ANOTHER_SYSTEMS
            .into_iter()
            .chain(["EBD0A0A2-B9E5-4433-87C0-68B6B72699C7"])
        {
            let printed = format!(
                r#"{{"blockdevices": [{{"name": "/dev/nvme1n1", "type": "disk", "ro": false,
                    "mountpoints": [null], "children": [
                    {{"name": "/dev/nvme1n1p1", "type": "part", "ro": false, "mountpoints": [null],
                      "parttype": "{kind}", "label": "DATA"}}]}}]}}"#
            );
            let disks = Disks::read(&printed).unwrap();
            assert_eq!(
                disks.may_receive(Path::new("/dev/nvme1n1"), Replacing::Nothing),
                Err(Unsuitable::HoldsAnotherSystem),
                "{kind}"
            );
        }
    }

    /// **A partition is refused as part of a disk**, and a disc drive as not a
    /// disk.
    #[test]
    fn part_of_a_disk_or_not_a_disk_is_refused() {
        let disks = Disks::read(THE_TEST_MACHINE).unwrap();
        assert_eq!(
            disks.may_receive(Path::new("/dev/vda3"), Replacing::Nothing),
            Err(Unsuitable::NotAWholeDisk)
        );
        assert_eq!(
            disks.may_receive(Path::new("/dev/sr0"), Replacing::Nothing),
            Err(Unsuitable::NotAWholeDisk)
        );
    }

    /// **A device the machine does not list is refused.**
    #[test]
    fn a_device_the_machine_does_not_list_is_refused() {
        let disks = Disks::read(THE_TEST_MACHINE).unwrap();
        assert_eq!(
            disks.may_receive(Path::new("/dev/vdc"), Replacing::Nothing),
            Err(Unsuitable::NotListed)
        );
        assert_eq!(
            Disks::read(r#"{"blockdevices": []}"#)
                .unwrap()
                .may_receive(Path::new("/dev/vdb"), Replacing::Nothing),
            Err(Unsuitable::NotListed)
        );
    }

    /// **A read-only disk, or one with anything mounted, is refused.**
    #[test]
    fn a_disk_in_use_or_read_only_is_refused() {
        let read_only = r#"{"blockdevices": [{"name": "/dev/sdb", "type": "disk", "ro": true, "mountpoints": [null]}]}"#;
        assert_eq!(
            Disks::read(read_only)
                .unwrap()
                .may_receive(Path::new("/dev/sdb"), Replacing::Nothing),
            Err(Unsuitable::CannotBeWritten)
        );

        let mounted = r#"{"blockdevices": [{"name": "/dev/sdb", "type": "disk", "ro": false, "mountpoints": [null],
            "children": [{"name": "/dev/sdb1", "type": "part", "ro": false, "mountpoints": ["/run/media/stick"],
                          "parttype": "0fc63daf-8483-4772-8e79-3d69d8477de4"}]}]}"#;
        assert_eq!(
            Disks::read(mounted)
                .unwrap()
                .may_receive(Path::new("/dev/sdb"), Replacing::Nothing),
            Err(Unsuitable::CannotBeWritten)
        );
    }

    /// A disk with only Linux partitions on it, none mounted, may be written:
    /// the refusals are about Windows and this installer, not about a disk
    /// having something on it.
    #[test]
    fn a_spare_disk_with_linux_partitions_may_be_written() {
        let printed = r#"{"blockdevices": [{"name": "/dev/sdb", "type": "disk", "ro": false, "mountpoints": [null],
            "children": [{"name": "/dev/sdb1", "type": "part", "ro": false, "mountpoints": [null],
                          "parttype": "0fc63daf-8483-4772-8e79-3d69d8477de4", "label": "old"}]}]}"#;
        assert_eq!(
            Disks::read(printed)
                .unwrap()
                .may_receive(Path::new("/dev/sdb"), Replacing::Nothing),
            Ok(())
        );
    }

    /// What is not the lister's JSON is not read as an empty machine.
    #[test]
    fn what_is_not_the_listers_json_is_an_error() {
        assert!(Disks::read("").is_err());
        assert!(Disks::read("NAME TYPE\nsda disk").is_err());
        assert!(Disks::read(r#"{"devices": []}"#).is_err());
    }

    /// **Replacing the system on the disk lifts that one refusal and no
    /// other.** The road that replaces Windows may write a disk holding
    /// Windows — that is what the person agreed to, twice — and may still not
    /// write the installer's own disk, something that is not a whole disk, or
    /// a disk in use.
    #[test]
    fn the_road_that_replaces_windows_may_write_a_disk_windows_is_on() {
        // The same machine, with the installer staged on the second disk
        // rather than beside Windows, so that the first disk is refused for
        // one reason only.
        let machine = THE_TEST_MACHINE.replace(r#""label": "ALO-INSTALL""#, r#""label": null"#);
        let disks = Disks::read(&machine).unwrap();
        assert_eq!(
            disks.may_receive(Path::new("/dev/vda"), Replacing::Nothing),
            Err(Unsuitable::HoldsAnotherSystem)
        );
        assert_eq!(
            disks.may_receive(Path::new("/dev/vda"), Replacing::TheSystemOnTheDisk),
            Ok(())
        );
        // And nothing else is lifted, on either road.
        for road in [Replacing::Nothing, Replacing::TheSystemOnTheDisk] {
            assert_eq!(
                disks.may_receive(Path::new("/dev/vda3"), road),
                Err(Unsuitable::NotAWholeDisk)
            );
        }
        // Not even for the disk the installer itself is running from.
        let whole = Disks::read(THE_TEST_MACHINE).unwrap();
        assert_eq!(
            whole.may_receive(Path::new("/dev/vda"), Replacing::TheSystemOnTheDisk),
            Err(Unsuitable::HoldsThisInstaller)
        );
    }

    /// The same machine after the installer on Windows has made and labelled
    /// the space for alo OS: Windows' start-up area, its reserved and recovery
    /// areas, its own volume, the installer's staging area, and the new space.
    const WINDOWS_MADE_THE_SPACE: &str = r#"{
       "blockdevices": [
          {"name": "/dev/vda", "type": "disk", "ro": false, "mountpoints": [null], "parttype": null, "label": null,
           "children": [
              {"name": "/dev/vda1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": null},
              {"name": "/dev/vda2", "type": "part", "ro": false, "mountpoints": [null], "parttype": "e3c9e316-0b5c-4db8-817d-f92df00215ae", "label": null},
              {"name": "/dev/vda3", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "Windows"},
              {"name": "/dev/vda4", "type": "part", "ro": false, "mountpoints": [null], "parttype": "de94bba4-06d1-4d40-a16a-bfd50179d6ac", "label": null},
              {"name": "/dev/vda5", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": "ALO-INSTALL"},
              {"name": "/dev/vda6", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "ALO-ROOT"}
           ]
          },
          {"name": "/dev/vdb", "type": "disk", "ro": false, "mountpoints": [null], "parttype": null, "label": null,
           "children": [
              {"name": "/dev/vdb1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "ALO-ROOT"}
           ]
          }
       ]
    }"#;

    /// **The space the installer made may be used, on the disk Windows is on**,
    /// with Windows' own start-up area — the one road where a disk holding
    /// Windows and holding this installer is not refused for either.
    #[test]
    fn the_space_the_installer_made_may_be_used() {
        let disks = Disks::read(WINDOWS_MADE_THE_SPACE).unwrap();
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda1"),
            ),
            Ok(())
        );
        // And the same disk is still refused for a whole-disk install, on both
        // of its roads: keeping what is there lifts nothing for anybody else.
        for road in [Replacing::Nothing, Replacing::TheSystemOnTheDisk] {
            assert_eq!(
                disks.may_receive(Path::new("/dev/vda"), road),
                Err(Unsuitable::HoldsThisInstaller)
            );
        }
    }

    /// **Every partition of Windows' own is refused as the space for alo OS**,
    /// and so is the installer's staging area and Windows' start-up area.
    ///
    /// This is the check that stands between a command line staged wrongly and
    /// somebody's Windows: the next thing that happens to whatever is named
    /// here is `mkfs.btrfs --force`. The label is the whole of the permission,
    /// so a partition of exactly the type the space for alo OS is — basic data,
    /// `/dev/vda3` — is refused when it does not carry the label, which is the
    /// case a type test would have let through.
    #[test]
    fn windows_own_partitions_are_refused_as_the_space_for_alo_os() {
        let disks = Disks::read(WINDOWS_MADE_THE_SPACE).unwrap();
        for named in [
            "/dev/vda1",
            "/dev/vda2",
            "/dev/vda3",
            "/dev/vda4",
            "/dev/vda5",
        ] {
            // Both of Windows' start-up areas are named here in turn, so the
            // other one stands in for the start-up area: naming one partition
            // twice is refused earlier, for a different reason, and would have
            // hidden what this test is asking.
            let start_up = if named == "/dev/vda1" {
                "/dev/vda5"
            } else {
                "/dev/vda1"
            };
            assert_eq!(
                disks.may_keep_what_is_there(
                    Path::new("/dev/vda"),
                    Path::new(named),
                    Path::new(start_up),
                ),
                Err(NotKeeping::NotTheSpaceTheInstallerMade),
                "{named} was taken for the space the installer made"
            );
        }
    }

    /// **A space on another disk is refused**, label and all.
    ///
    /// The second disk's partition carries the same label, because a disk that
    /// was in another machine can. What makes it refused is that it is not part
    /// of the disk the person chose.
    #[test]
    fn a_space_on_another_disk_is_refused() {
        let disks = Disks::read(WINDOWS_MADE_THE_SPACE).unwrap();
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vdb1"),
                Path::new("/dev/vda1"),
            ),
            Err(NotKeeping::NotOnThatDisk)
        );
        // And a disk the machine does not list at all.
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vdc"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda1"),
            ),
            Err(NotKeeping::DiskNotListed)
        );
    }

    /// **What is named for starting up has to be a start-up area**, and the two
    /// may not be the same partition.
    #[test]
    fn what_is_named_for_starting_up_has_to_be_one() {
        let disks = Disks::read(WINDOWS_MADE_THE_SPACE).unwrap();
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda3"),
            ),
            Err(NotKeeping::NotAStartUpArea)
        );
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda6"),
            ),
            Err(NotKeeping::OneSpaceForBoth)
        );
        // The installer's own staging area is a start-up area, and naming it
        // is not refused here: it is refused as the space for alo OS, which is
        // the partition that gets written over.
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda5"),
            ),
            Ok(())
        );
    }

    /// **A space that is mounted or read-only is refused**, either of the two.
    #[test]
    fn a_space_in_use_or_read_only_is_refused() {
        let mounted = WINDOWS_MADE_THE_SPACE.replace(
            r#""mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "ALO-ROOT"}
           ]
          },
          {"name": "/dev/vdb""#,
            r#""mountpoints": ["/run/somebody-else"], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": "ALO-ROOT"}
           ]
          },
          {"name": "/dev/vdb""#,
        );
        assert_ne!(mounted, WINDOWS_MADE_THE_SPACE, "the fixture was changed");
        assert_eq!(
            Disks::read(&mounted).unwrap().may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda1"),
            ),
            Err(NotKeeping::InUse)
        );

        let read_only = WINDOWS_MADE_THE_SPACE.replace(
            r#""name": "/dev/vda1", "type": "part", "ro": false"#,
            r#""name": "/dev/vda1", "type": "part", "ro": true"#,
        );
        assert_ne!(read_only, WINDOWS_MADE_THE_SPACE, "the fixture was changed");
        assert_eq!(
            Disks::read(&read_only).unwrap().may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/vda6"),
                Path::new("/dev/vda1"),
            ),
            Err(NotKeeping::CannotBeWritten)
        );
    }

    /// **Something stacked on a partition is not a partition**, so a decrypted
    /// volume inside the space cannot be named as the space.
    #[test]
    fn something_stacked_on_a_partition_is_not_one() {
        let printed = r#"{"blockdevices": [
            {"name": "/dev/vda", "type": "disk", "ro": false, "mountpoints": [null], "children": [
               {"name": "/dev/vda1", "type": "part", "ro": false, "mountpoints": [null], "parttype": "c12a7328-f81f-11d2-ba4b-00a0c93ec93b", "label": null},
               {"name": "/dev/vda6", "type": "part", "ro": false, "mountpoints": [null], "parttype": "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7", "label": null, "children": [
                  {"name": "/dev/dm-0", "type": "crypt", "ro": false, "mountpoints": [null], "label": "ALO-ROOT"}
               ]}
            ]}
        ]}"#;
        let disks = Disks::read(printed).unwrap();
        assert_eq!(
            disks.may_keep_what_is_there(
                Path::new("/dev/vda"),
                Path::new("/dev/dm-0"),
                Path::new("/dev/vda1"),
            ),
            Err(NotKeeping::NotOnThatDisk)
        );
    }
}
