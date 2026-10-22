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
//! **It does not turn an icon into a picture.** A desktop entry *names* one,
//! and that name is read here by [`read_with_their_icons`] on the same walk as
//! everything else. Turning it into a file is an icon-theme lookup — directory
//! precedence, sizes, fallbacks, rasterising — which is a separate decision and
//! not this crate's.
//!
//! *This said **it does not read an icon** until 2026-10-22, and that was the
//! whole sentence on the subject. It was true when written, it was read as the
//! boundary, and so nothing anywhere read the field: every application on the
//! Dock drew its first letter, which `docs/design/the-alo-dock.md` decided as
//! the fallback for an application **with no artwork** and which had quietly
//! become the answer for all of them. ADR 0101 is the owner's ruling that it
//! should not be — first letters are a missing-icon fallback, not the finished
//! Dock — and the name had to be read before anything could honour it.*
//!
//! **And it does not execute anything.** `Exec` is parsed by nobody here;
//! opening is `alo_applications::opener`'s, behind the grant that governs it.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use alo_applications::application::Application;
use alo_applications::declared::Declared;
use alo_applications::installed::Installed;

mod icons;
pub use icons::Icons;

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
        for entry in entries_under(place) {
            if let Ok(application) = Application::called(&entry.identifier, &entry.said.named) {
                // `add` answers false for an identifier already held, which is
                // how an earlier directory wins without this loop having to
                // ask first.
                let _ = installed.add(application);
            }
        }
    }
    installed
}

/// What each installed application declares it opens, read from the same
/// entries [`read`] reads.
///
/// **Both, from one walk.** `read` answers what is installed and this answers
/// what they open, and a caller wanting both — the portal answering *what opens
/// this* wants exactly both — would otherwise walk the filesystem twice and get
/// two lists that can disagree about which entries counted.
///
/// Earlier places win, which is [`read`]'s own rule and for its own reason: a
/// person's own `~/.local/share/applications` overrides what the image ships,
/// so an application they have replaced declares what their copy declares.
#[must_use]
pub fn read_with_what_they_open(places: &[PathBuf]) -> (Installed, Declared) {
    let mut installed = Installed::nothing();
    let mut declared = Declared::nothing();
    for place in places {
        for entry in entries_under(place) {
            let Ok(application) = Application::called(&entry.identifier, &entry.said.named) else {
                continue;
            };
            // **Declared before installed**, because `add` answers whether this
            // identifier was new and a second entry for one application must
            // not declare its kinds twice. The declaration is keyed by the
            // identifier either way, so the order only decides whether the
            // borrow is still ours.
            if installed.add(application.clone()) {
                declared.declares_media_types(
                    &application,
                    entry.said.opens.iter().map(String::as_str),
                );
            }
        }
    }
    (installed, declared)
}

/// What each installed application calls its icon, off the same walk again.
///
/// **A third answer from one reading, for [`read_with_what_they_open`]'s own
/// reason.** A caller wanting the list and the icon names would otherwise walk
/// twice and could get two lists disagreeing about which entries counted — and
/// here that disagreement would be invisible, because an application whose icon
/// went missing looks exactly like one that never named an icon.
///
/// **The name only, never a picture.** `Icon` holds either an absolute path or
/// a theme name, and turning the second into a file is an icon-theme lookup
/// that this crate's own header declines and `alo-icons` performs.
///
/// An application that named no icon is simply absent from [`Icons`], which is
/// ordinary: the specification does not require the key, and the Dock's answer
/// for one — its first letter — is already decided in
/// `docs/design/the-alo-dock.md`.
#[must_use]
pub fn read_with_their_icons(places: &[PathBuf]) -> (Installed, Icons) {
    let mut installed = Installed::nothing();
    let mut icons = Icons::nothing();
    for place in places {
        for entry in entries_under(place) {
            let Ok(application) = Application::called(&entry.identifier, &entry.said.named) else {
                continue;
            };
            // Named before added, exactly as the media types are and for the
            // same reason: `add` answers whether this identifier was new, and
            // `Icons::declares` keeps the first, so a second entry for one
            // application cannot overwrite the icon the first named.
            if installed.add(application)
                && let Some(named) = entry.said.icon.as_deref()
            {
                icons.declares(&entry.identifier, named);
            }
        }
    }
    (installed, icons)
}

