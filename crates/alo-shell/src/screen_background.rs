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

use alo_appearance::{Background, Colour};
use alo_displays::Warming;
use smithay::utils::{Physical, Rectangle};

use crate::RenderError;
use crate::lock_background::a_size_worth_painting;
use crate::painted::Solid;

/// One screen's background, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScreenBackground {
    /// Flat shapes, in painting order.
    pub(crate) solids: Vec<Solid>,
}

impl ScreenBackground {
    /// The surface `chosen` for this screen, at `size`, warmed by `warming`.
    ///
    /// It took two more arguments until ADR 0075 — how long the session had been
    /// running, and where the image put its wallpapers — and both existed only to
    /// choose which picture of a rotating folder was showing. There is no folder
    /// and no picture, so there is nothing to choose.
    ///
    /// # Errors
    /// [`RenderError::DesktopScene`] for a screen with no pixels, or more of them
    /// than this will allocate.
    /// # The area below is the screen's **room**, not its framebuffer
    ///
    /// It is built as `Rectangle<i32, Physical>` because that is the type the
    /// renderer takes, and **the marker is not the truth about the value**: the
    /// numbers came through `TheRoom`, which is the screen's pixels already
    /// divided by the scale it is drawn at.
    ///
    /// Taking [`TheRoom`] rather than a pair is what stops the two being
    /// swapped here. Three wrong diagnoses in three days came from reading that
    /// `Physical` marker as a claim about the units; the parameter now carries
    /// the claim instead, where it can be checked by a compiler.
    pub(crate) fn prepare(
        chosen: &Background,
        warming: Warming,
        room: crate::TheRoom,
    ) -> Result<Self, RenderError> {
        let size = room.across_and_along();
        if !a_size_worth_painting(size) {
            return Err(RenderError::DesktopScene);
        }
        let (width, height) = size;
        let area = Rectangle::<i32, Physical>::new((0, 0).into(), (width, height).into());
        Ok(Self {
            solids: vec![Solid {
                area,
                colour: as_painted(warming.applied_to(chosen.colour())),
            }],
        })
    }
}

/// A colour as the painter takes it.
fn as_painted(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

/// A refusal from reading a chosen image, said as the desktop's.
///
#[cfg(test)]
#[path = "screen_background_tests.rs"]
mod tests;
