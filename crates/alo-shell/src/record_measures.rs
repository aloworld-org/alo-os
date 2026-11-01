//! The record window's measures, read off the frames the owner approved on
//! 2026-11-01 (`560:142` at 1440 × 960, `560:316` at 1280 × 800, `560:490`
//! dark) and recorded in `docs/design/the-record-window.md`.
//!
//! Every figure is a **logical** measure: it goes through the person's text
//! size and the display's scale together (`crate::desktop_look::Measure`) at
//! the boundary, so a 200 per cent screen draws the same window sharper. The
//! window's own size is **a rule of the display**, not a figure — see
//! [`ABOVE_THE_FOOT`] and [`TALLEST`].

/// The window's width where the display has room: `560:175`, at both sizes.
pub(crate) const WIDEST: i32 = 880;
/// The least room either side of the window on a display too narrow for
/// [`WIDEST`].
pub(crate) const SIDE_ROOM: i32 = 16;
/// The window's height where the display has room: `560:175`'s 796.
pub(crate) const TALLEST: i32 = 796;
/// **How far the window's foot stays above the display's**: 108 in both
/// frames — the Dock's 72 and its 12 off the edge, and 24 between the two —
/// so the window never covers the Dock. The window is as tall as
/// [`TALLEST`] or the room above that allows, and its top is what is left:
/// 56 at 960 and 40 at 800, exactly as the two frames draw it.
pub(crate) const ABOVE_THE_FOOT: i32 = 108;
/// The least room above the window, which is what the 800-tall frame leaves.
pub(crate) const ABOVE_THE_TOP: i32 = 40;

/// The title strip's band, with the targets that reach above what is drawn.
pub(crate) const THE_STRIP_BAND: i32 = 44;
/// Where the strip that is drawn starts in its band, and how tall it is.
pub(crate) const THE_STRIP: (i32, i32) = (12, 32);
/// How round the strip's top corners are (`554:25`).
pub(crate) const THE_STRIPS_CORNER: i32 = 10;
/// The window's icon in the strip: a 16 square at (14, 20), on a tile at
/// corner 4.
pub(crate) const THE_ICON: (i32, i32, i32, i32) = (14, 20, 16, 4);
/// Where the title starts in the strip's band.
pub(crate) const THE_TITLE: (i32, i32) = (38, 19);
/// Close: a 44 square target ending 4 from the window's edge, its 14 mark at
/// (15, 21) inside it.
pub(crate) const CLOSE: (i32, i32, (i32, i32), i32) = (44, 4, (15, 21), 14);

/// How round the body is.
pub(crate) const THE_BODYS_CORNER: i32 = 18;
/// The body's inside room, across: everything starts 40 in.
pub(crate) const PAD: i32 = 40;
/// The heading's top.
pub(crate) const HEADING_AT: i32 = 32;
/// Between the heading and the sentence under it.
pub(crate) const AFTER_THE_HEADING: i32 = 8;
/// How wide the sentence under the heading may run.
pub(crate) const THE_INTRODUCTION_IS: i32 = 640;
/// Between the sentence and today's counts.
pub(crate) const BEFORE_THE_COUNTS: i32 = 10;
/// Today's counts' row: as tall as a count.
pub(crate) const A_COUNT_IS: i32 = 28;
/// Between *Today* and the first count, and between counts.
pub(crate) const BETWEEN_COUNTS: i32 = 8;
/// A count's inside: its icon 10 in, its words 30 in, 12 to spare after.
pub(crate) const IN_A_COUNT: (i32, i32, i32) = (10, 30, 12);
/// Between the counts and the rule under them.
pub(crate) const BEFORE_THE_RULE: i32 = 20;

/// The list's room inside its top and foot.
pub(crate) const THE_LIST_PADS: (i32, i32) = (20, 24);
/// A day's heading: 18 tall, set 4 in, the date 8 after the day.
pub(crate) const A_DAY: (i32, i32, i32) = (18, 4, 8);
/// Between a day's heading and its card.
pub(crate) const DAY_TO_CARD: i32 = 10;
/// Between one day's card and the next day's heading: the list's 10, the
/// design's 6 spacer and 10 again.
pub(crate) const BETWEEN_DAYS: i32 = 26;
/// How round a day's card is.
pub(crate) const A_CARDS_CORNER: i32 = 12;

/// An entry's inside: 16 above and below, 20 either side, 16 between
/// columns.
pub(crate) const IN_AN_ENTRY: (i32, i32, i32) = (16, 20, 16);
/// The time column's least width; a wider time widens it for every entry.
pub(crate) const THE_TIME_COLUMN: i32 = 44;
/// The icon's tile: 32 square at corner 8, its icon 16 in the middle.
pub(crate) const THE_TILE: (i32, i32, i32) = (32, 8, 16);
/// The head row: 22 tall; the agent's mark 12 at 5 down, 6 before the name
/// and 6 after it.
pub(crate) const THE_HEAD: (i32, i32, i32) = (22, 12, 6);
/// Between the lines of what happened.
pub(crate) const BETWEEN_LINES: i32 = 4;
/// The row of marks under them: 4 above, 16 tall, 12 between two, a 14 icon
/// 6 before its words.
pub(crate) const THE_MARKS: (i32, i32, i32, i32, i32) = (4, 16, 12, 14, 6);

/// The footer: 52 tall, its 14 icon at (40, 19), its words at (62, 18).
pub(crate) const THE_FOOTER: (i32, i32, i32, i32) = (52, 19, 14, 62);
/// The rail that says where the view is, when anything is out of it: 4 wide,
/// 14 in from the body's trailing edge.
pub(crate) const THE_RAIL: (i32, i32) = (4, 14);
