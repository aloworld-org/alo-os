//! Every figure the Settings window is drawn with, as a **logical** measure,
//! declared once with the frame it was taken off.
//!
//! Measured on 2026-10-31 from the owner's approved redesign, file
//! `nDxyF5Ho9oC4RObjVzwBNJ`, page `544:30910`; `docs/design/the-settings-window.md`
//! holds the frames, the sums that were checked and what the export did not
//! carry. Nothing here reaches a display without going through the person's
//! text size and that display's scale together (`crate::desktop_look::Measure`),
//! so larger text makes the controls larger with it — which is the
//! Accessibility page's own footnote, *larger text makes controls larger with
//! it, so they stay easy to reach*.
//!
//! The window's own size is not here: it is a **rule** of the display it is
//! on, in `crate::settings_frame`.

/// The window at its largest, across. Frame `550:98`, 1200 × 816.
pub(crate) const THE_WINDOW_AT_MOST_WIDE: u32 = 1200;
/// The window at its largest, down.
pub(crate) const THE_WINDOW_AT_MOST_TALL: u32 = 816;
/// The least room either side of the window. `550:420` at 1280 × 800 is 1152
/// wide, which is 1280 less 64 each side.
pub(crate) const BESIDE_THE_WINDOW: u32 = 64;
/// The least room above and below it. `550:420` is 688 tall: 800 less 56 each
/// way.
pub(crate) const ABOVE_THE_WINDOW: u32 = 56;
/// The window's corner, `radius/lg`.
pub(crate) const THE_WINDOWS_CORNER: u32 = 18;
/// Every hairline: the window's border, the side column's edge, a card's border
/// and the line between two rows.
pub(crate) const A_HAIRLINE: u32 = 1;

/// The side column, as a share of the window: 280 of 1200 in `550:99`, and
/// 269 (+1 for its edge) of 1152 in `550:421`. Both are seven thirtieths.
pub(crate) const THE_SIDE_COLUMN_IN_THIRTIETHS: u32 = 7;
/// The side column's padding, on every side.
pub(crate) const INSIDE_THE_SIDE_COLUMN: u32 = 16;
/// Between two things in the side column.
pub(crate) const BETWEEN_SIDE_ITEMS: u32 = 2;
/// The heading block's own padding.
pub(crate) const INSIDE_THE_HEADING: u32 = 12;
/// Between the title and the line under it.
pub(crate) const UNDER_THE_TITLE: u32 = 4;
/// A group's label: room above it.
pub(crate) const ABOVE_A_GROUP: u32 = 14;
/// A group's label: room below it.
pub(crate) const BELOW_A_GROUP: u32 = 6;
/// A category's row, and every other target in the window: 48.
pub(crate) const A_TARGET: u32 = 48;
/// A category's padding across, and a group label's.
pub(crate) const BESIDE_A_CATEGORY: u32 = 12;
/// Between a category's icon and its name.
pub(crate) const AFTER_A_CATEGORYS_ICON: u32 = 12;
/// A category's icon.
pub(crate) const A_CATEGORYS_ICON: u32 = 20;
/// The corner of a selected category, a lead icon's well and a risk box.
pub(crate) const A_SMALL_CORNER: u32 = 10;
/// The footer's room below its line.
pub(crate) const BELOW_THE_FOOTER: u32 = 8;