/// One desktop entry that counted, as the walk found it.
///
/// **A struct rather than the tuple this was.** The walk now carries four
/// things, and `(String, String, Vec<String>, Option<String>)` is a shape no
/// reader keeps straight — two of its four are `String`, and a call site with
/// those two swapped would compile and name every application after its icon.
struct Entry {
    /// Its desktop file ID.
    identifier: String,
    /// What the entry said about itself.
    said: Said,
}

/// What one entry said about itself, off one reading of one file.
struct Said {
    /// The name a person is shown.
    named: String,
    /// The media types it declares it opens, possibly none.
    opens: Vec<String>,
    /// What it calls its icon, where it named one.
    icon: Option<String>,
}

/// Every readable, listable entry under one `applications/` directory.
///
/// Walked into subdirectories, because a desktop file ID may contain them —
/// `kde4/konsole.desktop` is the identifier `kde4-konsole`.
fn entries_under(place: &Path) -> Vec<Entry> {
    let mut found = Vec::new();
    walk(place, place, &mut found);
    found
}

/// The walk itself, carrying the directory it began in so an identifier can be
/// built from the path relative to it.
fn walk(root: &Path, at: &Path, into: &mut Vec<Entry>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, into);
        } else if path.extension() == Some(OsStr::new("desktop"))
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Some(said) = what_the_entry_says(&text)
            && let Some(identifier) = the_identifier_of(root, &path)
        {
            into.push(Entry { identifier, said });
        }
    }
}

/// The desktop file ID: the path under `applications/`, `/` to `-`, no suffix.
fn the_identifier_of(root: &Path, path: &Path) -> Option<String> {
    let under = path.strip_prefix(root).ok()?;
    let spelled = under.to_str()?.strip_suffix(".desktop")?;
    Some(spelled.replace('/', "-"))
}

/// What an entry says about itself: the name it is shown under, the media types
/// it declares it opens, and what it calls its icon.
///
/// Reads only the `[Desktop Entry]` group: a desktop file may carry further
/// groups — one per action — and each has its own `Name` and its own `Icon`.
/// Taking the first of either anywhere in the file would name an application
/// after one of its right-click actions, and draw that action's picture.
///
/// **Localised keys are left for later and left alone.** `Name[de]` is not
/// `Name`, so this takes the unlocalised one; showing a person their own
/// language means asking `alo-strings` which language that is, and this crate
/// has no business holding that answer.
///
/// **One pass, three answers**, because they come off the same lines of the
/// same file and reading it three times would be three chances to disagree
/// about which entries counted. The refusals are the name's and are unchanged:
/// a `Type` that is not `Application`, a `NoDisplay` or a `Hidden`, or no
/// `Name` at all means this entry is not an application a person has.
///
/// **A missing `MimeType` is an empty list and not a refusal.** Most
/// applications declare none — a terminal, a settings panel — and an entry
/// without the line is an application that opens nothing, which is a true
/// answer about it rather than a reason to leave it out of what is installed.
///
/// **A missing `Icon` is [`None`] and not a refusal either**, for the same
/// reason and with a decided consequence: the Dock draws that application's
/// first letter.
fn what_the_entry_says(text: &str) -> Option<Said> {
    let mut inside = false;
    let mut named = None;
    let mut opens: Vec<String> = Vec::new();
    let mut icon: Option<String> = None;
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
            // **Semicolon separated, and a trailing one is ordinary.** The
            // desktop entry specification writes a list that way and most
            // entries end it with a separator, so an empty piece is the shape
            // of the format rather than an application declaring nothing.
            ("MimeType", listed) => {
                opens = listed
                    .split(';')
                    .map(str::trim)
                    .filter(|it| !it.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
            // **An empty Icon line is no icon, not an icon called nothing.** An
            // entry written by a packaging script sometimes carries the key
            // with nothing after it, and an empty name would send the lookup
            // hunting for a file called nothing and report *not found* for a
            // reason that is really *never named*.
            ("Icon", called) if !called.is_empty() => icon = Some(called.to_owned()),
            _ => {}
        }
    }
    named.map(|named| Said { named, opens, icon })
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
