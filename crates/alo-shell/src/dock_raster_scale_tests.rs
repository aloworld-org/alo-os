//! The Dock on a display drawn at 200 per cent, held against the same Dock at
//! 100: every measure exactly twice, every press landing where it did, and the
//! one-to-one Dock the same bytes it was before it took a scale.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dock::{Dock, Edge};
use alo_strings::Direction;
use smithay::utils::{Physical, Point, Rectangle};

use crate::a_press_on_the_dock::what_a_press_on_the_dock_does;
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::where_the_dock_is::{TheDocksPlaces, where_the_dock_is};
use crate::where_the_docks_surface_is::{
    WhatAPressOnTheSurfaceDoes, what_a_press_on_the_surface_does,
};

/// A display one to one, and the same display with twice the pixels each way.
const SPARSE: (i32, i32) = (1920, 1080);
/// The dense half of the pair: twice [`SPARSE`] each way.
const DENSE: (i32, i32) = (3840, 2160);

/// The dense display's scale.
fn two_times() -> DisplayScale {
    DisplayScale::of_per_cent(200)
}

/// `how_many` applications on the Dock, one window each, as `alo-dock`
/// decides the list.
fn on_the_dock(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).unwrap();
    let mut windows = alo_dock::Windows::none();
    for number in 0..how_many {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(u64::try_from(number).unwrap()),
            Some(alo_dock::AppId::named(&format!("app-{number}")).unwrap()),
            "",
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::Holding::nothing().showing(&windows)
}

/// The faces the shell draws with, and only those.
///
/// **Not `FontSystem::new`**, which also reads the machine's own fonts: the
/// first hashes here were taken that way and CI, with other fonts installed,
/// drew different letters from the same code.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new().unwrap().fonts
}

/// A rectangle twice as far from the corner and twice the size.
fn doubled(it: Rectangle<i32, Physical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (2 * it.loc.x, 2 * it.loc.y).into(),
        (2 * it.size.w, 2 * it.size.h).into(),
    )
}

/// The Dock on `edge` of a display of `size` at `scale`, holding five.
fn placed(edge: Edge, size: (i32, i32), scale: DisplayScale) -> TheDocksPlaces {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    where_the_dock_is(&dock, size, &on_the_dock(5), scale).unwrap()
}

/// The whole Dock drawn, with *More apps* open and the keyboard on a slot.
fn drawn(
    edge: Edge,
    size: (i32, i32),
    scale: DisplayScale,
    open: &crate::WhatTheDockHasOpen,
) -> super::DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let apps = on_the_dock(6);
    let strings = alo_strings::Strings::of(alo_saying::everything_this_machine_can_say().unwrap());
    let words = crate::the_docks_words::TheDocksWords::said(&strings, open);
    super::picture(
        &dock,
        noon_look(&an_appearance(), Direction::LeftToRight),
        size,
        &apps,
        &apps,
        (open, &words),
        Some(1),
        &alo_dock::artwork::NOTHING,
        &mut fonts(),
        scale,
    )
    .unwrap()
}

/// One FNV-1a hash of everything a Dock picture paints.
fn hashed(picture: &super::DockPicture) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    let area = |at: Rectangle<i32, Physical>, eat: &mut dyn FnMut(&[u8])| {
        for n in [at.loc.x, at.loc.y, at.size.w, at.size.h] {
            eat(&n.to_le_bytes());
        }
    };
    for washed in picture.surface.iter().chain(&picture.panels) {
        area(washed.area, &mut eat);
        eat(&washed.pixels);
    }
    for solid in &picture.solids {
        area(solid.area, &mut eat);
        eat(&solid.colour);
    }
    for inked in &picture.inked {
        area(inked.area, &mut eat);
        for pixel in &inked.pixels {
            eat(pixel);
        }
    }
    hash
}

