//! The four answers a press can get, and the boundaries between them.
//!
//! **Every one of these is a decision and none of them needs a display**, which
//! is the whole reason `what_a_press_on_the_surface_does` is a pure function
//! over a recorded rectangle. The half that needs hardware — a seat, a pointer,
//! a frame — is the three lines that call this, and
//! `crate::a_press_on_the_dock` says the same of its own split rather than
//! implying both halves are measured.
#![expect(
    clippy::panic,
    reason = "in a test, a formatted panic naming the point that was pressed is the failure \
              being reported"
)]

use super::{TheSurfaceAsDrawn, WhatAPressOnTheSurfaceDoes, WhichFace};
use smithay::utils::{Physical, Point, Rectangle};

/// A point, in the display's physical pixels.
fn at(x: i32, y: i32) -> Point<i32, Physical> {
    (x, y).into()
}

/// A popover at (500, 100), 448 wide, with its list starting 120 down.
///
/// The numbers are a plausible placement and nothing depends on them being the
/// real one: what is under test is the mapping from a point to an index, and a
/// recorded rectangle is a recorded rectangle wherever the draw put it.
fn an_overview(pinned: usize, others: usize) -> TheSurfaceAsDrawn {
    let grid = alo_dock::more_apps::TheOverview::of(pinned, others, alo_dock::Room::pixels(668));
    let viewport_is_tall = i32::try_from(grid.the_list_is_tall().as_pixels()).unwrap_or(0);
    TheSurfaceAsDrawn {
        popover: Rectangle::new(at(500, 100), (448, 536).into()),
        viewport: Rectangle::new(at(524, 220), (400, viewport_is_tall).into()),
        face: WhichFace::TheOverview(grid),
        scale: crate::display_scale::DisplayScale::ONE,
    }
}

/// The same, showing one application's windows.
fn a_window_list(how_many: usize) -> TheSurfaceAsDrawn {
    let rows = alo_dock::picking::the_pickers_panel(how_many, alo_dock::Room::pixels(668));
    let viewport_is_tall = i32::try_from(rows.the_list_is_tall().as_pixels()).unwrap_or(0);
    TheSurfaceAsDrawn {
        popover: Rectangle::new(at(500, 100), (448, 536).into()),
        viewport: Rectangle::new(at(524, 268), (400, viewport_is_tall).into()),
        face: WhichFace::TheWindowsOf(rows),
        scale: crate::display_scale::DisplayScale::ONE,
    }
}

/// **A press outside the popover dismisses it, on every side.**
///
/// All four sides and the two diagonals, because *outside* is one word and four
/// comparisons, and a comparison with the wrong sign is a surface a person
/// cannot close by clicking away from it — on one side only, which is how it
/// would survive a single-point test.
#[test]
fn a_press_anywhere_outside_dismisses_it() {
    let drawn = an_overview(4, 8);
    for (x, y, where_it_is) in [
        (499, 300, "just left"),
        (948, 300, "just right"),
        (700, 99, "just above"),
        (700, 636, "just below"),
        (0, 0, "the top-left of the display"),
        (1919, 1079, "the bottom-right of the display"),
        (499, 99, "diagonally off the top-left corner"),
        (948, 636, "diagonally off the bottom-right corner"),
    ] {
        assert_eq!(
            super::what_a_press_on_the_surface_does(&drawn, at(x, y)),
            WhatAPressOnTheSurfaceDoes::Dismiss,
            "a press {where_it_is} at ({x}, {y}) did not dismiss"
        );
    }
}

/// **A press on the popover's own furniture is taken and does nothing.**
///
/// The heading, the search field and the footer are all inside the popover and
/// hold no item. The fault this forbids is reading the arithmetic anyway: the
/// heading sits above the viewport, so a lookup that did not check the viewport
/// first would map it to a negative offset or — worse — to row zero.
#[test]
fn a_press_on_the_heading_or_the_footer_is_taken_and_does_nothing() {
    let drawn = an_overview(4, 8);
    for (x, y, what) in [
        (700, 110, "the heading"),
        (700, 180, "the search field"),
        (700, 630, "the footer"),
        (510, 300, "the padding left of the list"),
        (940, 300, "the padding right of the list"),
    ] {
        assert_eq!(
            super::what_a_press_on_the_surface_does(&drawn, at(x, y)),
            WhatAPressOnTheSurfaceDoes::TakenAndNothingDone,
            "a press on {what} at ({x}, {y}) was not taken-and-ignored"
        );
    }
}

/// **Every cell is chosen at the point the layout drew it.**
///
/// The round trip, carried through the recorded rectangle: this is the same
/// agreement `alo-dock`'s own tests hold between `where_a_cell_sits` and
/// `which_cell_is_at`, plus the one thing they cannot see — **that the shell
/// recorded the viewport's origin and subtracts it the right way round.** A
/// sign error there is invisible to both halves and would open the wrong
/// application for every press.
#[test]
fn every_cell_is_chosen_at_the_point_it_was_drawn() {
    let drawn = an_overview(4, 8);
    let WhichFace::TheOverview(grid) = drawn.face else {
        panic!("the premise: this fixture is the overview");
    };
    for which in 0..12 {
        let Some((across, down)) = grid.where_a_cell_sits(which) else {
            panic!("cell {which} was not placed");
        };
        // The middle of the cell, which a wrong stride misses.
        let x = drawn.viewport.loc.x
            + i32::try_from(across.as_pixels()).unwrap_or(0)
            + i32::try_from(alo_dock::measures::AN_APP_CELL_IS_WIDE / 2).unwrap_or(0);
        let y = drawn.viewport.loc.y
            + i32::try_from(down.as_pixels()).unwrap_or(0)
            + i32::try_from(alo_dock::measures::AN_APP_CELL_IS_TALL / 2).unwrap_or(0);
        assert_eq!(
            super::what_a_press_on_the_surface_does(&drawn, at(x, y)),
            WhatAPressOnTheSurfaceDoes::Choose(which),
            "a press in the middle of cell {which}, at ({x}, {y}), chose something else"
        );
    }
}

