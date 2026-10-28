//! The put-aside panel on a display drawn at 200 per cent, held against the
//! same panel at 100: the rail, its column, its control and every slot exactly
//! twice, a press on a slot bringing back the same window, and the one-to-one
//! panel the same bytes it was before it took a scale.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_put_aside::the_region_the_panel_claims::WhichEdge;
use smithay::utils::{Physical, Point, Rectangle};

use super::tests::{a_look, a_panel_holding, laid_out, laid_out_at};
use super::{PanelPicture, WhetherRevealed, picture, the_column_it_reserves};
use crate::a_picture_hashed::Hashed;
use crate::display_scale::DisplayScale;
use crate::which_preview_the_pointer_is_on::{
    OnThePanel, ThePanelAsDrawn, which_preview_the_pointer_is_on,
};

/// A display one to one.
const SPARSE: (i32, i32) = (1920, 1080);
/// The same display with twice the pixels each way.
const DENSE: (i32, i32) = (3840, 2160);

/// The dense display's scale.
fn two_times() -> DisplayScale {
    DisplayScale::of_per_cent(200)
}

/// A rectangle twice as far from the corner and twice the size.
fn doubled(it: Rectangle<i32, Physical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (2 * it.loc.x, 2 * it.loc.y).into(),
        (2 * it.size.w, 2 * it.size.h).into(),
    )
}

/// Everything a panel picture paints, and the column it owns, as one hash.
fn hashed(panel: &PanelPicture) -> u64 {
    let mut hash = Hashed::begun();
    hash.area(panel.rail);
    hash.area(panel.reserved);
    hash.solids(&panel.solids);
    hash.done()
}

/// **At 100 the panel is the bytes it drew before it took a scale**, on the
/// right edge and the left, holding four.
///
/// The hashes were taken once, by hand, on 2026-10-28 from this fixture run
/// against the tree this change was branched from (`56abab9a`), and this test
/// repeats the comparison on every build.
#[test]
fn at_one_hundred_the_panel_draws_the_bytes_it_drew_before() {
    for (edge, before) in [
        (WhichEdge::Right, RIGHT_BEFORE),
        (WhichEdge::Left, LEFT_BEFORE),
    ] {
        let drawn = picture(
            &a_panel_holding(4),
            a_look(),
            SPARSE,
            edge,
            WhetherRevealed::Revealed,
            DisplayScale::ONE,
        )
        .unwrap();
        let now = hashed(&drawn);
        assert_eq!(
            now, before,
            "the {edge:?} panel at 100 drew {now:#018x}, before {before:#018x}"
        );
    }
}

/// The hash of the right-hand panel before this change. See the test above.
const RIGHT_BEFORE: u64 = 0;
/// The hash of the left-hand panel before this change. See the test above.
const LEFT_BEFORE: u64 = 0;

/// **At 200 the rail, its column, its control and every slot are exactly
/// twice**, on a display with twice the pixels each way.
#[test]
fn at_two_hundred_every_part_of_the_panel_is_exactly_twice() {
    let sparse = laid_out(4, SPARSE);
    let dense = laid_out_at(4, DENSE, two_times());
    assert_eq!(dense.rail, doubled(sparse.rail), "the rail");
    assert_eq!(dense.reserved, doubled(sparse.reserved), "the column");
    assert_eq!(
        dense.control,
        sparse.control.map(doubled),
        "the expansion control"
    );
    assert_eq!(dense.slots.len(), sparse.slots.len(), "the slots");
    for (two, one) in dense.slots.iter().zip(&sparse.slots) {
        assert_eq!(*two, doubled(*one), "a slot");
    }
    let shapes: Vec<_> = sparse.solids.iter().map(|it| doubled(it.area)).collect();
    let now: Vec<_> = dense.solids.iter().map(|it| it.area).collect();
    assert_eq!(now, shapes, "every shape painted");
    // The control: twice the design's 64, so *twice* is twice the right thing.
    assert_eq!((sparse.rail.size.w, dense.rail.size.w), (64, 128));
}

/// The middle of a slot and its first and last pixel.
fn points_on(slot: Rectangle<i32, Physical>) -> [Point<i32, Physical>; 3] {
    [
        slot.loc,
        Point::from((slot.loc.x + slot.size.w / 2, slot.loc.y + slot.size.h / 2)),
        Point::from((slot.loc.x + slot.size.w - 1, slot.loc.y + slot.size.h - 1)),
    ]
}

/// **A press on any slot at 200 brings back the window it does at 100**, at
/// the slot's first, middle and last pixel — the rectangles a press is tested
/// against are the ones drawn, so a slot that doubled is a slot that is pressed
/// where it is seen.
#[test]
fn a_press_on_any_slot_at_two_hundred_names_the_window_it_names_at_one_hundred() {
    let panel = a_panel_holding(4);
    let sparse = ThePanelAsDrawn::of(&panel, laid_out(4, SPARSE));
    let dense_picture = laid_out_at(4, DENSE, two_times());
    let slots = dense_picture.slots.clone();
    let dense = ThePanelAsDrawn::of(&panel, dense_picture);
    assert_eq!(slots.len(), 4, "the premise: four slots to press");
    for slot in slots {
        for at in points_on(slot) {
            let half = Point::from((at.x / 2, at.y / 2));
            let there = which_preview_the_pointer_is_on(&panel, &dense, at);
            assert!(
                matches!(there, OnThePanel::APreview(_)),
                "{at:?} at 200 is on no preview"
            );
            assert_eq!(
                there,
                which_preview_the_pointer_is_on(&panel, &sparse, half),
                "{at:?} at 200 and {half:?} at 100 name different windows"
            );
        }
    }
}

/// **The column the Dock's overflow list keeps clear of is the column the
/// panel reserves, at every scale** — both ask [`the_column_it_reserves`], and
/// at 200 both are twice as wide.
#[test]
fn the_column_others_keep_clear_of_follows_the_scale_too() {
    let asked = the_column_it_reserves(a_look(), DENSE, WhichEdge::Right, two_times()).unwrap();
    assert_eq!(asked, laid_out_at(4, DENSE, two_times()).reserved);
    assert_eq!(asked.size.w, 224, "twice the design's 112");
}

/// **Two displays at two scales each get a panel laid out for their own**: a
/// two-times laptop and a one-to-one monitor beside it, as
/// `crate::desktop_raster` draws each display with that display's own scale.
#[test]
fn two_displays_at_two_scales_each_get_a_panel_of_their_own() {
    let laptop = laid_out_at(4, (2560, 1600), two_times());
    let monitor = laid_out(4, SPARSE);
    assert_eq!(laptop.rail.size.w, 128);
    assert_eq!(monitor.rail.size.w, 64);
    assert_eq!(laptop.reserved.size.w, 224);
    assert_eq!(monitor.reserved.size.w, 112);
}