/// **At 100 the whole Dock is the bytes it drew before it took a scale.**
///
/// Every surface, panel, flat shape and letter — the bar, its badges, its open
/// marks, the keyboard's ring, the composer's words and *More apps* open over
/// it — on the bottom edge and the left one. The hashes were taken once, by
/// hand, on 2026-10-28 from this fixture run against `main` at `3aeca5e8`
/// without this change, and this test repeats the comparison on every build.
///
/// **Retaken once.** The first hashes drew with `FontSystem::new`, which also
/// reads the machine's own fonts, so CI drew different letters from the same
/// code. Measured, not guessed: `main` gave the first hashes on this machine
/// and the change gave them too. The fixture now draws with the shell's own
/// bundled faces only (`fonts` below), which is what the shell draws with.
#[test]
fn at_one_hundred_the_whole_dock_draws_the_bytes_it_drew_before() {
    const BEFORE: [(Edge, u64); 2] = [(Edge::Bottom, BOTTOM_BEFORE), (Edge::Left, LEFT_BEFORE)];
    for (edge, before) in BEFORE {
        let now = hashed(&drawn(
            edge,
            SPARSE,
            DisplayScale::ONE,
            &crate::what_the_dock_has_open::MORE_APPS,
        ));
        assert_eq!(
            now, before,
            "the {edge:?} Dock at 100 drew {now:#018x}, before {before:#018x}"
        );
    }
}

/// The hash of the bottom Dock before this change. See the test above.
const BOTTOM_BEFORE: u64 = 0x4827_adf7_3434_8602;
/// The hash of the left Dock before this change. See the test above.
const LEFT_BEFORE: u64 = 0x6225_a4d4_ebf9_1906;

/// Whether `dense` is `sparse` doubled, allowing one pixel along the bar.
///
/// **One pixel, along the bar only, and that is centring rather than slack.**
/// A bar down a side or along the top holds the one-pixel rule before the
/// commands, so it is an odd length; centred on an even edge at 100 it is half
/// a pixel off centre and rounds one way, and at 200 it is exactly centred. Its
/// size and its distance from its own edge are exactly twice; only where it
/// starts along the edge may differ by that one pixel.
fn is_doubled_along(
    edge: Edge,
    dense: Rectangle<i32, Physical>,
    sparse: Rectangle<i32, Physical>,
) -> bool {
    let twice = doubled(sparse);
    let (along, across, along_twice, across_twice) = if edge.runs_across() {
        (dense.loc.x, dense.loc.y, twice.loc.x, twice.loc.y)
    } else {
        (dense.loc.y, dense.loc.x, twice.loc.y, twice.loc.x)
    };
    dense.size == twice.size && across == across_twice && (along - along_twice).abs() <= 1
}

/// **At 200 the band and every slot are exactly twice their size at 100**, on
/// every edge — and the slots themselves are the same logical answer, so the
/// same applications are on the bar.
#[test]
fn at_two_hundred_the_band_and_every_slot_are_exactly_twice_their_size() {
    for edge in Edge::EVERY {
        let sparse = placed(edge, SPARSE, DisplayScale::ONE);
        let dense = placed(edge, DENSE, two_times());
        assert!(
            is_doubled_along(edge, dense.band, sparse.band),
            "{edge:?}: {:?} is not {:?} doubled",
            dense.band,
            sparse.band
        );
        assert_eq!(dense.thickness, 2 * sparse.thickness, "{edge:?}");
        assert_eq!(dense.places, sparse.places, "{edge:?}: a different bar");
        assert_eq!(dense.over, sparse.over, "{edge:?}");
        for place in sparse.places.each() {
            let (from, across) = (
                i32::try_from(place.from_the_start()).unwrap(),
                i32::try_from(place.across()).unwrap(),
            );
            assert_eq!(two_times().px(place.from_the_start()), 2 * from);
            assert_eq!(two_times().px(place.across()), 2 * across);
        }
    }
    // The premise: the band is the measured 76 at 100, so 152 is a doubling
    // and not an accident of the display. And along the bottom, where the bar
    // is an even length, its place doubles exactly too.
    assert_eq!(
        placed(Edge::Bottom, DENSE, two_times()).band,
        doubled(placed(Edge::Bottom, SPARSE, DisplayScale::ONE).band)
    );
    assert_eq!(
        placed(Edge::Bottom, SPARSE, DisplayScale::ONE).thickness,
        76
    );
    assert_eq!(placed(Edge::Bottom, DENSE, two_times()).thickness, 152);
}

