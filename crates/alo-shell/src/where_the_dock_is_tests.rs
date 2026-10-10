//! **Where the Dock is, and that a press lands on what was drawn.**
//!
//! The geometry moved out of `crate::dock_raster` so that a press and a draw
//! read one answer. These tests are about the half a raster test cannot make:
//! turning a point on a screen into a distance along the bar, on four edges
//! whose axes differ.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use alo_dock::places::WhatIsHere;
use alo_dock::{Dock, Edge};

use super::{TheDocksPlaces, the_band_on, where_the_dock_is};

/// This many applications on a Dock, built the way `alo-dock` would hand them
/// over rather than assembled by hand.
fn on_the_dock(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    let mut windows = alo_dock::Windows::none();
    for number in 0..how_many {
        let named = format!("app-{number}");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(alo_dock::AppId::named(&named).expect("a named application")),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::Holding::nothing().showing(&windows)
}

/// **The band is wholly on the display and clear of its edge**, which is what
/// `the_band_on` promised in the raster and promises here.
///
/// Kept with the function rather than left behind in `dock_raster`, because a
/// test that stays where the code used to be is a test that stops being run
/// against it.
#[test]
fn the_band_sits_on_whichever_edge_it_was_laid_along() {
    let size = (1920, 1080);
    let (along, thick, floating) = (600, 70, 8);

    for edge in Edge::EVERY {
        let band = the_band_on(edge, size, along, thick, floating);

        let (expect_w, expect_h) = if edge.runs_across() {
            (along, thick)
        } else {
            (thick, along)
        };
        assert_eq!(
            (band.size.w, band.size.h),
            (expect_w, expect_h),
            "{edge:?} is laid out across the wrong axis"
        );

        let gap = match edge {
            Edge::Bottom => 1080 - (band.loc.y + band.size.h),
            Edge::Top => band.loc.y,
            Edge::Left => band.loc.x,
            Edge::Right => 1920 - (band.loc.x + band.size.w),
        };
        assert_eq!(gap, floating, "{edge:?} is not floating clear of its edge");

        let (before, after) = if edge.runs_across() {
            (band.loc.x, 1920 - (band.loc.x + band.size.w))
        } else {
            (band.loc.y, 1080 - (band.loc.y + band.size.h))
        };
        assert!(
            (before - after).abs() <= 1,
            "{edge:?} is not centred: {before} before, {after} after"
        );

        assert!(
            band.loc.x >= 0
                && band.loc.y >= 0
                && band.loc.x + band.size.w <= 1920
                && band.loc.y + band.size.h <= 1080,
            "{edge:?} leaves the screen at {band:?}"
        );
    }
}

/// **No two edges put the band in the same place.**
///
/// Four placements that each pass the checks above could still be two
/// placements written twice — a copied arm with its edge not changed is the
/// likeliest way this goes wrong, and every assertion above would hold.
#[test]
fn the_four_edges_are_four_different_bands() {
    let mut seen = Vec::new();
    for edge in Edge::EVERY {
        let band = the_band_on(edge, (1920, 1080), 600, 70, 8);
        assert!(
            !seen.contains(&band),
            "{edge:?} lands exactly where an earlier edge does: {band:?}"
        );
        seen.push(band);
    }
    assert_eq!(seen.len(), 4);
}

/// **A display `alo-dock` will not lay a dock out on is refused**, rather than
/// answered with a band nobody can press.
#[test]
fn a_display_too_small_or_too_large_is_refused() {
    let dock = Dock::shipped();
    assert!(where_the_dock_is(&dock, (100, 100), &on_the_dock(1)).is_err());
    assert!(where_the_dock_is(&dock, (100_000, 1080), &on_the_dock(1)).is_err());
    assert!(where_the_dock_is(&dock, (1920, 100_000), &on_the_dock(1)).is_err());
    // The premise: an ordinary display is not refused.
    assert!(where_the_dock_is(&dock, (1920, 1080), &on_the_dock(1)).is_ok());
}

/// The Dock on this edge of a 1920 × 1080 display, holding `how_many`.
fn on_the_edge(edge: Edge, how_many: usize) -> TheDocksPlaces {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    where_the_dock_is(&dock, (1920, 1080), &on_the_dock(how_many))
        .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"))
}

/// **A press in the middle of a slot reaches that slot, on every edge.**
///
/// The whole reason this file exists. `alo_dock::Places` lays slots out as
/// distances from the bar's own start without knowing which edge the bar is on,
/// so turning a point on a screen into that distance is the one step that can
/// get the axis wrong — and getting it wrong on the two edges nobody draws by
/// default is exactly how it would go unnoticed, which is what happened to the
/// accent two changes ago.
#[test]
fn a_press_in_the_middle_of_a_slot_reaches_that_slot_on_every_edge() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        assert_eq!(dock.places.how_many(), 5, "{edge:?}");

        for (which, place) in dock.places.each().iter().enumerate() {
            let middle = i32::try_from(place.from_the_start() + place.across() / 2)
                .expect("a place on a bar fits in an i32");
            let at = if edge.runs_across() {
                (
                    dock.band.loc.x + middle,
                    dock.band.loc.y + dock.band.size.h / 2,
                )
            } else {
                (
                    dock.band.loc.x + dock.band.size.w / 2,
                    dock.band.loc.y + middle,
                )
            };

            let along = dock
                .how_far_along(at.into())
                .unwrap_or_else(|| panic!("{edge:?}: slot {which}'s middle is not on the bar"));
            let found = dock
                .places
                .at(along)
                .unwrap_or_else(|| panic!("{edge:?}: nothing at {along} along the bar"));
            assert_eq!(
                found.what(),
                place.what(),
                "{edge:?}: pressing slot {which} reached something else"
            );
        }
    }
}

