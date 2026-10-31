//! A task window on a display drawn at 200 per cent, held against the same
//! window at 100: twice the size where it is placed, twice the size where it is
//! drawn, a press inside it still finding it, and the one-to-one window the
//! same bytes it was before it took a scale.
//!
//! **Sizes and places only.** What a press on the window's edge does is the
//! edge's (`crate::a_task_windows_edge`), and what a reader is told about it is
//! the accessibility tree's; neither is held here.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported"
)]

use alo_models::InferenceSource;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::utils::{Physical, Point, Rectangle};

use super::tests::{a_look, the_words, two_files, what_was_said};
use super::{TaskPicture, furniture};
use crate::display_scale::DisplayScale;
use crate::the_tasks_a_person_opened::TheTasks;

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

/// The window the design draws, at 120, 80 and 1040 × 800 on a one-to-one
/// display.
fn as_drawn() -> Rectangle<i32, Physical> {
    Rectangle::new((120, 80).into(), (1040, 800).into())
}

/// The shell's own faces and nothing else.
///
/// **Not `FontSystem::new`**, which also reads the machine's own fonts, so a
/// machine with other fonts installed draws other letters from the same code —
/// `crate::dock_raster_scale_tests` found that on CI.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .map(|labels| labels.fonts)
        .unwrap_or_else(|_| unreachable!("the shell's own fonts load"))
}

/// The design's window, with two files shared and three things said, at `at`
/// on a display drawn at `scale`.
fn drawn(at: Rectangle<i32, Physical>, scale: DisplayScale) -> TaskPicture {
    let files = two_files();
    let said = what_was_said();
    furniture(
        at,
        a_look(),
        2,
        &the_words(&files),
        &said,
        alo_tasks::TheScroll::at_the_newest(),
        alo_tasks::TheOneButton::Stop,
        &mut fonts(),
        scale,
    )
    .expect("the design's size, at either scale")
}

/// One FNV-1a hash of the panel's raster and every word on it.
fn hashed(window: &TaskPicture) -> u64 {
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
    area(window.panel.area, &mut eat);
    eat(&window.panel.pixels);
    for one in &window.inked {
        area(one.area, &mut eat);
        for pixel in &one.pixels {
            eat(pixel);
        }
    }
    hash
}

/// **At 100 the window is the bytes it drew before it took a scale.**
///
/// The panel with its shadow, chips, field, controls, bubbles and every word.
/// The hash was taken once, by hand, on 2026-10-28 from this fixture run
/// against `main` at `a65c898e` without this change, drawn with the shell's own
/// fonts, and this test
/// repeats the comparison on every build.
#[test]
fn at_one_hundred_the_window_draws_the_bytes_it_drew_before() {
    let now = hashed(&drawn(as_drawn(), DisplayScale::ONE));
    assert_eq!(
        now, BEFORE,
        "the task window at 100 drew {now:#018x}, before {BEFORE:#018x}"
    );
}

/// The hash of the window before this change. See the test above.
const BEFORE: u64 = 0xeb6a_85d1_d119_4c7d;

/// **At 200 the panel and its shadow are exactly twice**, the window measures
/// the same conversation in the same logical units, and every word is there
/// and about twice as tall — type is shaped, not multiplied.
#[test]
fn at_two_hundred_the_drawn_window_is_twice_the_size() {
    let sparse = drawn(as_drawn(), DisplayScale::ONE);
    let dense = drawn(doubled(as_drawn()), two_times());
    assert_eq!(dense.at, doubled(sparse.at), "the window");
    assert_eq!(
        dense.panel.area,
        doubled(sparse.panel.area),
        "the panel and its shadow"
    );
    assert_eq!(
        dense.the_conversation.1, sparse.the_conversation.1,
        "the conversation's view, in the layout's own units"
    );
    assert!(!sparse.inked.is_empty(), "the premise: there are words");
    assert_eq!(dense.inked.len(), sparse.inked.len(), "every line is drawn");
    for (two, one) in dense.inked.iter().zip(&sparse.inked) {
        let (two, one) = (f64::from(two.area.size.h), f64::from(one.area.size.h));
        assert!(
            (1.7..=2.3).contains(&(two / one)),
            "a line is {two} tall at 200 and {one} at 100"
        );
    }
}

