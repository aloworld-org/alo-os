//! Writing alo OS onto the chosen disk.
//!
//! `bootc install to-disk`, the tool inside the base the image is built on, and
//! the one `docs/booting.md` already gives — with two differences that are both
//! about where this runs.
//!
//! **`--source-imgref registry:…`.** The documented invocation runs *inside* the
//! image it installs, which needs the whole image in a container store first.
//! This environment lives in memory, and a 6.6 GB image unpacked into memory is
//! a machine that needs more of it than a certified laptop may have. Given a
//! source, the tool pulls the release straight into the new disk's own store
//! instead — measured in a virtual machine with 3 GB of memory on 2026-09-15.
//!
//! **The chosen disk by its own name**, `/dev/disk/by-id/…`, never the kernel's
//! `/dev/sdX` for it, so the disk written is the one the person named even if
//! the machine enumerated its disks in a different order on this start.
//!
//! The source is the pinned release **by digest**, the same reference
//! `crate::verifying` checked. Nothing else is passed about what the installed
//! machine follows afterwards: the tool records the reference it installed, and
//! what the machine updates to next is the updates plan's decision to make by
//! digest, not this environment's.
//!
//! **`--filesystem btrfs`**, which is the one argument here that a person can
//! never take back.
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) rewinds
//! *undo what the agent did* from the base's own read-only snapshot of a
//! person's home, and a snapshot needs a filesystem that has them. **A
//! filesystem is chosen at install and cannot be converted afterwards**, so a
//! machine installed on `ext4` — which has no subvolume and no snapshot — could
//! never undo anything without being reinstalled. The value is
//! [`alo_image::THE_ONLY_FILESYSTEM`] rather than a second spelling here,
//! because it is also written in `docs/booting.md`, and two spellings of a
//! decision that cannot be undone is one too many. What the base makes of it is
//! measured in `docs/quirks.md` rather than assumed.

use alo_image::{THE_ONLY_FILESYSTEM, ThePin};

use crate::disk::{DiskName, PartitionName};

/// The console a person's own machine shows its start-up on.
///
/// **The screen in front of them, said out loud rather than left to a default.**
/// With no `console=` at all the kernel picks, and what somebody sees when their
/// machine starts is whatever that choice happens to be. On a machine with no
/// serial port this changes nothing observable, which is the point: a real
/// machine boots exactly as before.
pub const THE_PERSONS_SCREEN: &str = "console=tty0";

/// The line a machine is watched on, when it was asked to be.
///
/// `115200n8` because that is the rate every walk already opens, and a console
/// whose rate the reader has to guess is a console nobody reads.
pub const A_WATCHED_LINE: &str = "console=ttyS0,115200n8";

/// Whether the installed machine opens a serial console.
///
/// **Not a flag on `Writing` but a thing the caller names**, because
/// [ADR 0082](../../../docs/decisions/0082-a-shipped-machine-shows-its-start-up-on-its-own-screen.md)
/// decided that a serial console is **asked for** rather than defaulted either
/// way: *a serial console is a console, and anything that can reach the port can
/// type at it.* A caller writing `Writing::of` gets a person's machine; a caller
/// that wants to watch one has to say so in the same breath as asking for the
/// install, where a reviewer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TheConsole {
    /// The screen in front of the person, and nothing else. The shipped default.
    #[default]
    TheirScreen,
    /// Their screen **and** a serial line, for a machine something is watching.
    ///
    /// Both, and in this order: the last console named takes `/dev/console`, so
    /// naming the serial line last is what lets a walk read the start-up —
    /// without taking the kernel's messages off the screen a person is looking
    /// at, which naming only the serial line would do.
    TheirScreenAndAWatchedLine,
}

impl TheConsole {
    /// The arguments this choice adds, in the order the kernel reads them.
    #[must_use]
    pub fn arguments(self) -> Vec<String> {
        match self {
            Self::TheirScreen => vec![THE_PERSONS_SCREEN.to_owned()],
            Self::TheirScreenAndAWatchedLine => {
                vec![THE_PERSONS_SCREEN.to_owned(), A_WATCHED_LINE.to_owned()]
            }
        }
    }
}

