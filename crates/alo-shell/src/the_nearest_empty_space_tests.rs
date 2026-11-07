//! The nearest empty space, against desks drawn out here rather than against a
//! restatement of the search.
//!
//! The strongest of these is the last: a brute-force walk over every whole
//! position in a region, which shares nothing with the search but the
//! sentence it implements — *the nearest free space, measured as how far the
//! view moves* — and has to agree with it on every desk it is given.

use super::{THE_GAP, WhereItGoes, the_nearest_empty_space};
use alo_strings::Direction;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// A rectangle on the plane.
fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((x, y)), Size::from((w, h)))
}

/// A size on the plane.
fn size(w: i32, h: i32) -> Size<i32, Logical> {
    Size::from((w, h))
}

/// The answer, or the test fails naming the desk.
fn found(
    view: Rectangle<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
    wanted: Size<i32, Logical>,
    reading: Direction,
) -> WhereItGoes {
    the_nearest_empty_space(view, open, wanted, &[], None, reading)
        .unwrap_or_else(|| unreachable!("no space found for {wanted:?} beside {open:?}"))
}

/// The new window, grown by the gap, overlaps nothing open — the owner's
/// *without covering any window*, with the gap rule 3 keeps.
fn covers_nothing(
    at: Point<i32, Logical>,
    wanted: Size<i32, Logical>,
    open: &[Rectangle<i32, Logical>],
) -> bool {
    let grown = rect(
        at.x - THE_GAP,
        at.y - THE_GAP,
        wanted.w + 2 * THE_GAP,
        wanted.h + 2 * THE_GAP,
    );
    !open.iter().any(|it| it.overlaps(grown))
}

/// Whether the window is wholly inside the view that will show it, a gap in.
fn is_shown(found: WhereItGoes, wanted: Size<i32, Logical>, view: Size<i32, Logical>) -> bool {
    found.window.x >= found.view.x + THE_GAP
        && found.window.y >= found.view.y + THE_GAP
        && found.window.x + wanted.w <= found.view.x + view.w - THE_GAP
        && found.window.y + wanted.h <= found.view.y + view.h - THE_GAP
}

/// A view tiled full: four windows filling a 1440 × 960 view with gaps.
fn a_full_view() -> (Rectangle<i32, Logical>, Vec<Rectangle<i32, Logical>>) {
    let view = rect(0, 0, 1440, 960);
    let open = vec![
        rect(16, 16, 696, 456),
        rect(728, 16, 696, 456),
        rect(16, 488, 696, 456),
        rect(728, 488, 696, 456),
    ];
    (view, open)
}

/// **Room in view: the view does not move**, and the window is in it.
#[test]
fn with_room_in_view_the_view_stays_where_it_is() {
    let view = rect(0, 0, 1440, 960);
    let open = [rect(16, 16, 600, 400)];
    let wanted = size(500, 400);
    let it = found(view, &open, wanted, Direction::LeftToRight);
    assert!(
        !it.the_view_moves(view.loc),
        "there was room and the view moved: {it:?}"
    );
    assert!(covers_nothing(it.window, wanted, &open));
    assert!(is_shown(it, wanted, view.size));
}

/// **A full view: the view moves to the nearest space, and only as far as it
/// must.** Above and below the desk are 496 away and beside it 656, so it goes
/// up — the tie between up and down going to the higher.
#[test]
fn a_full_view_glides_to_the_nearest_space_and_covers_nothing() {
    let (view, open) = a_full_view();
    let wanted = size(640, 480);
    let it = found(view, &open, wanted, Direction::LeftToRight);
    assert!(
        it.the_view_moves(view.loc),
        "the view was full and did not move"
    );
    assert!(
        covers_nothing(it.window, wanted, &open),
        "it covers a window: {it:?}"
    );
    assert!(
        is_shown(it, wanted, view.size),
        "the space is not in the view it glides to"
    );
    assert_eq!(
        it.window.y,
        16 - THE_GAP - wanted.h,
        "it is not the nearest space, one gap above the desk"
    );
    assert_eq!(
        (it.view.x, it.view.y),
        (view.loc.x, it.window.y - THE_GAP),
        "the view moved further than it had to"
    );
}

