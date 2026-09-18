//! Keep gesture preferences in gestures.toml at the session-supplied path.
//! Writes are atomic, read back, and refuse to overwrite a malformed file.

use std::path::Path;

use alo_kept::{Kept, Unread, Unwritten};

use crate::gesture_refusals::{NotRead, NotWritten};
use crate::gesture_settings::Preferences;

impl Kept for Preferences {
    const FILE: &'static str = "gestures.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &[
        "scroll",
        "pinch",
        "three-finger-swipe",
        "four-finger-swipe",
        "scrolling",
    ];
    type NotRead = NotRead;
    type NotWritten = NotWritten;

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(at: &Path, why: Unread) -> Self::NotRead {
        NotRead {
            at: at.to_owned(),
            why,
        }
    }

    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten {
        NotWritten {
            at: at.to_owned(),
            why,
        }
    }
}

/// Read the person's settings; a missing file means shipped preferences.
///
/// # Errors
/// A malformed or unreadable file is refused whole.
pub fn read(at: &Path) -> Result<Preferences, NotRead> {
    alo_kept::read(at)
}

/// Atomically keep and read back only the differences from shipped preferences.
///
/// # Errors
/// A bad existing file, wrong path or failed write leaves the old file intact.
pub fn keep(at: &Path, preferences: &Preferences) -> Result<(), NotWritten> {
    alo_kept::keep(at, preferences)
}