/// Where a write goes, and the two roads are not the same shape.
///
/// **A whole disk is erased and a partition is not**, and the tool is given a
/// different subcommand for each: `to-disk` makes the table itself, and
/// `to-filesystem` is handed a root somebody else has already made and mounted.
/// Carrying them as one enum rather than an optional partition beside a disk is
/// what stops a caller passing both or neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Onto {
    /// A whole disk, erased and partitioned by the tool. The older road, and
    /// still the one a machine with a spare disk takes.
    AWholeDisk(DiskName),
    /// One partition of a disk that keeps everything else on it.
    ///
    /// The installer plan's task 4: alo OS beside Windows. The Windows
    /// installer has already shrunk Windows and made this partition; the
    /// environment makes a filesystem on it, mounts it, and installs into it.
    ///
    /// **`esp` is the EFI System Partition that is already there** — Windows'
    /// own. A disk has one, firmware looks only there, and a second would be a
    /// partition nothing reads. The loader goes into a directory of its own
    /// beside Windows', which is what an ESP is for and what every dual-boot
    /// machine does.
    BesideWhatIsThere {
        /// The partition alo OS is installed into, and nothing outside it.
        root: PartitionName,
        /// The EFI System Partition already on the disk.
        esp: PartitionName,
    },
}

/// The write, with everything it is given already checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Writing {
    /// The release, by digest.
    reference: String,
    /// Where it goes.
    onto: Onto,
    /// What the installed machine shows its start-up on.
    console: TheConsole,
}

impl Writing {
    /// The pinned release onto this disk, as a person's own machine.
    ///
    /// Shows its start-up on the screen and opens no serial console — ADR 0082.
    /// A caller that wants to watch the installed machine says so with
    /// [`Self::watched_on_a_serial_line`], in the same breath as asking.
    #[must_use]
    pub fn of(pin: &ThePin, disk: &DiskName) -> Self {
        Self {
            reference: pin.reference(),
            onto: Onto::AWholeDisk(disk.clone()),
            console: TheConsole::TheirScreen,
        }
    }

    /// The pinned release **into one partition**, keeping everything else on
    /// the disk — which on the machine this is for is somebody's Windows.
    ///
    /// The installer plan's task 4. The Windows installer has already shrunk
    /// Windows and made the partition; this writes into it and touches nothing
    /// around it. The loader goes into the EFI System Partition that is already
    /// there, in a directory of its own beside Windows'.
    #[must_use]
    pub fn beside_what_is_there(pin: &ThePin, root: &PartitionName, esp: &PartitionName) -> Self {
        Self {
            reference: pin.reference(),
            onto: Onto::BesideWhatIsThere {
                root: root.clone(),
                esp: esp.clone(),
            },
            console: TheConsole::TheirScreen,
        }
    }

    /// The same write, on a machine something is watching.
    ///
    /// **The asking, made visible.** ADR 0082: a serial console is a console, so
    /// it is granted deliberately rather than shipped. A walk that wants to read
    /// an installed system's start-up calls this; anything that forgets gets a
    /// quiet line, which is the state that ADR exists to stop being mysterious.
    #[must_use]
    pub fn watched_on_a_serial_line(mut self) -> Self {
        self.console = TheConsole::TheirScreenAndAWatchedLine;
        self
    }

    /// What the installed machine will show its start-up on.
    #[must_use]
    pub const fn console(&self) -> TheConsole {
        self.console
    }

    /// Where it writes.
    #[must_use]
    pub const fn onto(&self) -> &Onto {
        &self.onto
    }

    /// The disk it writes, where it writes a whole one.
    ///
    /// [`None`] on the road that keeps what is already there: that write has a
    /// partition and no disk, **and a caller that wanted the disk around it
    /// wanted the wrong thing.**
    #[must_use]
    pub const fn disk(&self) -> Option<&DiskName> {
        match &self.onto {
            Onto::AWholeDisk(disk) => Some(disk),
            Onto::BesideWhatIsThere { .. } => None,
        }
    }

