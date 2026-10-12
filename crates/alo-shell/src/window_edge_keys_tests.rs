//! Tabbing to an edge, walking it, opening its menu and landing back on the
//! control that opened it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{AnEdgeKey, TheKeyboardOnTheEdge, WhatTheEdgeKeyDid};
use crate::window_edge::{
    Decorations, FULL_CONTROLS_NEED, OnTheEdge, WindowEdge, edge_of, the_usable_output,
};
use crate::window_menu::{
    THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, WindowMenu, the_menu_of,
};
use alo_dock::revealing::{AMenu, Revealing, TheKeyboard};
use smithay::utils::{Logical, Point, Rectangle, Size};

fn an_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// A narrow window, which is the layout with a Menu control.
fn a_compact_edge() -> WindowEdge {
    let window = Rectangle::new(Point::from((400, 200)), Size::from((160, 400)));
    edge_of(window, Decorations::TheShellDraws, true, an_output())
}

/// A wide one, which has no Menu control and so no menu.
fn a_full_edge() -> WindowEdge {
    let window = Rectangle::new(
        Point::from((400, 200)),
        Size::from((FULL_CONTROLS_NEED + 200, 400)),
    );
    edge_of(window, Decorations::TheShellDraws, true, an_output())
}

fn a_menu() -> WindowMenu {
    the_menu_of(
        &a_compact_edge(),
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control")
}

/// **An edge starts with no keyboard on it and no focus ring.**
#[test]
fn an_edge_nothing_has_reached_shows_no_focus_ring() {
    let keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(keyboard, TheKeyboardOnTheEdge::Nowhere);
    assert_eq!(keyboard.what_the_painter_shows(), None);
    assert_eq!(
        keyboard.whether_the_keyboard_holds_it(),
        TheKeyboard::IsElsewhere
    );
    assert_eq!(keyboard.whether_a_menu_holds_it(), AMenu::IsNot);
}

/// Tabbing to the edge lands on its first control.
#[test]
fn reaching_the_edge_lands_on_its_first_control() {
    let edge = a_compact_edge();
    let first = edge
        .controls
        .first()
        .expect("a revealed compact edge has controls")
        .does;
    let mut keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(
        keyboard.reached(&edge),
        WhatTheEdgeKeyDid::TheKeyboardIsOn(first)
    );
    assert_eq!(keyboard.what_the_painter_shows(), Some(first));
}

/// **A concealed edge has nothing to land on.**
///
/// `edge_of` lays a concealed edge out with no controls, so arriving must
/// reveal it first — and answering `TheKeyboardIsOn` for a control that was
/// not laid out would put a focus ring on nothing.
#[test]
fn a_concealed_edge_cannot_be_reached() {
    let window = Rectangle::new(Point::from((400, 200)), Size::from((160, 400)));
    let concealed = edge_of(window, Decorations::TheShellDraws, false, an_output());
    assert!(
        concealed.controls.is_empty(),
        "the fixture is not concealed, so this proves nothing"
    );
    let mut keyboard = TheKeyboardOnTheEdge::default();
    assert_eq!(keyboard.reached(&concealed), WhatTheEdgeKeyDid::Nothing);
    assert_eq!(keyboard.what_the_painter_shows(), None);
}

/// Next walks the controls in the order they are drawn, and wraps.
#[test]
fn next_walks_the_controls_in_the_order_they_are_drawn() {
    let edge = a_compact_edge();
    let drawn: Vec<OnTheEdge> = edge.controls.iter().map(|it| it.does).collect();
    assert_eq!(
        drawn,
        vec![OnTheEdge::Move, OnTheEdge::Menu, OnTheEdge::Close],
        "the compact layout"
    );

    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    let mut walked = vec![
        keyboard
            .what_the_painter_shows()
            .expect("it reached a control"),
    ];
    for _ in 0..drawn.len() {
        keyboard.told(AnEdgeKey::Next, &edge, None);
        walked.push(
            keyboard
                .what_the_painter_shows()
                .expect("it is still on a control"),
        );
    }
    assert_eq!(
        walked,
        vec![
            OnTheEdge::Move,
            OnTheEdge::Menu,
            OnTheEdge::Close,
            OnTheEdge::Move,
        ],
        "three controls and then a wrap"
    );
}

/// Previous from the first wraps to the last.
#[test]
fn previous_from_the_first_control_wraps_to_the_last() {
    let edge = a_compact_edge();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    assert_eq!(
        keyboard.told(AnEdgeKey::Previous, &edge, None),
        WhatTheEdgeKeyDid::TheKeyboardIsOn(OnTheEdge::Close)
    );
}

/// **Pressing the Menu control is the road into the menu**, which nothing had.
#[test]
fn pressing_the_menu_control_opens_the_menu() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    keyboard.told(AnEdgeKey::Next, &edge, None);
    assert_eq!(keyboard.what_the_painter_shows(), Some(OnTheEdge::Menu));
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &edge, Some(&menu)),
        WhatTheEdgeKeyDid::TheMenuOpened
    );
    assert_eq!(keyboard.whether_a_menu_holds_it(), AMenu::IsOpen);
    // **The trigger stays lit while the menu is open**, and the menu's own
    // focused row is a second thing, which starts as none.
    assert_eq!(keyboard.what_the_painter_shows(), Some(OnTheEdge::Menu));
    assert_eq!(keyboard.the_menu().and_then(|it| it.on()), None);
}

