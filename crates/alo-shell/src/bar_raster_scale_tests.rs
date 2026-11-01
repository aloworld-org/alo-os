//! The launcher on a display drawn at 200 per cent, held against the same
//! launcher at 100: every place exactly twice, and the panel and its shadow
//! twice the pixels.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::{Physical, Rectangle, Size};

use super::{BarPicture, TheBarOnTheDisplay, TheLaunchersWords, picture};
use crate::display_scale::DisplayScale;
use crate::the_bar_finds::{AResult, WhatARowDoes};

/// The size the design's numbers were read at.
const SPARSE: (i32, i32) = (1440, 960);
/// The same display with twice the pixels each way.
const DENSE: (i32, i32) = (2880, 1920);

/// Four applications found.
fn found() -> Vec<AResult> {
    ["Files", "Terminal", "Settings", "Mail"]
        .iter()
        .map(|name| AResult {
            shown: format!("{name}  \u{b7}  Application"),
            name: (*name).to_owned(),
            kind: "Application".to_owned(),
            does: WhatARowDoes::OpensAnApplication((*name).to_owned()),
        })
        .collect()
}

/// The launcher on a display of `size` at `scale`, the keyboard on row two.
fn drawn(size: (i32, i32), scale: DisplayScale) -> BarPicture {
    let results = found();
    let look = crate::desktop_testing::noon_look(
        &crate::desktop_testing::an_appearance(),
        alo_strings::Direction::LeftToRight,
    );
    let mut fonts = crate::WindowControlLabels::new().unwrap().fonts;
    picture(
        &TheBarOnTheDisplay {
            typed: "fil",
            origin: "On this computer",
            results: &results,
            chosen: Some(1),
            display: Size::from(size),
            words: TheLaunchersWords {
                heading: "Find, open, or ask",
                open_it: "Open",
                choose: "choose",
                open: "open",
                close: "close",
                esc: "Esc",
            },
            asks: None,
            artwork: None,
        },
        look.palette().unwrap(),
        [0xFA, 0xF7, 0xF2],
        &mut fonts,
        scale,
    )
    .unwrap()
}

/// A rectangle twice as far from the corner and twice the size.
fn doubled(it: Rectangle<i32, Physical>) -> Rectangle<i32, Physical> {
    Rectangle::new(
        (2 * it.loc.x, 2 * it.loc.y).into(),
        (2 * it.size.w, 2 * it.size.h).into(),
    )
}

/// **At 200 every place is twice itself and the same rows are shown.**
#[test]
fn at_two_hundred_every_place_in_the_launcher_is_exactly_twice() {
    let sparse = drawn(SPARSE, DisplayScale::ONE);
    let dense = drawn(DENSE, DisplayScale::of_per_cent(200));
    let (one, two) = (&sparse.places, &dense.places);
    assert_eq!(two.panel, doubled(one.panel));
    assert_eq!(two.close, doubled(one.close));
    assert_eq!(two.field, doubled(one.field));
    assert_eq!(two.keys, doubled(one.keys));
    assert_eq!(
        two.rows.iter().map(|(which, _)| *which).collect::<Vec<_>>(),
        one.rows.iter().map(|(which, _)| *which).collect::<Vec<_>>()
    );
    assert_eq!(
        dense.panel.area,
        doubled(sparse.panel.area),
        "the panel and its shadow"
    );
}