/// **Beside, when beside is nearest**, and the reading direction breaks the
/// tie between the two sides: the right in a left-to-right layout and the left
/// in a right-to-left one — rule 3's own preference, mirrored.
#[test]
fn beside_the_desk_the_tie_goes_the_way_a_reader_goes() {
    let (view, open) = a_full_view();
    // Tall enough that above or below is further than beside.
    let wanted = size(400, 900);
    let rightwards = found(view, &open, wanted, Direction::LeftToRight);
    assert_eq!(rightwards.window.x, 1424 + THE_GAP, "not to the right");
    let leftwards = found(view, &open, wanted, Direction::RightToLeft);
    assert_eq!(
        leftwards.window.x,
        16 - THE_GAP - wanted.w,
        "not to the left"
    );
    for it in [rightwards, leftwards] {
        assert!(covers_nothing(it.window, wanted, &open));
        assert!(is_shown(it, wanted, view.size));
        assert_eq!(it.view.y, view.loc.y, "the view moved up or down as well");
    }
}

/// **In any direction**: with the left, the right and above taken, it goes
/// down.
#[test]
fn the_nearest_space_can_be_in_any_direction() {
    let (view, mut open) = a_full_view();
    open.extend([
        rect(1440, -2000, 3000, 6000),
        rect(-3000, -2000, 3000, 6000),
        rect(0, -3000, 1440, 3000),
    ]);
    let wanted = size(640, 480);
    let it = found(view, &open, wanted, Direction::LeftToRight);
    assert!(it.window.y >= 944 + THE_GAP, "it did not go down: {it:?}");
    assert!(covers_nothing(it.window, wanted, &open));
    assert!(is_shown(it, wanted, view.size));
}

/// **A Place far out**, in the plane's own corner: above, the nearest space
/// elsewhere, is off the plane, so the space found is below — still on the
/// plane, and so is the view that shows it.
#[test]
fn a_full_place_at_the_edge_of_the_plane_finds_space_still_on_it() {
    let furthest = alo_canvas::plane::FURTHEST;
    let view = rect(-furthest, -furthest, 1440, 960);
    let open: Vec<_> = a_full_view()
        .1
        .into_iter()
        .map(|it| {
            rect(
                it.loc.x + view.loc.x,
                it.loc.y + view.loc.y,
                it.size.w,
                it.size.h,
            )
        })
        .collect();
    let wanted = size(640, 480);
    let it = found(view, &open, wanted, Direction::LeftToRight);
    assert!(it.window.x >= -furthest && it.window.y >= -furthest);
    assert!(it.view.x >= -furthest && it.view.y >= -furthest);
    assert!(
        it.window.y > view.loc.y,
        "it did not go down, away from the edge: {it:?}"
    );
    assert!(covers_nothing(it.window, wanted, &open));
    assert!(is_shown(it, wanted, view.size));
}

/// **Show all still gathers everything**: with a reach just tall enough for
/// one more window above the desk, that space is found and lies inside it.
#[test]
fn a_space_inside_what_show_all_can_reach_is_found() {
    let (view, open) = a_full_view();
    let wanted = size(640, 480);
    // The desk is 1408 across and 928 down; one window above it, one gap
    // apart, makes it 1424 down.
    let reach = Some(size(1408, 1424));
    let it = the_nearest_empty_space(view, &open, wanted, &[], reach, Direction::LeftToRight)
        .unwrap_or_else(|| unreachable!("there is room above, inside the reach"));
    assert!(covers_nothing(it.window, wanted, &open));
    assert!(
        it.window.y + wanted.h + THE_GAP <= 16,
        "it is not above the desk: {it:?}"
    );
}

/// **The refusal**: one unit less reach, and nothing fits anywhere — so there
/// is no nearest empty space, and the caller opens in front, offset, instead,
/// rather than putting a window where *Show all* could not bring it back.
#[test]
fn a_place_full_to_the_edge_of_the_reach_has_no_space() {
    let (view, open) = a_full_view();
    let reach = Some(size(1408, 1423));
    assert_eq!(
        the_nearest_empty_space(
            view,
            &open,
            size(640, 480),
            &[],
            reach,
            Direction::LeftToRight
        ),
        None,
        "a space was found outside what Show all can bring back"
    );
}

