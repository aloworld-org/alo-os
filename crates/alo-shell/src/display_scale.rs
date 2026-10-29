//! One display's scale, and the one road a logical measure takes to its pixels.
//!
//! `CLAUDE.md`: *nothing is built to one screen size*. A logical measure —
//! `alo_dock::measures::ICON` is 48 — is declared once and converted for each
//! display through that display's scale, so twice the pixels draws the same
//! Dock sharper rather than half as big. The fault the rule names is a figure
//! that reaches a display without passing through here.
//!
//! # Whole per cent, and no floating point
//!
//! A display's scale is a whole per cent, 100 to 300 (`alo_displays::Scale`),
//! and that is what this holds. So **at 100 every conversion is the identity**
//! — not nearly, exactly — which is what lets a Dock that goes through here
//! draw the same bytes it drew before it did. At 200 every measure is exactly
//! twice itself. At a fractional scale a measure is rounded to the nearest
//! pixel, so a sum of converted measures can differ from the converted sum by
//! one pixel; nothing here depends on the two agreeing.
//!
//! # Back again, for a press
//!
//! A press arrives in the display's pixels and `alo_dock` answers *what is
//! here* in logical ones, so [`DisplayScale::logical`] divides, rounding down.
//! At a whole scale that lands every pixel of a converted slot inside the
//! logical slot it was drawn from.

use alo_dock::Room;

/// How large one display draws, as a whole per cent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DisplayScale {
    /// The per cent, never zero.
    per_cent: u16,
}

impl DisplayScale {
    /// One pixel drawn for each logical one.
    ///
    /// **Only for a picture whose room is already logical**, such as the
    /// arrangement preview in `crate::screens_raster`, or for a surface that has
    /// not been made to follow its display yet — and a call that uses it says
    /// which of the two it is.
    pub(crate) const ONE: Self = Self { per_cent: 100 };

    /// A display drawn at this many per cent, as `Server::the_scale_of_display`
    /// answers it.
    ///
    /// Nought is not a scale and is read as one rather than refused: a frame
    /// with no answer for its display is drawn at one-to-one, which is what
    /// every frame did before this type existed.
    pub(crate) const fn of_per_cent(per_cent: u16) -> Self {
        if per_cent == 0 {
            return Self::ONE;
        }
        Self { per_cent }
    }

    /// The per cent.
    pub(crate) const fn per_cent(self) -> u16 {
        self.per_cent
    }

    /// The scale as a factor, for the figures that are drawn in floating point:
    /// a panel's corner and shadow (`crate::how_a_panel_is_raised`).
    pub(crate) fn factor(self) -> f64 {
        f64::from(self.per_cent) / 100.0
    }

    /// A logical measure in this display's pixels, to the nearest pixel.
    pub(crate) fn px(self, logical: u32) -> i32 {
        let scaled = (u64::from(logical) * u64::from(self.per_cent) + 50) / 100;
        i32::try_from(scaled).unwrap_or(i32::MAX)
    }

    /// A logical [`Room`] in this display's pixels.
    pub(crate) fn room(self, logical: Room) -> i32 {
        self.px(logical.as_pixels())
    }

    /// A type size in this display's pixels, for the shaper.
    #[expect(
        clippy::cast_precision_loss,
        reason = "a type size in pixels is a few hundred at most, well inside an f32"
    )]
    pub(crate) fn type_size(self, logical: u32) -> f32 {
        self.px(logical) as f32
    }

    /// A distance in this display's pixels as a logical one, rounded down.
    ///
    /// Negative is nought: a distance is measured from a surface's own edge,
    /// and a point before that edge is not on it.
    pub(crate) fn logical(self, pixels: i32) -> u32 {
        let pixels = u64::try_from(pixels).unwrap_or(0);
        u32::try_from(pixels * 100 / u64::from(self.per_cent)).unwrap_or(u32::MAX)
    }
}

/// [`DisplayScale::ONE`], for a file `tests/screens_source.rs` holds to never
/// naming the arrangement's own scale type: its guard reads any `Scale::` in a
/// line as that, and a path to this type's constant has one.
pub(crate) const fn one_to_one() -> DisplayScale {
    DisplayScale::ONE
}

#[cfg(test)]
#[path = "display_scale_tests.rs"]
mod tests;
