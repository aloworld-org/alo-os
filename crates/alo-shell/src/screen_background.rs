//! What is behind the windows on one screen: the background that screen wears,
//! fitted to its own room and warmed by its own night light.
//!
//! # Whose decisions these are
//!
//! **Which** background this screen wears is `alo-appearance`'s, asked for by
//! the name the shell knows the screen by, and handed here through
//! `alo_displays::Wearing`. **How warm** it is drawn is night light's, applied
//! per screen. Neither is decided here: this file fits the chosen picture to
//! this screen's room and multiplies each channel by what
//! `alo_displays::Warming` says, and it reads no file `alo-appearance` did not
//! name.
//!
//! # Warming is a lookup, and it is the same arithmetic
//!
//! A 4K background is eight million pixels, and warming each one by calling
//! into `alo-displays` for every one of them would be eight million calls a
//! frame. So the three channels are asked **once** each, for all 256 values a
//! channel can hold, through `Warming::applied_to` itself — which is exactly
//! the arithmetic a colour elsewhere in the shell gets, rather than a second
//! copy of it that could drift.
//!
//! # A picture and a plain colour are the same shape here
//!
//! Both come back as flat shapes and inked pixels in painting order, so the
//! surface above draws them the one way and never has to take the background
//! apart to find out which kind it is.

use std::path::Path;
use std::time::Duration;

use alo_appearance::{Background, Colour};
use alo_displays::Warming;
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::lock_background_path::selected;
use crate::lock_image_decode::{MAX_PIXELS, decode};
use crate::painted::{Inked, Solid};

/// The largest screen side a background is fitted to.
const LARGEST_SIDE: i32 = 8192;

/// One screen's background, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenBackground {
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
    /// Inked pixels, painted after the shapes.
    pub(crate) inked: Vec<Inked>,
}

impl ScreenBackground {
    /// The background `chosen` for this screen, fitted to `size` and warmed by
    /// `warming`.
    ///
    /// `running` is how long the session has been going, which is what
    /// `alo-appearance` turns into *which* of a rotating folder's pictures is
    /// showing now; the choice is that crate's and the reading is here.
    ///
    /// # Errors
    /// [`RenderError::DesktopScene`] for a screen with no pixels or more than
    /// this file fits, and for a chosen picture that is missing, malformed,
    /// larger than is decoded, or in a folder too large to read.
    pub(crate) fn prepare(
        chosen: &Background,
        warming: Warming,
        size: (i32, i32),
        running: Duration,
        shipped: &Path,
    ) -> Result<Self, RenderError> {
        let (width, height) = size;
        if width <= 0
            || height <= 0
            || width > LARGEST_SIDE
            || height > LARGEST_SIDE
            || u64::from(width.unsigned_abs()) * u64::from(height.unsigned_abs()) > MAX_PIXELS
        {
            return Err(RenderError::DesktopScene);
        }
        let area = Rectangle::<i32, Physical>::new((0, 0).into(), (width, height).into());
        if let Background::Colour(colour) = chosen {
            return Ok(Self {
                solids: vec![Solid {
                    area,
                    colour: as_painted(warming.applied_to(*colour)),
                }],
                inked: Vec::new(),
            });
        }

        let (path, fitting) = selected(chosen, running, shipped).map_err(on_the_desktop)?;
        let picture = decode(&path).map_err(on_the_desktop)?;
        let mut rgba = vec![0u8; width as usize * height as usize * 4];
        crate::lock_image_fit::paint(&picture, fitting, size, &mut rgba);
        Ok(Self {
            solids: Vec::new(),
            inked: vec![Inked {
                area,
                pixels: warmed(&rgba, warming),
            }],
        })
    }
}

/// Every fitted pixel as this screen's night light shows it, row-major.
fn warmed(rgba: &[u8], warming: Warming) -> Vec<[u8; 3]> {
    let channel = |pick: fn(Colour) -> u8| -> Vec<u8> {
        (0..=u8::MAX)
            .map(|value| pick(warming.applied_to(Colour::of(value, value, value))))
            .collect()
    };
    let (red, green, blue) = (
        channel(Colour::red),
        channel(Colour::green),
        channel(Colour::blue),
    );
    let through = |table: &[u8], value: u8| table.get(value as usize).copied().unwrap_or(value);
    rgba.as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| {
            let [r, g, b, _] = *pixel;
            [through(&red, r), through(&green, g), through(&blue, b)]
        })
        .collect()
}

/// A colour as the painter takes it.
fn as_painted(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

/// A refusal from reading a chosen image, said as the desktop's.
///
/// The two doors this file reads an image through are shared with the lock
/// screen, and both word every failure they have as that screen's — a missing
/// file comes back as a submission failure whose text names the lock
/// background. Neither of them refuses for any other reason, so every refusal
/// they make is this one: the desktop could not draw what a person chose. It is
/// said that way here, because a desktop frame that refused in the lock
/// screen's words would send somebody looking at the wrong surface.
fn on_the_desktop(_error: RenderError) -> RenderError {
    RenderError::DesktopScene
}

#[cfg(test)]
#[path = "screen_background_tests.rs"]
mod tests;