/// Pressing anything else on the edge answers the control, for the caller.
#[test]
fn pressing_a_control_that_is_not_the_menu_answers_the_control() {
    let edge = a_compact_edge();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &edge, None),
        WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Move),
        "Move has no action, so the caller decides what a press means"
    );
    keyboard.told(AnEdgeKey::Last, &edge, None);
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &edge, None),
        WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Close)
    );
    assert_eq!(
        keyboard.whether_a_menu_holds_it(),
        AMenu::IsNot,
        "pressing Close does not open a menu"
    );
}

/// Inside the menu, the keys walk rows rather than controls.
#[test]
fn inside_the_menu_the_keys_walk_rows() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    keyboard.told(AnEdgeKey::Next, &edge, None);
    keyboard.told(AnEdgeKey::Press, &edge, Some(&menu));
    assert_eq!(
        keyboard.told(AnEdgeKey::Next, &edge, Some(&menu)),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Minimise)
    );
    assert_eq!(
        keyboard.told(AnEdgeKey::Last, &edge, Some(&menu)),
        WhatTheEdgeKeyDid::TheKeyboardIsOnARow(WhatARowDoes::Close)
    );
}

/// **Escape in the menu puts the keyboard back on the control that opened it.**
///
/// The contract's *return focus to its trigger when dismissed*, kept here
/// rather than promised: `window_menu_keys` names the control and this is the
/// file that puts the keyboard on it.
#[test]
fn escape_in_the_menu_returns_the_keyboard_to_the_trigger() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    keyboard.told(AnEdgeKey::Next, &edge, None);
    keyboard.told(AnEdgeKey::Press, &edge, Some(&menu));
    keyboard.told(AnEdgeKey::Next, &edge, Some(&menu));

    assert_eq!(
        keyboard.told(AnEdgeKey::Leave, &edge, Some(&menu)),
        WhatTheEdgeKeyDid::TheMenuClosed
    );
    assert_eq!(
        keyboard,
        TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Menu),
        "and not nowhere, which would strand whoever arrived by keyboard"
    );
    assert_eq!(keyboard.whether_a_menu_holds_it(), AMenu::IsNot);
    assert_eq!(
        keyboard.whether_the_keyboard_holds_it(),
        TheKeyboard::IsOnIt,
        "the edge is still held revealed, because the keyboard is still on it"
    );
}

