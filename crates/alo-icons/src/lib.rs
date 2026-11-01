//! **Where an application's icon is on this machine**, found by the name its
//! desktop entry gave.
//!
//! `alo-installed` reads that name and says in as many words that turning it
//! into a file is *an icon-theme lookup — directory precedence, sizes,
//! fallbacks* — and not its job. This is that lookup, and nothing else: it
//! answers with a **path**, never with pixels, because decoding is a third job
//! again and belongs where the decoder already is.
//!
//! # Why this exists at all
//!
//! [ADR 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md),
//! the owner's ruling:
//!
//! > Real applications retain their actual application icons; the illustrative
//! > icons in Figma are not a replacement for every installed application's
//! > identity. **First letters are a missing-icon fallback, not the finished
//! > Dock.**
//!
//! Until this crate existed, the fallback was the answer for every application
//! on every machine, because nothing turned a name into a picture.
//!
//! # What an `Icon` name can be, and the two shapes are not alike
//!
//! The Desktop Entry specification allows either, and a lookup that handled
//! only the common one would silently lose the applications that use the other:
//!
//! - **a path** — `/usr/share/pixmaps/thing.png`. Used as given, and this
//!   crate's only job is to say whether it is there;
//! - **a name** — `org.gnome.Nautilus`, `firefox`. Searched for under the icon
//!   directories, by the Icon Theme specification's layout.
//!
//! A name is told from a path by whether it is absolute, which is the
//! specification's own rule and not a guess about what names look like.
//!
//! # A picture of pixels or a scalable one, and which is which
//!
//! An icon theme holds `.png` and `.svg`, and this crate offers both, saying
//! which it found: [`WhatWasFound::APicture`] or
//! [`WhatWasFound::AScalablePicture`]. Decoding either is the caller's job.
//!
//! **Until 2026-11-01 this offered only the PNG**, because nothing in this
//! repository could rasterise an application's SVG. A booted machine showed
//! the cost: Dolphin's Flatpak exports only
//! `hicolor/scalable/apps/org.kde.dolphin.svg`, so the Dock drew it as a
//! letter. [ADR 0144](../../../docs/decisions/0144-an-applications-scalable-icon-is-drawn-by-resvg.md)
//! brought the rasteriser in.
//!
//! # Sizes: the nearest at least as large, then a scalable one, then the
//! largest below
//!
//! The Dock draws artwork into a box of a known size. Asked for 48 on a theme
//! holding 32, 64 and 256, this answers **64**: scaling a picture down loses
//! nothing a person sees, and scaling one up is visibly soft. Where no picture
//! is large enough, **a scalable one is taken before a smaller picture**,
//! because it is drawn at the size asked for and is never soft. Only where
//! there is neither does it take the largest picture that is smaller.
//!
//! # What this crate does not do
//!
//! **It does not read `index.theme`, and does not follow `Inherits`.** A theme
//! may name a parent to fall back to, and the specification's full algorithm
//! walks that chain. This looks in each directory it is given, in order, and
//! the caller puts `hicolor` last — which is where the specification says every
//! application's own icon is installed, and is what makes the common case right
//! without parsing a second file format. A themed machine that moves an icon
//! out of `hicolor` is a case this does not serve yet, and it is written here
//! rather than discovered later.
//!
//! **It does not cache.** A caller that asks per frame would walk the
//! filesystem per frame; the Dock asks once when the desktop stands up, exactly
//! as it works its pins out once.

use std::path::{Path, PathBuf};