/// **Every row of a window list is chosen where it was drawn.**
#[test]
fn every_row_is_chosen_at_the_point_it_was_drawn() {
    let drawn = a_window_list(3);
    let WhichFace::TheWindowsOf(rows) = drawn.face else {
        panic!("the premise: this fixture is a window list");
    };
    for which in 0..3 {
        let Some(sits) = rows.where_a_row_sits(which) else {
            panic!("row {which} was not placed");
        };
        let y = drawn.viewport.loc.y
            + i32::try_from(sits.as_pixels()).unwrap_or(0)
            + i32::try_from(alo_dock::measures::A_WINDOW_ROW_IS_TALL / 2).unwrap_or(0);
        assert_eq!(
            super::what_a_press_on_the_surface_does(&drawn, at(700, y)),
            WhatAPressOnTheSurfaceDoes::Choose(which),
            "a press in the middle of row {which}, at y {y}, chose something else"
        );
    }
}

/// **A press in a gap inside the list is taken and chooses nothing.**
///
/// The rule `alo_dock::places` states for the bar, held for the surface: a gap
/// that reached its nearest neighbour would mean a person aiming at the space
/// between two applications opening one of them.
#[test]
fn a_press_in_a_gap_inside_the_list_chooses_nothing() {
    let drawn = an_overview(4, 8);
    let WhichFace::TheOverview(grid) = drawn.face else {
        panic!("the premise: this fixture is the overview");
    };
    let Some((across, down)) = grid.where_a_cell_sits(0) else {
        panic!("the first cell was not placed");
    };
    let gap = i32::try_from(alo_dock::measures::BETWEEN_APP_CELLS / 2).unwrap_or(4);
    let cell_is_wide = i32::try_from(alo_dock::measures::AN_APP_CELL_IS_WIDE).unwrap_or(94);
    let x =
        drawn.viewport.loc.x + i32::try_from(across.as_pixels()).unwrap_or(0) + cell_is_wide + gap;
    let y = drawn.viewport.loc.y + i32::try_from(down.as_pixels()).unwrap_or(0) + 10;
    assert_eq!(
        super::what_a_press_on_the_surface_does(&drawn, at(x, y)),
        WhatAPressOnTheSurfaceDoes::TakenAndNothingDone,
        "a press in the gap between the first two cells, at ({x}, {y}), chose one"
    );
}

/// **A scrolled list is read where its items now are.**
///
/// And the control that makes it mean something: the **unscrolled** surface
/// must answer differently at the same point. Without that, a test that passed
/// because the scroll happened to be zero would look identical.
#[test]
fn a_scrolled_surface_is_read_at_its_scrolled_position() {
    let unscrolled = an_overview(4, 56);
    let WhichFace::TheOverview(grid) = unscrolled.face else {
        panic!("the premise: this fixture is the overview");
    };
    let scrolled_grid = grid.showing(59);
    assert!(
        scrolled_grid.scrolled().as_pixels() > 0,
        "the premise: reaching the sixtieth scrolls the grid"
    );
    let drawn = TheSurfaceAsDrawn {
        face: WhichFace::TheOverview(scrolled_grid),
        ..unscrolled
    };
    let Some((across, down)) = scrolled_grid.where_a_cell_sits(59) else {
        panic!("the sixtieth cell was not placed");
    };
    let into_the_viewport = down
        .as_pixels()
        .saturating_sub(scrolled_grid.scrolled().as_pixels());
    let x = drawn.viewport.loc.x
        + i32::try_from(across.as_pixels()).unwrap_or(0)
        + i32::try_from(alo_dock::measures::AN_APP_CELL_IS_WIDE / 2).unwrap_or(0);
    let y = drawn.viewport.loc.y
        + i32::try_from(into_the_viewport).unwrap_or(0)
        + i32::try_from(alo_dock::measures::AN_APP_CELL_IS_TALL / 2).unwrap_or(0);
    assert_eq!(
        super::what_a_press_on_the_surface_does(&drawn, at(x, y)),
        WhatAPressOnTheSurfaceDoes::Choose(59),
        "the sixtieth application was not chosen where the scrolled grid drew it"
    );
    assert_ne!(
        super::what_a_press_on_the_surface_does(&unscrolled, at(x, y)),
        WhatAPressOnTheSurfaceDoes::Choose(59),
        "the unscrolled surface answered the same at the same point, so the scroll is \
         not being read and this test proves nothing"
    );
}

/// **An empty overview takes a press inside it and chooses nothing.**
///
/// A surface with no applications is a surface a person can still press, and
/// the honest answer is that there was nothing there — not a dismissal, because
/// the press was on the popover, and not a choice, because there is nothing to
/// choose.
#[test]
fn an_empty_surface_chooses_nothing_and_stays_open() {
    let drawn = an_overview(0, 0);
    assert_eq!(
        super::what_a_press_on_the_surface_does(&drawn, at(700, 300)),
        WhatAPressOnTheSurfaceDoes::TakenAndNothingDone
    );
    // And a press outside still dismisses it, which is the only way out of one.
    assert_eq!(
        super::what_a_press_on_the_surface_does(&drawn, at(100, 300)),
        WhatAPressOnTheSurfaceDoes::Dismiss
    );
}
