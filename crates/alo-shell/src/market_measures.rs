//! Where everything in the window of what a person can install goes, in
//! logical pixels before any scale is applied.
//!
//! **One file for the numbers**, so the drawing reads as shapes and the
//! arithmetic can be checked without reading a rasteriser. `alo_tasks::measures`
//! and `alo_dock::measures` are the same idea in the crates that own those
//! surfaces; this market has no crate of its own, so its numbers live beside
//! its drawing the way `crate::sign_in_raster`'s do.
//!
//! # These are the design's numbers, not invented ones
//!
//! Measured from the approved states on page *03 — alo OS · Living canvas*,
//! built 2026-10-17. Where a number is one of the design file's own variables
//! it is named as such, because a reader checking this against Figma should be
//! able to find the variable rather than a pixel count that happens to match.
//!
//! | here | the design's name | value |
//! |---|---|---|
//! | [`THE_WINDOW_IS_WIDE`] | the widest window this design has | 632 |
//! | [`A_WELLS_CORNER`] | `radius/md` | 12 |
//! | [`A_TILES_CORNER`] | `radius/sm` | 8 |
//! | [`BESIDE`], [`ABOVE`] | `space/6` | 24 |
//! | [`A_GAP`] | `space/3` | 12 |
//!
//! # The window's corner is the Dock's, not the design's
//!
//! The design asks `radius/xl`, 28. **It is drawn with 18**, because
//! `crate::dock_surface::a_panel` takes no radius — it uses
//! `alo_dock::measures::THE_DOCKS_CORNER` for every panel it draws, and this
//! window is its third caller.
//!
//! That is a gap rather than a decision, and it is written here rather than
//! fixed because making the radius a parameter widens a function the Dock, the
//! Ask alo window and this one all draw through. Its own change, with its own
//! callers to check. There is deliberately no constant for 28 below: a number
//! nothing reads is not a record, and this paragraph is.
//!
//! # Why the window is 632 and not 640
//!
//! It was 640 for an afternoon, and the title bar settled it: `Canvas / Window
//! title` is a component whose three controls sit at fixed offsets inside a
//! 632-wide bar. Matching that width puts minimise, focus and close exactly
//! where every other window keeps them, and 640 put them eight pixels adrift.
//! 632 is also the widest window anywhere in that design file, so this window
//! sets no new maximum.

/// How wide the window is.
pub(crate) const THE_WINDOW_IS_WIDE: u32 = 632;

/// How tall the title bar is — the height of `Canvas / Window title`.
pub(crate) const THE_TITLE_BAR_IS_TALL: u32 = 48;

/// The corner of a well — the search field and every application's card,
/// `radius/md`.
pub(crate) const A_WELLS_CORNER: u32 = 12;

/// The corner of an artwork tile, `radius/sm`.
pub(crate) const A_TILES_CORNER: u32 = 8;

/// The corner of a chip or a button: larger than either half, so the rasteriser
/// clamps it to a capsule.
///
/// **Deliberately not `radius/full`'s 999.** The design's variable is a number
/// that means *round*, and `AWell::corner`'s own rule is that a radius larger
/// than the shape becomes a capsule rather than nothing — so any number past
/// half the height does the same thing. This one is small enough to read as a
/// measurement rather than a sentinel.
pub(crate) const A_CAPSULE: u32 = 64;

/// The margin down each side of the body, `space/6`.
pub(crate) const BESIDE: u32 = 24;

/// The margin under the title bar and above the bottom, `space/6`.
pub(crate) const ABOVE: u32 = 24;

/// The ordinary gap between two things, `space/3`.
pub(crate) const A_GAP: u32 = 12;

/// The gap between two category chips, `space/1` and a half — smaller than
/// [`A_GAP`] because a row of chips reads as one control.
pub(crate) const BETWEEN_CHIPS: u32 = 6;

/// How tall the search field is.
pub(crate) const THE_SEARCH_IS_TALL: u32 = 44;

/// How tall one category chip is.
pub(crate) const A_CHIP_IS_TALL: u32 = 29;

/// How tall one application's card is.
///
/// **Fixed rather than hugging its text.** Two cards side by side must be the
/// same height or the row reads as broken, and the alternative — measuring
/// every blurb and taking the tallest — makes the layout depend on the
/// vocabulary, so a long translation would change the shape of the window.
pub(crate) const A_CARD_IS_TALL: u32 = 124;

