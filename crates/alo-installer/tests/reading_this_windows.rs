//! The installer's checks, run against the Windows this test is running on.
//!
//! **Reads only.** Every program is passed through a machine that refuses to
//! start one that changes anything, so running this on a person's own computer
//! is as safe as opening Task Manager. What it shows is that each check is
//! answered by Windows' own tools on a real Windows — the PowerShell scripts
//! parse, the cmdlets exist, the JSON is the shape `alo-installer` reads — and
//! it prints what was found, which is what a report pastes.
//!
//! Some answers need an administrator (Secure Boot above all), and an
//! unelevated run is told *could not be found out* for those — which is itself
//! the behaviour under test: not known is never read as off.
//!
//! Compiled on Windows only; on any other host there is no Windows to read.

#![cfg(windows)]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_installer::{OnThisMachine, Program, Ran, TheMachine, check, installer_words};
use alo_strings::{Said, Strings};

/// This Windows, with every change refused and every program recorded.
struct OnlyReading {
    /// The real machine.
    windows: OnThisMachine,
    /// Every program asked for.
    asked_for: Vec<Program>,
}

impl TheMachine for OnlyReading {
    fn say(&mut self, said: &Said) {
        eprintln!("{}", said.text());
    }

    fn ask(&mut self, _said: &Said) -> String {
        String::new()
    }

    fn run(&mut self, program: &Program) -> std::io::Result<Ran> {
        self.asked_for.push(program.clone());
        if program.changes() {
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        }
        self.windows.run(program)
    }

    fn downloaded_into(&mut self) -> std::io::Result<PathBuf> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }

    fn read(&mut self, _file: &Path) -> std::io::Result<Vec<u8>> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }

    fn write(&mut self, _file: &Path, _bytes: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
    }

    fn pause(&mut self, _for_as_long_as: Duration) {}
}

/// **This Windows answers the installer's checks, and nothing is changed.**
#[test]
fn this_windows_answers_every_check_and_nothing_is_changed() {
    let mut machine = OnlyReading {
        windows: OnThisMachine::found().expect("this is a Windows with a system directory"),
        asked_for: Vec::new(),
    };
    let elevated = alo_installer::is_an_administrator(&mut machine);
    let found = check(&mut machine);

    let strings = Strings::of(installer_words().expect("the installer's words declare"));
    eprintln!("administrator: {elevated}");
    for said in found.said(&strings) {
        eprintln!("{}", said.text());
    }
    eprintln!("{found:#?}");
    eprintln!(
        "decided: {:?}",
        alo_installer::decide(&found).map(|offer| offer.disks_for_alo_os)
    );

    assert!(
        machine.asked_for.iter().all(|program| !program.changes()),
        "{:?}",
        machine.asked_for
    );
    assert_eq!(machine.asked_for.len(), 9, "{:?}", machine.asked_for);

    // What Windows answers without an administrator, it answers here.
    assert!(
        found.starting.uefi.is_some(),
        "how this Windows starts was not read"
    );
    assert!(found.memory.is_some(), "memory was not read");
    let disks = found.disks.as_ref().expect("the disks were read");
    assert!(disks.every().next().is_some());
    // Secure Boot, the TPM, BitLocker, how far Windows can shrink, and the
    // firmware's list need an administrator: elevated, they are answered; not,
    // they are not known — and never off, absent or empty.
    if elevated {
        assert!(found.starting.secure_boot.is_some());
        assert!(found.an_entry_is_named_alo_os.is_some());
        assert_ne!(found.chip, alo_installer::SecurityChip::NotRead);
        let windows = found.windows.expect("the Windows volume was read");
        assert!(windows.size > 0 && windows.free <= windows.size);
        assert!(
            disks.numbered(windows.disk).is_some(),
            "the disk Windows is on is among the disks"
        );
    } else {
        assert_eq!(found.starting.secure_boot, None);
        assert_eq!(found.chip, alo_installer::SecurityChip::NotRead);
        assert_eq!(found.bitlocker, alo_installer::BitLocker::NotRead);
        assert_eq!(found.windows, None);
        assert_eq!(found.an_entry_is_named_alo_os, None);
    }
}