/// The content's padding across, at the window's full width. `550:154`.
pub(crate) const BESIDE_THE_CONTENT: u32 = 64;
/// The content's padding above, at the window's full width.
pub(crate) const ABOVE_THE_CONTENT: u32 = 48;
/// The content's padding across, once the window is narrower than its
/// largest. `550:476` at 1280 × 800.
pub(crate) const BESIDE_THE_CONTENT_WHEN_NARROWER: u32 = 48;
/// The content's padding above, once the window is narrower than its largest.
pub(crate) const ABOVE_THE_CONTENT_WHEN_NARROWER: u32 = 36;
/// The column the content is written in, at its widest: *Column · at most 720
/// wide*.
pub(crate) const THE_COLUMN_AT_MOST: u32 = 720;
/// Between the page's heading and its introduction.
pub(crate) const UNDER_THE_PAGE_HEADING: u32 = 8;
/// Between the page's blocks on the alo and AI page (`550:155`). The other
/// pages draw 36 or 28; [`between_blocks_on`] has each.
pub(crate) const BETWEEN_BLOCKS_ON_ALO_AND_AI: u32 = 32;
/// Between a section's heading, its card and its footnote.
pub(crate) const INSIDE_A_SECTION: u32 = 10;
/// A section heading's and a footnote's room across.
pub(crate) const BESIDE_A_SECTIONS_WORDS: u32 = 4;
/// A card's corner.
pub(crate) const A_CARDS_CORNER: u32 = 12;
/// A row's least height.
pub(crate) const A_ROW_AT_LEAST: u32 = 64;
/// A row's padding across.
pub(crate) const BESIDE_A_ROW: u32 = 20;
/// A row's padding above and below.
pub(crate) const ABOVE_A_ROW: u32 = 16;
/// Between a row's mark and its words, and its words and its control.
pub(crate) const INSIDE_A_ROW: u32 = 16;
/// Between a row's title and the line under it.
pub(crate) const UNDER_A_ROWS_TITLE: u32 = 4;
/// A choice's circle, on a row of its own.
pub(crate) const A_CHOICE: u32 = 22;
/// A choice's circle, on a row under another (`550:192`).
pub(crate) const A_NESTED_CHOICE: u32 = 18;
/// A row under another begins this far in: under its parent's words.
pub(crate) const A_NESTED_ROW_IN: u32 = 58;
/// A row under another: padding above and below.
pub(crate) const ABOVE_A_NESTED_ROW: u32 = 12;
/// A row under another: between its mark and its words.
pub(crate) const INSIDE_A_NESTED_ROW: u32 = 12;
/// The row that opens something, such as *Add a provider*: its least height.
pub(crate) const AN_OPENING_ROW_AT_LEAST: u32 = 56;
/// The chevron at the end of a row that opens something.
pub(crate) const A_CHEVRON: u32 = 20;
/// The small icon before a note under a row's words.
pub(crate) const A_NOTES_ICON: u32 = 14;
/// Between that icon and the note.
pub(crate) const AFTER_A_NOTES_ICON: u32 = 6;
/// A Quiet button's padding either side of its label: *Say where it runs* is
/// 174 across and *Take back* 115.
pub(crate) const BESIDE_A_BUTTONS_LABEL: u32 = 26;
/// A switch's track, across.
pub(crate) const A_SWITCH_IS_WIDE: u32 = 44;
/// A switch's track, down.
pub(crate) const A_SWITCH_IS_TALL: u32 = 26;
/// Between a switch's word and its track.
pub(crate) const BEFORE_A_SWITCH: u32 = 10;
/// A switch's knob, its radius.
pub(crate) const A_KNOB: u32 = 10;
/// A risk box's padding across.
pub(crate) const BESIDE_A_RISK: u32 = 12;
/// A risk box's padding above and below.
pub(crate) const ABOVE_A_RISK: u32 = 10;
/// A risk box's warning sign.
pub(crate) const A_RISKS_ICON: u32 = 18;
/// Between the sign and the words.
pub(crate) const AFTER_A_RISKS_ICON: u32 = 10;
/// Between the risk's heading and its sentence.
pub(crate) const UNDER_A_RISKS_HEADING: u32 = 2;
/// The keyboard's outline: how thick, and how far it stands off what it marks.
pub(crate) const THE_FOCUS_RING: u32 = 2;
/// How far the keyboard's outline stands outside what it marks.
pub(crate) const THE_FOCUS_RING_STANDS_OFF: u32 = 2;

/// A lead icon's well, square (`550:820`).
pub(crate) const A_LEADS_WELL: u32 = 36;
/// A key's cap: inside it across (`551:945`).
pub(crate) const INSIDE_A_KEY: u32 = 9;
/// A key's cap, down.
pub(crate) const A_KEY_IS_TALL: u32 = 30;
/// A key's cap's corner.
pub(crate) const A_KEYS_CORNER: u32 = 7;
/// Between two keys.
pub(crate) const BETWEEN_KEYS: u32 = 4;
/// The lock before *Always on* (`550:991`).
pub(crate) const A_LOCK: u32 = 16;
/// A segmented choice's box: inside it, and between its parts (`551:291`).
pub(crate) const INSIDE_A_SEGMENTED: u32 = 4;
/// A part of a segmented choice: inside it across.
pub(crate) const INSIDE_A_SEGMENT: u32 = 16;
/// The chosen part: inside it across.
pub(crate) const INSIDE_A_CHOSEN_SEGMENT: u32 = 14;
/// The chosen part's check.
pub(crate) const A_SEGMENTS_CHECK: u32 = 16;
/// Between the check and the label.
pub(crate) const AFTER_A_SEGMENTS_CHECK: u32 = 6;
/// A part's corner.
pub(crate) const A_SEGMENTS_CORNER: u32 = 9;
/// An accent colour, across, with its name under it (`551:308`).
pub(crate) const A_SWATCH_IS_WIDE: u32 = 72;
/// Between two accent colours.
pub(crate) const BETWEEN_SWATCHES: u32 = 16;
/// An accent colour's disc, its radius.
pub(crate) const A_SWATCH: u32 = 17;
/// The ring round the chosen colour, its radius.
pub(crate) const A_CHOSEN_SWATCHS_RING: u32 = 21;
/// The check on the chosen colour.
pub(crate) const A_SWATCHS_CHECK: u32 = 18;
/// Between a colour and its name.
pub(crate) const UNDER_A_SWATCH: u32 = 6;
/// Above the colours, under the row's line.
pub(crate) const ABOVE_THE_SWATCHES: u32 = 8;

/// Between a page's blocks, per page, as each frame draws it.
///
/// **Three figures for one relationship**, and recorded rather than tidied:
/// alo and AI draws 32, Appearance and Keyboard 28, and the other five 36.
/// `docs/design/the-settings-window.md` names this as a question for the
/// designer; until it is answered, each page is drawn as its own frame is.
pub(crate) const fn between_blocks_on(category: crate::SettingsCategory) -> u32 {
    use crate::SettingsCategory as C;
    match category {
        C::AloAndAi => BETWEEN_BLOCKS_ON_ALO_AND_AI,
        C::Appearance | C::KeyboardAndShortcuts => 28,
        C::FoldersAloCanUse
        | C::RunningCode
        | C::Terminal
        | C::RegionAndTime
        | C::Accessibility => 36,
    }
}
