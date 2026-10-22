//! **The picture each application draws on the Dock**, where it has one.
//!
//! [ADR 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md),
//! the owner's ruling:
//!
//! > Real applications retain their actual application icons … **First letters
//! > are a missing-icon fallback, not the finished Dock.**
//!
//! Until this existed the fallback was the answer for every application on
//! every machine, because nothing carried artwork to the Dock at all.
//!
//! # The shape of an answer, and not the finding of one
//!
//! This is [`crate::pinning::Discovery`]'s sibling and deliberately the same
//! shape. Which file holds an application's icon is **a reading of the
//! machine**, and `crates/alo-shell` is the crate forbidden to take one — so
//! the answer arrives as data, worked out by the binary that stands the desktop
//! up. This crate says what that data is; it does not go looking.
//!
//! So there is no path in here, no file, no format and no decoder. A caller
//! hands over pixels it has already made sense of, and a test hands over four
//! bytes without a PNG anywhere near it.
//!
//! # Keyed by the name the Dock knows
//!
//! [`crate::AppId`] is what the Dock holds and what it already draws a letter
//! from, so it is what artwork is looked up by. Keying this on a desktop file
//! identifier instead would be a second way of naming one application, and the
//! Dock would have to carry a translation between them on the drawing path —
//! which is the place least able to say what to do when the translation fails.
//!
//! # Straight-sided and upright, because the Dock draws it into a square
//!
//! The badge an application's letter sits in **is** the artwork box
//! (`dock_raster`), so a picture is drawn into a square of a known size. A
//! picture that is not square is not refused — plenty are not — it is the
//! caller's business to have scaled it, and [`APicture::across`] and
//! [`APicture::down`] are carried so the drawing can place it honestly rather
//! than assume.

use std::collections::BTreeMap;

use crate::AppId;

/// One application's artwork, as pixels somebody else has already decoded.
///
/// **Four bytes to a pixel, red green blue alpha, in rows from the top.** That
/// is what the shell's own raster works in, so it is what crosses this
/// boundary — converting here would put a second opinion about pixel order in
/// a crate that draws nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APicture {
    /// How many pixels wide it is.
    across: u32,
    /// How many pixels tall it is.
    down: u32,
    /// Its pixels, four bytes each, in rows from the top.
    pixels: Vec<u8>,
}

/// Why a run of bytes is not a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPicture {
    /// It has no area, so there is nothing to draw.
    Empty,
    /// The bytes do not divide into four-byte pixels of that size.
    ///
    /// **A refusal and not a crop**, because the two ways to be wrong here are
    /// a truncated decode and a size read off the wrong field, and both draw
    /// something that looks like artwork while being somebody else's bytes.
    TheWrongNumberOfBytes {
        /// How many there should have been.
        wanted: usize,
        /// How many there were.
        given: usize,
    },
}

impl APicture {
    /// A picture of this size from these bytes, or why it is not one.
    ///
    /// # Errors
    ///
    /// [`NotAPicture`] where the size has no area, or the bytes do not match
    /// it exactly.
    pub fn of(across: u32, down: u32, pixels: Vec<u8>) -> Result<Self, NotAPicture> {
        if across == 0 || down == 0 {
            return Err(NotAPicture::Empty);
        }
        let wanted = across as usize * down as usize * 4;
        if pixels.len() != wanted {
            return Err(NotAPicture::TheWrongNumberOfBytes {
                wanted,
                given: pixels.len(),
            });
        }
        Ok(Self {
            across,
            down,
            pixels,
        })
    }

    /// How many pixels across it is.
    #[must_use]
    pub const fn across(&self) -> u32 {
        self.across
    }

    /// How many pixels down it is.
    #[must_use]
    pub const fn down(&self) -> u32 {
        self.down
    }

    /// Its pixels, red green blue alpha, in rows from the top.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// What artwork this machine has, by the application that draws it.
///
/// **An application that is absent has no artwork**, and the Dock draws its
/// first letter — which is the decided answer rather than a gap
/// (`docs/design/the-alo-dock.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheArtwork(BTreeMap<AppId, APicture>);

impl TheArtwork {
    /// A machine whose applications all draw their first letter.
    ///
    /// **This is a real answer and the honest one for three different
    /// machines**: one with no artwork installed, one whose artwork is all
    /// scalable, and one where the reading never ran. None of those may be
    /// told apart from the drawing path, and none of them should be guessed
    /// at — whoever did the reading says which it was, in its own report.
    #[must_use]
    pub fn none() -> Self {
        Self(BTreeMap::new())
    }

    /// Remember that this application draws this picture.
    ///
    /// **The first wins**, which is the icon directories' own rule: a person's
    /// own icon directory is searched before the one the image shipped, so the
    /// first answer for an application is the one they chose.
    pub fn draws(&mut self, application: AppId, picture: APicture) {
        self.0.entry(application).or_insert(picture);
    }

    /// What this application draws, or [`None`] where it draws a letter.
    #[must_use]
    pub fn of(&self, application: &AppId) -> Option<&APicture> {
        self.0.get(application)
    }

    /// How many applications have artwork.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.0.len()
    }

    /// Whether every application on this machine draws its first letter.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
#[path = "artwork_tests.rs"]
mod tests;
