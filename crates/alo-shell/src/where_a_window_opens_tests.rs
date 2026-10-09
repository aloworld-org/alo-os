//! The three examples `docs/design/where-a-new-window-opens.md` owes, and the
//! rules that are easiest to break without noticing.
//!
//! **Written against the owner's situations rather than against this code.**
//! The contract names three — room beside the active window, overlap when
//! space is limited, repeated openings — and each is a test below with the
//! rule it holds quoted in its own words.

use super::*;

/// A window of this size at this corner.
fn at(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((x, y)), Size::from((w, h)))
}

/// A view 1366 by 768 at the origin, which is this machine's own fixture size.
fn a_view() -> Rectangle<i32, Logical> {
    at(0, 0, 1366, 768)
}

/// A window the size the design draws one, rounded: 400 by 300.
fn wanted() -> Size<i32, Logical> {
    Size::from((400, 300))
}

/// **Room beside the active window.** Rule 3: *prefer free space beside the
/// active window, with a small consistent gap. Try the right side first in
/// left-to-right layouts.*
#[test]
fn it_opens_to_the_right_when_there_is_room() {
    let active = at(100, 100, 400, 300);
    let where_it_goes = where_a_window_opens(
        a_view(),
        &[active],
        Some(active),
        wanted(),
        Direction::LeftToRight,
    );
    assert_eq!(
        where_it_goes,
        Point::from((100 + 400 + A_SMALL_GAP, 100)),
        "a gap to the right of the active window, at its top"
    );
}

/// **And the mirror.** Rule 3: *mirror that preference for right-to-left
/// layouts.* The same rule, not a second one.
#[test]
fn it_opens_to_the_left_when_the_person_reads_right_to_left() {
    let active = at(600, 100, 400, 300);
    let where_it_goes = where_a_window_opens(
        a_view(),
        &[active],
        Some(active),
        wanted(),
        Direction::RightToLeft,
    );
    assert_eq!(
        where_it_goes,
        Point::from((600 - A_SMALL_GAP - 400, 100)),
        "a gap to the left of the active window"
    );
}

/// **It takes the other side rather than overlapping.** The preferred side is
/// a preference, and rule 5's overlap is for when there is genuinely no room.
#[test]
fn it_takes_the_other_side_before_it_overlaps() {
    let active = at(500, 100, 400, 300);
    // Everything to the right of the active window is taken.
    let taken = at(916, 0, 450, 768);
    let where_it_goes = where_a_window_opens(
        a_view(),
        &[active, taken],
        Some(active),
        wanted(),
        Direction::LeftToRight,
    );
    assert_eq!(
        where_it_goes,
        Point::from((500 - A_SMALL_GAP - 400, 100)),
        "the left side was free and overlap was not needed"
    );
}

/// **Overlap when space is limited.** Rule 5: *if no suitable space exists,
/// overlap deliberately. Open the new window in front, slightly offset from
/// the active window.*
///
/// And the reason the offset is a whole band: the window **underneath** keeps
/// its own name band completely uncovered, so rule 5's *keep its title band
/// and controls reachable* holds for both of them.
#[test]
fn it_overlaps_on_purpose_when_the_view_is_full() {
    let active = at(0, 0, 1366, 768);
    let where_it_goes = where_a_window_opens(
        a_view(),
        &[active],
        Some(active),
        wanted(),
        Direction::LeftToRight,
    );
    assert_eq!(where_it_goes, Point::from((48, 48)));
    assert!(
        where_it_goes.y >= crate::window_name_band::BAND_IS_TALL,
        "the window underneath must keep its whole band uncovered"
    );
}

/// **Nothing already open is moved or resized.** Rule 4, held by the only
/// means a pure function can: it returns one point and there is nothing in its
/// answer that could rearrange a desk.
#[test]
fn it_never_asks_for_anything_open_to_move() {
    let active = at(0, 0, 1366, 768);
    let open = [active, at(10, 10, 20, 20)];
    let before = open;
    let _ = where_a_window_opens(
        a_view(),
        &open,
        Some(active),
        wanted(),
        Direction::LeftToRight,
    );
    assert_eq!(open, before, "the windows that were open are untouched");
}

/// **Repeated openings must not drift off-screen.** Rule 7: *when another
/// offset would make the new window inaccessible, restart the placement within
/// the usable view.*
///
/// Ten terminals must end as ten reachable terminals, which is the sentence
/// the contract uses and the thing this asserts.
#[test]
fn opening_many_never_walks_one_out_of_reach() {
    let view = a_view();
    let mut open: Vec<Rectangle<i32, Logical>> = Vec::new();
    let mut active = at(0, 0, 1366, 768);
    open.push(active);
    for opened in 0..10 {
        let where_it_goes =
            where_a_window_opens(view, &open, Some(active), wanted(), Direction::LeftToRight);
        let handle = Rectangle::new(where_it_goes, Size::from((44, 24)));
        assert_eq!(
            view.intersection(handle),
            Some(handle),
            "window {opened} opened at {where_it_goes:?}, whose handle is outside the view"
        );
        active = Rectangle::new(where_it_goes, wanted());
        open.push(active);
    }
}

/// **With no active window, near the centre.** Rule 6.
#[test]
fn with_nothing_active_it_opens_near_the_middle() {
    let where_it_goes = where_a_window_opens(a_view(), &[], None, wanted(), Direction::LeftToRight);
    assert_eq!(
        where_it_goes,
        Point::from(((1366 - 400) / 2, (768 - 300) / 2))
    );
}

/// **A window that would hang out of the view is not placed there.** The
/// preferred side is refused when the window would not fit whole, which is
/// what rule 3's *free space* means.
#[test]
fn it_refuses_a_side_the_window_would_hang_out_of() {
    let active = at(900, 100, 400, 300);
    let where_it_goes = where_a_window_opens(
        a_view(),
        &[active],
        Some(active),
        wanted(),
        Direction::LeftToRight,
    );
    assert_ne!(
        where_it_goes,
        Point::from((900 + 400 + A_SMALL_GAP, 100)),
        "that position runs past the right edge of the view"
    );
    assert_eq!(
        where_it_goes,
        Point::from((900 - A_SMALL_GAP - 400, 100)),
        "so it went to the left instead"
    );
}
