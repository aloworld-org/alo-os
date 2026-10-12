//! What the picker answers, and the one question it exists for.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::on_the_canvas::{Patch, Spot};
use crate::previews::Opened;
use crate::window::{AppId, HowItSits, Window};
use crate::windows::Windows;

fn app(name: &str) -> AppId {
    AppId::named(name).unwrap()
}

/// `how_many` windows of one application, numbered from one.
fn a_browser_with(how_many: u64) -> Windows {
    let mut windows = Windows::none();
    for number in 1..=how_many {
        windows.opened(Window::of(
            crate::window::WindowId::numbered(number),
            Some(app("Browser")),
            &format!("Window {number}"),
            Patch::of(Spot::at(0, 0), 800, 600).unwrap(),
            HowItSits::OnTheCanvas,
        ));
    }
    windows
}

fn picker_of(how_many: u64, room: u32) -> ThePicker {
    let windows = a_browser_with(how_many);
    let previews = Previews::of(&windows, &app("Browser"), Opened::ByHovering);
    ThePicker::of(previews, TextScale::default(), Room::pixels(room))
}

/// **A row on the screen names the window it is drawn against**, when nothing
/// has scrolled.
#[test]
fn an_unscrolled_row_names_the_window_beside_it() {
    let picker = picker_of(3, 1_000);
    let (_, showing) = picker.panel().showing();
    assert_eq!(showing, 3, "the premise: all three fit");

    for which in 0..showing {
        let named = picker.which_window(which).unwrap();
        let drawn = picker.rows().get(which).unwrap().id();
        assert_eq!(named, drawn, "row {which}");
    }
}

/// **And when it has scrolled, it still does** — which is the whole reason this
/// type exists.
///
/// A list scrolled down makes the first row on the screen some later window. A
/// caller indexing `rows()` directly would focus the wrong one, silently, and
/// only for somebody with enough windows to scroll. The counts are printed in
/// the failure because *which* wrong window it picked says whether the fault is
/// an off-by-one or a missing offset entirely.
#[test]
fn a_scrolled_row_names_the_window_that_is_actually_drawn_against_it() {
    // Room for a few rows out of many, so the list must scroll.
    let picker = picker_of(20, 300).showing_window(15);
    let (first, showing) = picker.panel().showing();
    assert!(picker.panel().scrolls(), "the premise: it scrolls");
    assert!(first > 0, "the premise: it has scrolled, first={first}");

    for which in 0..showing {
        let named = picker.which_window(which).unwrap();
        let drawn = picker.rows().get(first + which).unwrap().id();
        assert_eq!(
            named, drawn,
            "row {which} on the screen is window {first} + {which} in the list"
        );
    }

    // And the top row is emphatically not the first window.
    assert_ne!(
        picker.which_window(0),
        picker.rows().first().map(crate::previews::Preview::id),
        "the top row on a scrolled list named the first window in the list"
    );
}

/// **A row that is not on the screen names nothing.**
///
/// Including every row of an empty list. Returning the last window, or the
/// first, would make a press below the rows focus something.
#[test]
fn a_row_that_is_not_shown_names_nothing() {
    let picker = picker_of(20, 300);
    let (_, showing) = picker.panel().showing();
    assert!(showing < 20, "the premise: not all of them are shown");

    assert_eq!(
        picker.which_window(showing),
        None,
        "one past the last shown"
    );
    assert_eq!(picker.which_window(usize::MAX), None, "far past it");
    assert_eq!(
        picker.where_a_row_sits(showing),
        None,
        "and it sits nowhere"
    );
}

/// **Where a row sits and which window it is are the same counting.**
///
/// Asserted together rather than separately, because the fault worth catching
/// is them disagreeing — a list drawn at one set of offsets and pressed at
/// another. Each shown row has both answers or neither.
#[test]
fn where_a_row_sits_and_what_it_names_agree_about_which_rows_exist() {
    for (how_many, room) in [(3_u64, 1_000_u32), (20, 300), (2, 120), (40, 90)] {
        let picker = picker_of(how_many, room).showing_window(usize::from(how_many > 10) * 15);
        for which in 0..40 {
            assert_eq!(
                picker.which_window(which).is_some(),
                picker.where_a_row_sits(which).is_some(),
                "{how_many} windows in {room}: row {which} is named by one and not the other"
            );
        }
    }
}