/// **A press off the bar is not on the bar**, on every edge and on every side of
/// it.
///
/// `how_far_along` answering for a point beside the Dock would make every press
/// on the canvas near it open an application.
#[test]
fn a_press_beside_the_bar_is_not_on_it() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        let (left, top) = (dock.band.loc.x, dock.band.loc.y);
        let (right, bottom) = (left + dock.band.size.w, top + dock.band.size.h);
        let middle = (left + dock.band.size.w / 2, top + dock.band.size.h / 2);

        for (named, at) in [
            ("just before its start", (left - 1, middle.1)),
            ("just past its end", (right, middle.1)),
            ("just above it", (middle.0, top - 1)),
            ("just below it", (middle.0, bottom)),
            ("the middle of the screen", (960, 540)),
            ("the origin", (0, 0)),
        ] {
            assert_eq!(
                dock.how_far_along(at.into()),
                None,
                "{edge:?}: a point {named}, at {at:?}, was read as being on the bar whose band \
                 is {:?}",
                dock.band
            );
        }

        // **The premise**: a point that really is on the bar does answer, so the
        // six above are not all `None` because nothing ever is.
        assert!(
            dock.how_far_along(middle.into()).is_some(),
            "{edge:?}: the middle of the band is not on the bar, so this test proves nothing"
        );
    }
}

/// **The overflow control is reachable by a press, on every edge.**
///
/// The slot a person goes for when they have the most open.
#[test]
fn the_overflow_control_can_be_pressed_on_every_edge() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 200);
        assert!(!dock.over.is_empty(), "{edge:?}: nothing overflowed");

        let control = dock
            .places
            .each()
            .last()
            .unwrap_or_else(|| panic!("{edge:?}: no slots at all"));
        assert!(
            control.is_the_overflow(),
            "{edge:?}: the last slot is not the control"
        );

        let middle = i32::try_from(control.from_the_start() + control.across() / 2)
            .expect("a place on a bar fits in an i32");
        let at = if edge.runs_across() {
            (
                dock.band.loc.x + middle,
                dock.band.loc.y + dock.band.size.h / 2,
            )
        } else {
            (
                dock.band.loc.x + dock.band.size.w / 2,
                dock.band.loc.y + middle,
            )
        };
        let along = dock
            .how_far_along(at.into())
            .unwrap_or_else(|| panic!("{edge:?}: the control's middle is not on the bar"));
        assert_eq!(
            dock.places.at(along).map(|place| place.what()),
            Some(&WhatIsHere::TheOverflow),
            "{edge:?}: pressing the control reached an application"
        );
    }
}