/// **What is drawn on the bar doubles too**: the panel with its shadow, and
/// every flat shape on it — the open marks and the rule before the commands.
/// Along the bottom exactly; down the left side to within the one pixel of
/// centring [`is_doubled_along`] explains.
#[test]
fn at_two_hundred_the_drawn_bar_is_twice_the_size() {
    let closed = &crate::what_the_dock_has_open::NOTHING;
    for edge in [Edge::Bottom, Edge::Left] {
        let sparse = drawn(edge, SPARSE, DisplayScale::ONE, closed);
        let dense = drawn(edge, DENSE, two_times(), closed);
        assert!(is_doubled_along(edge, dense.band, sparse.band), "{edge:?}");
        let (Some(one), Some(two)) = (sparse.surface.as_ref(), dense.surface.as_ref()) else {
            panic!("{edge:?}: a Dock with no surface");
        };
        assert!(
            is_doubled_along(edge, two.area, one.area),
            "{edge:?}: the panel and its shadow, {:?} against {:?}",
            two.area,
            one.area
        );
        assert!(
            !sparse.solids.is_empty(),
            "the premise: there are marks to compare"
        );
        assert_eq!(dense.solids.len(), sparse.solids.len(), "{edge:?}");
        for (two, one) in dense.solids.iter().zip(&sparse.solids) {
            assert!(
                is_doubled_along(edge, two.area, one.area),
                "{edge:?}: the mark {:?} is not {:?} doubled",
                two.area,
                one.area
            );
        }
        if edge == Edge::Bottom {
            let twice: Vec<_> = sparse
                .solids
                .iter()
                .map(|solid| doubled(solid.area))
                .collect();
            let now: Vec<_> = dense.solids.iter().map(|solid| solid.area).collect();
            assert_eq!(now, twice, "along the bottom every mark doubles exactly");
        }
        // Every letter is inked. Along the bottom each is inside the doubled
        // band; down a side, *History* is shaped across the slot's length and
        // runs past the bar's thickness at 100 as well, which is the side
        // Dock's own open question rather than this change's.
        assert_eq!(dense.inked.len(), sparse.inked.len(), "{edge:?}");
        for letter in dense.inked.iter().filter(|_| edge == Edge::Bottom) {
            assert!(
                dense.band.contains_rect(letter.area),
                "{edge:?}: {:?} is outside {:?}",
                letter.area,
                dense.band
            );
        }
    }
}

/// The middle of a slot, and its first and last pixel along the bar, on the
/// display the Dock was placed for.
fn points_on(dock: &TheDocksPlaces, place: &alo_dock::places::APlace) -> [Point<i32, Physical>; 3] {
    let from = dock.scale.px(place.from_the_start());
    let past = dock.scale.px(place.past_its_end());
    let across = dock.thickness / 2;
    let at = |along: i32| {
        if dock.layout.edge().runs_across() {
            Point::from((dock.band.loc.x + along, dock.band.loc.y + across))
        } else {
            Point::from((dock.band.loc.x + across, dock.band.loc.y + along))
        }
    };
    [at(from), at((from + past) / 2), at(past - 1)]
}

/// **A press anywhere on a slot at 200 does what a press on it does at 100**,
/// on every edge — its first pixel, its middle and its last — and a press on
/// the gap after it does nothing at either.
#[test]
fn a_press_on_any_slot_at_two_hundred_does_what_it_does_at_one_hundred() {
    for edge in Edge::EVERY {
        let sparse = placed(edge, SPARSE, DisplayScale::ONE);
        let dense = placed(edge, DENSE, two_times());
        let mut pressed = 0;
        for place in sparse.places.each() {
            for (one, two) in points_on(&sparse, place)
                .into_iter()
                .zip(points_on(&dense, place))
            {
                let wanted = what_a_press_on_the_dock_does(&sparse, one);
                assert_eq!(
                    what_a_press_on_the_dock_does(&dense, two),
                    wanted,
                    "{edge:?} {place:?}"
                );
                if wanted != crate::a_press_on_the_dock::WhatAPressOnTheDockDoes::Nothing {
                    pressed += 1;
                }
            }
            // One pixel past the slot is in the gap, or past the bar.
            let [_, _, last] = points_on(&dense, place);
            let gap = if edge.runs_across() {
                Point::from((last.x + 1, last.y))
            } else {
                Point::from((last.x, last.y + 1))
            };
            if sparse
                .places
                .each()
                .iter()
                .all(|other| !other.holds(place.past_its_end()))
            {
                assert_eq!(
                    what_a_press_on_the_dock_does(&dense, gap),
                    crate::a_press_on_the_dock::WhatAPressOnTheDockDoes::Nothing,
                    "{edge:?}: the gap after {place:?}"
                );
            }
        }
        assert!(
            pressed > 0,
            "{edge:?}: the premise — some slot does something"
        );
    }
}

