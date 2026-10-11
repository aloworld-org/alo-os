//! How big *More apps* is, and where everything in it sits.
//!
//! The owner's design of 2026-10-11 — `More apps / Popover`, `406:26881`,
//! captured in `docs/design/figma-snapshot/406-26881.xml`. A 448-wide popover
//! with a fixed heading, a fixed search field, a **scrolling grid** of
//! applications and a fixed line of words under it.
//!
//! # The heights are caps, which is the whole of this file
//!
//! > These are caps, not heights that increase with application count. Sixty
//! > applications must scroll inside the same bounded viewport. A 644px-tall
//! > popover exceeds the specification.
//!
//! That last sentence is about this repository: on 2026-10-11 the old list grew
//! until it ran out of screen, and with sixty applications it drew **644 of
//! 768 pixels**. It was not a bug in the drawing — it was the rule the earlier
//! design had, *grow when space permits, then scroll* — and it is the rule this
//! replaces.
//!
//! So nothing here grows with how many applications there are. The number of
//! applications decides only **whether it scrolls**.
//!
//! # And they shrink on a short output
//!
//! > On shorter screens, reduce the list viewport further so the complete
//! > popover fits the available space.
//!
//! The heading, the search field and the footer are fixed heights and the list
//! is what gives way. [`TheOverview::of`] is handed the room there is and
//! takes the smaller of the cap and what is left after the fixed parts.
//!
//! # Why the grid is here and the drawing is not
//!
//! Where a cell sits is arithmetic over measured figures, and arithmetic can be
//! tested without a screen. Which pixels it is made of cannot. The same split
//! every other layout in this crate uses.

use crate::measures::{
    AN_APP_CELL_IS_TALL, AN_APP_CELL_IS_WIDE, AN_APP_GROUP_HEADING_IS_TALL, BETWEEN_APP_CELLS,
    DOWN_MORE_APPS, INSIDE_MORE_APPS, MORE_APPS_IS_AT_MOST_TALL, MORE_APPS_IS_WIDE,
    THE_APP_LIST_IS_AT_MOST_TALL, THE_DOCKS_EDGE, THE_MORE_APPS_FOOTER_IS_TALL,
    THE_MORE_APPS_HEADING_IS_TALL, THE_SEARCH_FIELD_IS_TALL,
};
use crate::room::Room;

/// Everything in the popover that is not the scrolling list.
///
/// Added up once, because *how tall is the list allowed to be* is this
/// subtracted from the room, and a caller doing the sum itself is a caller
/// that will do it differently.
const FIXED: u32 = THE_DOCKS_EDGE * 2
    + INSIDE_MORE_APPS * 2
    + THE_MORE_APPS_HEADING_IS_TALL
    + THE_SEARCH_FIELD_IS_TALL
    + THE_MORE_APPS_FOOTER_IS_TALL
    + DOWN_MORE_APPS * 3;

// **The design's own numbers add up to the owner's own cap, exactly.**
//
//   1 + 24          border and padding, top          25
//   32              heading                          57
//   16              gap                              73
//   48              search                          121
//   16              gap                             137
//   336             the list, at its cap            473
//   16              gap                             489
//   22              footer                          511
//   24 + 1          padding and border, bottom      536
//
// They were six short until the footer was read as 22 rather than 16 — 16 is
// the glyphs and 22 is the box. This holds the two halves together: the
// measured parts are checked against the stated cap at compile time, so a
// mistyped measurement is a build failure rather than a popover that is
// slightly the wrong size on a screen.
const _: () = assert!(
    FIXED + THE_APP_LIST_IS_AT_MOST_TALL == MORE_APPS_IS_AT_MOST_TALL,
    "the measured parts and the owner's cap do not agree: one of them is wrong"
);

/// One group of applications in the list: *Pinned*, then *Other open apps*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AGroup {
    /// How many applications are in it.
    how_many: usize,
    /// How far down the list's own content it begins.
    at: u32,
    /// How tall it is, heading and rows together.
    tall: u32,
}

impl AGroup {
    /// How many applications are in it.
    #[must_use]
    pub const fn how_many(self) -> usize {
        self.how_many
    }

    /// How far down the list's content it starts.
    #[must_use]
    pub const fn at(self) -> Room {
        Room::pixels(self.at)
    }

    /// How tall it is.
    #[must_use]
    pub const fn tall(self) -> Room {
        Room::pixels(self.tall)
    }
}

/// *More apps*, laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheOverview {
    /// How many columns the grid has.
    columns: usize,
    /// The pinned applications.
    pinned: AGroup,
    /// Everything else that is open.
    others: AGroup,
    /// How tall the scrolling viewport is.
    viewport: u32,
    /// How tall the whole popover is.
    tall: u32,
    /// How far the list has been scrolled.
    scrolled: u32,
}

