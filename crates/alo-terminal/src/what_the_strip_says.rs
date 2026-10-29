//! What the strip across the top of the terminal says: the folder the shell is
//! in, where the shell runs, and who is signed in.
//!
//! # What the design draws, and what the owner decided it means
//!
//! The design's `Project and session identity` strip (`479:29013`) reads
//! `~/Projects/launch` at its left and `Local · disan` at its right. The design
//! does not say what those stand for, and on 2026-10-28 the owner decided:
//!
//! - **the folder is the one the shell is in now**, and it follows the person
//!   as they change folder, written with `~` for their home;
//! - **`Local` means the shell runs on this machine** rather than on a paired
//!   one;
//! - **the name is the signed-in person's.**
//!
//! # How the folder is known without anything watching
//!
//! The kernel keeps every process's working folder as a link,
//! `/proc/PID/cwd`, and [`crate::what_the_kernel_says::where_this_process_is`]
//! reads it once when asked. The terminal asks when the shell has written
//! something — a `cd` is always followed by a new prompt — so the strip moves
//! when the person does and nothing reads in between. **No background reader
//! and no shell integration is needed**: alo's integration
//! (`docs/contracts/shell-integration-marks.md`) sends the marks of ADR 0123,
//! which do not include a folder, and a terminal that only knew the folder
//! when a snippet was installed would show nothing on most machines.
//!
//! This file is the part with no kernel in it — how a folder is written for a
//! person and which word says where the shell runs — so it is held to its cases
//! on any platform.

use std::path::{Component, Path};

use alo_strings::Word;

/// Where the shell a person is typing into runs.
///
/// **One answer today, and the type is the promise that there will be two.**
/// A terminal reaching a paired machine is not built, so every shell runs here;
/// the strip still says so in words rather than leaving a person to assume it,
/// and the second answer arrives as a second arm a `match` cannot forget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhereTheShellRuns {
    /// On this machine, the one the person is sitting at.
    OnThisMachine,
}

impl WhereTheShellRuns {
    /// The word the strip shows for it.
    #[must_use]
    pub const fn the_word(self) -> Word {
        match self {
            Self::OnThisMachine => crate::words::ON_THIS_MACHINE,
        }
    }
}

/// How a folder is written in the strip: with `~` standing for the person's
/// home, and in full anywhere else.
///
/// `~` alone is the home itself and `~/Projects/launch` is a folder inside it —
/// the form the design draws and the form every shell prompt uses. **A folder
/// outside the home is written in full**, because `~` means *your home* and
/// nothing else, and **a home of `/` is not abbreviated**: every folder on the
/// disk is inside `/`, and writing `/etc` as `~/etc` would tell a person they
/// were somewhere they are not.
///
/// The comparison is by **whole folder names**, so a home of `/home/disan` does
/// not swallow `/home/disandra`. A name that is not valid Unicode is shown with
/// its unreadable bytes replaced rather than not shown at all.
#[must_use]
pub fn the_folder_as_a_person_reads_it(folder: &Path, home: Option<&Path>) -> String {
    let home = home.filter(|home| home.components().any(|part| part != Component::RootDir));
    let Some(inside) = home.and_then(|home| folder.strip_prefix(home).ok()) else {
        return folder.to_string_lossy().into_owned();
    };
    if inside.as_os_str().is_empty() {
        return "~".to_owned();
    }
    format!("~/{}", inside.to_string_lossy())
}

#[cfg(test)]
#[path = "what_the_strip_says_tests.rs"]
mod tests;
