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

    /// The same picture at this size, for drawing into a box.
    ///
    /// **Here rather than in the raster**, because it is arithmetic with a
    /// right answer and the raster is the file nobody wants to open. The Dock
    /// draws into a box whose size depends on the display's scale, so the
    /// scaling cannot be done where the picture is read — and a wrong answer
    /// here is a soft or fringed icon rather than a crash, which is exactly the
    /// kind of fault that survives a review and not a test.
    ///
    /// # Averaged over alpha, which is what stops a dark fringe
    ///
    /// Each destination pixel is the average of the source pixels inside it.
    /// Averaging red, green and blue **on their own** would mix in the colour
    /// of the fully transparent pixels around an icon's edge — usually black —
    /// and every icon would come out with a dark halo. So the colours are
    /// weighted by alpha and divided by the alpha that was there, which is the
    /// same thing a premultiplied composite does and is why the edge of a
    /// rounded icon stays its own colour.
    ///
    /// A destination pixel with no opacity under it keeps no colour, because
    /// there is no colour there to keep.
    #[must_use]
    pub fn scaled_to(&self, across: u32, down: u32) -> Self {
        if across == 0 || down == 0 || (across == self.across && down == self.down) {
            return self.clone();
        }
        let mut out = vec![0_u8; across as usize * down as usize * 4];
        for row in 0..down {
            // The source rows this destination row covers, at least one.
            let from_row = row * self.down / down;
            let to_row = (((row + 1) * self.down).div_ceil(down))
                .max(from_row + 1)
                .min(self.down);
            for column in 0..across {
                let from_column = column * self.across / across;
                let to_column = (((column + 1) * self.across).div_ceil(across))
                    .max(from_column + 1)
                    .min(self.across);
                let (mut red, mut green, mut blue, mut alpha, mut counted) =
                    (0_u64, 0_u64, 0_u64, 0_u64, 0_u64);
                for source_row in from_row..to_row {
                    for source_column in from_column..to_column {
                        let at = (source_row as usize * self.across as usize
                            + source_column as usize)
                            * 4;
                        // **Asked for rather than indexed**, which the lints
                        // require and which is right here for a reason beyond
                        // them: every bound in this loop is arithmetic on two
                        // sizes, and a picture whose bytes did not match its
                        // size would panic in a compositor rather than draw one
                        // icon wrongly. `APicture::of` already refuses that, so
                        // this is unreachable — and unreachable is exactly
                        // where a panic is least welcome and least noticed.
                        let Some([one_red, one_green, one_blue, opacity]) = self
                            .pixels
                            .get(at..at + 4)
                            .and_then(|four| <[u8; 4]>::try_from(four).ok())
                        else {
                            continue;
                        };
                        let opacity = u64::from(opacity);
                        red += u64::from(one_red) * opacity;
                        green += u64::from(one_green) * opacity;
                        blue += u64::from(one_blue) * opacity;
                        alpha += opacity;
                        counted += 1;
                    }
                }
                let at = (row as usize * across as usize + column as usize) * 4;
                let mixed = |total: u64| {
                    total
                        .checked_div(alpha)
                        .map_or(0, |each| u8::try_from(each).unwrap_or(u8::MAX))
                };
                if let Some(there) = out.get_mut(at..at + 4) {
                    there.copy_from_slice(&[
                        mixed(red),
                        mixed(green),
                        mixed(blue),
                        u8::try_from(alpha / counted.max(1)).unwrap_or(u8::MAX),
                    ]);
                }
            }
        }
        Self {
            across,
            down,
            pixels: out,
        }
    }
}

/// What artwork this machine has, by the application that draws it.
///
/// **An application that is absent has no artwork**, and the Dock draws its
/// first letter — which is the decided answer rather than a gap
/// (`docs/design/the-alo-dock.md`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheArtwork(BTreeMap<AppId, APicture>);

/// A machine with no artwork, borrowable for as long as the program runs.
///
/// **For the frame types that carry a borrow of this**, and for every example
/// and test that draws a Dock without caring what is on it. The alternative is
/// an `Option<&TheArtwork>` on each of them, and this repository has already
/// paid for one of those: `market` was written `None` at a frame-building site,
/// reached the raster as nothing, and cost a release. An empty map says the
/// same thing and cannot be confused with *nobody filled this in*.
///
/// **A `static` and not an associated `const`**, which it was for an hour. A
/// `const` is a value copied in at each use, and `&TheArtwork::NOTHING` is then
/// a borrow of a temporary -- which the compiler cannot promote to `'static`
/// here, because a `BTreeMap` has a `Drop` implementation and nothing with one
/// is ever promoted. Five call sites refused to compile and said so.
pub static NOTHING: TheArtwork = TheArtwork(BTreeMap::new());

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