impl TheOverview {
    /// How many applications the grid puts on one row.
    ///
    /// **Four columns, and fewer when there is not the width for four.** The
    /// design's description: *4 columns at this size, fewer on narrow screens.*
    /// The width is fixed at 448 today, so this is four — it is computed rather
    /// than written down so that a narrower popover gets the right answer
    /// without this file being edited again.
    ///
    /// It takes no arguments because it depends on nothing a caller has: the
    /// popover's width is this crate's own measure and the same on every
    /// display. **It is public so the keyboard can ask it.** An arrow key moves
    /// focus a whole row, which is this number, and the shell's router works
    /// that out before any `TheOverview` exists for the frame — given its own
    /// copy of the arithmetic it would be a second place for *four* to live,
    /// and a popover narrowed to three columns would move the ring four.
    #[must_use]
    pub const fn how_many_across() -> usize {
        let across = MORE_APPS_IS_WIDE.saturating_sub(INSIDE_MORE_APPS * 2);
        let columns =
            ((across + BETWEEN_APP_CELLS) / (AN_APP_CELL_IS_WIDE + BETWEEN_APP_CELLS)) as usize;
        if columns == 0 { 1 } else { columns }
    }

    /// Lay it out for `pinned` pinned applications and `others` others, in
    /// `room` of vertical space.
    ///
    /// `room` is what the display leaves after the Dock and the gap to it —
    /// the caller knows where the bar is and this does not.
    #[must_use]
    pub fn of(pinned: usize, others: usize, room: Room) -> Self {
        let columns = Self::how_many_across();

        let group = |how_many: usize, at: u32| {
            let rows = how_many.div_ceil(columns);
            let tall = if how_many == 0 {
                0
            } else {
                let rows = u32::try_from(rows).unwrap_or(u32::MAX);
                AN_APP_GROUP_HEADING_IS_TALL
                    .saturating_add(BETWEEN_APP_CELLS)
                    .saturating_add(rows.saturating_mul(AN_APP_CELL_IS_TALL))
                    .saturating_add(rows.saturating_sub(1).saturating_mul(BETWEEN_APP_CELLS))
            };
            AGroup { how_many, at, tall }
        };
        let pinned = group(pinned, 0);
        // An empty first group takes no room and no gap: two headings with
        // nothing between them would be a gap a person can see and cannot
        // explain.
        let after = if pinned.tall == 0 {
            0
        } else {
            pinned.tall.saturating_add(BETWEEN_APP_CELLS)
        };
        let others = group(others, after);

        // **The cap, then the room, then the contents** — in that order,
        // because each is a bound the one after it may not exceed.
        let viewport = THE_APP_LIST_IS_AT_MOST_TALL
            .min(room.as_pixels().saturating_sub(FIXED))
            .min(others.at.saturating_add(others.tall).max(pinned.tall));

        Self {
            columns,
            pinned,
            others,
            viewport,
            tall: FIXED.saturating_add(viewport),
            scrolled: 0,
        }
    }

    /// How wide it is, which is a measurement and never changes.
    #[must_use]
    pub const fn width() -> Room {
        Room::pixels(MORE_APPS_IS_WIDE)
    }

    /// How tall it is, heading and footer included.
    ///
    /// **Never more than [`crate::measures::MORE_APPS_IS_AT_MOST_TALL`]**, and
    /// less on an output too short for that.
    #[must_use]
    pub const fn height(self) -> Room {
        Room::pixels(self.tall)
    }

    /// How tall the scrolling list inside it is.
    ///
    /// **Never more than [`crate::measures::THE_APP_LIST_IS_AT_MOST_TALL`].**
    #[must_use]
    pub const fn the_list_is_tall(self) -> Room {
        Room::pixels(self.viewport)
    }

    /// How many columns the grid has.
    #[must_use]
    pub const fn columns(self) -> usize {
        self.columns
    }

    /// The pinned group.
    #[must_use]
    pub const fn pinned(self) -> AGroup {
        self.pinned
    }

    /// Everything else that is open.
    #[must_use]
    pub const fn others(self) -> AGroup {
        self.others
    }

    /// How tall everything in the list is, scrolled or not.
    #[must_use]
    pub const fn the_whole_list_is_tall(self) -> Room {
        Room::pixels(self.others.at + self.others.tall)
    }

    /// Whether there is more than the viewport can hold.
    #[must_use]
    pub const fn scrolls(self) -> bool {
        self.others.at + self.others.tall > self.viewport
    }

    /// How far down the list has been scrolled.
    #[must_use]
    pub const fn scrolled(self) -> Room {
        Room::pixels(self.scrolled)
    }

    /// The most it can be scrolled, which is everything below the viewport.
    #[must_use]
    pub const fn the_furthest_it_scrolls(self) -> Room {
        Room::pixels((self.others.at + self.others.tall).saturating_sub(self.viewport))
    }

