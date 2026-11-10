//! Walking the window panel's choices, and landing back on Window actions.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{AMenuKey, TheKeyboardInTheMenu, WhatTheKeyDid, the_edge_stays_revealed};
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_menu::{HowTheWindowIs, WhatARowDoes, WindowMenu, the_menu_of};
use alo_dock::revealing::AMenu;
use smithay::utils::{Point, Rectangle, Size};

/// The Notes window's panel.
fn a_panel() -> WindowMenu {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let room = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, room);
    the_menu_of(&edge, HowTheWindowIs::default(), room).expect("a panel")
}

/// A panel just opened from Window actions.
fn opened() -> TheKeyboardInTheMenu {
    TheKeyboardInTheMenu::opened_from(OnTheEdge::WindowActions)
}

/// What each of these keys did, in turn.
fn pressing(keys: &[AMenuKey], menu: &WindowMenu) -> Vec<WhatTheKeyDid> {
    let mut keyboard = opened();
    keys.iter().map(|key| keyboard.told(*key, menu)).collect()
}

/// **The panel opens with the keyboard on no choice**, so nothing is in hand
/// until the person moves.
#[test]
fn a_panel_opens_with_the_keyboard_on_no_choice() {
    assert_eq!(opened().on(), None);
    assert_eq!(opened().trigger(), OnTheEdge::WindowActions);
}

/// **Down walks every choice in the keyboard order, and wraps.**
#[test]
fn down_walks_every_choice_and_wraps() {
    let panel = a_panel();
    assert_eq!(
        pressing(&[AMenuKey::Next; 5], &panel),
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Left),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Maximise),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Right),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::FullScreen),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Left),
        ]
    );
}

/// **Up from nothing reaches the last choice.**
#[test]
fn up_from_no_choice_reaches_the_last() {
    assert_eq!(
        pressing(&[AMenuKey::Previous], &a_panel()),
        vec![WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::FullScreen)]
    );
}

/// **Home and End reach the first and the last.**
#[test]
fn home_and_end_reach_the_first_and_the_last() {
    assert_eq!(
        pressing(&[AMenuKey::Last, AMenuKey::First], &a_panel()),
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::FullScreen),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Left),
        ]
    );
}

/// **Enter does the choice in hand and gives focus back to Window actions.**
#[test]
fn enter_does_the_choice_in_hand_and_returns_focus() {
    assert_eq!(
        pressing(&[AMenuKey::Next, AMenuKey::Run], &a_panel()).last(),
        Some(&WhatTheKeyDid::ItRuns {
            does: WhatARowDoes::Left,
            focus_goes_back_to: OnTheEdge::WindowActions,
        })
    );
}

/// **Enter before anything is in hand does nothing**, rather than guessing.
#[test]
fn enter_before_any_choice_is_in_hand_does_nothing() {
    assert_eq!(
        pressing(&[AMenuKey::Run], &a_panel()),
        vec![WhatTheKeyDid::Nothing]
    );
}

/// **Escape closes with nothing done, and gives focus back.**
#[test]
fn escape_closes_with_nothing_done_and_returns_focus() {
    assert_eq!(
        pressing(&[AMenuKey::Next, AMenuKey::Close], &a_panel()).last(),
        Some(&WhatTheKeyDid::ItCloses {
            focus_goes_back_to: OnTheEdge::WindowActions,
        })
    );
}

/// **An open panel is what keeps the band revealed**, and none is not.
#[test]
fn an_open_panel_keeps_the_band_revealed() {
    assert_eq!(the_edge_stays_revealed(Some(&opened())), AMenu::IsOpen);
    assert_eq!(the_edge_stays_revealed(None), AMenu::IsNot);
}

/// **A panel a key opened has the keyboard on its first choice**, so the
/// keyboard is never somewhere a person cannot see, and Down moves on from it.
#[test]
fn a_panel_a_key_opened_starts_on_the_first_choice() {
    let menu = a_panel();
    let mut keyboard = TheKeyboardInTheMenu::opened_by_the_keyboard(OnTheEdge::WindowActions);
    assert_eq!(keyboard.on(), Some(crate::window_menu::THE_FIRST_CHOICE));
    assert_eq!(
        menu.what_can_be_used().first().map(|row| row.does),
        Some(crate::window_menu::THE_FIRST_CHOICE),
        "the first choice is the one laid out first"
    );
    assert_eq!(
        keyboard.told(AMenuKey::Next, &menu),
        WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Maximise)
    );
    assert_eq!(keyboard.trigger(), OnTheEdge::WindowActions);
}
