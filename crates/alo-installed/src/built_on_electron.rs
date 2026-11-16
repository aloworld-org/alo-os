//! **Whether an installed application is built on Electron**, read from its
//! own package — so that alo can read the page inside any Electron
//! application's window, not only the three it names.
//!
//! `docs/features.md` promises that *Electron applications started the right
//! way* describe their window to alo, *which alo does for them by default*.
//! What "the right way" is, is a command-line switch Chromium reads
//! (`alo_launching::THE_PAGE_TOO`), and it is handed only to an application
//! known to be drawn by Chromium, because a switch handed to a program that
//! does not know it may be read as a file to open. Until 2026-11-16 the only
//! way to be known was the closed list `alo_launching::DRAWN_BY_CHROMIUM` —
//! VSCodium, VS Code and Chromium — so every other Electron application a
//! person installed showed alo its window frame and nothing inside it.
//!
//! # What is read
//!
//! **The manifest its build left in the package.** `flatpak-builder` writes
//! the manifest an application was built from to `files/manifest.json`, and
//! an application built on Electron names Electron's base application there:
//! `"base": "org.electronjs.Electron2.BaseApp"`. That base is what carries
//! Electron, so an application built on it is drawn by Chromium and takes
//! Chromium's switches. Nothing else is read, and nothing is run.
//!
//! **Unmeasured on an alo machine**: that a Flathub package holds the file, and
//! that each such application hands the switch on to Chromium, are
//! `flatpak-builder`'s and Electron's documented behaviour, not something a
//! booted machine has shown. The closed list stays beside this, so an
//! application installed some other way is still known by name.

use std::path::Path;

use crate::where_flatpak_exports::{THE_MACHINES_INSTALLATION, the_persons_installation};

/// The base application Electron applications on Flathub are built on.
pub const ELECTRON_BASE_APP: &str = "org.electronjs.Electron2.BaseApp";

/// Whether the application with this identifier is built on Electron, read
/// from its package in the person's installation (`persons`) or the
/// machine's (`machines`), the person's first, as flatpak runs it.
///
/// `false` where there is no package or no manifest: **not known is not
/// Electron**, because the switch is handed only to what is known to take it.
#[must_use]
pub fn is_built_on_electron(machines: &Path, persons: Option<&Path>, identifier: &str) -> bool {
    // An identifier names a directory below, so one that could name another
    // place is not looked for at all.
    if identifier.is_empty()
        || identifier.starts_with('.')
        || identifier.contains("..")
        || !identifier
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return false;
    }
    persons
        .into_iter()
        .chain(std::iter::once(machines))
        .find_map(|installation| {
            std::fs::read_to_string(
                installation
                    .join("app")
                    .join(identifier)
                    .join("current")
                    .join("active")
                    .join("files")
                    .join("manifest.json"),
            )
            .ok()
        })
        .is_some_and(|manifest| says_its_base_is_electron(&manifest))
}

/// Whether the application with this identifier is drawn by Chromium **on
/// this machine, for the person this process runs as**: on the closed list
/// (`alo_launching::is_drawn_by_chromium`), or built on Electron by its own
/// package, read from the machine's installation and the person's (`HOME`,
/// `XDG_DATA_HOME`) now.
#[must_use]
pub fn is_drawn_by_chromium(identifier: &str) -> bool {
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    let data_home = std::env::var_os("XDG_DATA_HOME");
    let persons = the_persons_installation(home.as_deref(), data_home.as_deref());
    is_drawn_by_chromium_under(
        Path::new(THE_MACHINES_INSTALLATION),
        persons.as_deref(),
        identifier,
    )
}

/// [`is_drawn_by_chromium`], reading these installations.
#[must_use]
pub fn is_drawn_by_chromium_under(
    machines: &Path,
    persons: Option<&Path>,
    identifier: &str,
) -> bool {
    alo_launching::is_drawn_by_chromium(identifier)
        || is_built_on_electron(machines, persons, identifier)
}

/// Whether a manifest's `base` key holds exactly [`ELECTRON_BASE_APP`].
///
/// Read as the key and its value — `"base"`, a colon, the quoted name — with
/// any spacing, rather than as a phrase found anywhere, so that another key
/// holding the name, or a longer name beginning with it, is not Electron.
fn says_its_base_is_electron(manifest: &str) -> bool {
    let key = "\"base\"";
    let value = format!("\"{ELECTRON_BASE_APP}\"");
    manifest.match_indices(key).any(|(at, _)| {
        let rest = manifest[at + key.len()..].trim_start();
        rest.strip_prefix(':')
            .is_some_and(|rest| rest.trim_start().starts_with(&value))
    })
}

#[cfg(test)]
#[path = "built_on_electron_tests.rs"]
mod tests;
