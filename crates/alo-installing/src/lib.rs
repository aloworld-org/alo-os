//! The installer's reboot half: the boot environment that installs alo OS.
//!
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §2–3: the installer a person downloads *stages a minimal boot environment
//! and a UEFI boot entry, then reboots*, and that environment *runs `bootc
//! install`, pulling alo OS from the registry over HTTPS, verifying signatures
//! before writing.* This crate is the program inside that environment, and
//! `image/installing/` is the recipe the environment is built from.
//!
//! # What the environment is
//!
//! A kernel and an initramfs from the same pinned base the operating system is
//! built on, holding the tool that writes the disk, the container tooling it
//! needs, the checker for the owner's signature, and this program — and
//! nothing else. It is not a second operating system: it has no shell anybody
//! is offered, no login, and one unit, which runs [`install`] and stops.
//!
//! # What it is told, and what it never assumes
//!
//! What arrives from outside it is the choice the person made on the machine
//! they still had, carried on the kernel command line ([`Told`]): which disk,
//! and — on the road that keeps the system already on that disk — which two
//! partitions of it. What it installs and which key it accepts were built into
//! it ([`Environment`]) and never arrive over the network. It never touches a
//! disk it was not told to; it refuses a disk that holds this installer or
//! another operating system, and on the road that keeps one it refuses any
//! partition but the one the installer on Windows made and labelled
//! ([`Disks`]); and it writes nothing until the release is shown to be the
//! owner's ([`Verifying`]).
//!
//! # What it says
//!
//! Every step, as it begins, on every console the machine has (`console`),
//! in the words `alo-saying` collects (`words`). The person watching has no
//! other window.
//!
//! # What this crate is not
//!
//! **It is not the program a person downloads.** That is task 3 of the
//! installer plan — the Windows program that checks the machine, takes the
//! typed consent, stages this environment and adds the boot entry. What that
//! program owes this one is written in `docs/booting.md`: the environment's
//! files on a FAT partition labelled [`THIS_INSTALLER`], the chosen disk's name
//! in `EFI/BOOT/chosen.cfg` beside the loader's configuration, and a boot
//! entry reached once, through the firmware's next-boot choice, so that the
//! restart this environment ends with never lands back in it.
//!
//! On the road that keeps the system already on the disk it owes one thing
//! more: the space it made for alo OS carries the label
//! [`THE_SPACE_THE_INSTALLER_MADE`], and the two partitions' names are on the
//! command line ([`THE_PARTITION`], [`THE_EFI`]). The label is not a
//! convenience — it is the whole of this environment's permission to make a
//! file system on that partition, and without it a command line staged wrongly
//! would be the end of somebody's Windows.

mod complaint;
mod console;
mod disk;
mod disks;
mod ended;
mod entries;
mod environment;
mod machine;
mod program;
#[cfg(target_os = "linux")]
mod running;
mod sequence;
mod tidying;
mod told;
mod verifying;
mod words;
mod writing;

pub use complaint::{Complaint, THE_LAST_LINES};
pub use console::{ACTIVE, every_console, every_serial_line};
pub use disk::{BY_ID, DiskName, NotADisk, NotAPartition, PartitionName};
pub use disks::{
    ANOTHER_SYSTEMS, Disks, NotKeeping, Replacing, THE_SPACE_THE_INSTALLER_MADE, THE_START_UP_AREA,
    THIS_INSTALLER, Unsuitable,
};
pub use ended::{Ended, Refusal};
pub use entries::{Entries, Entry, THE_ENTRYS_NAME, THE_LOADER_THE_BASE_INSTALLS};
pub use environment::{Environment, WHERE_IT_IS};
pub use machine::{BEFORE_RESTARTING, STILL_EVERY, THE_DISK_APPEARS_WITHIN, TheMachine};
pub use program::{EVERY_PROGRAM, Program, Ran};
#[cfg(target_os = "linux")]
pub use running::OnThisMachine;
pub use sequence::install;
pub use tidying::{Tidied, tidy_up};
pub use told::{
    Beside, NotTold, REPLACING, THE_CHOICE, THE_EFI, THE_PARTITION, THE_REPLACING, Told,
};
pub use verifying::{Verified, Verifying};
pub use words::{EVERY_WORD, WordsError, declare_into, installing_words};
pub use writing::{A_WATCHED_LINE, THE_PERSONS_SCREEN, TheConsole, Writing};
