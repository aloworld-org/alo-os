//! **The logical measures of a window's miniature in the right-hand panel**,
//! taken off the owner's approved design `10 / Minimised windows, Mac-style`
//! (`27:2` in `XbSmsBYix0TrcQ0Qzaup64`, 2026-11-12) and recorded in
//! `docs/design/the-windows-put-aside.md`.
//!
//! Every figure is **logical**, inside a 48-square slot (`alo_dock::measures::ICON`),
//! and reaches a display as a share of the slot the panel laid out — which is
//! already through the display's scale and the person's text size — so none of
//! them is a pixel count on any screen.

/// The miniature's left inset inside the slot, `27:10`.
pub(crate) const MINIATURE_LEFT: u32 = 2;
/// The miniature's top inset inside the slot, `27:10`.
pub(crate) const MINIATURE_TOP: u32 = 8;
/// The miniature's width, `27:10`.
pub(crate) const MINIATURE_WIDE: u32 = 44;
/// The miniature's height, `27:10`.
pub(crate) const MINIATURE_TALL: u32 = 32;
/// How round the miniature's corners are, `27:10`.
pub(crate) const MINIATURE_ROUNDING: u32 = 4;
/// The miniature's border, `27:10`.
pub(crate) const MINIATURE_BORDER: u32 = 1;

/// Where the application's badge starts, across and down, `27:16`.
pub(crate) const BADGE_AT: u32 = 30;
/// The badge's side, `27:16`.
pub(crate) const BADGE: u32 = 18;
/// How round the badge's corners are, `27:16` (5.04 drawn).
pub(crate) const BADGE_ROUNDING: u32 = 5;
/// The badge's white border, in **halves** of a logical unit: 1.5 drawn.
pub(crate) const BADGE_BORDER_HALVES: u32 = 3;
/// The badge's letter, for an application with no icon of its own, `27:17`.
pub(crate) const BADGE_LETTER: u32 = 9;

/// The name's pill: its padding across, `27:39`.
pub(crate) const NAME_ACROSS: u32 = 10;
/// The name's pill: its padding down, `27:39`.
pub(crate) const NAME_DOWN: u32 = 6;
/// The name's pill: how round its corners are, `27:39`.
pub(crate) const NAME_ROUNDING: u32 = 8;
/// The name's words, 12 SemiBold, `27:40`.
pub(crate) const NAME_TEXT: u32 = 12;
/// How far inward of the rail the name sits, `27:39` at 130 beside a rail
/// ending at 120.
pub(crate) const NAME_FROM_THE_RAIL: u32 = 10;
