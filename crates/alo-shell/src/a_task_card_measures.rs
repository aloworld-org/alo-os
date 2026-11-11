//! The logical measures of a task's square and card in the right-hand panel.
//!
//! Every figure here was taken off the owner's approved frame `23:2`, *07 / A
//! running task in the put-aside panel* (approved as drawn, 2026-11-11), and is
//! recorded with the frame's node ids in `docs/design/the-windows-put-aside.md`.
//! They are **logical**: `crate::task_card_raster` converts each one through
//! the person's text size and the display's scale before it reaches a pixel.
//! The square itself is `alo_dock::measures::ICON` and is not repeated here.

/// The card's width (`23:29`).
pub(crate) const CARD_WIDE: u32 = 216;
/// The card's height.
pub(crate) const CARD_TALL: u32 = 106;
/// The card's corner, and the square's.
pub(crate) const CORNER: u32 = 12;
/// The coloured edge down the card's left side, which carries the state.
pub(crate) const EDGE: u32 = 3;
/// From the card's left to its words.
pub(crate) const INSET: u32 = 12;
/// From the card's left to the state word beside alo's mark.
pub(crate) const STATE_FROM_LEFT: u32 = 36;
/// The top of each of the card's four lines.
pub(crate) const LINE_TOPS: [u32; 4] = [10, 32, 54, 78];
/// How tall each line's box is: 16, 18, 16, 16.
pub(crate) const LINE_TALL: [u32; 4] = [16, 18, 16, 16];
/// The type size of each line: 11, 13, 11, 11.
pub(crate) const LINE_TYPE: [u32; 4] = [11, 13, 11, 11];
/// How wide the state word may run.
pub(crate) const STATE_WIDE: u32 = 160;
/// How wide lines two to four may run with no button beside them.
pub(crate) const LINE_WIDE: u32 = 192;
/// How wide lines three and four may run beside the button.
pub(crate) const LINE_BESIDE_A_BUTTON: u32 = 124;
/// The button's width.
pub(crate) const BUTTON_WIDE: u32 = 64;
/// The button's height.
pub(crate) const BUTTON_TALL: u32 = 30;
/// The button's corner.
pub(crate) const BUTTON_CORNER: u32 = 8;
/// From the card's right edge to the button's.
pub(crate) const BUTTON_FROM_RIGHT: u32 = 12;
/// From the card's bottom edge to the button's.
pub(crate) const BUTTON_FROM_BOTTOM: u32 = 10;
/// The button's type size.
pub(crate) const BUTTON_TYPE: u32 = 12;
/// The ring round a square for a task alo is working on or waiting in.
pub(crate) const STRONG_RING: u32 = 2;
/// The ring round any other square, solid or dashed.
pub(crate) const QUIET_RING: u32 = 1;
/// The dot that says a task needs the person (`23:16`).
pub(crate) const DOT: u32 = 10;
/// The dot's top-left inside the square.
pub(crate) const DOT_AT: (u32, u32) = (34, 4);
/// alo's mark in a square: its type size.
pub(crate) const MARK_TYPE: u32 = 13;
/// How long each dash of a dashed ring is, and the gap after it.
pub(crate) const DASH: u32 = 4;
/// The card's shadow: falls 2, blurs 8, at six per cent (`0 2 8 #0F2942 6%`).
pub(crate) const SHADOW_FALLS: u32 = 2;
/// How far the shadow blurs.
pub(crate) const SHADOW_BLURS: u32 = 8;
/// How dark it is: six per cent.
pub(crate) const SHADOW_DARKNESS: f32 = 0.06;
