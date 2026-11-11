#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
//! The matches rise from the Dock's field on every edge, stay on the display
//! and out of the panel's column, and keep the row the keyboard is on in view.

use super::*;

/// A display of `w` × `h` with a Dock on `edge`, its field where a Dock of
/// that family draws it.
fn a_dock(w: i32, h: i32, edge: alo_dock::Edge) -> TheFieldOnTheDock {
    let (band, field) = match edge {
        alo_dock::Edge::Bottom => (
            Rectangle::new((300, h - 88).into(), (840, 76).into()),
            Rectangle::new((560, h - 74).into(), (344, 48).into()),
        ),
        alo_dock::Edge::Top => (
            Rectangle::new((300, 12).into(), (840, 76).into()),
            Rectangle::new((560, 26).into(), (344, 48).into()),
        ),
        alo_dock::Edge::Left => (
            Rectangle::new((12, 100).into(), (76, 700).into()),
            Rectangle::new((26, 300).into(), (48, 344).into()),
        ),
        alo_dock::Edge::Right => (
            Rectangle::new((w - 88, 100).into(), (76, 700).into()),
            Rectangle::new((w - 74, 300).into(), (48, 344).into()),
        ),
    };
    TheFieldOnTheDock { field, band, edge }
}

/// Two applications found and the row that asks offered.
const TWO_AND_ASK: WhatItHolds = WhatItHolds {
    found: 2,
    asks: true,
    chosen: None,
};

fn rises(edge: alo_dock::Edge, holds: WhatItHolds) -> TheMatchesPlaces {
    where_the_matches_rise(
        Size::from((1440, 960)),
        DisplayScale::ONE,
        holds,
        a_dock(1440, 960, edge),
        None,
    )
    .expect("there is room for it")
}

#[test]
fn above_a_bottom_dock_the_list_ends_its_gap_above_the_bar_where_the_field_starts() {
    let dock = a_dock(1440, 960, alo_dock::Edge::Bottom);
    let at = rises(alo_dock::Edge::Bottom, TWO_AND_ASK);
    assert_eq!(at.panel.loc.y + at.panel.size.h, dock.band.loc.y - 12);
    assert_eq!(at.panel.loc.x, dock.field.loc.x);
    assert_eq!(at.rows.len(), 2);
    assert!(at.ask.is_some() && at.hairline.is_some());
    assert!(
        at.typed.is_none(),
        "the words are in the Dock's field itself"
    );
}

#[test]
fn below_a_top_dock_the_list_starts_its_gap_below_the_bar() {
    let dock = a_dock(1440, 960, alo_dock::Edge::Top);
    let at = rises(alo_dock::Edge::Top, TWO_AND_ASK);
    assert_eq!(at.panel.loc.y, dock.band.loc.y + dock.band.size.h + 12);
}

#[test]
fn beside_a_side_dock_the_list_opens_inward_and_carries_what_is_typed() {
    let left = a_dock(1440, 960, alo_dock::Edge::Left);
    let at = rises(alo_dock::Edge::Left, TWO_AND_ASK);
    assert_eq!(at.panel.loc.x, left.band.loc.x + left.band.size.w + 12);
    assert!(at.typed.is_some());

    let right = a_dock(1440, 960, alo_dock::Edge::Right);
    let at = rises(alo_dock::Edge::Right, TWO_AND_ASK);
    assert_eq!(at.panel.loc.x + at.panel.size.w, right.band.loc.x - 12);
    assert!(at.typed.is_some());
}

#[test]
fn every_part_of_the_list_is_inside_its_panel_on_every_edge() {
    for edge in alo_dock::Edge::EVERY {
        let at = rises(edge, TWO_AND_ASK);
        let inside = |part: Rectangle<i32, Physical>| {
            part.loc.x >= at.panel.loc.x
                && part.loc.y >= at.panel.loc.y
                && part.loc.x + part.size.w <= at.panel.loc.x + at.panel.size.w
                && part.loc.y + part.size.h <= at.panel.loc.y + at.panel.size.h
        };
        assert!(at.rows.iter().all(|(_, row)| inside(*row)), "{edge:?}");
        assert!(at.ask.is_some_and(inside), "{edge:?}");
        assert!(at.typed.is_none_or(inside), "{edge:?}");
        assert!(!at.panel.overlaps(a_dock(1440, 960, edge).band), "{edge:?}");
    }
}

