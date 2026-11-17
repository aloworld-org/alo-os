//! The switches on the What alo can use page that let alo use **pictures and
//! keys** in one application each — for the applications that tell alo nothing
//! about their windows, such as games
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q11; the owner's *"Yes, build it"* of 2026-11-17).
//!
//! # Kept beside the level, read at every act
//!
//! The answer is `pictures-and-keys-in` in `/var/lib/alo/how-code-runs.toml`
//! (`docs/contracts/how-code-runs-file.md`). `alo_remembering` switches one
//! application at a time and keeps every other answer in the file as it was.
//! Whatever takes a picture of a window reads the same file at that moment, so
//! a switch turned off here is what the next picture meets.
//!
//! **The switch moves only once the file says so**, for the level's reason: a
//! switch that moved first would tell somebody alo cannot look at a window it
//! can.
//!
//! # Off, and drawn off over a file that will not read
//!
//! There is no *not set*: ADR 0143 says off until the person switches it on.
//! A file that would not read is drawn off everywhere, because that is what a
//! reader of it does — and switching over such a file is refused rather than
//! written, because writing it would throw away a level the person may have
//! chosen.

use std::path::{Path, PathBuf};

use alo_capability::PicturesAndKeys;

use crate::settings_kept::SettingsKept;

/// The switches, as Settings found them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PicturesAndKeysHere {
    /// What the file says: none where it says nothing, and none where it
    /// would not read.
    is: PicturesAndKeys,
    /// Where it is kept.
    at: PathBuf,
}

impl PicturesAndKeysHere {
    /// The switches at sign-in.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        Self {
            is: PicturesAndKeys::as_read(alo_remembering::pictures_and_keys_remembered(at)),
            at: at.to_owned(),
        }
    }

    /// Whether the switch for this application is on.
    pub(crate) fn is_on_in(&self, application: &str) -> bool {
        self.is.allows(application)
    }

    /// One press on one application's switch: written first and drawn second.
    pub(crate) fn pressed(&mut self, application: &str) -> SettingsKept {
        let on = !self.is.allows(application);
        match alo_remembering::pictures_and_keys_switched(&self.at, application, on) {
            Ok(now) => {
                self.is = now;
                SettingsKept::Written
            }
            Err(_) => SettingsKept::NotWritten,
        }
    }
}

#[cfg(test)]
#[path = "pictures_and_keys_in_settings_tests.rs"]
mod tests;
