//! **Whether an installed application is drawn by Chromium**, read from the
//! files its package carries — so that alo can read the page inside any
//! Chromium browser's or Electron application's window, whatever it is
//! called and however it was built.
//!
//! `crate::built_on_electron` knows an application built on Electron's
//! Flathub base by its manifest. That leaves out every Chromium browser —
//! Chrome, Brave, Edge — which are not built on that base, and any Electron
//! application packaged another way; before this they were known only by the
//! closed list `alo_launching::DRAWN_BY_CHROMIUM`, one name at a time. **The
//! owner's rule of 2026-11-17 is that what alo can reach in an application is
//! decided by its toolkit, never by its name.**
//!
//! # What is read
//!
//! **Whether the package carries `chrome_100_percent.pak`**, the resource file
//! Chromium itself ships beside its program, which every Chromium browser and
//! every Electron application carries and no other toolkit does — Qt's web
//! engine, which is Chromium inside but reads none of Chromium's switches,
//! names its own `qtwebengine_resources`. It is looked for below the package's
//! files, including `extra`, where a browser that downloads itself at install
//! lands. **Only a regular file of exactly that name counts**; a link is never
//! followed, so a package cannot point at somebody else's Chromium and be
//! taken for one. The look is bounded — [`DEEPEST`] folders down and
//! [`MOST_ENTRIES`] entries — because it is asked again at each act.
//!
//! **Not known is not Chromium**: the switch it decides is handed only to a
//! program known to take it, because one handed to a program that does not
//! may be read as a file to open.
//!
//! **Unmeasured on an alo machine**: that each Flathub browser carries the file
//! where its build put it is how Chromium lays itself out, read from the
//! packages' build recipes, not from a booted machine.

use std::path::{Path, PathBuf};

use crate::built_on_electron::{is_a_plain_identifier, the_packages_files};

/// The file whose presence says a package is drawn by Chromium.
pub const CHROMIUMS_OWN_FILE: &str = "chrome_100_percent.pak";

/// How many folders below the package's files are looked into.
pub const DEEPEST: usize = 6;

/// How many entries one look reads at most.
pub const MOST_ENTRIES: usize = 20_000;

/// Whether the application with this identifier carries Chromium's own file,
/// in the person's installation (`persons`) or the machine's (`machines`).
#[must_use]
pub fn is_built_on_chromium(machines: &Path, persons: Option<&Path>, identifier: &str) -> bool {
    if !is_a_plain_identifier(identifier) {
        return false;
    }
    persons
        .into_iter()
        .chain(std::iter::once(machines))
        .any(|installation| carries_chromiums_file(&the_packages_files(installation, identifier)))
}

/// Whether Chromium's own file is somewhere below `files`, within the bounds.
fn carries_chromiums_file(files: &Path) -> bool {
    let mut waiting: Vec<(PathBuf, usize)> = vec![(files.to_path_buf(), 0)];
    let mut read = 0_usize;
    while let Some((folder, depth)) = waiting.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            read += 1;
            if read > MOST_ENTRIES {
                return false;
            }
            // **The entry itself, never what a link points at.**
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_file() && entry.file_name() == CHROMIUMS_OWN_FILE {
                return true;
            }
            if kind.is_dir() && depth + 1 < DEEPEST {
                waiting.push((entry.path(), depth + 1));
            }
        }
    }
    false
}

#[cfg(test)]
#[path = "built_on_chromium_tests.rs"]
mod tests;