    /// Scroll the least it can so that this application is wholly in view.
    ///
    /// The owner's *scroll keyboard-focused items into view*. `which` counts
    /// the pinned applications first and then the others, which is the order a
    /// keyboard moves through them.
    #[must_use]
    pub fn showing(mut self, which: usize) -> Self {
        let Some(top) = self
            .where_a_cell_sits(which)
            .map(|(_, down)| down.as_pixels())
        else {
            return self;
        };
        let bottom = top.saturating_add(AN_APP_CELL_IS_TALL);
        // **A cell on a group's first row brings its heading with it.**
        //
        // Without this, scrolling up to the first application stops with the
        // cell's top exactly at the viewport's top and *Pinned* cut off just
        // above it. That is the least scrolling that puts the cell in view, and
        // it is the wrong answer: a person who moved to the first thing in a
        // group is told which group by a word that is no longer on the screen.
        //
        // Only the first row, and only upward — a heading that followed the
        // focus down the list would be a sticky heading, which is a different
        // thing nobody has asked for.
        let top = if self.is_on_a_groups_first_row(which) {
            top.saturating_sub(AN_APP_GROUP_HEADING_IS_TALL + BETWEEN_APP_CELLS)
        } else {
            top
        };
        if top < self.scrolled {
            self.scrolled = top;
        } else if bottom > self.scrolled.saturating_add(self.viewport) {
            self.scrolled = bottom.saturating_sub(self.viewport);
        }
        self.scrolled = self
            .scrolled
            .min(self.the_furthest_it_scrolls().as_pixels());
        self
    }

    /// Whether this application is on the first row of its group, and so has a
    /// heading directly above it.
    #[must_use]
    fn is_on_a_groups_first_row(self, which: usize) -> bool {
        let inside = if which < self.pinned.how_many {
            which
        } else {
            match which.checked_sub(self.pinned.how_many) {
                Some(inside) if inside < self.others.how_many => inside,
                _ => return false,
            }
        };
        inside < self.columns
    }

    /// Where the `which`th application's cell sits inside the list's content,
    /// across and down, before any scrolling.
    ///
    /// Counted through both groups: the pinned ones first, then the others.
    /// [`None`] past the last one.
    #[must_use]
    pub fn where_a_cell_sits(self, which: usize) -> Option<(Room, Room)> {
        let (group, inside) = if which < self.pinned.how_many {
            (self.pinned, which)
        } else {
            let inside = which.checked_sub(self.pinned.how_many)?;
            if inside >= self.others.how_many {
                return None;
            }
            (self.others, inside)
        };
        let column = u32::try_from(inside % self.columns).ok()?;
        let row = u32::try_from(inside / self.columns).ok()?;
        let across = column.saturating_mul(AN_APP_CELL_IS_WIDE + BETWEEN_APP_CELLS);
        let down = group
            .at
            .saturating_add(AN_APP_GROUP_HEADING_IS_TALL)
            .saturating_add(BETWEEN_APP_CELLS)
            .saturating_add(row.saturating_mul(AN_APP_CELL_IS_TALL + BETWEEN_APP_CELLS));
        Some((Room::pixels(across), Room::pixels(down)))
    }

    /// Whether the `which`th application is at least partly in view.
    #[must_use]
    pub fn is_shown(self, which: usize) -> bool {
        let Some((_, down)) = self.where_a_cell_sits(which) else {
            return false;
        };
        let top = down.as_pixels();
        top.saturating_add(AN_APP_CELL_IS_TALL) > self.scrolled
            && top < self.scrolled.saturating_add(self.viewport)
    }
}

#[cfg(test)]
#[path = "more_apps_tests.rs"]
mod tests;

/// Where a scrolling list's thumb sits and how long it is.
///
/// **A rule and not a measurement.** The design draws one thumb at one length
/// in one mock; what a drawing needs is where it goes for any list, and the
/// only honest generalisation is proportional — the thumb is to the track what
/// the viewport is to the content, and where it sits is how far down the
/// content the viewport has reached.
///
/// Returned as a pair of distances down the track rather than a rectangle,
/// because the track's own position is the drawing's and this does not know
/// where on a screen the list is.
///
/// [`None`] when everything fits: a full-length thumb in a list that does not
/// scroll is a control that says there is somewhere to go and there is not.
#[must_use]
pub fn a_scroll_thumb(viewport: Room, content: Room, scrolled: Room) -> Option<(Room, Room)> {
    use crate::measures::A_SCROLL_THUMB_IS_AT_LEAST;
    let (viewport, content, scrolled) = (
        viewport.as_pixels(),
        content.as_pixels(),
        scrolled.as_pixels(),
    );
    if content <= viewport || viewport == 0 {
        return None;
    }
    // **At least as long as it is wide times four**, or a long enough list
    // makes it a dot nobody can see and nobody can take hold of.
    let long = ((viewport.saturating_mul(viewport)) / content).max(A_SCROLL_THUMB_IS_AT_LEAST);
    let long = long.min(viewport);
    // How far down the track, scaled so that the thumb's **end** reaches the
    // track's end exactly when the content's end does — which is what makes it
    // read as a position rather than as a proportion of something.
    let travel = viewport.saturating_sub(long);
    let furthest = content.saturating_sub(viewport);
    // `checked_div` rather than a guard: `furthest` is zero exactly when the
    // content fits, which the early return above already refuses — so the
    // `None` arm is unreachable and says so by answering the top of the track.
    let down = scrolled
        .min(furthest)
        .saturating_mul(travel)
        .checked_div(furthest)
        .unwrap_or(0);
    Some((Room::pixels(down), Room::pixels(long)))
}
