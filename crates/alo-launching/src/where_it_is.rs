//! Where an entry is, and where the program its `Exec` line names is.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// The file of the entry with this desktop file ID, in the first place that
/// has one.
///
/// **Earlier places win**, `alo-installed`'s rule and the specification's: a
/// person's own `~/.local/share/applications` copy is the one started, which is
/// also how a copy saying `Hidden=true` masks the system's.
///
/// A desktop file ID turns each `/` under `applications/` into `-`, so
/// `kde4-konsole` may be `kde4/konsole.desktop`. Each way of reading the dashes
/// as folders, left to right, is tried after the plain name.
///
/// **An identifier that would climb out of the place is never looked for** — a
/// `.` or `..` part, or a `/` of its own — whatever is on the disk.
#[must_use]
pub fn where_the_entry_is(identifier: &str, places: &[PathBuf]) -> Option<PathBuf> {
    let candidates = the_ways_to_read(identifier);
    for place in places {
        for candidate in &candidates {
            let path = place.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

/// The relative files an identifier may be, plain name first.
fn the_ways_to_read(identifier: &str) -> Vec<PathBuf> {
    if identifier.is_empty() || identifier.contains('/') || identifier.contains('\\') {
        return Vec::new();
    }
    let mut ways = vec![format!("{identifier}.desktop")];
    let dashes: Vec<usize> = identifier.match_indices('-').map(|(at, _)| at).collect();
    for count in 1..=dashes.len() {
        let mut spelled = String::with_capacity(identifier.len());
        for (at, letter) in identifier.char_indices() {
            let a_folder = letter == '-' && dashes.iter().take(count).any(|dash| *dash == at);
            spelled.push(if a_folder { '/' } else { letter });
        }
        ways.push(format!("{spelled}.desktop"));
    }
    ways.into_iter()
        .map(PathBuf::from)
        .filter(|way| {
            way.components()
                .all(|part| matches!(part, Component::Normal(_)))
        })
        .collect()
}

/// The program an `Exec` line's first word names, as an absolute path.
///
/// An absolute name is used as it is; a bare name is looked for in each
/// absolute folder of `search` (the desktop's own `PATH`, or the usual
/// `/usr/local/bin:/usr/bin:/bin` when it has none) — the specification's *the
/// executable program … is looked up in the $PATH*. A relative name with a `/`
/// in it is refused: it would be read against whatever folder the desktop
/// happens to be in.
///
/// `is_runnable` answers whether a path is a program this person may run; the
/// desktop asks the filesystem, a test answers from a list.
#[must_use]
pub fn the_program(
    first: &str,
    search: Option<&OsStr>,
    is_runnable: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    if first.starts_with('/') {
        let path = PathBuf::from(first);
        return is_runnable(&path).then_some(path);
    }
    if first.is_empty() || first.contains('/') {
        return None;
    }
    let search = search
        .and_then(OsStr::to_str)
        .filter(|it| !it.is_empty())
        .unwrap_or("/usr/local/bin:/usr/bin:/bin");
    search
        .split(':')
        .filter(|folder| folder.starts_with('/'))
        .map(|folder| Path::new(folder).join(first))
        .find(|path| is_runnable(path))
}

/// Linux paths, so Linux only: on Windows `join` writes a `\` and the search
/// path's answers would be spelled differently from what they are about.
#[cfg(all(test, unix))]
#[path = "where_it_is_tests.rs"]
mod tests;
