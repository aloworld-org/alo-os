//! **Where the Dock is, and that a press lands on what was drawn.**
//!
//! The geometry moved out of `crate::dock_raster` so that a press and a draw
//! read one answer. These tests are about the half a raster test cannot make:
//! turning a point on a screen into a distance along the bar, on four edges
//! whose axes differ.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              `expect` names what was wanted. `clippy::panic` was expected here too and is not \
              any more: the three tests that reached for a formatted panic went with \
              `how_far_along`, and an expectation that has stopped being needed is itself a \
              clippy error, which is how it was found"
)]

use alo_dock::{Dock, Edge};

use super::{the_band_on, where_the_dock_is};

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
