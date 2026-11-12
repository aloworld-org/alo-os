//! Native pixels alo draws itself: flat shapes, inked text and rasters with
//! their alpha, each with where it goes.
//!
//! **Pixels in memory, and nothing that puts them on a screen.** `alo-shell`
//! keeps `paint`, which hands these to a compositor's frame, and converts
//! each [`Area`] to the compositor's rectangle at that one seam.

use crate::area::Area;

/// One flat rectangle of one colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solid {
    /// Where, in output pixels.
    pub area: Area,
    /// Red, green, blue.
    pub colour: [u8; 3],
}

/// Text inked onto its own ground, row-major and opaque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inked {
    /// Where, in output pixels; always inside the output.
    pub area: Area,
    /// `area.size.w * area.size.h` pixels.
    pub pixels: Vec<[u8; 3]>,
}

/// A raster with an alpha channel, composited over whatever is behind it.
///
/// **What [`Solid`] cannot be.** A `Solid` is one opaque colour over an
/// axis-aligned rectangle, which is all a flat band or a line of text needs and
/// is why that is what this file draws. A rounded corner and a shadow are
/// **partial coverage**, and no number of rectangles makes one: the pixel at a
/// corner is some of the panel and some of what is behind it.
///
/// So this carries `RGBA` — four bytes a pixel, row-major, the memory order
/// `Fourcc::Abgr8888` wants — and is uploaded as a texture by whoever holds the
/// renderer.
///
/// **What a renderer is handed must be premultiplied**: each colour already
/// multiplied by its own alpha. Both of this shell's renderers blend that way
/// — GLES with `ONE, ONE_MINUS_SRC_ALPHA`, pixman because its `a8b8g8r8` *is*
/// premultiplied — so a pixel is drawn as *its colour plus what is behind it,
/// times one less its alpha*. A straight pixel of shadow colour at almost no
/// alpha is therefore drawn as the background **plus** the shadow's colour,
/// which on the cream canvas runs past 255 in every channel: the shadow round a
/// window came out as an opaque white rectangle, seen on Dolphin on 2026-11-08.
/// A raster made in straight alpha — `alo-shell`'s `settings_canvas.rs` makes one, for
/// pictures that are also read as icons — goes through
/// [`Washed::premultiplied`] on its way to a renderer. It is **not** painted by
/// `alo-shell`'s `paint`, because
/// importing a texture needs the `Renderer` and `paint` only has a `Frame`:
/// `alo-shell`'s `scene_drawing.rs` imports these before it opens its frame, which is the
/// same order it already uses for every client surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Washed {
    /// Where, in output pixels.
    pub area: Area,
    /// `area.size.w * area.size.h` pixels, four bytes each: red, green, blue,
    /// then alpha.
    pub pixels: Vec<u8>,
}

impl Washed {
    /// The same raster with each colour multiplied by its own alpha, which is
    /// what a renderer blends (see [`Washed`]).
    ///
    /// Rounded to nearest, so an opaque pixel is unchanged and a transparent
    /// one is black: nothing at all, to a premultiplied blend.
    #[must_use]
    pub fn premultiplied(mut self) -> Self {
        for [red, green, blue, alpha] in self.pixels.as_chunks_mut::<4>().0 {
            let alpha = u32::from(*alpha);
            for channel in [red, green, blue] {
                let times = (u32::from(*channel) * alpha + 127) / 255;
                *channel = u8::try_from(times).unwrap_or(u8::MAX);
            }
        }
        self
    }
}

#[cfg(test)]
#[path = "painted_tests.rs"]
mod tests;
