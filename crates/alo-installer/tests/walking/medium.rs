//! The two discs the guest is handed: the one that installs Windows
//! unattended, and the one that tells a boot what to do and carries the
//! download.
//!
//! A disc rather than a disk, deliberately. A third disk would be a third disk
//! `Get-Disk` reports, and `deciding.rs` decides what to offer from exactly
//! that list — so the walk would be measuring an installer that was shown a
//! machine no person has. A CD-ROM is not a disk to Windows' storage cmdlets,
//! so the guest sees the two disks the acceptance asks for and no others.

use std::path::{Path, PathBuf};

use super::guest;
use super::machine::run;

/// What the guest's every-start script looks for on the walk disc.
pub const WHERE_THE_INSTRUCTION_IS: &str = "alo-walk";

/// What one boot is told to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Told {
    /// Read the manifest and the state, and change nothing. This is how a boot
    /// that only has to show Windows starting says so.
    JustLook,
    /// Run the installer the same way and type nothing at the consent, so it
    /// makes every read and every side effect running it has on Windows, and
    /// refuses.
    RefuseAtTheConsent,
    /// Run the installer and kill it the moment this step of `staging.rs` has
    /// happened.
    KillAfterStep(u8),
    /// Run the installer all the way, and let it restart the computer.
    TheWholeRoad,
}

impl Told {
    /// The instruction, as the guest reads it.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::JustLook => "mode=manifest-only\nstep=0\n".to_owned(),
            Self::RefuseAtTheConsent => "mode=refuse\nstep=0\n".to_owned(),
            Self::KillAfterStep(step) => format!("mode=kill-at-step\nstep={step}\n"),
            Self::TheWholeRoad => "mode=whole-road\nstep=0\n".to_owned(),
        }
    }
}

/// Build the disc that installs Windows unattended.
///
/// # Panics
/// When the disc cannot be written.
#[must_use]
pub fn the_answer_disc(yard: &Path) -> PathBuf {
    let tree = yard.join("answer");
    let _ = std::fs::remove_dir_all(&tree);
    std::fs::create_dir_all(tree.join("alo")).expect("somewhere for the answer");
    write_for_windows(&tree.join("autounattend.xml"), guest::THE_ANSWER_FILE);
    write_for_windows(
        &tree.join("alo/setup-guest.cmd"),
        guest::SETTING_THE_GUEST_UP,
    );
    write_for_windows(&tree.join("alo/at-every-start.cmd"), guest::AT_EVERY_START);
    an_iso(yard, &tree, "answer.iso", "UNATTEND")
}

/// Build the disc that tells this boot what to do and carries the download.
///
/// # Panics
/// When the disc cannot be written.
#[must_use]
pub fn the_walk_disc(yard: &Path, told: &Told, download: &Path) -> PathBuf {
    let tree = yard.join("walk");
    let _ = std::fs::remove_dir_all(&tree);
    let inside = tree.join(WHERE_THE_INSTRUCTION_IS);
    std::fs::create_dir_all(&inside).expect("somewhere for the instruction");
    write_for_windows(&inside.join("instruction.txt"), &told.written());
    write_for_windows(&inside.join("walk.cmd"), guest::THE_WALKS_COMMAND);
    write_for_windows(&inside.join("walk.ps1"), guest::THE_WALK);
    run(
        "cp",
        &[
            "-r",
            &download.display().to_string(),
            &inside.join("payload").display().to_string(),
        ],
    );
    an_iso(yard, &tree, "walk.iso", "ALOWALK")
}

/// Build the disc that settles the installed Windows once, before it becomes
/// the base: fast startup off, ten minutes for Windows' own first-start work,
/// and a shutdown it asks for itself.
///
/// # Panics
/// When the disc cannot be written.
#[must_use]
pub fn the_settle_disc(yard: &Path) -> PathBuf {
    let tree = yard.join("settle");
    let _ = std::fs::remove_dir_all(&tree);
    let inside = tree.join(WHERE_THE_INSTRUCTION_IS);
    std::fs::create_dir_all(&inside).expect("somewhere for the settling");
    write_for_windows(&inside.join("walk.cmd"), guest::SETTLING);
    an_iso(yard, &tree, "settle.iso", "ALOWALK")
}

/// A file Windows reads, with the line endings Windows' own shell wants.
///
/// A `.cmd` whose lines end without a carriage return is read by `cmd.exe` in
/// ways that depend on what is in it; the walk does not depend on finding out
/// which ones.
fn write_for_windows(at: &Path, text: &str) {
    let windows: String = text.replace("\r\n", "\n").replace('\n', "\r\n");
    std::fs::write(at, windows.as_bytes())
        .unwrap_or_else(|why| panic!("{} could not be written: {why}", at.display()));
}

/// A directory as a disc.
fn an_iso(yard: &Path, tree: &Path, called: &str, label: &str) -> PathBuf {
    let iso = yard.join(called);
    let _ = std::fs::remove_file(&iso);
    run(
        "genisoimage",
        &[
            "-quiet",
            "-o",
            &iso.display().to_string(),
            "-J",
            "-r",
            "-V",
            label,
            &tree.display().to_string(),
        ],
    );
    iso
}