/// The directories icons live in, most specific first.
///
/// `$XDG_DATA_HOME/icons`, then each `$XDG_DATA_DIRS/icons`, then
/// `/usr/share/pixmaps`. The specification's defaults are used where a variable
/// is unset or empty — `$HOME/.local/share` and `/usr/local/share:/usr/share` —
/// which is `alo_installed::where_entries_live`'s rule and deliberately the
/// same one: the two lists are read off the same machine and a person who has
/// moved their data directory has moved both.
///
/// **`pixmaps` is last and is not a theme.** It is a flat directory from before
/// icon themes existed, holding `thing.png` with no size in the path, and
/// plenty of applications still install there. Searching it first would hand
/// back a 32-pixel legacy icon when a themed 256 was available.
#[must_use]
pub fn where_icons_live(
    home: Option<&Path>,
    data_home: Option<&std::ffi::OsStr>,
    data_dirs: Option<&std::ffi::OsStr>,
) -> Vec<PathBuf> {
    let mut places = Vec::new();
    match data_home.filter(|set| !set.is_empty()) {
        Some(set) => places.push(Path::new(set).join("icons")),
        None => {
            if let Some(home) = home {
                places.push(home.join(".local").join("share").join("icons"));
            }
        }
    }
    let dirs = data_dirs.filter(|set| !set.is_empty()).map_or_else(
        || "/usr/local/share:/usr/share".to_owned(),
        |set| set.to_string_lossy().into_owned(),
    );
    for dir in dirs.split(':').filter(|part| !part.is_empty()) {
        places.push(Path::new(dir).join("icons"));
    }
    // Not under `icons/`, and not a theme. See this function's own note.
    places.push(PathBuf::from("/usr/share/pixmaps"));
    places
}

/// What came of looking for one application's icon.
///
/// **Three answers**, because a caller decodes a picture of pixels and a
/// scalable one differently, and reports a machine with neither apart from
/// both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatWasFound {
    /// A PNG, at this path. The nearest size at least as large as was asked
    /// for, or the largest there is where nothing reaches it and there is no
    /// scalable copy.
    APicture(PathBuf),
    /// An SVG, at this path: no PNG reached the size asked for, and this is
    /// drawn at that size.
    AScalablePicture(PathBuf),
    /// Nothing anywhere is called this.
    Nothing,
}

impl WhatWasFound {
    /// The path, where one was found.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::APicture(at) | Self::AScalablePicture(at) => Some(at),
            Self::Nothing => None,
        }
    }
}

/// Find the icon an entry called `named`, preferring one at least `wanted`
/// across.
///
/// `places` is [`where_icons_live`]'s answer, in its order. An absolute name is
/// a path and is answered about directly; anything else is searched for.
#[must_use]
pub fn the_icon_called(named: &str, places: &[PathBuf], wanted: u32) -> WhatWasFound {
    if named.is_empty() {
        return WhatWasFound::Nothing;
    }

    // **A path is used, not searched for.** The specification says an `Icon`
    // may be an absolute path, and a machine that installed artwork outside a
    // theme is a machine whose entry says exactly where it put it.
    let given = Path::new(named);
    if given.is_absolute() {
        return match (given.is_file(), is_scalable(given)) {
            (true, false) => WhatWasFound::APicture(given.to_path_buf()),
            (true, true) => WhatWasFound::AScalablePicture(given.to_path_buf()),
            (false, _) => WhatWasFound::Nothing,
        };
    }

    let mut best: Option<(u32, PathBuf)> = None;
    // The first scalable copy, in the order of `places`: an earlier directory
    // wins, as it does for pictures.
    let mut scalable: Option<PathBuf> = None;
    for place in places {
        for candidate in candidates_under(place, named) {
            if is_scalable(&candidate.at) {
                scalable.get_or_insert(candidate.at);
                continue;
            }
            best = better_of(best, (candidate.across, candidate.at), wanted);
        }
    }
    match (best, scalable) {
        // A picture large enough: scaled down, nothing is lost.
        (Some((across, at)), _) if across >= wanted => WhatWasFound::APicture(at),
        // Otherwise a scalable one, drawn at exactly the size asked for.
        (_, Some(at)) => WhatWasFound::AScalablePicture(at),
        (Some((_, at)), None) => WhatWasFound::APicture(at),
        (None, None) => WhatWasFound::Nothing,
    }
}

