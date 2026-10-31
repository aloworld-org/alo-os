//! Where the Settings window goes on a display, and how its two columns divide
//! it — as **rules** of the display, never as the size somebody drew.
//!
//! The design draws the window twice: 1200 × 816 on a 1440 × 960 screen
//! (`550:98`) and 1152 × 688 on a 1280 × 800 one (`550:420`). One rule draws
//! both, and every other screen:
//!
//! - **the window is as large as the design draws it, and no larger** — 1200
//!   across and 816 down at most;
//! - **and never closer to the screen's edge than 64 across or 56 down**, which
//!   is what the smaller frame leaves;
//! - **centred** in what is left;
//! - the side column is **seven thirtieths** of the window, which both frames
//!   draw (280 of 1200, 270 of 1152);
//! - the content's padding is the larger pair while the window is at its full
//!   width and the smaller pair once it is not, which is the one difference
//!   between the two frames' content that a proportion does not explain;
//! - the content's column is **at most 720**, and narrower when the window is.
//!
//! Everything here is in the window's own **design units**: a display's pixels
//! divided by its scale and by the person's text size, so the window keeps its
//! proportions when either changes and simply meets its least margin sooner.

use crate::settings_measures as m;

/// The least the content's column may be before the window refuses to draw:
/// one row's mark, its words and a switch side by side.
pub(crate) const THE_COLUMN_AT_LEAST: u32 = 240;

/// The window and its two columns on one display, in design units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheFrame {
    /// The window's left edge.
    pub(crate) left: u32,
    /// Its top edge.
    pub(crate) top: u32,
    /// Its width.
    pub(crate) wide: u32,
    /// Its height.
    pub(crate) tall: u32,
    /// The side column's width, its edge included.
    pub(crate) side: u32,
    /// The content's padding across.
    pub(crate) beside_the_content: u32,
    /// The content's padding above.
    pub(crate) above_the_content: u32,
    /// The column the content is written in.
    pub(crate) column: u32,
}

/// Why the window cannot be drawn on a display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TooSmall;

impl TheFrame {
    /// The window on a screen `wide` by `tall` design units.
    ///
    /// # Errors
    /// [`TooSmall`] for a screen with no room for the content's column at
    /// [`THE_COLUMN_AT_LEAST`].
    pub(crate) fn on(wide: u32, tall: u32) -> Result<Self, TooSmall> {
        let window_wide =
            m::THE_WINDOW_AT_MOST_WIDE.min(wide.saturating_sub(2 * m::BESIDE_THE_WINDOW));
        let window_tall =
            m::THE_WINDOW_AT_MOST_TALL.min(tall.saturating_sub(2 * m::ABOVE_THE_WINDOW));
        // **Of the window's inside**, its border taken off both sides, and
        // rounded up: 1198 × 7 / 30 is 279.5 and the design draws 280; 1150 is
        // 268.3 and it draws 269.
        let inside = window_wide.saturating_sub(2 * m::A_HAIRLINE);
        let side = (inside * m::THE_SIDE_COLUMN_IN_THIRTIETHS).div_ceil(30);
        let at_full_width = window_wide >= m::THE_WINDOW_AT_MOST_WIDE;
        let (beside, above) = if at_full_width {
            (m::BESIDE_THE_CONTENT, m::ABOVE_THE_CONTENT)
        } else {
            (
                m::BESIDE_THE_CONTENT_WHEN_NARROWER,
                m::ABOVE_THE_CONTENT_WHEN_NARROWER,
            )
        };
        let content = inside.saturating_sub(side);
        let column = m::THE_COLUMN_AT_MOST.min(content.saturating_sub(2 * beside));
        if column < THE_COLUMN_AT_LEAST || window_tall < 2 * m::A_TARGET + above {
            return Err(TooSmall);
        }
        Ok(Self {
            left: (wide - window_wide) / 2,
            top: (tall - window_tall) / 2,
            wide: window_wide,
            tall: window_tall,
            side,
            beside_the_content: beside,
            above_the_content: above,
            column,
        })
    }

    /// Where the content's column begins, across, from the window's left.
    pub(crate) const fn the_column_starts(self) -> u32 {
        m::A_HAIRLINE + self.side + self.beside_the_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both frames the design draws come out of the one rule**, to the unit.
    #[test]
    fn both_frames_come_out_of_one_rule() {
        let large = TheFrame::on(1440, 960).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            (large.left, large.top, large.wide, large.tall),
            (120, 72, 1200, 816)
        );
        assert_eq!(large.side, 280);
        assert_eq!(
            (large.beside_the_content, large.above_the_content),
            (64, 48)
        );
        assert_eq!(large.column, 720);
        // `550:637` begins at x 281, after the window's border and the side
        // column, and the column is 64 into it.
        assert_eq!(large.the_column_starts(), 345);

        let small = TheFrame::on(1280, 800).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            (small.left, small.top, small.wide, small.tall),
            (64, 56, 1152, 688)
        );
        // `550:421` is 269 wide, and `550:476` begins at x 270.
        assert_eq!(small.side, 269);
        assert_eq!(small.the_column_starts(), 318);
        assert_eq!(
            (small.beside_the_content, small.above_the_content),
            (48, 36)
        );
        assert_eq!(small.column, 720);
    }

    /// **A screen with no room is refused rather than drawn cramped**, and a
    /// tall narrow one still centres what it can hold.
    #[test]
    fn a_screen_with_no_room_is_refused() {
        assert_eq!(TheFrame::on(500, 400), Err(TooSmall));
        let narrow = TheFrame::on(800, 1280).unwrap_or_else(|_| unreachable!());
        assert_eq!(narrow.wide, 672);
        assert_eq!(narrow.tall, 816);
        assert_eq!(narrow.top, 232);
        assert!(narrow.column >= THE_COLUMN_AT_LEAST);
    }
}
