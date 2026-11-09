//! The keyboard on a band and in its panel: reaching, walking, opening,
//! choosing and giving focus back.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use crate::window_menu::{HowTheWindowIs, WhatARowDoes, WindowMenu, the_menu_of};
use alo_dock::revealing::{AMenu, TheKeyboard};
use smithay::utils::{Logical, Point, Rectangle, Size};

/// A display large enough that nothing is clamped.
fn an_output() -> Size<i32, Logical> {
    Size::from((4000, 3000))
}

/// The Notes window's band, revealed.
fn a_band() -> WindowEdge {
    edge_of(
        Rectangle::new(Point::from((112, 314)), Size::from((1056, 466))),
        Decorations::TheShellDraws,
        true,
        an_output(),
    )
}

/// Its panel.
fn a_panel() -> WindowMenu {
    the_menu_of(&a_band(), HowTheWindowIs::default(), an_output()).expect("a panel")
}

/// **Nothing has the keyboard until something gives it**, and then no ring
/// is drawn and nothing is held revealed.
#[test]
fn a_band_nothing_has_reached_shows_no_focus_ring_and_holds_nothing() {
    let keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(keyboard, TheKeyboardOnTheEdge::Nowhere);
    assert_eq!(keyboard.what_the_painter_shows(), None);
    assert_eq!(
        keyboard.whether_the_keyboard_holds_it(),
        TheKeyboard::IsElsewhere
    );
}

/// **Reaching the band lands on Window actions**, the first control.
#[test]
fn reaching_the_band_lands_on_window_actions() {
    let mut keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(
        keyboard.reached(&a_band()),
        WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::WindowActions)
    );
}

/// **The arrows walk the three in the order they are drawn, and wrap.**
#[test]
fn the_keys_walk_the_three_controls_in_drawn_order_and_wrap() {
    let band = a_band();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let walked: Vec<WhatTheEdgeKeyDid> = (0..3)
        .map(|_| keyboard.told(AnEdgeKey::Next, &band, None))
        .collect();
    assert_eq!(
        walked,
        vec![
            WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::Minimise),
            WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::Close),
            WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::WindowActions),
        ]
    );
    assert_eq!(
        keyboard.told(AnEdgeKey::Previous, &band, None),
        WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::Close)
    );
}

/// **Enter on Window actions opens the panel**, and the ring stays on it.
#[test]
fn pressing_window_actions_opens_the_panel() {
    let band = a_band();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &band, None),
        WhatTheEdgeKeyDid::TheMenuOpened
    );
    assert_eq!(
        keyboard.what_the_painter_shows(),
        Some(OnTheEdge::WindowActions)
    );
    assert_eq!(keyboard.whether_a_menu_holds_it(), AMenu::IsOpen);
}

/// **Enter on Minimise or Close answers that control**, for the caller to do.
#[test]
fn pressing_minimise_or_close_answers_the_control() {
    let band = a_band();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let _ = keyboard.told(AnEdgeKey::Next, &band, None);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &band, None),
        WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Minimise)
    );
    let _ = keyboard.told(AnEdgeKey::Last, &band, None);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &band, None),
        WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Close)
    );
}

/// **Inside the panel the keys walk its choices**, and Enter does the one in
/// hand and gives focus back to Window actions.
#[test]
fn inside_the_panel_the_keys_walk_and_enter_chooses() {
    let (band, panel) = (a_band(), a_panel());
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let _ = keyboard.told(AnEdgeKey::Press, &band, None);
    assert_eq!(
        keyboard.told(AnEdgeKey::Next, &band, Some(&panel)),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Left)
    );
    assert_eq!(
        keyboard.told(AnEdgeKey::Next, &band, Some(&panel)),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Maximise)
    );
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &band, Some(&panel)),
        WhatTheEdgeKeyDid::ARowRuns(WhatARowDoes::Maximise)
    );
    assert_eq!(
        keyboard,
        TheKeyboardOnTheEdge::OnAControl(OnTheEdge::WindowActions),
        "focus goes back to the opener"
    );
}

/// **Escape closes the deepest layer first**: the panel, then the band.
#[test]
fn escape_closes_the_panel_and_then_leaves_the_band() {
    let (band, panel) = (a_band(), a_panel());
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let _ = keyboard.told(AnEdgeKey::Press, &band, None);
    let _ = keyboard.told(AnEdgeKey::Next, &band, Some(&panel));
    assert_eq!(
        keyboard.told(AnEdgeKey::Leave, &band, Some(&panel)),
        WhatTheEdgeKeyDid::TheMenuClosed
    );
    assert_eq!(
        keyboard,
        TheKeyboardOnTheEdge::OnAControl(OnTheEdge::WindowActions)
    );
    assert_eq!(
        keyboard.told(AnEdgeKey::Leave, &band, None),
        WhatTheEdgeKeyDid::TheKeyboardLeft
    );
    assert_eq!(keyboard, TheKeyboardOnTheEdge::Nowhere);
}

/// **Escape in the panel chooses nothing** — no choice is answered.
#[test]
fn escape_in_the_panel_chooses_nothing() {
    let (band, panel) = (a_band(), a_panel());
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let _ = keyboard.told(AnEdgeKey::Press, &band, None);
    let _ = keyboard.told(AnEdgeKey::Next, &band, Some(&panel));
    let did = keyboard.told(AnEdgeKey::Leave, &band, Some(&panel));
    assert!(!matches!(did, WhatTheEdgeKeyDid::ARowRuns(_)), "{did:?}");
}

/// **A key on a band nothing reached does nothing**, so a key never acts on a
/// window the person was not on.
#[test]
fn keys_do_nothing_on_a_band_the_keyboard_has_not_reached() {
    let band = a_band();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    for key in [
        AnEdgeKey::Next,
        AnEdgeKey::Previous,
        AnEdgeKey::First,
        AnEdgeKey::Last,
        AnEdgeKey::Press,
        AnEdgeKey::Leave,
    ] {
        assert_eq!(keyboard.told(key, &band, None), WhatTheEdgeKeyDid::Nothing);
    }
    assert_eq!(keyboard, TheKeyboardOnTheEdge::Nowhere);
}

/// **A key inside a panel that cannot be laid out does nothing**, rather than
/// acting on a choice nobody can see.
#[test]
fn a_key_in_the_panel_with_no_panel_to_read_does_nothing() {
    let band = a_band();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    let _ = keyboard.reached(&band);
    let _ = keyboard.told(AnEdgeKey::Press, &band, None);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &band, None),
        WhatTheEdgeKeyDid::Nothing
    );
}

/// **A concealed band cannot be reached by these keys**: it lays out no
/// controls. The server lays it out revealed before reaching it
/// (`Server::the_keyboard_reached_this_windows_edge`).
#[test]
fn a_concealed_band_has_nothing_to_land_on() {
    let concealed = edge_of(
        Rectangle::new(Point::from((112, 314)), Size::from((1056, 466))),
        Decorations::TheShellDraws,
        false,
        an_output(),
    );
    let mut keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(keyboard.reached(&concealed), WhatTheEdgeKeyDid::Nothing);
}
