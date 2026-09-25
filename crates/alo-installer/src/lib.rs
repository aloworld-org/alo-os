//! The installer a person downloads: check, say, consent, stage, restart.
//!
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §1–2: *a signed Windows executable, downloaded from the website. It checks
//! the machine, says exactly what will happen, and takes a typed consent — not
//! a checkbox — for anything destructive. It stages a minimal boot environment
//! and a UEFI boot entry, then reboots.* This crate is that program, and
//! `crates/alo-installing` is the environment it stages.
//!
//! # What it does, in order
//!
//! [`install`] is the whole of it, and `sequence.rs` says each step. It checks
//! that it may change the computer, that the environment beside it is genuine,
//! and what the computer is — UEFI, Secure Boot, TPM, BitLocker, free space,
//! memory, disks — each read from Windows' own tools ([`check`]) and each said.
//! **With Secure Boot on it refuses and says why, and never suggests the
//! setting be changed** (ADR 0033 §4). It says exactly what will happen, takes
//! the name of the disk alo OS replaces as a typed consent, shrinks Windows by
//! the area through Windows' own tool, stages the environment there, adds a
//! start-up entry named alo OS, makes it the next start once, and restarts.
//!
//! # What it never does
//!
//! - **Change anything before the person agreed**, or claim nothing was changed
//!   when something was ([`Ended::NotPutBack`] says exactly what remains).
//! - **Run anything but the programs in `program.rs`**, each a variant with
//!   typed arguments, or write a raw IOCTL: `unsafe_code` is forbidden, and
//!   disks change through Windows' own storage cmdlets and `bcdedit`.
//! - **Show a key, a password or a signature** (ADR 0036). A download that is
//!   not genuine is *this download is not a genuine alo OS, so nothing was
//!   changed*, with no way past it.
//! - **Install onto the disk Windows is on.** The environment replaces one whole
//!   empty disk; putting alo OS into the same disk beside Windows is the
//!   installer plan's task 4.
//!
//! # And it runs on Windows
//!
//! On any other host this crate compiles to its types and its decisions, which
//! is where it is tested; the program says so and ends.

mod administrator;
mod asking;
mod bitlocker;
mod checking;
mod consent;
mod deciding;
mod defaulting;
mod disks;
mod encoded;
mod ended;
mod entries;
mod environment;
mod fast_startup;
mod found;
mod identities;
mod machine;
mod memory;
mod naming;
#[cfg(windows)]
mod on_windows;
mod program;
mod security_chip;
mod sequence;
mod sizes;
mod staging;
mod starting;
mod switching;
mod windows_volume;
mod words;

pub use administrator::HIGH_MANDATORY_LEVEL;
pub use bitlocker::BitLocker;
pub use checking::{check, is_an_administrator};
pub use deciding::{ForAloOs, Offer, decide};
pub use defaulting::{
    THE_DEFAULTS_WORD, THE_START_PARTITIONS_LETTER, TheDefault, the_block, which_system_starts,
};
pub use disks::{Disk, Disks, Standing};
pub use ended::{Ended, Refusal, Remains};
pub use environment::{
    EVERY_FILE_IT_NEEDS, NotStaged, Released, THE_CHOICE, THE_CHOICE_BEGINS, THE_DIRECTORY,
    THE_LIST, THE_RELEASED_LIST, TheEnvironment, sha256_hex,
};
pub use fast_startup::FastStartup;
pub use found::Found;
pub use identities::{DiskNumber, Entry, Letter, PartitionNumber};
pub use machine::{BEFORE_RESTARTING, Ran, TheMachine};
pub use memory::MADE_FOR;
#[cfg(windows)]
pub use on_windows::OnThisMachine;
pub use program::{
    BASIC_DATA, Program, THE_ENTRYS_NAME, THE_LOADER, THE_PROGRAMS_HOME, THE_PROGRAMS_NAME,
    THE_SHORTCUT, Tool,
};
pub use security_chip::SecurityChip;
pub use sequence::install;
pub use sizes::{GIB, MIB, THE_AREA, THE_LEAST_DISK, WINDOWS_KEEPS_FREE};
pub use starting::Starting;
pub use switching::{Switched, THE_SWITCHS_WORD, restart_into_alo_os};
pub use windows_volume::{NotEnoughSpace, Shrink, WindowsVolume};
pub use words::{
    ANSWER_LEAVE_ON, ANSWER_TURN_OFF, ASK_FAST_STARTUP, DEFAULT_CHANGE_IT, DEFAULT_CHANGED,
    DEFAULT_IS, DEFAULT_KEPT, DEFAULT_NOT_REACHED, DEFAULT_NOT_READ, DEFAULT_NOT_THERE,
    EVERY_REFUSAL, EVERY_WORD, FAST_STARTUP_LEFT_ON, PRESS_ENTER_TO_CLOSE,
    REMAINS_FAST_STARTUP_OFF, SWITCH_AGREED, SWITCH_NOT_AGREED, SWITCH_NOT_READ, SWITCH_NOT_SET,
    SWITCH_NOT_THERE, SWITCH_WILL_RESTART, WordsError, declare_into, installer_words,
};
