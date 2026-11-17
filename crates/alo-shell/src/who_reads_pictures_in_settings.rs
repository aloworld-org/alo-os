//! The choice on the What alo can use page of **what reads the pictures** alo
//! takes of a window, beneath the switches for pictures and keys
//! ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §4; the owner's answer of 2026-11-17).
//!
//! # Kept beside the switches, read at every picture
//!
//! The answer is `pictures-read-by` in `/var/lib/alo/how-code-runs.toml`
//! (`docs/contracts/how-code-runs-file.md`). Whatever takes a picture of a
//! window reads the same file at that moment, so a choice made here is what
//! the next picture meets.
//!
//! **The tick moves only once the file says so**, for the level's reason: a
//! tick that moved first could tell somebody nothing leaves this computer when
//! a picture would.
//!
//! # No one, and drawn as no one over a file that will not read
//!
//! A machine arrives with no one reading pictures, and that is the true thing
//! to tick until the person chooses. A file that would not read is drawn as no
//! one too, because that is what a reader of it does — and choosing over such a
//! file is refused rather than written, because writing it would throw away a
//! level the person may have chosen.

use std::path::{Path, PathBuf};

use alo_capability::WhoReadsPictures;

use crate::settings_kept::SettingsKept;

/// The choice, as Settings found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WhoReadsPicturesHere {
    /// What the file says: no one where it says nothing, and no one where it
    /// would not read.
    is: WhoReadsPictures,
    /// Where it is kept.
    at: PathBuf,
}

impl WhoReadsPicturesHere {
    /// The choice at sign-in.
    pub(crate) fn at_sign_in(at: &Path) -> Self {
        Self {
            is: WhoReadsPictures::as_read(alo_remembering::who_reads_pictures_remembered(at)),
            at: at.to_owned(),
        }
    }

    /// Which reader is chosen.
    pub(crate) const fn is(&self) -> &WhoReadsPictures {
        &self.is
    }

    /// One press on one reader: written first and drawn second.
    pub(crate) fn picked(&mut self, reader: &WhoReadsPictures) -> SettingsKept {
        match alo_remembering::who_reads_pictures_chosen(&self.at, reader.clone()) {
            Ok(now) => {
                self.is = now;
                SettingsKept::Written
            }
            Err(_) => SettingsKept::NotWritten,
        }
    }
}
