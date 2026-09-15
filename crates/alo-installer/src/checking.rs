//! Asking Windows about the computer, with reads and nothing else.
//!
//! Each question is one read-only program from `crate::program`, and each
//! answer is handed to the file that knows how to read it. A program that did
//! not start, did not succeed, or printed something that is not its answer is
//! a question not answered — never a default.

use crate::bitlocker::BitLocker;
use crate::disks::Disks;
use crate::entries;
use crate::found::Found;
use crate::machine::TheMachine;
use crate::memory;
use crate::program::Program;
use crate::security_chip::SecurityChip;
use crate::starting::Starting;
use crate::windows_volume::WindowsVolume;

/// Whether this process has an administrator's rights; `false` when that could
/// not be read.
pub fn is_an_administrator(machine: &mut impl TheMachine) -> bool {
    answered(machine, &Program::AskingWhetherThisIsAnAdministrator)
        .is_some_and(|printed| crate::administrator::is_elevated(&printed))
}

/// Everything the installer asks about the computer.
///
/// Runs only programs that change nothing, which
/// `tests/the_installer_checks_consents_and_stages.rs` holds it to.
pub fn check(machine: &mut impl TheMachine) -> Found {
    Found {
        starting: Starting::read(answered(machine, &Program::ReadingHowItStarts).as_deref()),
        chip: SecurityChip::read(answered(machine, &Program::ReadingTheTpm).as_deref()),
        bitlocker: BitLocker::read(answered(machine, &Program::ReadingBitLocker).as_deref()),
        memory: memory::read(answered(machine, &Program::ReadingTheMemory).as_deref()),
        windows: WindowsVolume::read(
            answered(machine, &Program::ReadingTheWindowsVolume).as_deref(),
        ),
        disks: Disks::read(answered(machine, &Program::ListingTheDisks).as_deref()),
        an_entry_is_named_alo_os: answered(machine, &Program::ListingTheStartEntries)
            .map(|printed| entries::one_is_named_alo_os(&printed)),
    }
}

/// What a read printed, when it ran and succeeded.
fn answered(machine: &mut impl TheMachine, program: &Program) -> Option<String> {
    debug_assert!(!program.changes(), "{program:?} is not a read");
    if program.changes() {
        return None;
    }
    machine
        .run(program)
        .ok()
        .filter(|ran| ran.succeeded)
        .map(|ran| ran.printed)
}