/// One file that might be this icon, and how wide the path says it is.
struct Candidate {
    /// Where it is.
    at: PathBuf,
    /// Its size, from the directory that holds it. `0` where the path says
    /// nothing — a flat `pixmaps` directory — which sorts it below every sized
    /// candidate without needing a second rule.
    across: u32,
}

/// Every file under one icon directory that is called this.
///
/// Two layouts, because both exist on a real machine:
///
/// - **themed**: `<place>/<theme>/<size>/<context>/<named>.png`, where `size`
///   is `48x48` or `scalable` and `context` is `apps` or another word. Walked
///   rather than guessed at, because a theme may use `apps`, `applications` or
///   a context this code has never heard of, and listing what is there cannot
///   be wrong about it;
/// - **flat**: `<place>/<named>.png`, which is what `pixmaps` is.
fn candidates_under(place: &Path, named: &str) -> Vec<Candidate> {
    let mut found = Vec::new();
    // Flat, which is pixmaps and also a theme directory somebody flattened.
    for suffix in ["png", "svg"] {
        let flat = place.join(format!("{named}.{suffix}"));
        if flat.is_file() {
            found.push(Candidate {
                at: flat,
                across: 0,
            });
        }
    }
    let Ok(themes) = std::fs::read_dir(place) else {
        return found;
    };
    for theme in themes.flatten() {
        let theme = theme.path();
        if !theme.is_dir() {
            continue;
        }
        let Ok(sizes) = std::fs::read_dir(&theme) else {
            continue;
        };
        for size in sizes.flatten() {
            let size_at = size.path();
            if !size_at.is_dir() {
                continue;
            }
            let across = how_wide(&size_at);
            let Ok(contexts) = std::fs::read_dir(&size_at) else {
                continue;
            };
            for context in contexts.flatten() {
                let context = context.path();
                if !context.is_dir() {
                    continue;
                }
                for suffix in ["png", "svg"] {
                    let at = context.join(format!("{named}.{suffix}"));
                    if at.is_file() {
                        found.push(Candidate { at, across });
                    }
                }
            }
        }
    }
    found
}

/// How wide a size directory says its contents are.
///
/// `48x48` is 48, and so is `48`. `scalable` and anything unreadable are `0`,
/// which is below every real size — and an SVG is set aside before its size is
/// ever compared, so nothing turns on `scalable` having a number.
fn how_wide(at: &Path) -> u32 {
    let Some(named) = at.file_name().and_then(std::ffi::OsStr::to_str) else {
        return 0;
    };
    let first = named.split(['x', 'X']).next().unwrap_or(named);
    first.parse().unwrap_or(0)
}

/// Whether this file is scalable rather than a picture of pixels.
fn is_scalable(at: &Path) -> bool {
    at.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case("svg"))
}

/// The better of two candidates for a box `wanted` across.
///
/// **At least as large wins, and among those the smallest.** Scaling a picture
/// down loses nothing a person can see at Dock size; scaling one up is soft,
/// and an icon is the first thing a person looks at. Where nothing reaches
/// `wanted`, the largest below it is the least bad.
fn better_of(
    best: Option<(u32, PathBuf)>,
    candidate: (u32, PathBuf),
    wanted: u32,
) -> Option<(u32, PathBuf)> {
    let Some(held) = best else {
        return Some(candidate);
    };
    let (held_across, _) = held;
    let (across, _) = candidate;
    let held_reaches = held_across >= wanted;
    let reaches = across >= wanted;
    let take = match (held_reaches, reaches) {
        // Both big enough: the smaller, which is the least scaling down.
        (true, true) => across < held_across,
        // Only one is: that one, whichever it is.
        (false, true) => true,
        (true, false) => false,
        // Neither: the larger, which is the least scaling up.
        (false, false) => across > held_across,
    };
    if take { Some(candidate) } else { Some(held) }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
