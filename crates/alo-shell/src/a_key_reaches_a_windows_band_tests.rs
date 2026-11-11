//! Which keys a band and its panel take — and that every other key is the
//! application's.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// **The keys the specification names are the band's**: arrows, Home, End,
/// Enter, Space and Escape.
#[test]
fn the_keys_the_specification_names_are_taken() {
    assert_eq!(which_key(codes::ESCAPE), Some(AnEdgeKey::Leave));
    for next in [codes::RIGHT, codes::DOWN] {
        assert_eq!(which_key(next), Some(AnEdgeKey::Next));
    }
    for back in [codes::LEFT, codes::UP] {
        assert_eq!(which_key(back), Some(AnEdgeKey::Previous));
    }
    assert_eq!(which_key(codes::HOME), Some(AnEdgeKey::First));
    assert_eq!(which_key(codes::END), Some(AnEdgeKey::Last));
    for press in [codes::ENTER, codes::PAD_ENTER, codes::SPACE] {
        assert_eq!(which_key(press), Some(AnEdgeKey::Press));
    }
}

/// **Every other key is the application's** — a letter, a digit, Backspace —
/// and Tab is the shell's, between surfaces.
#[test]
fn every_other_key_goes_on_to_the_application() {
    // A, 1, Backspace, F3, Tab.
    for code in [30, 2, 14, 61, 15] {
        assert_eq!(which_key(code), None, "{code}");
    }
}

/// **`Alt+F3` then Right reaches Maximise**: the boot walk of 2026-11-09, on
/// the road the machine takes — the state `Alt+F3` leaves
/// ([`crate::the_edge_takes_the_keyboard::once_opened`], by the keyboard), the
/// evdev code read by [`which_key`], and the band's own machine answering it.
///
/// The walk pressed Right once after `Alt+F3` and saw *Left*, and Down once and
/// saw *Left*, because the panel opened on no choice and the first arrow only
/// reached the first. What is not on this road is the `WlSurface` lookup in
/// `Server::a_key_on_a_band`, which needs a client to reach.
#[test]
fn alt_f3_then_right_reaches_maximise_and_down_walks_on() {
    use crate::the_edge_takes_the_keyboard::{ByWhat, once_opened};
    use crate::window_menu::{HowTheWindowIs, WhatARowDoes, the_menu_of};
    use smithay::utils::{Point, Rectangle, Size};

    let room = Size::from((1280, 800));
    let band = crate::window_edge::edge_of(
        Rectangle::new(Point::from((0, 84)), Size::from((584, 600))),
        crate::window_edge::Decorations::TheShellDraws,
        true,
        room,
    );
    let panel = the_menu_of(
        &band,
        HowTheWindowIs {
            maximised: true,
            full_screen: false,
            ..HowTheWindowIs::default()
        },
        room,
    );
    let mut keyboard = once_opened(TheKeyboardOnTheEdge::Nowhere, ByWhat::TheKeyboard);
    let mut press = |code: u32| {
        let key = which_key(code).expect("a key the band takes");
        keyboard.told(key, &band, panel.as_ref())
    };
    assert_eq!(
        press(codes::RIGHT),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Restore)
    );
    assert_eq!(
        press(codes::DOWN),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Right)
    );
    assert_eq!(
        press(codes::DOWN),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::FullScreen)
    );
    assert_eq!(
        press(codes::LEFT),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Right)
    );
}

/// **A panel the pointer opened has no choice in hand**, so the first arrow
/// lands on the first choice — no ring nobody asked for.
#[test]
fn a_panel_the_pointer_opened_starts_on_no_choice() {
    use crate::the_edge_takes_the_keyboard::{ByWhat, once_opened};
    let opened = once_opened(TheKeyboardOnTheEdge::Nowhere, ByWhat::ThePointer);
    assert_eq!(
        opened
            .the_menu()
            .and_then(crate::window_menu_keys::TheKeyboardInTheMenu::on),
        None
    );
    let again = once_opened(opened, ByWhat::ThePointer);
    assert_eq!(
        again,
        TheKeyboardOnTheEdge::Nowhere,
        "asked again, it closes"
    );
}
