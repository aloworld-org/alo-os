//! Where F6 goes, which areas it passes over, and the one it always comes back
//! to.

use super::{AnArea, WhichWay, where_the_keyboard_goes};

/// Every area has something to act on.
fn everything(_: AnArea) -> bool {
    true
}

/// Nothing anywhere does.
fn nothing(_: AnArea) -> bool {
    false
}

/// Walk F6 forward from the canvas, collecting where it lands each time.
fn walking(
    pressed: WhichWay,
    has_anything: &dyn Fn(AnArea) -> bool,
    how_many: usize,
) -> Vec<Option<AnArea>> {
    let mut at = None;
    let mut where_it_went = Vec::new();
    for _ in 0..how_many {
        at = where_the_keyboard_goes(at, pressed, has_anything);
        where_it_went.push(at);
    }
    where_it_went
}

/// **F6 walks the four in the design's order and comes back to the work.**
///
/// Five presses, not four: the work is where the fifth lands, which is what
/// makes this a cycle a person can leave rather than a trap they have to
/// Escape out of. The design names five areas and the canvas is the absence —
/// a model holding both had two canvases in the walk and visited each.
#[test]
fn f6_walks_every_area_and_returns_to_the_canvas() {
    assert_eq!(
        walking(WhichWay::Onward, &everything, 5),
        vec![
            Some(AnArea::TheDock),
            Some(AnArea::ThePanel),
            Some(AnArea::Notifications),
            Some(AnArea::TheStatusArea),
            None,
        ]
    );
}

/// **Shift+F6 walks the other way**, ending at the work for the same reason.
#[test]
fn shift_f6_walks_back_the_way_it_came() {
    assert_eq!(
        walking(WhichWay::Back, &everything, 5),
        vec![
            Some(AnArea::TheStatusArea),
            Some(AnArea::Notifications),
            Some(AnArea::ThePanel),
            Some(AnArea::TheDock),
            None,
        ]
    );
}

/// **An area with nothing to act on is passed over.**
///
/// A person who presses F6 and lands on an empty panel has pressed a key for
/// nothing — and somebody who cannot see the screen has no way to tell *this
/// area is empty* from *that key did nothing*.
///
/// The panel and notifications are empty here, which is the ordinary state of a
/// machine somebody has just signed in to.
#[test]
fn an_area_with_nothing_in_it_is_passed_over() {
    let only_the_dock_and_the_status =
        |area: AnArea| matches!(area, AnArea::TheDock | AnArea::TheStatusArea);

    assert_eq!(
        walking(WhichWay::Onward, &only_the_dock_and_the_status, 3),
        vec![Some(AnArea::TheDock), Some(AnArea::TheStatusArea), None,],
        "F6 stopped on an area with nothing in it, or skipped one that had something"
    );
}

/// **A machine where nothing has anything leaves the keyboard on the work.**
///
/// Not spinning, and not stuck on an empty area. One lap at most, which is why
/// the walk is bounded rather than looping until it finds something.
#[test]
fn with_nothing_anywhere_the_keyboard_stays_with_the_work() {
    assert_eq!(
        where_the_keyboard_goes(None, WhichWay::Onward, &nothing),
        None
    );
    assert_eq!(
        where_the_keyboard_goes(None, WhichWay::Back, &nothing),
        None
    );
    assert_eq!(
        where_the_keyboard_goes(Some(AnArea::TheDock), WhichWay::Onward, &nothing),
        None,
        "a person on the Dock when everything emptied was left nowhere"
    );
}

/// **Forward then back returns to where it started**, for every area.
///
/// The property rather than a worked example: a cycle whose two directions
/// disagreed would strand somebody who overshot by one and pressed Shift+F6 to
/// come back, which is the commonest thing a person does with a cycle.
#[test]
fn forward_then_back_is_where_it_started() {
    for start in [
        None,
        Some(AnArea::TheDock),
        Some(AnArea::ThePanel),
        Some(AnArea::Notifications),
        Some(AnArea::TheStatusArea),
    ] {
        let onward = where_the_keyboard_goes(start, WhichWay::Onward, &everything);
        let back = where_the_keyboard_goes(onward, WhichWay::Back, &everything);
        assert_eq!(
            back, start,
            "F6 then Shift+F6 did not come back from {start:?}"
        );
    }
}

/// **The canvas is reachable from every area**, which is the way out.
///
/// Pressing F6 enough times from anywhere reaches the work. A cycle a person
/// cannot leave by the key they entered it with is one they have to be taught
/// to escape.
#[test]
fn the_work_is_reachable_from_everywhere() {
    for start in AnArea::EVERY {
        let mut at = Some(start);
        let mut reached = false;
        for _ in 0..=AnArea::EVERY.len() {
            at = where_the_keyboard_goes(at, WhichWay::Onward, &everything);
            if at.is_none() {
                reached = true;
                break;
            }
        }
        assert!(
            reached,
            "the work is not reachable from {start:?} by pressing F6"
        );
    }
}
