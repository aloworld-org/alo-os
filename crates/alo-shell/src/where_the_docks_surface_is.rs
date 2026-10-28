//! Where the Dock's open surface was drawn, so a press can be classified
//! against it.
//!
//! # The gap this closes
//!
//! `crate::where_the_dock_is::TheDocksPlaces` is the Dock's hit test and it
//! covers the **band**: the slots, the overflow control, the divider. Nothing
//! covered the popover the control opens. So until this file, *More apps* and
//! the window picker were drawn to the design, said the right things to a
//! screen reader, and did nothing whatever when pressed — and after the arrow
//! keys landed, the keyboard was the only way to choose from either.
//!
//! It is the same shape the window edge was in on the other lane the same day:
//! the halves existed and nothing joined them.
//!
//! # The decision is separated from the act, which is `a_press_on_the_dock`'s
//! rule and the reason it is repeated here
//!
//! [`what_a_press_on_the_surface_does`] takes where the surface was drawn and
//! where the pointer landed, and returns what should happen. It touches
//! nothing. A press is the one input a test cannot conjure without a
//! compositor, a seat and a frame, so the decision is a value in and a value
//! out and the act is three lines elsewhere.
//!
//! # What this does not decide
//!
//! It does not perform anything. Choosing is
//! `Server::choose_from_the_open_surface`, which the keyboard's Enter already
//! calls — **one performer, two callers**, because migrating one way in and not
//! the other is how `crate::window_control_input` came to have a working
//! keyboard and a broken button at the same time.

use smithay::utils::{Physical, Point, Rectangle};

/// Which face is open, with the layout it was drawn from.
///
/// **The layout and not a list of rectangles**, because the layout already
/// answers *which item is at this point* — `TheOverview::which_cell_is_at` and
/// `ThePickersPanel::which_row_is_at` — and it answers it with the scroll it
/// was drawn at. A list of rectangles recorded here would be a second answer to
/// a question `alo-dock` has, and the two could disagree about a scrolled list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhichFace {
    /// The application overview, with its grid.
    TheOverview(alo_dock::more_apps::TheOverview),
    /// One application's windows, with its column of rows.
    TheWindowsOf(alo_dock::picking::ThePickersPanel),
}

/// Where the open surface was drawn, and what is in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheSurfaceAsDrawn {
    /// The whole popover, shadow excluded: what a press inside belongs to.
    pub(crate) popover: Rectangle<i32, Physical>,
    /// The scrolling list inside it, which is the only part that holds items.
    ///
    /// Separate from the popover because the heading, the search field and the
    /// footer are inside the popover and hold no item — a press on the heading
    /// is taken and does nothing, and must not be read as a press on whatever
    /// item the arithmetic would put there.
    pub(crate) viewport: Rectangle<i32, Physical>,
    /// Which face, and its layout — in `alo_dock`'s logical measures.
    pub(crate) face: WhichFace,
    /// The display's scale the face was drawn at.
    ///
    /// **The face is logical and a press is not.** The popover and the viewport
    /// are rectangles on the screen, in pixels; the grid and the rows inside them
    /// are `alo_dock`'s, in its measures. So a press inside the viewport is
    /// divided by this before the face is asked which item it is on — the same
    /// scale the draw multiplied by, read from the draw rather than asked again.
    pub(crate) scale: crate::display_scale::DisplayScale,
}

/// What a press on, or off, the open surface should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatAPressOnTheSurfaceDoes {
    /// Not on the surface at all. **The caller closes it**, which is what every
    /// list on this machine does and what `docs/design/the-alo-dock.md` says of
    /// these: a press outside dismisses.
    ///
    /// It says *dismiss* rather than *nothing* because the two are opposite
    /// here: a press outside is the commonest way a person closes this, and a
    /// variant meaning *nothing* would have left it undismissable.
    Dismiss,
    /// On the surface but not on an item: the heading, the search field, the
    /// footer, a gap between items, or the empty end of a short row.
    ///
    /// **Taken and nothing done.** A press that fell through to the canvas
    /// would move the view when a person aimed at a popover, and a press that
    /// reached the nearest item would open an application somebody aimed
    /// between — `alo_dock::places` settled that for the bar and it is the same
    /// rule.
    TakenAndNothingDone,
    /// On the `which`th item, counting through the whole open list.
    Choose(usize),
}

/// What a press at this point on the screen does to the open surface.
///
/// `at` is in the display's own physical pixels, the same space the popover's
/// rectangle is in.
pub(crate) fn what_a_press_on_the_surface_does(
    drawn: &TheSurfaceAsDrawn,
    at: Point<i32, Physical>,
) -> WhatAPressOnTheSurfaceDoes {
    if !drawn.popover.contains(at) {
        return WhatAPressOnTheSurfaceDoes::Dismiss;
    }
    if !drawn.viewport.contains(at) {
        return WhatAPressOnTheSurfaceDoes::TakenAndNothingDone;
    }
    // **Inside the viewport, measured from the viewport**, which is the space
    // `which_cell_is_at` and `which_row_is_at` both take — and they add their
    // own scroll, so nothing here may add it as well.
    let across = at.x.saturating_sub(drawn.viewport.loc.x);
    let down = at.y.saturating_sub(drawn.viewport.loc.y);
    if across < 0 || down < 0 {
        // `contains` already refused a point above or left of the viewport, so
        // this is unreachable rather than a case — and it is written as a
        // refusal rather than an `unwrap` because `clippy::unwrap_used` is
        // denied and because a negative offset is not worth ending a session
        // over.
        return WhatAPressOnTheSurfaceDoes::TakenAndNothingDone;
    }
    // **Back to logical**, which is what the face measures in.
    let (across, down) = (
        alo_dock::Room::pixels(drawn.scale.logical(across)),
        alo_dock::Room::pixels(drawn.scale.logical(down)),
    );
    let which = match drawn.face {
        WhichFace::TheOverview(grid) => grid.which_cell_is_at(across, down),
        WhichFace::TheWindowsOf(rows) => rows.which_row_is_at(down),
    };
    which.map_or(
        WhatAPressOnTheSurfaceDoes::TakenAndNothingDone,
        WhatAPressOnTheSurfaceDoes::Choose,
    )
}

#[cfg(test)]
#[path = "where_the_docks_surface_is_tests.rs"]
mod tests;