/// **A picker of one window is not worth showing.**
///
/// `clicking` answers `ChooseAWindow` only above one window, so a picker
/// holding one is an application whose other windows closed between the press
/// and the draw. That is an ordinary race and the answer is to close, not to
/// show a person a list with a single thing on it.
#[test]
fn a_picker_of_one_window_or_none_is_not_worth_showing() {
    assert!(!picker_of(1, 1_000).is_worth_showing(), "one window");
    assert!(!picker_of(0, 1_000).is_worth_showing(), "no windows");
    assert!(picker_of(2, 1_000).is_worth_showing(), "two windows");
}

/// **The picker's panel stops at the same cap as the overview.**
///
/// The owner's 536 is about the popover and not about what is in it, so a
/// window list obeys it exactly as a grid of applications does. Both ends,
/// because a cap that is never reached is not being tested.
#[test]
fn the_pickers_panel_stops_at_the_cap() {
    use crate::measures::MORE_APPS_IS_AT_MOST_TALL;
    let room = Room::pixels(768 - 76 - 12 - 12);

    let two = the_pickers_panel(2, room);
    let many = the_pickers_panel(40, room);

    assert_eq!(
        many.height().as_pixels(),
        MORE_APPS_IS_AT_MOST_TALL,
        "forty windows did not stop at the cap"
    );
    assert!(many.scrolls(), "forty windows fitted without scrolling");
    assert!(
        two.height().as_pixels() < many.height().as_pixels(),
        "two windows are already at the cap, so this proves nothing: {} and {}",
        two.height().as_pixels(),
        many.height().as_pixels()
    );
    assert!(!two.scrolls(), "two windows should not need to scroll");
}

/// **Its list is shorter than the overview's**, because its heading is taller.
///
/// 320 against 336: the picker's heading is 48 and holds a back control, the
/// overview's is 32 and holds a line of text. Asserted as the difference rather
/// than as two numbers, so that moving either heading is caught by one test.
#[test]
fn the_pickers_list_is_shorter_by_exactly_the_taller_heading() {
    use crate::measures::{THE_MORE_APPS_HEADING_IS_TALL, THE_PICKERS_HEADING_IS_TALL};
    let room = Room::pixels(768 - 76 - 12 - 12);
    let overview = crate::more_apps::TheOverview::of(4, 56, room);
    let picker = the_pickers_panel(40, room);

    assert_eq!(
        overview.the_list_is_tall().as_pixels() - picker.the_list_is_tall().as_pixels(),
        THE_PICKERS_HEADING_IS_TALL - THE_MORE_APPS_HEADING_IS_TALL,
        "the two lists differ by something other than their headings: {} and {}",
        overview.the_list_is_tall().as_pixels(),
        picker.the_list_is_tall().as_pixels()
    );
}

/// **A short output shrinks the list and not the fixed parts.**
#[test]
fn a_short_output_shrinks_the_pickers_list() {
    use crate::measures::MORE_APPS_IS_AT_MOST_TALL;
    for room in [800_u32, 600, 400, 300, 200, 100, 0] {
        let it = the_pickers_panel(40, Room::pixels(room));
        assert!(
            it.height().as_pixels() <= MORE_APPS_IS_AT_MOST_TALL,
            "in {room} of room it was {} tall",
            it.height().as_pixels()
        );
    }
}

/// **Rows are where the design draws them**, and there is none past the last.
#[test]
fn the_rows_sit_where_the_design_draws_them() {
    let picker = the_pickers_panel(3, Room::pixels(1_000));
    let at: Vec<u32> = (0..3)
        .map(|which| picker.where_a_row_sits(which).unwrap().as_pixels())
        .collect();
    assert_eq!(at, vec![0, 114, 228], "a row is 106 tall with 8 between");
    assert_eq!(picker.where_a_row_sits(3), None, "one past the last");
}
