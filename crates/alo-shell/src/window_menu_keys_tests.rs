//! Walking five rows, stepping over the ones that will not act, and landing
//! back on the control that opened the menu.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{AMenuKey, TheKeyboardInTheMenu, WhatTheKeyDid, the_edge_stays_revealed};
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_menu::{
    THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, WindowMenu, the_menu_of,
};
use alo_dock::revealing::AMenu;
use smithay::utils::{Logical, Point, Rectangle, Size};

fn an_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// The same compact window `window_menu_tests` uses, so both files are asking
/// about one menu rather than about two fixtures that happen to agree.
fn a_compact_edge() -> WindowEdge {
    let window = Rectangle::new(Point::from((400, 200)), Size::from((160, 400)));
    edge_of(window, Decorations::TheShellDraws, true, an_output())
}

fn a_menu_allowing(allows: WhatTheWindowAllows) -> WindowMenu {
    the_menu_of(
        &a_compact_edge(),
        allows,
        false,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control")
}

fn everything() -> WindowMenu {
    a_menu_allowing(WhatTheWindowAllows::everything())
}

fn opened() -> TheKeyboardInTheMenu {
    TheKeyboardInTheMenu::opened_from(OnTheEdge::Menu)
}

/// Press these in order and collect where the keyboard ended up each time.
fn pressing(keys: &[AMenuKey], menu: &WindowMenu) -> Vec<WhatTheKeyDid> {
    let mut keyboard = opened();
    keys.iter()
        .map(|key| keyboard.told(*key, menu))
        .collect::<Vec<_>>()
}

/// **The trigger is a control the edge really has.**
///
/// The control focus is handed back to must be one a person could have
/// pressed. `the_menu_of` returns nothing for an edge without a Menu control,
/// so this asserts the agreement rather than assuming it.
#[test]
fn the_control_focus_returns_to_is_one_the_edge_has() {
    let edge = a_compact_edge();
    let keyboard = opened();
    assert!(
        edge.controls
            .iter()
            .any(|control| control.does == keyboard.trigger()),
        "focus would return to {:?}, which this edge does not have: {:?}",
        keyboard.trigger(),
        edge.controls.iter().map(|it| it.does).collect::<Vec<_>>()
    );
}

/// **Opened with the keyboard on nothing**, so a menu opened by pointer draws
/// no focus ring somebody would have to explain.
#[test]
fn a_menu_opens_with_the_keyboard_on_no_row() {
    assert_eq!(opened().on(), None);
}

/// Down walks the design's order from the top.
#[test]
fn down_walks_every_row_in_the_designs_order() {
    let menu = everything();
    let walked = pressing(&[AMenuKey::Next; 5], &menu);
    assert_eq!(
        walked,
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Maximise),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Move),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Resize),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Close),
        ],
        "and the order a reader meets them is the same list"
    );
    // **The same order the menu was laid out in**, not an order this file
    // believes. A second list here would be the second string ADR 0089 is
    // about, and it would be the one that stopped agreeing.
    let laid_out: Vec<WhatARowDoes> = menu.rows.iter().map(|row| row.does).collect();
    let by_keyboard: Vec<WhatARowDoes> = walked
        .iter()
        .filter_map(|did| match did {
            WhatTheKeyDid::TheKeyboardIsOn(does) => Some(*does),
            _ => None,
        })
        .collect();
    assert_eq!(laid_out, by_keyboard);
}

/// Down from the last row reaches the first.
#[test]
fn down_from_the_last_row_wraps_to_the_first() {
    let menu = everything();
    let walked = pressing(&[AMenuKey::Next; 6], &menu);
    assert_eq!(
        walked.last(),
        Some(&WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise))
    );
}

/// Up from nothing reaches the last row, and up again the one before it.
#[test]
fn up_from_no_row_reaches_the_last_and_keeps_going_back() {
    let menu = everything();
    assert_eq!(
        pressing(&[AMenuKey::Previous, AMenuKey::Previous], &menu),
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Close),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Resize),
        ]
    );
}

/// Home and End reach the ends.
#[test]
fn home_and_end_reach_the_first_and_last_usable_rows() {
    let menu = everything();
    assert_eq!(
        pressing(&[AMenuKey::Last, AMenuKey::First], &menu),
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Close),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise),
        ]
    );
}

/// **Rows that will not act are stepped over.**
///
/// A window that cannot be maximised or resized still draws five rows — they
/// stay for a reader, which `MenuRow::available` explains — and the keyboard
/// must reach only the three that work.
#[test]
fn the_keyboard_steps_over_rows_this_window_will_not_allow() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        minimise: true,
        maximise: false,
        move_it: true,
        resize: false,
        close: true,
    });
    // The control: all five rows are still there to be stepped over.
    assert_eq!(menu.rows.len(), 5, "{:?}", menu.rows);
    assert_eq!(menu.what_can_be_used().len(), 3);
    assert_eq!(
        pressing(&[AMenuKey::Next; 4], &menu),
        vec![
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Move),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Close),
            WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise),
        ],
        "four presses must land on three usable rows and then wrap"
    );
}

/// Enter runs the focused row and hands the keyboard back.
#[test]
fn enter_runs_the_focused_row_and_returns_focus_to_the_trigger() {
    let menu = everything();
    let mut keyboard = opened();
    keyboard.told(AMenuKey::Next, &menu);
    assert_eq!(
        keyboard.told(AMenuKey::Run, &menu),
        WhatTheKeyDid::ItRuns {
            does: WhatARowDoes::Minimise,
            focus_goes_back_to: OnTheEdge::Menu,
        }
    );
}