#[test]
fn nothing_typed_on_a_bottom_dock_is_no_list_and_on_a_side_dock_is_the_line_alone() {
    let nothing = WhatItHolds {
        found: 0,
        asks: false,
        chosen: None,
    };
    assert!(
        where_the_matches_rise(
            Size::from((1440, 960)),
            DisplayScale::ONE,
            nothing,
            a_dock(1440, 960, alo_dock::Edge::Bottom),
            None,
        )
        .is_none()
    );
    let side = rises(alo_dock::Edge::Left, nothing);
    assert!(side.rows.is_empty() && side.ask.is_none());
    assert!(side.typed.is_some());
}

#[test]
fn with_ai_off_there_is_no_row_that_asks() {
    let at = rises(
        alo_dock::Edge::Bottom,
        WhatItHolds {
            found: 1,
            asks: false,
            chosen: None,
        },
    );
    assert!(at.ask.is_none() && at.hairline.is_none());
    assert_eq!(at.rows.len(), 1);
}

#[test]
fn the_list_is_kept_clear_of_the_panels_column() {
    let column = Rectangle::new((1340, 0).into(), (100, 960).into());
    let mut dock = a_dock(1440, 960, alo_dock::Edge::Bottom);
    dock.field.loc.x = 1100;
    let at = where_the_matches_rise(
        Size::from((1440, 960)),
        DisplayScale::ONE,
        TWO_AND_ASK,
        dock,
        Some(column),
    )
    .expect("room");
    assert!(at.panel.loc.x + at.panel.size.w <= column.loc.x - 12);
}

#[test]
fn on_a_small_display_the_list_stays_on_it_and_scrolls_to_the_row_chosen() {
    for (w, h) in [(800, 480), (1024, 600), (1280, 720)] {
        let mut dock = a_dock(w, h, alo_dock::Edge::Bottom);
        dock.field.loc.x = 40;
        let holds = WhatItHolds {
            found: 30,
            asks: true,
            chosen: Some(29),
        };
        let at = where_the_matches_rise(Size::from((w, h)), DisplayScale::ONE, holds, dock, None)
            .expect("room for some rows");
        assert!(at.panel.loc.y >= 0 && at.panel.loc.x >= 0, "{w}x{h}");
        assert!(at.panel.loc.x + at.panel.size.w <= w, "{w}x{h}");
        assert!(
            at.panel.loc.y + at.panel.size.h <= dock.band.loc.y,
            "{w}x{h}"
        );
        assert!(at.rows.len() < 30, "{w}x{h}: it scrolls");
        assert_eq!(at.rows.last().map(|(which, _)| *which), Some(29), "{w}x{h}");
        assert!(
            at.ask.is_some(),
            "{w}x{h}: the row that asks is never scrolled away"
        );
    }
}

#[test]
fn a_display_too_short_for_the_row_that_asks_draws_no_list() {
    let dock = TheFieldOnTheDock {
        field: Rectangle::new((10, 40).into(), (344, 48).into()),
        band: Rectangle::new((0, 30).into(), (800, 76).into()),
        edge: alo_dock::Edge::Bottom,
    };
    assert!(
        where_the_matches_rise(
            Size::from((800, 106)),
            DisplayScale::ONE,
            TWO_AND_ASK,
            dock,
            None
        )
        .is_none()
    );
}

#[test]
fn at_two_hundred_the_list_is_twice_as_wide() {
    let one = rises(alo_dock::Edge::Bottom, TWO_AND_ASK);
    let mut dock = a_dock(2880, 1920, alo_dock::Edge::Bottom);
    dock.band = Rectangle::new((600, 1920 - 176).into(), (1680, 152).into());
    dock.field = Rectangle::new((1120, 1920 - 148).into(), (688, 96).into());
    let two = where_the_matches_rise(
        Size::from((2880, 1920)),
        DisplayScale::of_per_cent(200),
        TWO_AND_ASK,
        dock,
        None,
    )
    .expect("room");
    assert_eq!(two.panel.size.w, one.panel.size.w * 2);
    assert_eq!(two.panel.size.h, one.panel.size.h * 2);
}
