//! **What applications this machine has**, read from its desktop entries.
//!
//! `docs/features.md` promises *Launcher and window management: open, focus,
//! close, tile* at `[v0.01]`, and the alo Dock *shows what you can open*.
//! Neither is answerable without a list of what is here, and until this crate
//! existed **nothing built one**: `alo_applications::Installed` had
//! `holding` and `nothing`, and every caller of either was a test.
//!
//! # Why this is its own crate and not a module in `alo-applications`
//!
//! That crate says of itself, deliberately: *whatever builds this list reads it
//! off the machine … this crate holds no opinion about where the entries come
//! from — exactly as `alo_capability::Grants` holds none about where grants are
//! stored.* A reader **is** an opinion about where they come from, so putting
//! one inside it would bend a boundary somebody chose rather than honour it.
//!
//! Grants are read by `alo-remembering`, one crate along. This is the same
//! shape: `alo-applications` holds what an entry has to be and how one is
//! matched; this holds where they live on a Linux machine.
//!
//! # What a desktop entry is, and which ones a person should see
//!
//! The freedesktop Desktop Entry specification: an INI-like file ending
//! `.desktop`, in `applications/` under each XDG data directory, whose
//! `[Desktop Entry]` section carries at least a `Type` and a `Name`.
//!
//! **Three kinds are left out, and each for a reason a person would give:**
//!
//! - `Type` other than `Application` — a link or a directory is not something
//!   to open;
//! - `NoDisplay=true` — the entry's own author saying *do not list me*, which
//!   is how a helper that exists only to handle a file type stays out of a
//!   launcher;
//! - `Hidden=true` — the specification's *deleted*, which a user-level
//!   directory uses to mask a system entry.
//!
//! **Earlier directories win.** XDG's order is most-specific first, so a
//! person's own `~/.local/share/applications` overrides `/usr/share`. That is
//! what lets somebody rename or hide an application the system installed, and
//! reversing it would silently ignore their copy.
//!
//! # No clever matching, and the reason is next door
//!
//! An identifier is the entry's **desktop file ID** — its path under
//! `applications/`, with `/` becoming `-`, minus the suffix. `alo-applications`
//! matches an identifier **exactly, with no case folding**, because *matching
//! loosely matches more than the person picked*, and the same list has to be
//! matched the same way or *is it granted* and *is it here* are answered about
//! different applications. So nothing here lowercases, trims or guesses.
//!
//! # What this crate does not do
//!
//! **It does not read an icon.** A desktop entry names one, and turning a name
//! into an image is an icon-theme lookup — directory precedence, sizes,
//! fallbacks, rasterising — which is a separate decision and not this crate's.
//! What is here is the list.
//!
//! **And it does not execute anything.** `Exec` is parsed by nobody here;
//! opening is `alo_applications::opener`'s, behind the grant that governs it.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use alo_applications::application::Application;
use alo_applications::installed::Installed;

/// The directories desktop entries live in, most specific first.
///
/// `$XDG_DATA_HOME/applications`, then each of `$XDG_DATA_DIRS/applications`.
/// The specification's defaults are used where a variable is unset or empty:
/// `$HOME/.local/share` and `/usr/local/share:/usr/share`.
///
/// **An empty variable means unset**, which the specification says in as many
/// words and which a shell produces by exporting nothing. Treating `""` as a
/// directory would read entries out of the process's working directory.
#[must_use]
pub fn where_entries_live(
    home: Option<&Path>,
    data_home: Option<&OsStr>,
    data_dirs: Option<&OsStr>,
) -> Vec<PathBuf> {
    let mut places = Vec::new();
    match data_home.filter(|set| !set.is_empty()) {
        Some(set) => places.push(Path::new(set).join("applications")),
        None => {
            if let Some(home) = home {
                places.push(home.join(".local").join("share").join("applications"));
            }
        }
    }
    let dirs = data_dirs.filter(|set| !set.is_empty()).map_or_else(
        || "/usr/local/share:/usr/share".to_owned(),
        |set| set.to_string_lossy().into_owned(),
    );
    for dir in dirs.split(':').filter(|part| !part.is_empty()) {
        places.push(Path::new(dir).join("applications"));
    }
    places
}

/// Every application these directories declare, earlier directories winning.
///
/// A directory that is not there is not an error: a machine with no
/// `~/.local/share/applications` has installed nothing of its own, which is
/// ordinary. A file that does not parse is skipped rather than failing the
/// walk, for the same reason `Reported::of` drops one display it cannot
/// describe instead of losing the others.
#[must_use]
pub fn read(places: &[PathBuf]) -> Installed {
    let mut installed = Installed::nothing();
    for place in places {
        for (identifier, named) in entries_under(place) {
            if let Ok(application) = Application::called(&identifier, &named) {
                // `add` answers false for an identifier already held, which is
                // how an earlier directory wins without this loop having to
                // ask first.
                let _ = installed.add(application);
            }
        }
    }
    installed
}

/// Every readable, listable entry under one `applications/` directory.
///
/// Walked into subdirectories, because a desktop file ID may contain them —
/// `kde4/konsole.desktop` is the identifier `kde4-konsole`.
fn entries_under(place: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    walk(place, place, &mut found);
    found
}

/// The walk itself, carrying the directory it began in so an identifier can be
/// built from the path relative to it.
fn walk(root: &Path, at: &Path, into: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, into);
        } else if path.extension() == Some(OsStr::new("desktop"))
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Some(named) = the_name_in(&text)
            && let Some(identifier) = the_identifier_of(root, &path)
        {
            into.push((identifier, named));
        }
    }
}

/// The desktop file ID: the path under `applications/`, `/` to `-`, no suffix.
fn the_identifier_of(root: &Path, path: &Path) -> Option<String> {
    let under = path.strip_prefix(root).ok()?;
    let spelled = under.to_str()?.strip_suffix(".desktop")?;
    Some(spelled.replace('/', "-"))
}

/// The `Name` of an entry a person should see, or [`None`].
///
/// Reads only the `[Desktop Entry]` group: a desktop file may carry further
/// groups — one per action — and each has its own `Name`. Taking the first
/// `Name` anywhere in the file would name an application after one of its
/// right-click actions.
///
/// **Localised keys are left for later and left alone.** `Name[de]` is not
/// `Name`, so this takes the unlocalised one; showing a person their own
/// language means asking `alo-strings` which language that is, and this crate
/// has no business holding that answer.
fn the_name_in(text: &str) -> Option<String> {
    let mut inside = false;
    let mut named = None;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            if inside {
                // The next group begins, so the entry's own keys are over.
                break;
            }
            inside = line == "[Desktop Entry]";
            continue;
        }
        if !inside {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match (key.trim(), value.trim()) {
            ("Type", kind) if kind != "Application" => return None,
            ("NoDisplay" | "Hidden", "true") => return None,
            ("Name", shown) if !shown.is_empty() => named = Some(shown.to_owned()),
            _ => {}
        }
    }
    named
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
