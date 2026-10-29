//! Where the window of what a person can install goes on a display.
//!
//! **Centred, and the reason is that there is exactly one of them.**
//! `crate::where_a_window_opens` is the owner's specification and its rules are
//! about a window arriving *beside the active one* with offsets for repeated
//! openings — and neither applies here: one seat has one of these windows, so
//! there is never a second to sit beside and never a second opening to offset
//! from. A window a person opened by pressing an icon belongs where their eyes
//! already are.
//!
//! This follows `crate::where_a_task_window_opens` in every other respect, and
//! that file's two notes hold unchanged:
//!
//! - **Screen space, not the canvas.** The three shell windows that came before
//!   — what is running, what is filling the disk, a task — are laid out and
//!   painted against the display. `crate::market_raster` draws into a frame at
//!   physical pixels, and a surface on the plane a person pans would need the
//!   camera applied to its texture.
//! - **Whether it ought to live on the canvas instead is a real question and is
//!   not answered here.** The owner has not been asked.
//!
//! # Its size is the design's, and its height follows the list
//!
//! `crate::market_measures` holds every number. The width is fixed at 632 — the
//! widest window in the design file — and the height is
//! [`crate::market_measures::the_window_is_tall`] of however many rows of cards
//! the list needs. So narrowing to *Painting* shrinks the window rather than
//! leaving a field of empty surface, which is what the design draws.
//!
//! # Placed every frame rather than kept
//!
//! A task window's position is **kept** once it has one, because
//! `docs/design/where-a-new-window-opens.md` says *later redraws must not
//! unexpectedly reposition it* and a person drags a task window about. Nothing
//! moves this one: it has no edge to drag by yet, it is centred rather than
//! placed beside anything, and its height is a function of what the list shows.
//! Computing it per frame is therefore the same answer every frame **until the
//! person narrows the list**, which is exactly when it should change.
//!
//! The moment that stops being true is the moment a person can move this
//! window, and then its place belongs beside the window like a task's does. It
//! is written here so that whoever adds that can see what they are changing.
//!
//! # A tall window on a short screen is cut, and this is where it was found
//!
//! The whole catalogue needs 755 logical pixels of height. At the largest text
//! size a person may choose — `alo_appearance`'s `LARGEST` is **300 per cent** —
//! that is 2265 physical pixels, which is **taller than a 4K display**. The
//! first version of this file refused such a window, and a test asking whether
//! somebody who reads larger gets a larger window is what found it.
//!
//! **Refusing was the wrong answer.** A person who reads at 300 per cent would
//! have had no market at all, on any display sold today, while everybody else
//! had one. So the height is cut to the display and the window opens.
//!
//! `crate::market_raster` already draws a cut window correctly: *a card that
//! would reach past the promise is not drawn, and neither is any after it —
//! half a card says something the list does not.* So a cut window shows whole
//! cards and its own closing line, and nothing is drawn off the edge.
//!
//! ## What a cut window costs, stated plainly
//!
//! **The applications past the cut cannot be reached yet.** Nothing scrolls:
//! `MarketWindow`'s cursor walks the whole list and the raster always begins at
//! the first row, so pressing Down past the last visible card moves a selection
//! a person cannot see. On an ordinary display at an ordinary text size the
//! whole catalogue fits and none of this happens; it bites at large text sizes
//! and on short screens, which is exactly where accessibility is already
//! hardest.
//!
//! **That is the next change on this window** and it is a real one: the raster
//! has to draw from a first row rather than from the top, and the cursor has to
//! carry that row with it. It is written down rather than half-built, because a
//! window that scrolls badly is harder to find than one that says it does not
//! scroll.

use alo_appearance::TextScale;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::market_measures as m;
use crate::market_window::MarketShows;

/// Where the window goes on this display, at this text size and this
/// display's scale — the two multiplied, as every measure in the market is
/// (`crate::market_raster`), so on a 200 per cent screen it asks for twice the
/// pixels.
///
/// Answers [`None`] for a shut window — which is what a display with no market
/// on it is — and for a display too small to hold one. The second is a true
/// answer about a very small screen rather than a failure: there is no position
/// at which this window fits there, and `crate::market_raster::picture` would
/// refuse it anyway, so refusing here says so one step earlier and in the file
/// that knows about displays.
#[must_use]
pub(crate) fn where_the_market_opens(
    view: Rectangle<i32, Physical>,
    showing: &MarketShows<'_>,
    scale: TextScale,
    display: DisplayScale,
) -> Option<Rectangle<i32, Physical>> {
    let MarketShows::Browsing(browsing) = showing else {
        return None;
    };
    let measure = Measure::of(scale).on(display);
    // **Two cards to a row, and the last row may hold one.** `div_ceil` rather
    // than a division and a remainder, so a list of nine does not lose its
    // ninth — the fault a plain `/ 2` makes, and it loses exactly the last
    // application in the catalogue.
    let rows = u32::try_from(browsing.how_many().div_ceil(m::CARDS_ACROSS as usize)).unwrap_or(0);
    let wide = measure.px(i32::try_from(m::THE_WINDOW_IS_WIDE).unwrap_or(i32::MAX));
    let tall = measure.px(i32::try_from(m::the_window_is_tall(rows)).unwrap_or(i32::MAX));
    // **The height is cut to the display; the width is not.** Cutting the width
    // would narrow the cards, and `market_measures`' own assertions hold a card
    // wide enough for its artwork tile and its words beside it — a narrower one
    // is a layout nothing has measured. A window too wide for the display is
    // therefore refused rather than squeezed, and this file's header says what
    // cutting the height costs.
    let wanted = Size::from((wide, tall.min(view.size.h)));
    if wanted.w > view.size.w {
        return None;
    }
    // **And a window with no room for even its furniture is refused.** The
    // title bar, the search field, the chips and the closing line are what a
    // person needs before any card; a window shorter than those is one
    // `crate::market_raster` refuses anyway, so saying so here refuses it one
    // step earlier and in the file that knows about displays.
    if wanted.h < measure.px(i32::try_from(m::the_window_is_tall(0)).unwrap_or(i32::MAX)) {
        return None;
    }
    // **Integer division, and the leftover pixel goes below and to the right.**
    // A window one pixel off centre on an odd-width display is not something a
    // person can see; rounding it away with floating point would be arithmetic
    // nobody can check against the design's own numbers.
    let at = Point::from((
        view.loc.x + (view.size.w - wanted.w) / 2,
        view.loc.y + (view.size.h - wanted.h) / 2,
    ));
    Some(Rectangle::new(at, wanted))
}

#[cfg(test)]
#[path = "where_the_market_opens_tests.rs"]
mod tests;
