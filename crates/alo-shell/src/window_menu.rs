//! The window panel: where it opens and what it holds.
//!
//! **Window actions opens it** — the first of a window's three controls — and
//! so do `Alt+F3` and a right-click on the band. The owner's specification of
//! 2026-11-09 (`docs/design/the-window-controls.md`) draws it at `11:24`:
//! heading *Window*, three placement pictures, *Full screen*, *Move to…*, *Help
//! with this window* and *Capture & export*.
//!
//! # Only what works is here
//!
//! **A choice that does not work yet is not drawn** — not greyed, not *coming
//! soon* — which is the specification's rule and law 3's. So this panel holds
//! what this machine can do today: **Left**, **Maximise** (or **Restore**) and
//! **Right**, **Full screen** (or **Exit full screen**), and **Help with this
//! window** — drawn only while alo may be asked, and never while the person
//! has set AI to *Not at all* (`crate::alo_in_a_window`). Move to… and Capture
//! & export each join it in the change that makes them work, and the layout
//! below leaves their places measured rather than invented.
//!
//! It replaced a window menu with five rows — Minimise, Maximise, Move, Resize,
//! Close — and a title block. Minimise and Close are on the band and are not
//! repeated here (*no duplicates*); Move and Resize did nothing when pressed
//! (`carry_out_the_row` matched them to an empty arm) and are gone rather than
//! carried over.
//!
//! # Measured, from `11:51`
//!
//! | | |
//! |---|---|
//! | panel | 348 wide, corner 16 |
//! | where | 8 below the band, its leading edge 168 before Window actions' |
//! | heading | at (22, 20) |
//! | placement pictures | 94 × 94, 10 apart, from (22, 50) |
//! | divider | 304 × 1 at y 160 |
//! | Full screen | a 304 × 40 row at y 172 |
//! | the divider before alo | 11 below the row above it — `11:75` at 267, under *Move to…* ending at 256 |
//! | Help with this window | a 304 × 52 row, 17 below that divider — `11:76` at 284 |
//! | below the last thing drawn | 20 — `397 − 377`, under `11:80`'s `›` |
//!
//! **Without *Move to…* the divider and Help keep their distances, not their
//! places**: they follow Full screen as `11:75` and `11:76` follow *Move to…*,
//! so nothing floats where a row that is not built would be.

use alo_strings::Word;
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::window_edge::{OnTheEdge, WindowEdge};

/// How wide the panel is. `11:51`.
pub const THE_PANEL_IS_WIDE: i32 = 348;

/// How far below the band the panel opens: `11:51` at `y = 132` in the canvas
/// under a window at `y = 70` with a 54-tall band — `132 − 70 − 54`.
pub const BELOW_THE_BAND: i32 = 8;

/// How far the panel's leading edge is before Window actions': `11:51` at
/// `x = 786` against a window at 56 whose Window actions is at 898 — so the
/// control is at 954 in the canvas and `954 − 786 = 168`.
pub const BEFORE_WINDOW_ACTIONS: i32 = 168;

/// The panel's inset on every side its contents are measured from: the
/// heading, the pictures, the divider and the rows all start at `x = 22`.
pub const THE_INSET: i32 = 22;

/// Where the heading *Window* sits down the panel. `11:52`.
pub const THE_HEADING_SITS_AT: i32 = 20;

/// Where the row of placement pictures starts down the panel. `11:54`.
pub const THE_PICTURES_START_AT: i32 = 50;

/// How big one placement picture is. `11:55`.
pub const A_PICTURE_IS: i32 = 94;

/// The gap between placement pictures: `11:55` at 22, `11:59` at 126.
pub const BETWEEN_PICTURES: i32 = 10;

/// Where the divider under the pictures is. `11:67`.
pub const THE_DIVIDER_IS_AT: i32 = 160;

/// How wide the divider and every row are: `11:67` and `11:68`.
pub const A_ROW_IS_WIDE: i32 = 304;

/// Where the first row under the divider starts. `11:68`.
pub const THE_ROWS_START_AT: i32 = 172;

/// How tall a row is. `11:68`.
pub const A_ROW_IS_TALL: i32 = 40;

/// What is left below the last thing drawn: `11:80` ends at 377 in a 397-tall
/// panel.
pub const BELOW_THE_LAST: i32 = 20;

/// How far below the row above it the divider before alo is: `11:75` at 267
/// under *Move to…*, which ends at `216 + 40`.
pub const BEFORE_THE_DIVIDER_TO_ALO: i32 = 11;