/// Where the first task window goes on a view of `size` at `per_cent`, and the
/// tasks it was placed in.
fn placed(size: (i32, i32), per_cent: u16) -> (TheTasks, Rectangle<i32, Physical>) {
    let mut tasks = TheTasks::none_yet();
    let _first = tasks.and_now_this_one(ATask::asked_for(
        "Prepare the launch presentation",
        InferenceSource::ThisMachine,
    ));
    let view = Rectangle::new((0, 0).into(), size.into());
    assert_eq!(
        tasks.put_whatever_has_nowhere_to_be(view, Direction::LeftToRight, &[], per_cent),
        1,
        "the window was not placed on {size:?} at {per_cent}"
    );
    let (_, at) = tasks
        .each_open_window()
        .into_iter()
        .next()
        .expect("one window, placed");
    (tasks, at)
}

/// **At 200 a task window asks for twice the pixels**, on a display with twice
/// the pixels each way — the design's 1040 × 800 becomes 2080 × 1600 — and it
/// still leaves room above it for its edge at that scale.
#[test]
fn at_two_hundred_a_task_window_is_placed_twice_the_size() {
    let (_, sparse) = placed((1920, 1080), 100);
    let (_, dense) = placed((3840, 2160), 200);
    assert_eq!((sparse.size.w, sparse.size.h), (1040, 800));
    assert_eq!((dense.size.w, dense.size.h), (2080, 1600));
    let edge = 2 * crate::window_edge::THE_REGION_IS_TALL;
    assert!(
        dense.loc.y >= edge,
        "the window at {dense:?} leaves no room for its edge, {edge} tall at 200"
    );
}

/// **A press anywhere inside a window placed at 200 finds that window**: its
/// first pixel, its middle and its last — the rectangle a press is tested
/// against is the rectangle it is drawn at, in the same pixels.
#[test]
fn a_press_inside_a_window_at_two_hundred_finds_it() {
    let (tasks, at) = placed((3840, 2160), 200);
    let drawn_at = drawn(at, two_times()).at;
    assert_eq!(drawn_at, at, "drawn where it was placed");
    for point in [
        at.loc,
        Point::from((at.loc.x + at.size.w / 2, at.loc.y + at.size.h / 2)),
        Point::from((at.loc.x + at.size.w - 1, at.loc.y + at.size.h - 1)),
    ] {
        assert!(
            tasks.the_one_under(point).is_some(),
            "a press at {point:?} inside {at:?} found no window"
        );
    }
    let outside = Point::from((at.loc.x + at.size.w, at.loc.y));
    assert!(
        tasks.the_one_under(outside).is_none(),
        "the control: a press just past the window found one"
    );
}

/// **Two displays at two scales each get a window laid out for their own**: a
/// two-times laptop and a one-to-one monitor beside it.
#[test]
fn two_displays_at_two_scales_each_get_a_window_of_their_own() {
    let (_, laptop) = placed((2560, 1600), 200);
    let (_, monitor) = placed((1920, 1080), 100);
    assert_eq!(monitor.size.w, 1040);
    assert_eq!(laptop.size.w, 2 * monitor.size.w);
    let on_the_laptop = drawn(laptop, two_times());
    let on_the_monitor = drawn(monitor, DisplayScale::ONE);
    let (Some(big), Some(small)) = (on_the_laptop.inked.first(), on_the_monitor.inked.first())
    else {
        unreachable!("a window with a title drew no words");
    };
    let ratio = f64::from(big.area.size.h) / f64::from(small.area.size.h);
    assert!(
        (1.7..=2.3).contains(&ratio),
        "the laptop's title is {ratio} times the monitor's, not about twice"
    );
}