/// **Enter with the keyboard on no row runs nothing.**
///
/// A menu opened by pointer and then given Enter must not act on whichever
/// row happens to be first. That would be the menu choosing for a person.
#[test]
fn enter_before_any_row_has_the_keyboard_runs_nothing() {
    let menu = everything();
    assert_eq!(
        pressing(&[AMenuKey::Run], &menu),
        vec![WhatTheKeyDid::Nothing]
    );
}

/// **A row that stopped being usable does not act**, even with the keyboard on
/// it.
///
/// The window changed under the open menu, which is why `told` reads the menu
/// every time rather than trusting where it put the keyboard earlier.
#[test]
fn a_row_that_stopped_being_usable_runs_nothing() {
    let mut keyboard = opened();
    let was = everything();
    assert_eq!(
        keyboard.told(AMenuKey::Next, &was),
        WhatTheKeyDid::TheKeyboardIsOn(WhatARowDoes::Minimise)
    );
    let now = a_menu_allowing(WhatTheWindowAllows {
        minimise: false,
        ..WhatTheWindowAllows::everything()
    });
    assert_eq!(
        keyboard.told(AMenuKey::Run, &now),
        WhatTheKeyDid::Nothing,
        "the keyboard was on Minimise and this window no longer allows it"
    );
}

/// Escape closes and hands the keyboard back.
#[test]
fn escape_closes_the_menu_and_returns_focus_to_the_trigger() {
    let menu = everything();
    assert_eq!(
        pressing(&[AMenuKey::Close], &menu),
        vec![WhatTheKeyDid::ItCloses {
            focus_goes_back_to: OnTheEdge::Menu,
        }]
    );
}

/// **Escape works on a window that allows nothing at all.**
///
/// The trap this avoids: a person opens the menu of a window with no
/// capabilities, every other key refuses, and a dismissal that needed a usable
/// row would leave them with the keyboard inside a menu they cannot leave.
#[test]
fn a_window_that_allows_nothing_can_still_be_escaped() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        minimise: false,
        maximise: false,
        move_it: false,
        resize: false,
        close: false,
    });
    assert!(
        menu.what_can_be_used().is_empty(),
        "the fixture must have nothing usable or this proves nothing: {:?}",
        menu.rows
    );
    assert_eq!(
        pressing(
            &[
                AMenuKey::Next,
                AMenuKey::Previous,
                AMenuKey::First,
                AMenuKey::Last,
                AMenuKey::Run,
            ],
            &menu
        ),
        vec![WhatTheKeyDid::Nothing; 5],
        "nothing can be reached or run"
    );
    assert_eq!(
        pressing(&[AMenuKey::Close], &menu),
        vec![WhatTheKeyDid::ItCloses {
            focus_goes_back_to: OnTheEdge::Menu,
        }],
        "and it can still be closed"
    );
}

/// A press that cannot move leaves the keyboard where it was.
#[test]
fn a_press_that_cannot_move_does_not_lose_a_persons_place() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        minimise: false,
        maximise: false,
        move_it: false,
        resize: false,
        close: false,
    });
    let mut keyboard = opened();
    keyboard.told(AMenuKey::Next, &menu);
    assert_eq!(keyboard.on(), None, "there was nowhere to go");

    let usable = everything();
    let mut keyboard = opened();
    keyboard.told(AMenuKey::Last, &usable);
    assert_eq!(keyboard.on(), Some(WhatARowDoes::Close));
    keyboard.told(AMenuKey::Run, &menu);
    assert_eq!(
        keyboard.on(),
        Some(WhatARowDoes::Close),
        "a refused Run must not clear where the keyboard was"
    );
}

/// **Every row the full menu offers is reachable by keyboard alone.**
///
/// The sibling `every_action_the_full_edge_offers_is_reachable_when_compact`
/// asks the same question of the pointer road. This is the keyboard's half of
/// *every action must be reachable by pointer, touch, keyboard and assistive
/// technology*: press Down until it wraps, and every usable row must have been
/// landed on.
#[test]
fn every_usable_row_is_reachable_by_pressing_down() {
    let menu = everything();
    let usable: Vec<WhatARowDoes> = menu.what_can_be_used().iter().map(|it| it.does).collect();
    assert!(!usable.is_empty(), "the fixture offers nothing");

    let mut keyboard = opened();
    let mut reached = Vec::new();
    for _ in 0..usable.len() {
        if let WhatTheKeyDid::TheKeyboardIsOn(does) = keyboard.told(AMenuKey::Next, &menu) {
            reached.push(does);
        }
    }
    assert_eq!(reached, usable);

    // And each of them runs from there, which is the second half: reachable
    // and then actable, not merely focusable.
    for does in usable {
        let mut keyboard = opened();
        while keyboard.on() != Some(does) {
            keyboard.told(AMenuKey::Next, &menu);
        }
        assert_eq!(
            keyboard.told(AMenuKey::Run, &menu),
            WhatTheKeyDid::ItRuns {
                does,
                focus_goes_back_to: OnTheEdge::Menu,
            }
        );
    }
}

/// **An open menu holds the edge revealed, and nothing else reports it.**
///
/// `the_window_edge_reveals`' header records `a_menu` as present in
/// `Revealing` and unfed. This is the test that it is now fed, and the
/// negative case so that the function cannot be one that always says open.
#[test]
fn an_open_menu_is_what_keeps_the_edge_revealed() {
    let keyboard = opened();
    assert_eq!(the_edge_stays_revealed(Some(&keyboard)), AMenu::IsOpen);
    assert_eq!(the_edge_stays_revealed(None), AMenu::IsNot);
}