/// Running a row closes the menu and also returns focus.
#[test]
fn running_a_row_returns_the_keyboard_to_the_trigger_too() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    keyboard.told(AnEdgeKey::Next, &edge, None);
    keyboard.told(AnEdgeKey::Press, &edge, Some(&menu));
    keyboard.told(AnEdgeKey::Next, &edge, Some(&menu));
    assert_eq!(
        keyboard.told(AnEdgeKey::Press, &edge, Some(&menu)),
        WhatTheEdgeKeyDid::ARowRuns(WhatARowDoes::Minimise)
    );
    assert_eq!(keyboard, TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Menu));
}

/// Escape on the edge itself gives the edge up.
#[test]
fn escape_on_the_edge_gives_it_up_and_stops_holding_it_revealed() {
    let edge = a_compact_edge();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    assert_eq!(
        keyboard.told(AnEdgeKey::Leave, &edge, None),
        WhatTheEdgeKeyDid::TheKeyboardLeft
    );
    assert_eq!(keyboard, TheKeyboardOnTheEdge::Nowhere);
    assert_eq!(
        keyboard.whether_the_keyboard_holds_it(),
        TheKeyboard::IsElsewhere
    );
    assert_eq!(keyboard.what_the_painter_shows(), None);
}

/// A key pressed on an edge nothing has reached does nothing.
#[test]
fn keys_do_nothing_on_an_edge_the_keyboard_has_not_reached() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    for key in [
        AnEdgeKey::Next,
        AnEdgeKey::Previous,
        AnEdgeKey::First,
        AnEdgeKey::Last,
        AnEdgeKey::Press,
        AnEdgeKey::Leave,
    ] {
        assert_eq!(
            keyboard.told(key, &edge, Some(&menu)),
            WhatTheEdgeKeyDid::Nothing,
            "{key:?} acted on an edge nothing had reached"
        );
    }
}

/// **A menu key with no laid-out menu guesses at nothing.**
#[test]
fn a_key_inside_the_menu_with_no_menu_to_read_does_nothing() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    keyboard.told(AnEdgeKey::Next, &edge, None);
    keyboard.told(AnEdgeKey::Press, &edge, Some(&menu));
    assert_eq!(
        keyboard.told(AnEdgeKey::Next, &edge, None),
        WhatTheEdgeKeyDid::Nothing
    );
    assert_eq!(
        keyboard.whether_a_menu_holds_it(),
        AMenu::IsOpen,
        "and the menu is still open, so the edge is still held"
    );
}

/// **A wide window has no Menu control, so the keyboard reaches its three.**
///
/// `the_menu_of` returns nothing for a full edge and `window_menu`'s header
/// records that as the design's arrangement. The keyboard road must not invent
/// a menu where the design draws none.
#[test]
fn a_full_edge_has_no_menu_to_open() {
    let edge = a_full_edge();
    let drawn: Vec<OnTheEdge> = edge.controls.iter().map(|it| it.does).collect();
    assert!(
        !drawn.contains(&OnTheEdge::Menu),
        "the full layout draws no menu target: {drawn:?}"
    );
    assert!(
        the_menu_of(
            &edge,
            WhatTheWindowAllows::everything(),
            false,
            THE_TITLE_BLOCK_IS_TALL,
            an_output(),
        )
        .is_none()
    );

    let mut keyboard = TheKeyboardOnTheEdge::default();
    keyboard.reached(&edge);
    for _ in 0..drawn.len() {
        let did = keyboard.told(AnEdgeKey::Next, &edge, None);
        assert!(
            matches!(did, WhatTheEdgeKeyDid::TheKeyboardIsOn(_)),
            "{did:?}"
        );
    }
    assert_eq!(keyboard.whether_a_menu_holds_it(), AMenu::IsNot);
}