/// How far below that divider *Help with this window* starts: `11:76` at 284.
pub const AFTER_THE_DIVIDER_TO_ALO: i32 = 17;

/// How tall *Help with this window* is. `11:76`.
pub const THE_ALO_ROW_IS_TALL: i32 = 52;

/// Where the divider before alo is, down the panel: under Full screen.
const THE_DIVIDER_TO_ALO_IS_AT: i32 = THE_ROWS_START_AT + A_ROW_IS_TALL + BEFORE_THE_DIVIDER_TO_ALO;

/// Where *Help with this window* starts, down the panel.
const THE_ALO_ROW_IS_AT: i32 = THE_DIVIDER_TO_ALO_IS_AT + AFTER_THE_DIVIDER_TO_ALO;

/// The choice the keyboard lands on when a key opens the panel: the first, as
/// the pictures and rows are laid out in [`the_menu_of`].
pub const THE_FIRST_CHOICE: WhatARowDoes = WhatARowDoes::Left;

/// One thing the panel can do to the window.
///
/// In the order a keyboard walks them and a reader meets them — the pictures
/// left to right, then the rows top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatARowDoes {
    /// Put the window on the left half of its display's work area, moving no
    /// other (`crate::a_window_on_one_side`).
    Left,
    /// Make the window fill the work area, keeping its band and the shell's
    /// areas. The same picture as [`Self::Restore`]; only one is present.
    Maximise,
    /// Put it back to the size and place it had. While it is maximised, or
    /// on a half.
    Restore,
    /// Put the window on the right half, moving no other.
    Right,
    /// Make it cover the whole display, hiding its band and the shell's areas.
    FullScreen,
    /// Put it back the way it was before full screen. While it is full screen.
    ExitFullScreen,
    /// **Open an alo request with this window as its context**, asking
    /// nothing until the person sends it (`crate::alo_in_a_window`). Only
    /// while alo may be asked.
    HelpWithThisWindow,
}

impl WhatARowDoes {
    /// What a person reads on it, and what a reader speaks.
    #[must_use]
    pub const fn its_own_word(self) -> Word {
        match self {
            Self::Left => alo_access::words::LEFT,
            Self::Right => alo_access::words::RIGHT,
            Self::Maximise => alo_access::words::MAXIMISE,
            Self::Restore => alo_access::words::RESTORE,
            Self::FullScreen => alo_access::words::FULL_SCREEN,
            Self::ExitFullScreen => alo_access::words::EXIT_FULL_SCREEN,
            Self::HelpWithThisWindow => alo_access::words::ASK_ABOUT_THIS_WINDOW,
        }
    }

    /// Whether this is drawn as a placement picture rather than a row.
    #[must_use]
    pub const fn is_a_picture(self) -> bool {
        matches!(
            self,
            Self::Left | Self::Maximise | Self::Restore | Self::Right
        )
    }

    /// The half this choice puts a window on, for Left and Right.
    #[must_use]
    pub(crate) const fn the_side(self) -> Option<crate::a_window_on_one_side::OneSide> {
        match self {
            Self::Left => Some(crate::a_window_on_one_side::OneSide::Left),
            Self::Right => Some(crate::a_window_on_one_side::OneSide::Right),
            _ => None,
        }
    }
}

/// How the window is now, which decides which word each choice reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HowTheWindowIs {
    /// Maximised or on a half now: the middle picture reads *Restore*.
    pub maximised: bool,
    /// Full screen now: the row reads *Exit full screen*.
    pub full_screen: bool,
    /// **Whether alo may be asked now**, so *Help with this window* is in the
    /// panel: something is chosen to answer, and AI is not set to *Not at
    /// all*. Not about the window, but carried with it because it decides what
    /// the panel holds; `false`, the default, is the panel without it.
    pub alo_may_help: bool,
}

/// One choice in the panel, and where it answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuRow {
    /// What choosing it does.
    pub does: WhatARowDoes,
    /// Where it is drawn and answers a press.
    pub row: Rectangle<i32, Logical>,
}

/// The panel of one window, laid out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowMenu {
    /// The whole panel. A press anywhere inside it is the panel's.
    pub panel: Rectangle<i32, Logical>,
    /// Where the heading *Window* starts.
    pub heading: Point<i32, Logical>,
    /// The divider under the pictures.
    pub divider: Rectangle<i32, Logical>,
    /// The divider before *Help with this window*, while it is there.
    pub before_alo: Option<Rectangle<i32, Logical>>,
    /// The choices, in keyboard order.
    pub rows: Vec<MenuRow>,
}

