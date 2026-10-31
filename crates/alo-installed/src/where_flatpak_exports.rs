//! **Where this machine's flatpak installations put the applications they
//! install**, so that reading the desktop entries finds them whatever the
//! process that reads them was started with.
//!
//! # Why this cannot be left to the environment
//!
//! Flatpak installs an application under its own tree and *exports* its
//! desktop entry and its icons to `<installation>/exports/share`. Nothing reads
//! that directory unless it is on `XDG_DATA_DIRS`, and flatpak puts it there
//! from two places only: a login shell's `/etc/profile.d/flatpak.sh`, and a
//! systemd **user** environment generator. A process started any other way does
//! not have it.
//!
//! **`alo-desktop` is started another way.** It is a system unit run as the
//! person (`alo-desktop@.service`, ADR 0141): its unit sets no
//! `XDG_DATA_DIRS` and its PAM stack has no `pam_env`, so the variable is
//! unset, the specification's default is `/usr/local/share:/usr/share`, and
//! every flatpak on the machine is invisible to it. A terminal on the same
//! desktop printed `/var/lib/flatpak/exports/share:/usr/local/share/:/usr/share/`
//! on a booted machine — the trailing separators are `flatpak.sh`'s own
//! spelling, which is the login shell adding it and not the desktop having it.
//! **That the desktop's own value is unset is read from its unit, not measured
//! in its process**; the next walk is where it is measured.
//!
//! So the directories are added here, by the rule flatpak's own scripts follow,
//! rather than hoped for from whatever started the reader.
//!
//! # The rule
//!
//! The person's own installation first, then the machine's, then whatever
//! `XDG_DATA_DIRS` already said (or the specification's default when it said
//! nothing). **One that is already there is not added twice**, so a process
//! that was given the directories reads each one once and in the order it was
//! given — compared without a trailing `/`, because `flatpak.sh` writes
//! `/usr/share/` where the specification writes `/usr/share`.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Where the machine's own flatpak installation lives.
///
/// `alo-shipping` installs a fresh machine's applications here — the
/// system-wide installation, which `alo_software` calls nobody's.
pub const THE_MACHINES_INSTALLATION: &str = "/var/lib/flatpak";

/// `XDG_DATA_DIRS` as it would read with both flatpak installations on it.
///
/// `machines` is [`THE_MACHINES_INSTALLATION`] on a running machine and a
/// directory of a test's own elsewhere. `home`, `data_home` and `data_dirs`
/// are the process's own `HOME`, `XDG_DATA_HOME` and `XDG_DATA_DIRS`, read by
/// the caller; an empty one means unset, which is the specification's rule
/// and `crate::where_entries_live`'s.
///
/// The answer is meant to be handed to `crate::where_entries_live` and
/// `alo_icons::where_icons_live` as their `data_dirs`, so the entries and the
/// pictures are looked for in the same places.
#[must_use]
pub fn with_what_flatpak_exports(
    machines: &Path,
    home: Option<&Path>,
    data_home: Option<&OsStr>,
    data_dirs: Option<&OsStr>,
) -> OsString {
    let persons_data = match data_home.filter(|set| !set.is_empty()) {
        Some(set) => Some(PathBuf::from(set)),
        None => home.map(|home| home.join(".local").join("share")),
    };
    let given = data_dirs.filter(|set| !set.is_empty()).map_or_else(
        || "/usr/local/share:/usr/share".to_owned(),
        |set| set.to_string_lossy().into_owned(),
    );
    let mut places: Vec<String> = Vec::new();
    let exported = [
        persons_data.map(|data| data.join("flatpak").join("exports").join("share")),
        Some(machines.join("exports").join("share")),
    ];
    for place in exported.into_iter().flatten() {
        let place = place.to_string_lossy().into_owned();
        let already = given
            .split(':')
            .any(|part| the_same_place(part, &place));
        if !already {
            places.push(place);
        }
    }
    places.extend(
        given
            .split(':')
            .filter(|part| !part.is_empty())
            .map(str::to_owned),
    );
    OsString::from(places.join(":"))
}

/// Whether two spellings of a directory name the same one, ignoring a
/// trailing separator.
fn the_same_place(one: &str, other: &str) -> bool {
    let one = one.trim_end_matches('/');
    let other = other.trim_end_matches('/');
    !one.is_empty() && one == other
}

#[cfg(test)]
#[path = "where_flatpak_exports_tests.rs"]
mod tests;
