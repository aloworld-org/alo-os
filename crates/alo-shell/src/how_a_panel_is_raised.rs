//! How a rounded panel is lifted off what is behind it — its corner and its
//! shadow — and the display scale those figures are drawn at.
//!
//! # Two looks, both from the design file
//!
//! | | corner | shadow |
//! |---|---|---|
//! | [`HowItIsRaised::THE_DOCKS`] | 18 | `0 8px 24px` `#07131F` at 8% — ADR 0101 |
//! | [`HowItIsRaised::FLOATING`] | `radius/lg` 18 | `Elevation/Floating`: `#07131F24`, offset (0, 16), blur 40, spread −4 |
//!
//! The second was asked of the live file on 2026-10-28 for the terminal's body,
//! `479:29012`, and is recorded in `docs/design/the-terminal-window.md`.
//!
//! # Every figure here is logical, and passes through one scale
//!
//! `CLAUDE.md`: *nothing is built to one screen size*. So a look is a set of
//! logical measures and [`Raised`] is that look **at a display's scale** — the
//! corner, the hairline, the shadow's fall, blur and spread, a well's corner
//! and a focus ring all go through the same multiplication, in
//! `crate::dock_surface`, and none of them through any other.
//!
//! # Which panels draw at their display's scale
//!
//! **The Dock and *More apps* do**, since 2026-10-28: their band, slots and
//! marks go through `crate::display_scale::DisplayScale` in
//! `crate::dock_raster`, so their corner and shadow ask for
//! [`HowItIsRaised::at`] with the same scale and the whole panel moves together.
//!
//! **The Bar and the task window do not yet.** Their geometry is still laid out
//! in logical measures used straight as pixels, and drawing their corner at a
//! dense display's scale would round a panel that is not scaled — an 18 radius
//! drawn as 36 on a band still 76 tall, which is the fault `crate::dock_raster`
//! records. So they ask for [`HowItIsRaised::unscaled`], which names that fact
//! at the call rather than hiding it in a `1.0`; when their layout goes through
//! the display's scale, those calls become [`HowItIsRaised::at`] and nothing in
//! the rasteriser changes.

use alo_dock::measures::{THE_DOCKS_CORNER, THE_SHADOW_FALLS, THE_SHADOW_SPREADS};

/// A panel's corner and shadow, in logical measures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HowItIsRaised {
    /// The panel's corner radius.
    corner: u32,
    /// How far the shadow falls below the panel.
    falls: u32,
    /// How wide the shadow's fade is.
    blurs: u32,
    /// How much bigger than the panel the shadow is before it fades — negative
    /// for a shadow tucked in under the panel, as `Elevation/Floating`'s is.
    spreads: i32,
    /// How dark the shadow is at its darkest, from nought to one.
    darkness: f32,
}

impl HowItIsRaised {
    /// The Dock's own panel: ADR 0101's corner and its `Overlay / Glass`
    /// shadow, `rgba(7,19,31,0.08)`, measured on `348:23874`.
    ///
    /// **Every figure is the one the rasteriser drew before it took a look at
    /// all**, and `crate::dock_surface`'s tests hold the pixels to that.
    pub(crate) const THE_DOCKS: Self = Self {
        corner: THE_DOCKS_CORNER,
        falls: THE_SHADOW_FALLS,
        blurs: THE_SHADOW_SPREADS,
        spreads: 0,
        darkness: 0.08,
    };

    /// `Elevation/Floating` on a `radius/lg` panel: the terminal's body.
    ///
    /// The colour's alpha is `0x24`, thirty-six out of 255.
    pub(crate) const FLOATING: Self = Self {
        corner: 18,
        falls: 16,
        blurs: 40,
        spreads: -4,
        darkness: 36.0 / 255.0,
    };

    /// This look at a display's scale, as a fraction: 1.0 at 100%, 2.0 at
    /// 200%.
    ///
    /// A scale that is not a positive number is taken as one, so a display
    /// described wrongly draws an ordinary panel rather than none.
    pub(crate) fn at(self, scale: f64) -> Raised {
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        Raised { how: self, scale }
    }

    /// This look for a panel whose own geometry is not scaled — the Bar and
    /// the task window, today. See this file's header.
    pub(crate) const fn unscaled(self) -> Raised {
        Raised {
            how: self,
            scale: 1.0,
        }
    }
}

/// A look at one display's scale: every figure the rasteriser needs, in that
/// display's pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Raised {
    /// The look.
    how: HowItIsRaised,
    /// The display's scale, as a fraction.
    scale: f64,
}

impl Raised {
    /// Any logical measure, in this display's pixels.
    ///
    /// **The one conversion**, which is the point: a well's corner and a focus
    /// ring's thickness go through this exactly as the panel's corner does.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a figure a few dozen units long, at a display's scale, is well inside an f32"
    )]
    pub(crate) fn a_measure(self, logical: u32) -> f32 {
        (f64::from(logical) * self.scale) as f32
    }

    /// The panel's corner radius.
    pub(crate) fn corner(self) -> f32 {
        self.a_measure(self.how.corner)
    }

    /// How far the shadow falls.
    pub(crate) fn falls(self) -> f32 {
        self.a_measure(self.how.falls)
    }

    /// How wide the shadow's fade is, never less than one pixel.
    pub(crate) fn blurs(self) -> f32 {
        self.a_measure(self.how.blurs).max(1.0)
    }

    /// How much bigger than the panel the shadow is, before it fades.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a spread of a few units, at a display's scale, is well inside an f32"
    )]
    pub(crate) fn spreads(self) -> f32 {
        (f64::from(self.how.spreads) * self.scale) as f32
    }

    /// How dark the shadow is at its darkest.
    pub(crate) const fn darkness(self) -> f32 {
        self.how.darkness
    }

    /// How far past the panel's own edge the raster has to reach to hold the
    /// whole shadow, in whole pixels, on every side.
    ///
    /// The fall and the blur together, plus the spread where it grows the
    /// shadow; taken on every side because one number is easier to be right
    /// about than four, and the cost is a few transparent pixels.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a reach of a few dozen units, at a display's scale, is a small whole number"
    )]
    pub(crate) fn reach(self) -> i32 {
        let logical = f64::from(self.how.falls)
            + f64::from(self.how.blurs)
            + f64::from(self.how.spreads.max(0));
        (logical * self.scale).ceil() as i32
    }
}

#[cfg(test)]
#[path = "how_a_panel_is_raised_tests.rs"]
mod tests;
