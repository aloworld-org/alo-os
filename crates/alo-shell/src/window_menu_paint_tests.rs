//! What the window panel draws.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use crate::window_menu::{HowTheWindowIs, the_menu_of};
use smithay::utils::{Point, Size};

/// The Notes window's open panel.
fn a_panel() -> WindowMenu {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let roomy = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, roomy);
    the_menu_of(&edge, HowTheWindowIs::default(), roomy).expect("a panel")
}

/// The fonts every drawing shapes with.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// **The body is drawn first and reaches round the whole panel**, so every
/// choice is drawn on it and its shadow falls outside it.
#[test]
fn the_body_is_drawn_first_and_holds_the_whole_panel() {
    let menu = a_panel();
    let (rasters, _) = the_panel_drawn(&menu, PointingAtTheMenu::default(), &mut fonts(), 100, (4000, 3000));
    let body = rasters.first().expect("a body").area;
    let panel = in_pixels(menu.panel, 100);
    assert!(body.contains_rect(panel), "{body:?} does not hold {panel:?}");
    assert!(body.size.h > panel.size.h, "the shadow reaches below the panel");
}

/// **The heading and one word per choice are written**, and nothing else.
#[test]
fn the_heading_and_each_choice_are_written() {
    let menu = a_panel();
    let (_, words) = the_panel_drawn(&menu, PointingAtTheMenu::default(), &mut fonts(), 100, (4000, 3000));
    assert_eq!(words.len(), 1 + menu.rows.len());
}

/// **The choice in hand is the keyboard's before the pointer's**, so a person
/// moving the pointer while using the keys does not lose their place.
#[test]
fn the_keyboard_s_choice_is_the_one_in_hand() {
    let both = PointingAtTheMenu {
        hovered: Some(WhatARowDoes::FullScreen),
        focused: Some(WhatARowDoes::Maximise),
    };
    assert_eq!(both.in_hand(), Some(WhatARowDoes::Maximise));
    let pointer_only = PointingAtTheMenu {
        hovered: Some(WhatARowDoes::FullScreen),
        focused: None,
    };
    assert_eq!(pointer_only.in_hand(), Some(WhatARowDoes::FullScreen));
}

/// **The keyboard's choice gets its navy ring, as well as its colour** — one
/// more raster than the same panel with nothing focused.
#[test]
fn the_keyboard_s_choice_is_ringed_and_not_told_by_colour_alone() {
    let menu = a_panel();
    let (plain, _) = the_panel_drawn(&menu, PointingAtTheMenu::default(), &mut fonts(), 100, (4000, 3000));
    let focused = PointingAtTheMenu {
        hovered: None,
        focused: Some(WhatARowDoes::FullScreen),
    };
    let (ringed, _) = the_panel_drawn(&menu, focused, &mut fonts(), 100, (4000, 3000));
    assert_eq!(ringed.len(), plain.len() + 1);
}

/// **The expand mark reads**, so Full screen is never drawn without it.
#[test]
fn the_full_screen_mark_reads() {
    assert!(the_full_screen_mark().is_some());
}