    /// Where the root is mounted while the tool installs into it.
    ///
    /// Not a path anybody chooses: the tool is handed a root that is already
    /// mounted, and this environment is the only thing mounting it.
    pub const THE_ROOT: &'static str = "/run/alo-os-root";

    /// Where the EFI System Partition is mounted under that root.
    ///
    /// `bootc install to-filesystem` looks for the loader's home beneath the
    /// root it is given, which is where a booted machine would have it.
    pub const THE_ESP: &'static str = "/run/alo-os-root/boot/efi";

    /// The writer's arguments.
    ///
    /// On the whole-disk road, `--wipe` because the person agreed to replace
    /// this disk, and a disk with a table on it is otherwise refused by the
    /// tool — which is the tool asking the question the person already
    /// answered.
    ///
    /// **On the road beside Windows there is no `--wipe` and no `--filesystem`,
    /// and that is the point.** `to-filesystem` is handed a root that already
    /// exists, so the filesystem was chosen when it was made — by
    /// [`Program::MakingTheRoot`](crate::Program::MakingTheRoot), which uses the
    /// same [`THE_ONLY_FILESYSTEM`] for ADR 0045's reason. A `--wipe` on this
    /// road would be an instruction to erase the disk the person is keeping.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let mut arguments = match &self.onto {
            Onto::AWholeDisk(_) => vec![
                "install".to_owned(),
                "to-disk".to_owned(),
                "--source-imgref".to_owned(),
                format!("registry:{}", self.reference),
                "--wipe".to_owned(),
                "--filesystem".to_owned(),
                THE_ONLY_FILESYSTEM.to_owned(),
            ],
            Onto::BesideWhatIsThere { .. } => vec![
                "install".to_owned(),
                "to-filesystem".to_owned(),
                "--source-imgref".to_owned(),
                format!("registry:{}", self.reference),
            ],
        };
        // **One `--karg` per argument**, which is how the tool takes them: a
        // single flag holding both would be one string the kernel never splits.
        // Before this the install passed none at all, and the machine it left
        // behind said nothing on any line from the moment the firmware handed
        // over — so both starts of a kept computer read as a hang while the
        // screen showed a login prompt. ADR 0082.
        for karg in self.console.arguments() {
            arguments.push("--karg".to_owned());
            arguments.push(karg);
        }
        // **What the tool is pointed at, and the two roads point at different
        // kinds of thing.** `to-disk` takes the disk it will partition;
        // `to-filesystem` takes a directory somebody has already mounted a root
        // on. Naming a disk where a root is expected would hand the tool a
        // block device to walk as a filesystem.
        match &self.onto {
            Onto::AWholeDisk(disk) => arguments.push(disk.path().display().to_string()),
            Onto::BesideWhatIsThere { .. } => arguments.push(Self::THE_ROOT.to_owned()),
        }
        arguments
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::Path;

    use super::*;

    /// The write pulls the pinned digest onto the disk by its own name, and
    /// passes nothing a shell would read.
    #[test]
    fn the_write_pulls_the_pinned_digest_onto_the_named_disk() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk).arguments();
        assert_eq!(
            arguments,
            [
                "install",
                "to-disk",
                "--source-imgref",
                &format!("registry:ghcr.io/aloworld-org/alo-os@{}", pin.digest()),
                "--wipe",
                "--filesystem",
                "btrfs",
                "--karg",
                "console=tty0",
                "/dev/disk/by-id/virtio-alo-target",
            ]
        );
        assert!(
            !arguments.iter().any(|argument| argument.contains(':')
                && argument.contains(pin.version())
                && !argument.contains('@')),
            "the release is never named by its tag"
        );
    }

    /// **The disk is written with the one filesystem an undo can be taken on**,
    /// named once, and taken from the crate that holds every writer to it.
    ///
    /// A person cannot convert this afterwards, so an installer that named
    /// `ext4` here would be a machine that can never undo what an agent did
    /// without being reinstalled (ADR 0045). A second `--filesystem` in the
    /// arguments would be the same defect arriving quietly, which is why this
    /// counts them rather than looking one up.
    #[test]
    fn the_disk_is_written_with_the_one_filesystem_an_undo_can_be_taken_on() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk).arguments();

        let named: Vec<&String> = arguments
            .iter()
            .zip(arguments.iter().skip(1))
            .filter(|(argument, _)| argument.as_str() == "--filesystem")
            .map(|(_, value)| value)
            .collect();
        assert_eq!(
            named,
            [THE_ONLY_FILESYSTEM],
            "the installer names {} filesystem(s): {named:?}",
            named.len()
        );
    }

    /// **A person's machine shows its start-up on its own screen and opens no
    /// serial console.**
    ///
    /// ADR 0082's decision, asserted as the default rather than as an option:
    /// a caller who asks for nothing gets a machine with no extra way in.
    #[test]
    fn a_persons_machine_opens_no_serial_console() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let writing = Writing::of(&pin, &disk);

        assert_eq!(writing.console(), TheConsole::TheirScreen);
        let arguments = writing.arguments();
        assert!(
            arguments.iter().any(|it| it == THE_PERSONS_SCREEN),
            "a person's machine was not told which screen to start on"
        );
        assert!(
            !arguments.iter().any(|it| it.contains("ttyS")),
            "a person's machine was given a serial console nobody asked for"
        );
    }

    /// **A watched machine gets both, and the serial line is named last.**
    ///
    /// The last console named takes `/dev/console`, so the order is what lets a
    /// walk read the start-up **without** taking the kernel's messages off the
    /// screen a person is looking at. A test that only checked both were present
    /// would pass for the arrangement that breaks a laptop.
    #[test]
    fn a_watched_machine_keeps_the_screen_and_names_the_line_last() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk)
            .watched_on_a_serial_line()
            .arguments();

        let screen = arguments.iter().position(|it| it == THE_PERSONS_SCREEN);
        let line = arguments.iter().position(|it| it == A_WATCHED_LINE);
        assert!(
            screen.is_some() && line.is_some(),
            "a watched machine is missing a console"
        );
        assert!(
            screen < line,
            "the serial line was not named last, so the kernel's messages leave the person's screen"
        );
    }

    /// **Every console argument is its own `--karg`.**
    ///
    /// The tool takes one per flag. A single flag holding both would be one
    /// string the kernel never splits, which is a machine that boots with a
    /// console named `tty0 console=ttyS0,115200n8` and no console at all.
    #[test]
    fn each_console_argument_is_its_own_flag() {
        let pin = ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let arguments = Writing::of(&pin, &disk)
            .watched_on_a_serial_line()
            .arguments();

        assert_eq!(
            arguments.iter().filter(|it| *it == "--karg").count(),
            2,
            "the two consoles did not arrive as two flags"
        );
        assert!(
            !arguments.iter().any(|it| it.contains(' ')),
            "an argument holds a space, so something was packed into one string"
        );
    }

    /// A pin to build writes from.
    fn a_pin() -> ThePin {
        ThePin::read(
            &std::fs::read_to_string(Path::new(alo_image::THE_IMAGE).join(alo_image::THE_PIN))
                .unwrap(),
        )
        .unwrap()
    }

    /// **The road that keeps Windows never says `--wipe`.**
    ///
    /// This is the one that would cost somebody their computer. `--wipe` tells
    /// the tool to erase the disk and make its own table, and on this road the
    /// disk is the one the person is keeping. The whole-disk road says it
    /// because the person agreed to replace that disk; this road has no such
    /// agreement and must never carry the flag that acts on one.
    #[test]
    fn the_road_that_keeps_windows_never_wipes() {
        let root = PartitionName::named("virtio-alo-target-part4").unwrap();
        let esp = PartitionName::named("virtio-alo-target-part1").unwrap();
        let arguments = Writing::beside_what_is_there(&a_pin(), &root, &esp).arguments();

        assert!(
            !arguments.iter().any(|it| it == "--wipe"),
            "the write that keeps Windows carries --wipe, which erases the disk it is keeping: \
             {arguments:?}"
        );
    }

    /// **And it never names a file system**, because it did not make one.
    ///
    /// `to-filesystem` is handed a root that already exists. The choice of
    /// file system was made when it was made — by `Program::MakingTheRoot`,
    /// with the same `THE_ONLY_FILESYSTEM` for ADR 0045's reason. A
    /// `--filesystem` here would be an argument the tool has nothing to do
    /// with, and a reader would think this road chose it.
    #[test]
    fn the_road_that_keeps_windows_names_no_filesystem() {
        let root = PartitionName::named("virtio-alo-target-part4").unwrap();
        let esp = PartitionName::named("virtio-alo-target-part1").unwrap();
        let arguments = Writing::beside_what_is_there(&a_pin(), &root, &esp).arguments();

        assert!(
            !arguments.iter().any(|it| it == "--filesystem"),
            "the write that keeps Windows names a file system it did not make: {arguments:?}"
        );
    }

    /// **It is handed a root, not a disk.**
    ///
    /// The last argument is what the tool acts on, and the two roads point at
    /// different kinds of thing: a block device to partition, or a directory
    /// somebody already mounted a root on. Naming a disk where a root is
    /// expected hands the tool a device to walk as a file system.
    #[test]
    fn the_road_that_keeps_windows_is_handed_a_root() {
        let root = PartitionName::named("virtio-alo-target-part4").unwrap();
        let esp = PartitionName::named("virtio-alo-target-part1").unwrap();
        let arguments = Writing::beside_what_is_there(&a_pin(), &root, &esp).arguments();

        assert_eq!(arguments.first().map(String::as_str), Some("install"));
        assert_eq!(arguments.get(1).map(String::as_str), Some("to-filesystem"));
        assert_eq!(
            arguments.last().map(String::as_str),
            Some(Writing::THE_ROOT)
        );
        assert!(
            !arguments.iter().any(|it| it.contains("/dev/disk/by-id/")),
            "the write that keeps Windows names a device, and it should name a mounted root: \
             {arguments:?}"
        );
    }

    /// **A write that keeps Windows has no disk to give**, and says so rather
    /// than giving the disk the partition happens to sit on.
    ///
    /// Everything downstream — what is said, what is tidied — is about the disk
    /// a person chose, and it gets that from what they were told, not from the
    /// write. A caller reaching here for a disk is a caller asking the wrong
    /// thing, and `None` is how it finds that out at compile time.
    #[test]
    fn a_write_that_keeps_windows_has_no_disk() {
        let root = PartitionName::named("virtio-alo-target-part4").unwrap();
        let esp = PartitionName::named("virtio-alo-target-part1").unwrap();

        assert_eq!(
            Writing::beside_what_is_there(&a_pin(), &root, &esp).disk(),
            None
        );
        let disk = DiskName::named("virtio-alo-target").unwrap();
        assert_eq!(Writing::of(&a_pin(), &disk).disk(), Some(&disk));
    }

    /// **Both roads still pull the same pinned release, by digest.**
    ///
    /// Whatever changes about where it lands, what lands is the release
    /// `crate::verifying` checked — not a tag, not a different reference.
    #[test]
    fn both_roads_pull_the_pinned_release() {
        let pin = a_pin();
        let disk = DiskName::named("virtio-alo-target").unwrap();
        let root = PartitionName::named("virtio-alo-target-part4").unwrap();
        let esp = PartitionName::named("virtio-alo-target-part1").unwrap();

        let whole = Writing::of(&pin, &disk).arguments();
        let beside = Writing::beside_what_is_there(&pin, &root, &esp).arguments();

        let reference = |arguments: &[String]| {
            arguments
                .iter()
                .zip(arguments.iter().skip(1))
                .find(|(argument, _)| argument.as_str() == "--source-imgref")
                .map(|(_, value)| value.clone())
        };

        assert_eq!(reference(&whole), reference(&beside));
        assert!(
            reference(&whole).is_some_and(|it| it.contains("@sha256:")),
            "the release is pulled by something other than a digest"
        );
    }
}