/// ***More apps* at 200 is twice the size, opens in the doubled place, and a
/// press on any cell chooses the cell it chooses at 100.**
#[test]
fn more_apps_at_two_hundred_is_twice_the_size_and_a_press_chooses_the_same_cell() {
    let open = &crate::what_the_dock_has_open::MORE_APPS;
    let sparse = drawn(Edge::Bottom, SPARSE, DisplayScale::ONE, open);
    let dense = drawn(Edge::Bottom, DENSE, two_times(), open);
    let (one, two) = (sparse.overflow_list.unwrap(), dense.overflow_list.unwrap());
    assert_eq!(two.popover, doubled(one.popover));
    assert_eq!(two.viewport, doubled(one.viewport));
    assert_eq!(two.face, one.face, "the same overview, in logical measures");
    assert_eq!(dense.panels.len(), sparse.panels.len());

    let crate::where_the_docks_surface_is::WhichFace::TheOverview(grid) = one.face else {
        panic!("the overview was asked for");
    };
    let mut chosen = 0;
    for which in 0..6 {
        let Some((across, down)) = grid.where_a_cell_sits(which) else {
            continue;
        };
        let (across, down) = (
            i32::try_from(across.as_pixels()).unwrap(),
            i32::try_from(down.as_pixels()).unwrap(),
        );
        // A point well inside the cell at 100, and the same point at 200.
        let at_one = Point::from((
            one.viewport.loc.x + across + 10,
            one.viewport.loc.y + down + 10,
        ));
        let at_two = Point::from((2 * at_one.x, 2 * at_one.y));
        let wanted = what_a_press_on_the_surface_does(&one, at_one);
        assert_eq!(wanted, WhatAPressOnTheSurfaceDoes::Choose(which));
        assert_eq!(
            what_a_press_on_the_surface_does(&two, at_two),
            wanted,
            "cell {which}"
        );
        // And the cell's last pixel at 200 is still that cell.
        let cell = i32::try_from(alo_dock::measures::AN_APP_CELL_IS_WIDE).unwrap();
        let last = Point::from((
            two.viewport.loc.x + 2 * (across + cell) - 1,
            two.viewport.loc.y + 2 * down + 1,
        ));
        assert_eq!(
            what_a_press_on_the_surface_does(&two, last),
            wanted,
            "cell {which}'s edge"
        );
        chosen += 1;
    }
    assert!(chosen > 0, "the premise: some cell was placed");
    // A press beside the popover dismisses it at either scale.
    let outside = Point::from((two.popover.loc.x - 1, two.popover.loc.y));
    assert_eq!(
        what_a_press_on_the_surface_does(&two, outside),
        WhatAPressOnTheSurfaceDoes::Dismiss
    );
}

/// **Two displays at two scales each get a Dock laid out for their own**: the
/// same Dock drawn for a one-to-one laptop and a two-times monitor beside it,
/// as `crate::direct_desktop` draws one per display with that display's own
/// scale. Neither is the other's size.
#[test]
fn two_displays_at_two_scales_each_get_a_dock_of_their_own() {
    let closed = &crate::what_the_dock_has_open::NOTHING;
    let laptop = drawn(Edge::Bottom, (2560, 1600), two_times(), closed);
    let monitor = drawn(Edge::Bottom, SPARSE, DisplayScale::ONE, closed);
    assert_eq!(laptop.band.size.h, 152);
    assert_eq!(monitor.band.size.h, 76);
    assert_eq!(laptop.band.size.w, 2 * monitor.band.size.w);
    assert_eq!(laptop.standing.scale, two_times());
    assert_eq!(monitor.standing.scale, DisplayScale::ONE);
}
