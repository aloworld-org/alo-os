//! **alo's cursor theme**, as the files a toolkit reads: one XCursor file per
//! name the protocol defines, the older names that point at them, and the
//! theme's own `index.theme`.
//!
//! Built from the same vector source the compositor draws from, so an
//! application drawing its own pointer from this theme shows exactly what alo
//! shows over its own windows. [`the_theme`] makes the files and does no I/O;
//! [`write_into`] puts them on disk, which only the build step that installs the
//! theme does.
//!
//! # The older names
//!
//! Toolkits written before the CSS names ask for `left_ptr`, `xterm`, `hand2`
//! and so on. Each is an alias of the CSS name `cursor-icon` lists it under
//! ([`cursor_icon::CursorIcon::alt_names`]). Where two CSS names list the same old name — both
//! `col-resize` and `ew-resize` claim `h_double_arrow` — the first in the
//! protocol's order keeps it; and an old name that is also a CSS name (`copy`,
//! `move`) is the CSS cursor, never an alias.

use std::collections::BTreeMap;
use std::path::Path;

use crate::artwork::EVERY_SHAPE;
use crate::xcursor_file::{NotWritten, xcursor_file};
use crate::{NotDrawn, Silhouette, draw};

/// The theme's directory name, and what `XCURSOR_THEME` is set to.
pub const THE_NAME: &str = "alo";

/// The nominal sizes every cursor file carries, in pixels.
///
/// 24 is the logical size ([`crate::measures::LOGICAL_SIZE`]); the rest are
/// what that becomes at 125, 150, 200, 250, 300 and 400 per cent, so a toolkit
/// asking for `XCURSOR_SIZE` times its scale finds an exact size rather than a
/// stretched one.
pub const SIZES: [u32; 7] = [24, 30, 36, 48, 60, 72, 96];

/// Everything in the theme, by its path inside the theme's directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    /// `index.theme`'s text.
    pub index: String,
    /// Each cursor file: its name under `cursors/`, and its bytes.
    pub cursors: BTreeMap<String, Vec<u8>>,
    /// Each older name under `cursors/`, and the cursor file it points at.
    pub aliases: BTreeMap<String, String>,
}

/// Why the theme was not made or written.
#[derive(Debug)]
pub enum NotMade {
    /// A cursor did not draw.
    NotDrawn(NotDrawn),
    /// A cursor file did not write.
    NotWritten(NotWritten),
    /// The disk refused.
    Io(std::io::Error),
}

impl core::fmt::Display for NotMade {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotDrawn(why) => write!(f, "{why}"),
            Self::NotWritten(why) => write!(f, "{why}"),
            Self::Io(why) => write!(f, "the theme could not be written: {why}"),
        }
    }
}

impl std::error::Error for NotMade {}

/// The whole theme, drawn with `silhouette` as the arrow.
///
/// # Errors
/// [`NotMade`] when a cursor does not draw or does not write, which no shipped
/// shape does and a test makes the whole theme to show.
pub fn the_theme(silhouette: Silhouette) -> Result<Theme, NotMade> {
    let mut cursors = BTreeMap::new();
    for icon in EVERY_SHAPE {
        let pictures = SIZES
            .iter()
            .map(|&size| draw(icon, silhouette, size))
            .collect::<Result<Vec<_>, _>>()
            .map_err(NotMade::NotDrawn)?;
        let file = xcursor_file(&pictures).map_err(NotMade::NotWritten)?;
        cursors.insert(icon.name().to_owned(), file);
    }
    let mut aliases = BTreeMap::new();
    for icon in EVERY_SHAPE {
        for &old in icon.alt_names() {
            if !cursors.contains_key(old) && !aliases.contains_key(old) {
                aliases.insert(old.to_owned(), icon.name().to_owned());
            }
        }
    }
    Ok(Theme {
        index: format!(
            "[Icon Theme]\nName={THE_NAME}\nComment=alo OS's pointer, drawn from one vector source\n"
        ),
        cursors,
        aliases,
    })
}

/// Write `theme` into `directory`, as `directory/index.theme` and
/// `directory/cursors/…`, making both.
///
/// An alias is a symbolic link to its cursor's file, as every cursor theme
/// spells one; on a system with no symbolic links it is a copy.
///
/// # Errors
/// [`NotMade::Io`] for whatever the disk refused.
pub fn write_into(theme: &Theme, directory: &Path) -> Result<(), NotMade> {
    let cursors = directory.join("cursors");
    std::fs::create_dir_all(&cursors).map_err(NotMade::Io)?;
    std::fs::write(directory.join("index.theme"), &theme.index).map_err(NotMade::Io)?;
    for (name, bytes) in &theme.cursors {
        std::fs::write(cursors.join(name), bytes).map_err(NotMade::Io)?;
    }
    for (alias, target) in &theme.aliases {
        let at = cursors.join(alias);
        match std::fs::remove_file(&at) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(NotMade::Io(why)),
        }
        an_alias(target, &at, &cursors).map_err(NotMade::Io)?;
    }
    Ok(())
}

/// `at` made to point at `target`, a file beside it in `cursors`.
#[cfg(unix)]
fn an_alias(target: &str, at: &Path, _cursors: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, at)
}

/// `at` made a copy of `target`, a file beside it in `cursors`.
#[cfg(not(unix))]
fn an_alias(target: &str, at: &Path, cursors: &Path) -> std::io::Result<()> {
    std::fs::copy(cursors.join(target), at).map(|_| ())
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
