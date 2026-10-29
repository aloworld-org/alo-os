//! The alo Bar on a display drawn at 200 per cent, held against the same Bar at
//! 100: every measure exactly twice, the panel and its shadow twice, its words
//! larger, and the one-to-one Bar the same bytes it was before it took a scale.
//!
//! **There is no press to hold here**, and that is a fact about the Bar rather
//! than a gap in these tests: nothing in `crate::pointer` hit-tests the Bar's
//! rows. A person moves among them with the arrow keys and Enter
//! (`crate::the_composer_a_person_types_in`), which reads the row by its number
//! and never by a position, so a scale cannot move what a key chooses.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle, Size};

use super::{BarPicture, TheBarOnTheDisplay, picture};
use crate::a_picture_hashed::Hashed;
use crate::display_scale::DisplayScale;
use crate::the_bar_finds::{AResult, WhatARowDoes};
use crate::where_the_bar_opens::where_the_bar_opens;

/// The size the design's numbers were read at, at one to one.
const SPARSE: (i32, i32) = (1440, 960);
/// The same display with twice the pixels each way.
const DENSE: (i32, i32) = (2880, 1920);

/// The dense display's scale.
fn two_times() -> DisplayScale {
    DisplayScale::of_per_cent(200)
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

/// Four applications found, more than fit, so the list is the design's.
fn found() -> Vec<AResult> {
    ["Files", "Terminal", "Settings", "Mail"]
        .iter()
        .map(|name| AResult {
            shown: format!("{name} · On this computer"),
            does: WhatARowDoes::OpensAnApplication((*name).to_owned()),
        })
        .collect()
}

/// The Bar drawn on a display of `size` at `scale`, with the keyboard on the
/// second row.
fn drawn(size: (i32, i32), scale: DisplayScale) -> BarPicture {
    let results = found();
    let look = crate::desktop_testing::noon_look(
        &crate::desktop_testing::an_appearance(),
        Direction::LeftToRight,
    );
    picture(
        &TheBarOnTheDisplay {
            typed: "fil",
            origin: "On this computer",
            results: &results,
            chosen: Some(1),
            display: Size::from(size),
        },
        look.palette().unwrap(),
        &mut fonts(),
        scale,
    )
    .unwrap()
}

/// Everything a Bar picture paints, as one hash.
fn hashed(bar: &BarPicture) -> u64 {
    let mut hash = Hashed::begun();
    hash.washed(&bar.panel);
    hash.inked(&bar.inked);
    hash.done()
}

/// A rectangle twice as far from the corner and twice the size.
fn doubled(it: Rectangle<i32, Physical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (2 * it.loc.x, 2 * it.loc.y).into(),
        (2 * it.size.w, 2 * it.size.h).into(),
    )
}

/// **At 100 the Bar is the bytes it drew before it took a scale.**
///
/// The panel with its shadow, its wells, the keyboard's ring and every word, on
/// a 1920 × 1080 display. The hash was taken once, by hand, on 2026-10-28 from
/// this fixture run against `main` at `a65c898e` without this change, drawn
/// with the shell's own fonts, and this test repeats the comparison on every
/// build.
#[test]
fn at_one_hundred_the_bar_draws_the_bytes_it_drew_before() {
    let now = hashed(&drawn((1920, 1080), DisplayScale::ONE));
    assert_eq!(
        now, BEFORE,
        "the Bar at 100 drew {now:#018x}, before {BEFORE:#018x}"
    );
}

/// The hash of the Bar before this change. See the test above.
const BEFORE: u64 = 0x3c8c_c3f2_1d67_79b2;

/// **At 200 every measure is exactly twice itself**, and so is every share of
/// the display, on the display the design was drawn at and the same display
/// with twice the pixels. The rows that fit are the same rows.
#[test]
fn at_two_hundred_every_place_in_the_bar_is_exactly_twice() {
    let sparse = where_the_bar_opens(Size::from(SPARSE), DisplayScale::ONE).unwrap();
    let dense = where_the_bar_opens(Size::from(DENSE), two_times()).unwrap();
    assert_eq!(dense.panel, doubled(sparse.panel), "the panel");
    assert_eq!(dense.field, doubled(sparse.field), "the field");
    assert_eq!(dense.origin, doubled(sparse.origin), "the line under it");
    assert_eq!(dense.first_row, doubled(sparse.first_row), "the first row");
    assert_eq!(dense.row_to_row, 2 * sparse.row_to_row, "row to row");
    assert_eq!(dense.rows_that_fit, sparse.rows_that_fit, "the rows shown");
    assert_eq!(dense.scale, two_times());
    // The control: the field at 100 is the design's 56, so *twice* is 112 and
    // not twice some other number that happened to double.
    assert_eq!((sparse.field.size.h, dense.field.size.h), (56, 112));
}

/// **What is drawn doubles too**: the panel and its shadow exactly, and every
/// word is there and larger — a line of type is shaped, not multiplied, so its
/// height is held to near twice rather than to exactly.
#[test]
fn at_two_hundred_the_drawn_bar_is_twice_the_size() {
    let sparse = drawn(SPARSE, DisplayScale::ONE);
    let dense = drawn(DENSE, two_times());
    assert_eq!(
        dense.panel.area,
        doubled(sparse.panel.area),
        "the panel and its shadow"
    );
    assert!(!sparse.inked.is_empty(), "the premise: there are words");
    assert_eq!(dense.inked.len(), sparse.inked.len(), "every word is drawn");
    for (two, one) in dense.inked.iter().zip(&sparse.inked) {
        let (two, one) = (f64::from(two.area.size.h), f64::from(one.area.size.h));
        assert!(
            (1.8..=2.2).contains(&(two / one)),
            "a line is {two} tall at 200 and {one} at 100"
        );
    }
    for word in &dense.inked {
        assert!(
            dense.panel.area.contains_rect(word.area),
            "{:?} is drawn outside the panel {:?}",
            word.area,
            dense.panel.area
        );
    }
}

/// **Two displays at two scales each get a Bar laid out for their own**: a
/// two-times laptop and a one-to-one monitor beside it, as `crate::desktop_raster`
/// draws each display with that display's own scale.
#[test]
fn two_displays_at_two_scales_each_get_a_bar_of_their_own() {
    let laptop = where_the_bar_opens(Size::from((2560, 1600)), two_times()).unwrap();
    let monitor = where_the_bar_opens(Size::from((1920, 1080)), DisplayScale::ONE).unwrap();
    assert_eq!(laptop.field.size.h, 112);
    assert_eq!(monitor.field.size.h, 56);
    assert_eq!(laptop.scale, two_times());
    assert_eq!(monitor.scale, DisplayScale::ONE);
}
