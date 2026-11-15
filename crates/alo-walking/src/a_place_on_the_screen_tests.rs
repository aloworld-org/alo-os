//! Finding a place to point at in a picture: a colour, what changed, and the
//! middle — and nothing where there is nothing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a place not found is the failure being reported"
)]

use super::{Target, where_is};
use crate::frame::Frame;

/// A grey screen.
const GREY: [u8; 3] = [128, 128, 128];

/// Magenta, as the walk's page draws its text field.
const MAGENTA: [u8; 3] = [255, 0, 255];

/// `frame` with a rectangle of `rgb` from `(x, y)`, `w` by `h`.
fn with_block(mut frame: Frame, (x, y, w, h): (usize, usize, usize, usize), rgb: [u8; 3]) -> Frame {
    for py in y..y + h {
        for px in x..x + w {
            frame = frame.with_pixel(px, py, rgb);
        }
    }
    frame
}

/// **A colour is aimed at inside its patch**, and the larger of two patches
/// wins.
#[test]
fn a_colour_is_aimed_at_inside_its_largest_patch() {
    let screen = with_block(
        with_block(Frame::filled(320, 200, GREY), (16, 16, 48, 48), MAGENTA),
        (160, 64, 128, 96),
        MAGENTA,
    );
    let Some((x, y)) = where_is(Target::Colour(MAGENTA), &screen, None) else {
        panic!("no magenta found");
    };
    assert!(
        (160..288).contains(&x) && (64..160).contains(&y),
        "aimed at ({x}, {y})"
    );
}

/// **What changed is the window that opened**, not the few pixels of a clock
/// that also changed.
#[test]
fn what_changed_is_the_large_patch_not_the_small_one() {
    let before = Frame::filled(320, 200, GREY);
    let now = with_block(
        with_block(before.clone(), (300, 0, 8, 8), [0, 0, 0]),
        (40, 40, 160, 120),
        [250, 250, 250],
    );
    let Some((x, y)) = where_is(Target::WhatChanged, &now, Some(&before)) else {
        panic!("no change found");
    };
    assert!(
        (40..200).contains(&x) && (40..160).contains(&y),
        "aimed at ({x}, {y})"
    );
}

/// **An L-shaped patch is aimed at inside it**, never at the empty corner its
/// bounding box would put the middle in.
#[test]
fn an_l_shaped_patch_is_aimed_at_inside_it() {
    let screen = with_block(
        with_block(Frame::filled(320, 320, GREY), (0, 0, 320, 64), MAGENTA),
        (0, 0, 64, 320),
        MAGENTA,
    );
    let Some((x, y)) = where_is(Target::Colour(MAGENTA), &screen, None) else {
        panic!("no magenta found");
    };
    assert_eq!(screen.rgb(x, y), Some(MAGENTA), "aimed at ({x}, {y})");
}

/// **The refusals**: a colour that is not there, a patch too small to aim at,
/// nothing changed, and no picture kept to compare with, each find nothing.
#[test]
fn nothing_there_is_nowhere() {
    let screen = Frame::filled(320, 200, GREY);
    assert_eq!(where_is(Target::Colour(MAGENTA), &screen, None), None);
    let speck = with_block(screen.clone(), (32, 32, 8, 8), MAGENTA);
    assert_eq!(where_is(Target::Colour(MAGENTA), &speck, None), None);
    assert_eq!(where_is(Target::WhatChanged, &screen, Some(&screen)), None);
    assert_eq!(where_is(Target::WhatChanged, &screen, None), None);
}

/// The middle is the middle.
#[test]
fn the_middle_is_the_middle() {
    assert_eq!(
        where_is(Target::Middle, &Frame::filled(1280, 800, GREY), None),
        Some((640, 400))
    );
}

/// **A button with its label across the middle is found**: Firefox's
/// *Continue*, 92 by 32 pixels with white letters through its middle, is aimed
/// at on its colour, not on a letter.
#[test]
fn a_button_with_a_label_is_aimed_at_on_its_colour() {
    let button = with_block(
        with_block(Frame::filled(320, 200, GREY), (138, 80, 92, 32), MAGENTA),
        (150, 91, 68, 11),
        [255, 255, 255],
    );
    let Some((x, y)) = where_is(Target::Colour(MAGENTA), &button, None) else {
        panic!("the button was not found");
    };
    assert_eq!(button.rgb(x, y), Some(MAGENTA), "aimed at ({x}, {y})");
}