/// **Repeated openings stay findable** (rule 7): ten windows opened one after
/// another each find a space, none covers another, and every one is within
/// what *Show all* reaches on a 1440 × 960 display.
#[test]
fn ten_openings_in_a_row_cover_nothing_and_stay_within_reach() {
    let (view, mut open) = a_full_view();
    let wanted = size(640, 480);
    // 1440 * 1000 / 50, the furthest out a Place can be shown.
    let reach = size(28_800, 19_200);
    let mut looking = view;
    for _ in 0..10 {
        let it = the_nearest_empty_space(
            looking,
            &open,
            wanted,
            &[],
            Some(reach),
            Direction::LeftToRight,
        )
        .unwrap_or_else(|| unreachable!("a space within reach"));
        assert!(
            covers_nothing(it.window, wanted, &open),
            "{it:?} covers a window"
        );
        assert!(is_shown(it, wanted, view.size));
        open.push(Rectangle::new(it.window, wanted));
        looking = Rectangle::new(it.view, view.size);
    }
    let left = open.iter().map(|it| it.loc.x).min().unwrap_or_default();
    let right = open
        .iter()
        .map(|it| it.loc.x + it.size.w)
        .max()
        .unwrap_or_default();
    let top = open.iter().map(|it| it.loc.y).min().unwrap_or_default();
    let bottom = open
        .iter()
        .map(|it| it.loc.y + it.size.h)
        .max()
        .unwrap_or_default();
    assert!(right - left <= reach.w && bottom - top <= reach.h);
}

/// **A fixed control never buries the new window's handle** once the view
/// has moved: a space whose corner would sit under the top controls is passed
/// over for one whose corner does not.
#[test]
fn a_space_whose_handle_would_be_under_a_fixed_control_is_passed_over() {
    let (view, open) = a_full_view();
    let wanted = size(640, 480);
    let unhindered = found(view, &open, wanted, Direction::LeftToRight);
    // A control across the whole top of the screen, deep enough to cover the
    // handle wherever a window sits one gap in from the view's top.
    let top_controls = [rect(view.loc.x, view.loc.y, view.size.w, THE_GAP + 24)];
    let it = the_nearest_empty_space(
        view,
        &open,
        wanted,
        &top_controls,
        None,
        Direction::LeftToRight,
    )
    .unwrap_or_else(|| unreachable!("a space with its handle clear"));
    assert_ne!(
        it, unhindered,
        "the control: the unhindered space has its handle under the bar"
    );
    let moved_by = (it.view.x - view.loc.x, it.view.y - view.loc.y);
    let bar_then = rect(
        top_controls[0].loc.x + moved_by.0,
        top_controls[0].loc.y + moved_by.1,
        top_controls[0].size.w,
        top_controls[0].size.h,
    );
    let handle = rect(it.window.x, it.window.y, 44, 24);
    assert_ne!(
        bar_then.intersection(handle),
        Some(handle),
        "the handle is under the bar"
    );
    assert!(covers_nothing(it.window, wanted, &open));
}

/// **A window larger than the view** is shown from its corner, one gap in,
/// where its handle is.
#[test]
fn a_window_larger_than_the_view_is_shown_from_its_handle() {
    let (view, open) = a_full_view();
    let wanted = size(2000, 1200);
    let it = found(view, &open, wanted, Direction::LeftToRight);
    assert!(covers_nothing(it.window, wanted, &open));
    assert_eq!(
        (it.window.x - it.view.x, it.window.y - it.view.y),
        (THE_GAP, THE_GAP),
        "its corner is not one gap into the view"
    );
}

/// **Nothing to place, or nothing to look through**, is no answer rather than
/// a guess.
#[test]
fn an_empty_window_or_an_empty_view_has_no_space() {
    let (view, open) = a_full_view();
    for (view, wanted) in [
        (view, size(0, 480)),
        (view, size(640, -1)),
        (rect(0, 0, 0, 960), size(640, 480)),
    ] {
        assert_eq!(
            the_nearest_empty_space(view, &open, wanted, &[], None, Direction::LeftToRight),
            None
        );
    }
}