/// **The edge really is held revealed, asked of `Revealing` itself.**
///
/// The two answers this file gives are only useful if `Revealing` agrees, so
/// this drives the actual machine rather than asserting the arguments handed
/// to it. The pointer is told it is `Elsewhere` throughout, so nothing but the
/// keyboard and the menu can be what holds it.
#[test]
fn the_keyboard_and_an_open_menu_hold_the_edge_revealed() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let mut keyboard = TheKeyboardOnTheEdge::default();

    let held = |keyboard: &TheKeyboardOnTheEdge| {
        Revealing::covered()
            .the_pointer_is(alo_dock::revealing::ThePointer::Elsewhere)
            .the_keyboard(keyboard.whether_the_keyboard_holds_it())
            .a_menu(keyboard.whether_a_menu_holds_it())
            .is_revealed()
    };

    assert!(!held(&keyboard), "nothing has reached it yet");
    keyboard.reached(&edge);
    assert!(held(&keyboard), "the keyboard is on a control");
    keyboard.told(AnEdgeKey::Next, &edge, None);
    keyboard.told(AnEdgeKey::Press, &edge, Some(&menu));
    assert!(held(&keyboard), "its menu is open");
    keyboard.told(AnEdgeKey::Leave, &edge, Some(&menu));
    assert!(
        held(&keyboard),
        "the menu closed and the keyboard came back"
    );
    keyboard.told(AnEdgeKey::Leave, &edge, None);
    assert!(!held(&keyboard), "the keyboard left");
}

/// **Every action the compact edge moves off the strip is reachable without a
/// pointer**, which is the keyboard's half of the contract's one sentence.
#[test]
fn every_action_a_narrow_window_offers_is_reachable_by_keyboard_alone() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let wanted = [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ];
    for does in wanted {
        let mut keyboard = TheKeyboardOnTheEdge::default();
        keyboard.reached(&edge);
        // Walk the edge to the Menu control, open it, walk to the row, press.
        while keyboard.what_the_painter_shows() != Some(OnTheEdge::Menu) {
            keyboard.told(AnEdgeKey::Next, &edge, None);
        }
        assert_eq!(
            keyboard.told(AnEdgeKey::Press, &edge, Some(&menu)),
            WhatTheEdgeKeyDid::TheMenuOpened
        );
        while keyboard.the_menu().and_then(|it| it.on()) != Some(does) {
            keyboard.told(AnEdgeKey::Next, &edge, Some(&menu));
        }
        assert_eq!(
            keyboard.told(AnEdgeKey::Press, &edge, Some(&menu)),
            WhatTheEdgeKeyDid::ARowRuns(does),
            "{does:?} could not be run from the keyboard"
        );
    }
}

/// **The output's own scale changes nothing about the keyboard road.**
///
/// The edge is clamped inside the usable output, which moves every control;
/// the order the keyboard walks them must not move with it. One of the
/// owner's verification points — 125%, 150% and 200% — asked of this road.
#[test]
fn the_keyboard_walks_the_same_controls_at_every_scale() {
    let window = Rectangle::new(Point::from((0, 200)), Size::from((160, 400)));
    let mut seen = Vec::new();
    for scale in [100, 125, 150, 200] {
        let room = the_usable_output(Size::from((1920, 1080)), scale);
        let edge = edge_of(window, Decorations::TheShellDraws, true, room);
        let mut keyboard = TheKeyboardOnTheEdge::default();
        keyboard.reached(&edge);
        let mut walked = vec![
            keyboard
                .what_the_painter_shows()
                .expect("it reached a control"),
        ];
        for _ in 1..edge.controls.len() {
            keyboard.told(AnEdgeKey::Next, &edge, None);
            walked.push(
                keyboard
                    .what_the_painter_shows()
                    .expect("it is still on a control"),
            );
        }
        seen.push((scale, walked));
    }
    let (_, first) = seen.first().expect("four scales were walked").clone();
    assert_eq!(
        first,
        vec![OnTheEdge::Move, OnTheEdge::Menu, OnTheEdge::Close]
    );
    for (scale, walked) in seen {
        assert_eq!(walked, first, "at {scale} hundredths the order changed");
    }
}
