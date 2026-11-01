//! Where the record window goes on one output, and the measures it is drawn
//! at: the person's text size times the display's own scale.
//!
//! **A rule of the display, not a figure.** The window is
//! `record_measures::WIDEST` wide where there is room and narrower where
//! there is not; it is as tall as `record_measures::TALLEST` or the room above
//! the Dock allows, and its foot stays `record_measures::ABOVE_THE_FOOT` above
//! the display's, so it never covers the Dock. Both approved frames come out of
//! that rule exactly: 880 × 796 at (280, 56) on 1440 × 960, and 880 × 652 at
//! (200, 40) on 1280 × 800.
//!
//! Apart from `crate::record_raster` because it changes for a different reason:
//! this is where the window is, and that file is what goes in it.

use alo_appearance::{Scheme, TextScale};
use alo_strings::Direction;
use smithay::utils::Rectangle;

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::record_measures as m;
use crate::settings_canvas::Area;
use crate::{Contrast, RenderError};

/// How the record window looks, as the person's appearance and language
/// decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordLook {
    /// Light or dark, as `alo-appearance` decides it.
    pub scheme: Scheme,
    /// The person's text scale, applied to every measure.
    pub scale: TextScale,
    /// Which way the person reads, which decides which edge everything starts
    /// from and which the rail is on.
    pub reading: Direction,
    /// The design's palette, or the one high contrast decides
    /// (`crate::access_contrast`).
    pub contrast: Contrast,
}

/// The largest output side, in pixels, the window is laid out for.
const LARGEST_SIDE: i32 = 16_384;

/// The least the window may be, logical, before it refuses the frame: room
/// for the heading's line and one short entry.
const THE_LEAST: (i32, i32) = (320, 300);

/// Where the window is on one output, in its pixels.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Room {
    /// The text size and the display's scale, together.
    pub(crate) measure: Measure,
    /// One logical pixel through the display's scale alone, so a larger text
    /// size never thickens a hairline.
    pub(crate) hairline: i32,
    /// The whole window: the strip's band and the body under it.
    pub(crate) window: Area,
    /// The strip that is drawn, inside its band.
    pub(crate) strip: Area,
    /// The body.
    pub(crate) body: Area,
    /// Which way the person reads.
    pub(crate) reading: Direction,
}

impl Room {
    /// The window on an output of `size` drawn at `display`, or the refusal
    /// for an output too small for it or larger than any it is laid out for.
    pub(crate) fn on(
        size: (i32, i32),
        look: RecordLook,
        display: DisplayScale,
    ) -> Result<Self, RenderError> {
        let (width, height) = size;
        if width > LARGEST_SIDE || height > LARGEST_SIDE {
            return Err(RenderError::RecordScene);
        }
        let measure = Measure::of(look.scale).on(display);
        let px = |at_one: i32| measure.px(at_one);
        let wide = px(m::WIDEST).min(width - 2 * px(m::SIDE_ROOM));
        let foot = height - px(m::ABOVE_THE_FOOT);
        let tall = px(m::TALLEST).min(foot - px(m::ABOVE_THE_TOP));
        if wide < px(THE_LEAST.0) || tall < px(THE_LEAST.1) {
            return Err(RenderError::RecordScene);
        }
        let window = Rectangle::new(((width - wide) / 2, foot - tall).into(), (wide, tall).into());
        let (strip_at, strip_tall) = m::THE_STRIP;
        let strip = Rectangle::new(
            (window.loc.x, window.loc.y + px(strip_at)).into(),
            (wide, px(strip_tall)).into(),
        );
        let band = px(m::THE_STRIP_BAND);
        let body = Rectangle::new(
            (window.loc.x, window.loc.y + band).into(),
            (wide, tall - band).into(),
        );
        Ok(Self {
            measure,
            hairline: display.px(1),
            window,
            strip,
            body,
            reading: look.reading,
        })
    }

    /// `at_one` logical pixels, scaled.
    pub(crate) fn px(self, at_one: i32) -> i32 {
        self.measure.px(at_one)
    }

    /// `area`, laid out as though read left to right, where it is drawn for
    /// the person's direction: turned round the window for a reader of a
    /// right-to-left language, as the canvas turns everything it draws.
    pub(crate) fn as_drawn(self, area: Area) -> Area {
        match self.reading {
            Direction::LeftToRight => area,
            Direction::RightToLeft => Rectangle::new(
                (
                    2 * self.window.loc.x + self.window.size.w - area.loc.x - area.size.w,
                    area.loc.y,
                )
                    .into(),
                area.size,
            ),
        }
    }

    /// The canvas the window is drawn on: the window and room for its shadow,
    /// within the output.
    pub(crate) fn canvas(self, size: (i32, i32)) -> Area {
        let reach = self.px(48);
        let around = Rectangle::new(
            (self.window.loc.x - reach, self.window.loc.y - reach).into(),
            (
                self.window.size.w + 2 * reach,
                self.window.size.h + 2 * reach,
            )
                .into(),
        );
        around
            .intersection(Rectangle::from_size(size.into()))
            .unwrap_or(self.window)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::record_testing::light;

    /// **Both approved frames come out of the one rule**: 880 × 796 at
    /// (280, 56) on 1440 × 960, and 880 × 652 at (200, 40) on 1280 × 800.
    #[test]
    fn both_frames_come_out_of_the_rule() {
        let big = Room::on((1440, 960), light(), DisplayScale::ONE).unwrap();
        assert_eq!(big.window, Rectangle::new((280, 56).into(), (880, 796).into()));
        assert_eq!(big.body, Rectangle::new((280, 100).into(), (880, 752).into()));
        assert_eq!(big.strip, Rectangle::new((280, 68).into(), (880, 32).into()));
        let small = Room::on((1280, 800), light(), DisplayScale::ONE).unwrap();
        assert_eq!(small.window, Rectangle::new((200, 40).into(), (880, 652).into()));
    }

    /// **At 200 per cent it is twice the size, in the same place to the
    /// eye**, and a narrow display narrows it rather than cutting it.
    #[test]
    fn it_scales_and_narrows() {
        let doubled = Room::on((2880, 1920), light(), DisplayScale::of_per_cent(200)).unwrap();
        assert_eq!(
            doubled.window,
            Rectangle::new((560, 112).into(), (1760, 1592).into())
        );
        let narrow = Room::on((800, 960), light(), DisplayScale::ONE).unwrap();
        assert_eq!(narrow.window.size.w, 800 - 32);
    }

    /// **An output too small for the window refuses it**, rather than
    /// drawing a window nobody could read.
    #[test]
    fn too_small_is_refused() {
        assert!(Room::on((1920, 90), light(), DisplayScale::ONE).is_err());
        assert!(Room::on((300, 960), light(), DisplayScale::ONE).is_err());
        assert!(Room::on((20_000, 960), light(), DisplayScale::ONE).is_err());
    }
}