/// How many cards sit side by side.
pub(crate) const CARDS_ACROSS: u32 = 2;

/// The padding inside one card.
pub(crate) const INSIDE_A_CARD: u32 = 16;

/// How wide and tall an artwork tile is.
pub(crate) const A_TILE_IS: u32 = 36;

/// How tall the line under the list is.
pub(crate) const THE_PROMISE_IS_TALL: u32 = 18;

/// How wide one card is, from the window's width and the margins.
///
/// Two cards, one gap, two margins. Returns zero for a window too narrow to
/// hold them, which [`crate::market_raster`] treats as *do not draw* rather
/// than drawing something negative.
pub(crate) const fn a_card_is_wide() -> u32 {
    let room = THE_WINDOW_IS_WIDE.saturating_sub(2 * BESIDE);
    let gaps = A_GAP * (CARDS_ACROSS - 1);
    let left = room.saturating_sub(gaps);
    left / CARDS_ACROSS
}

/// How wide the body is, inside both margins.
pub(crate) const fn the_body_is_wide() -> u32 {
    THE_WINDOW_IS_WIDE.saturating_sub(2 * BESIDE)
}

/// How tall a window holding this many rows of cards is.
///
/// Title bar, the search field, the chips, the rows with gaps between them,
/// the promise, and the margins above and below.
pub(crate) const fn the_window_is_tall(rows: u32) -> u32 {
    let cards = if rows == 0 {
        0
    } else {
        rows * A_CARD_IS_TALL + (rows - 1) * A_GAP
    };
    THE_TITLE_BAR_IS_TALL
        + ABOVE
        + THE_SEARCH_IS_TALL
        + A_GAP
        + A_CHIP_IS_TALL
        + A_GAP
        + cards
        + A_GAP
        + THE_PROMISE_IS_TALL
        + ABOVE
}

// **These hold at compile time rather than at test time**, because every value
// in them is a constant — and an `assert!` over constants is something the
// compiler already knows. Clippy named them and is right: such a test cannot
// fail while the build succeeds, so running it proves nothing compiling did
// not. A `const` block refuses the build instead, which is the earliest
// anything here can be wrong.

/// **The capsule radius exceeds half of anything it is used on**, which is what
/// makes `AWell` clamp it to a capsule rather than drawing a corner.
const _: () = assert!(A_CAPSULE > A_CHIP_IS_TALL / 2);
const _: () = assert!(A_CAPSULE > THE_SEARCH_IS_TALL / 2);

/// The artwork tile fits inside a card's padding with room for its text beside
/// it.
const _: () = assert!(A_TILE_IS + 2 * INSIDE_A_CARD < a_card_is_wide());
const _: () = assert!(A_TILE_IS + 2 * INSIDE_A_CARD < A_CARD_IS_TALL);

#[cfg(test)]
mod tests {
    use super::*;

    /// **Two cards and a gap fill the body exactly**, so neither a stray pixel
    /// down the right-hand side nor a card running under the margin.
    #[test]
    fn two_cards_and_a_gap_are_the_body() {
        let cards = CARDS_ACROSS * a_card_is_wide() + A_GAP * (CARDS_ACROSS - 1);
        // Integer division can leave at most one pixel per card unassigned.
        let short = the_body_is_wide() - cards;
        assert!(short < CARDS_ACROSS, "{short} pixels unaccounted for");
    }

    /// A window of no rows is still a window: the title bar, the search field,
    /// the chips and the promise are there whatever the list holds.
    #[test]
    fn a_window_with_no_rows_still_has_its_furniture() {
        let bare = the_window_is_tall(0);
        assert!(bare > THE_TITLE_BAR_IS_TALL + THE_SEARCH_IS_TALL);
        assert!(bare < the_window_is_tall(1));
    }

    /// Each row adds exactly one card and one gap, so a window's height is
    /// readable from the number of rows rather than from a table.
    #[test]
    fn each_row_adds_a_card_and_a_gap() {
        for rows in 1..6_u32 {
            assert_eq!(
                the_window_is_tall(rows + 1) - the_window_is_tall(rows),
                A_CARD_IS_TALL + A_GAP,
                "{rows}"
            );
        }
    }
}