/// How far the view moves on one axis, written out again from the sentence
/// for the brute force: inside with a gap to spare, or its start one gap in
/// where it is longer than the view.
fn travel(view_from: i32, view_length: i32, at: i32, wanted: i32) -> i64 {
    let (view_from, view_length, at, wanted) = (
        i64::from(view_from),
        i64::from(view_length),
        i64::from(at),
        i64::from(wanted),
    );
    let gap = i64::from(THE_GAP);
    if wanted > view_length - 2 * gap {
        return at - (view_from + gap);
    }
    let lowest = view_from + gap;
    let highest = view_from + view_length - gap - wanted;
    if at < lowest {
        at - lowest
    } else if at > highest {
        at - highest
    } else {
        0
    }
}

/// **The search agrees with a brute-force walk over every whole position**,
/// on desks at several display sizes and zooms — the view being the display
/// divided by the zoom, as `crate::a_new_window_is_placed` derives it.
///
/// Small numbers so the walk is cheap: each display is scaled down by the same
/// factor as its windows, which changes nothing about which space is nearest.
#[test]
fn the_search_agrees_with_walking_every_position_at_every_display_size() {
    // (display width, height, zoom in thousandths), each divided by 16 so the
    // walk visits a few hundred thousand points rather than billions.
    let displays: [(i32, i32, i32); 5] = [
        (1280, 800, 1000),
        (1440, 960, 1000),
        (1920, 1080, 500),
        (3840, 2160, 2000),
        (800, 1280, 1000),
    ];
    for (width, height, zoom) in displays {
        let view = rect(37, -11, width * 1000 / zoom / 16, height * 1000 / zoom / 16);
        let (w, h) = (view.size.w, view.size.h);
        let desks: [Vec<Rectangle<i32, Logical>>; 3] = [
            vec![rect(view.loc.x, view.loc.y, w, h)],
            vec![
                rect(view.loc.x - 5, view.loc.y + 3, w / 2, h + 4),
                rect(view.loc.x + w / 2 + 2, view.loc.y - 6, w / 2, h / 2),
                rect(view.loc.x + w / 2 + 2, view.loc.y + h / 2 + 1, w * 2, h),
            ],
            vec![
                rect(view.loc.x, view.loc.y, w, h),
                rect(view.loc.x + w + 3, view.loc.y - h, w, 3 * h),
            ],
        ];
        for open in desks {
            let wanted = size(w / 3 + 1, h / 3 + 2);
            let Some(it) =
                the_nearest_empty_space(view, &open, wanted, &[], None, Direction::LeftToRight)
            else {
                unreachable!("an open plane always has room somewhere");
            };
            let cost = |x: i32, y: i32| {
                let (a, d) = (
                    travel(view.loc.x, w, x, wanted.w),
                    travel(view.loc.y, h, y, wanted.h),
                );
                a * a + d * d
            };
            let mut best = i64::MAX;
            for x in (view.loc.x - 3 * w)..(view.loc.x + 4 * w) {
                for y in (view.loc.y - 3 * h)..(view.loc.y + 4 * h) {
                    if covers_nothing(Point::from((x, y)), wanted, &open) {
                        best = best.min(cost(x, y));
                    }
                }
            }
            assert!(
                covers_nothing(it.window, wanted, &open),
                "{it:?} covers a window"
            );
            assert_eq!(
                cost(it.window.x, it.window.y),
                best,
                "at {width}x{height} and zoom {zoom}, the search did not find the nearest space \
                 the walk found, for {open:?}"
            );
            assert_eq!(
                (
                    i64::from(it.view.x - view.loc.x),
                    i64::from(it.view.y - view.loc.y)
                ),
                (
                    travel(view.loc.x, w, it.window.x, wanted.w),
                    travel(view.loc.y, h, it.window.y, wanted.h)
                ),
                "the view does not move by exactly what shows the window"
            );
        }
    }
}