impl WindowMenu {
    /// Every choice, in the order the keyboard walks them.
    #[must_use]
    pub fn what_can_be_used(&self) -> Vec<MenuRow> {
        self.rows.clone()
    }

    /// The choice under this point, if any.
    #[must_use]
    pub fn what_is_under(&self, at: Point<i32, Logical>) -> Option<MenuRow> {
        self.rows.iter().copied().find(|it| it.row.contains(at))
    }
}

/// How tall the panel is with these choices: to the last row drawn, and
/// [`BELOW_THE_LAST`] under it.
const fn how_tall(alo_may_help: bool) -> i32 {
    if alo_may_help {
        THE_ALO_ROW_IS_AT + THE_ALO_ROW_IS_TALL + BELOW_THE_LAST
    } else {
        THE_ROWS_START_AT + A_ROW_IS_TALL + BELOW_THE_LAST
    }
}

/// The panel of a band whose Window actions is shown, or [`None`] on a band
/// that has no Window actions to open it from.
///
/// **Kept on the display**: anchored `BEFORE_WINDOW_ACTIONS` before the
/// control and `BELOW_THE_BAND` under the band, then moved — never cut — to
/// stay inside `output`. At 348 × 312 with *Help with this window* it fits the
/// smallest display this product lays out for (360 × 360,
/// `crate::sign_in_layout`), so nothing in it needs to scroll; the rows that
/// would not fit join with the change that draws them.
#[must_use]
pub fn the_menu_of(
    edge: &WindowEdge,
    how: HowTheWindowIs,
    output: Size<i32, Logical>,
) -> Option<WindowMenu> {
    let anchor = edge.the_control(OnTheEdge::WindowActions)?;
    let tall = how_tall(how.alo_may_help);
    let at_x = (anchor.target.loc.x - BEFORE_WINDOW_ACTIONS)
        .clamp(0, (output.w - THE_PANEL_IS_WIDE).max(0));
    let at_y = (edge.region.loc.y + edge.region.size.h + BELOW_THE_BAND)
        .clamp(0, (output.h - tall).max(0));
    let panel = Rectangle::new(
        Point::from((at_x, at_y)),
        Size::from((THE_PANEL_IS_WIDE, tall)),
    );
    let maximise = if how.maximised {
        WhatARowDoes::Restore
    } else {
        WhatARowDoes::Maximise
    };
    let full_screen = if how.full_screen {
        WhatARowDoes::ExitFullScreen
    } else {
        WhatARowDoes::FullScreen
    };
    let a_picture = |nth: i32, does: WhatARowDoes| MenuRow {
        does,
        row: Rectangle::new(
            Point::from((
                at_x + THE_INSET + nth * (A_PICTURE_IS + BETWEEN_PICTURES),
                at_y + THE_PICTURES_START_AT,
            )),
            Size::from((A_PICTURE_IS, A_PICTURE_IS)),
        ),
    };
    let mut rows = vec![
        a_picture(0, THE_FIRST_CHOICE),
        a_picture(1, maximise),
        a_picture(2, WhatARowDoes::Right),
        MenuRow {
            does: full_screen,
            row: Rectangle::new(
                Point::from((at_x + THE_INSET, at_y + THE_ROWS_START_AT)),
                Size::from((A_ROW_IS_WIDE, A_ROW_IS_TALL)),
            ),
        },
    ];
    let before_alo = how.alo_may_help.then(|| {
        rows.push(MenuRow {
            does: WhatARowDoes::HelpWithThisWindow,
            row: Rectangle::new(
                Point::from((at_x + THE_INSET, at_y + THE_ALO_ROW_IS_AT)),
                Size::from((A_ROW_IS_WIDE, THE_ALO_ROW_IS_TALL)),
            ),
        });
        Rectangle::new(
            Point::from((at_x + THE_INSET, at_y + THE_DIVIDER_TO_ALO_IS_AT)),
            Size::from((A_ROW_IS_WIDE, 1)),
        )
    });
    Some(WindowMenu {
        panel,
        heading: Point::from((at_x + THE_INSET, at_y + THE_HEADING_SITS_AT)),
        divider: Rectangle::new(
            Point::from((at_x + THE_INSET, at_y + THE_DIVIDER_IS_AT)),
            Size::from((A_ROW_IS_WIDE, 1)),
        ),
        before_alo,
        rows,
    })
}

#[cfg(test)]
#[path = "window_menu_tests.rs"]
mod tests;
