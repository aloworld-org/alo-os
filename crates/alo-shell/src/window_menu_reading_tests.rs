//! The menu has a name, its rows are named by what they do, and the row that
//! will not act says so.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{every_word_this_surface_says, what_a_reader_meets};
use crate::window_edge::{Decorations, OnTheEdge, WindowEdge, edge_of};
use crate::window_menu::{
    THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, WindowMenu, the_menu_of,
};
use crate::window_menu_keys::{AMenuKey, TheKeyboardInTheMenu};
use alo_access::{Role, State};
use smithay::utils::{Logical, Point, Rectangle, Size};

fn an_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// The fixture both other menu test files use.
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

/// **The menu has an accessible name**, which is the contract's own words.
#[test]
fn the_menu_is_named_and_the_name_is_one_a_translator_can_reach() {
    let met = what_a_reader_meets(&everything(), None, None);
    assert_eq!(met.panel.role, Role::List);
    assert_eq!(met.panel.name.key().as_str(), "access.the-window-menu");
    assert_eq!(
        met.panel.state,
        State::ReadOnly,
        "the panel is not pressed; its rows are"
    );
    // **Declared in the crate `alo-saying` collects.** A name declared in
    // `alo-shell` would be a string a person can see and no translator can
    // reach, which is why `alo-access` holds it — and this asserts the word is
    // in that crate's own list rather than merely spelt like one of its keys.
    assert!(
        alo_access::words::EVERY_WORD
            .iter()
            .any(|word| word.key() == met.panel.name.key()),
        "the menu's name is not a word alo-access declares"
    );
}

/// **Every word this surface says is one that crate declares.**
///
/// The failure this catches is the one `EVERY_WORD_ANOTHER_CRATES_SURFACE_SAYS`
/// was written about: a word put in front of a person that no vocabulary
/// holds, so no translator ever sees it.
#[test]
fn every_word_this_surface_says_is_declared_somewhere_a_translator_looks() {
    let said = every_word_this_surface_says();
    assert_eq!(said.len(), 7, "one panel and six rows: {said:?}");
    for word in said {
        let in_access = alo_access::words::EVERY_WORD
            .iter()
            .any(|it| it.key() == word.key());
        let in_shortcuts = alo_shortcuts::words::EVERY_WORD
            .iter()
            .any(|it| it.key() == word.key());
        assert!(
            in_access || in_shortcuts,
            "{} is said in the menu and declared by nobody",
            word.key()
        );
    }
}

/// Five rows, each named by what it does.
#[test]
fn every_row_is_named_by_what_it_does() {
    let menu = everything();
    let met = what_a_reader_meets(&menu, None, None);
    assert_eq!(met.rows.len(), 5, "{:?}", met.rows);
    for (row, told) in menu.rows.iter().zip(&met.rows) {
        assert_eq!(told.role, Role::Button);
        assert_eq!(
            told.name.key(),
            row.does.its_own_word().key(),
            "the row and what a reader hears disagree"
        );
        // ADR 0089 as `window_edge_reading_tests` keeps it: where there is an
        // action, the name is that action's own word and not a second string.
        match told.does {
            Some(does) => assert_eq!(told.name.key(), does.word().key()),
            None => assert!(
                matches!(row.does, WhatARowDoes::Move | WhatARowDoes::Resize),
                "{:?} performs nothing and is not one of the two that cannot",
                row.does
            ),
        }
    }
    // **Three act and two do not**, so the loop above cannot pass by every
    // row taking one branch.
    assert_eq!(met.rows.iter().filter(|it| it.does.is_some()).count(), 3);
    assert_eq!(met.rows.iter().filter(|it| it.does.is_none()).count(), 2);
}

/// **A row this window will not allow is announced read-only, not hidden.**
#[test]
fn an_unavailable_row_is_still_announced_and_says_it_cannot_be_used() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        resize: false,
        ..WhatTheWindowAllows::everything()
    });
    let met = what_a_reader_meets(&menu, None, None);
    assert_eq!(met.rows.len(), 5, "the row stays: {:?}", met.rows);
    let resize = met
        .rows
        .iter()
        .find(|it| it.name.key() == alo_access::words::RESIZE_THE_WINDOW.key())
        .expect("the resize row is still announced");
    assert_eq!(resize.state, State::ReadOnly);
    // And the control: every other row still says it can be used, so this is
    // not a reading in which nothing can be used.
    assert_eq!(
        met.rows
            .iter()
            .filter(|it| it.state == State::CanBeUsed)
            .count(),
        4
    );
}

/// **The agreement with what the window itself announces.**
///
/// `window_edge_reading::what_a_reader_is_told` lists what can be done to a
/// window at any width. Every row the menu offers must be in that list, or the
/// menu would act on something a reader was never told about.
#[test]
fn every_row_the_menu_offers_is_an_action_the_edge_announces() {
    let announced = crate::window_edge_reading::what_a_reader_is_told(Decorations::TheShellDraws);
    assert!(!announced.is_empty(), "nothing was announced at all");
    let met = what_a_reader_meets(&everything(), None, None);
    for row in &met.rows {
        assert!(
            announced
                .iter()
                .any(|control| control.name.key() == row.name.key()),
            "the menu offers {} and a reader is never told the window can do it",
            row.name.key()
        );
    }
}

/// The title is carried as the application's own words.
#[test]
fn the_window_title_is_carried_as_the_applications_own_words() {
    let met = what_a_reader_meets(&everything(), None, Some("Minutes of the 4th — draft.odt"));
    assert_eq!(met.title.as_deref(), Some("Minutes of the 4th — draft.odt"));
    // **Nothing is worded here for a window that named nothing.**
    // `access_nodes` falls back to `AN_APPLICATION`, and a second fallback
    // would be a second answer to one question.
    assert_eq!(what_a_reader_meets(&everything(), None, None).title, None);
}

/// Which row has the keyboard travels with the reading.
#[test]
fn the_row_with_the_keyboard_is_part_of_what_a_reader_meets() {
    let menu = everything();
    assert_eq!(
        what_a_reader_meets(&menu, None, None).the_keyboard_is_on,
        None
    );

    let mut keyboard = TheKeyboardInTheMenu::opened_from(OnTheEdge::Menu);
    assert_eq!(
        what_a_reader_meets(&menu, Some(keyboard), None).the_keyboard_is_on,
        None,
        "a menu just opened has the keyboard on no row"
    );
    keyboard.told(AMenuKey::Last, &menu);
    assert_eq!(
        what_a_reader_meets(&menu, Some(keyboard), None).the_keyboard_is_on,
        Some(WhatARowDoes::Close)
    );
}

/// **Restore and Maximise are one row and one word.**
///
/// Only one of the two is ever drawn, and both say *Maximise the window, or
/// put it back* — so a reader of a maximised window hears the capability
/// rather than a second string somebody has to keep in step.
#[test]
fn a_maximised_window_says_the_same_word_for_putting_it_back() {
    let edge = a_compact_edge();
    let restored = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control");
    let maximised = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        true,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control");
    // The control: the two menus really do differ, or this proves nothing.
    assert_ne!(restored.rows, maximised.rows);

    let one = what_a_reader_meets(&restored, None, None);
    let other = what_a_reader_meets(&maximised, None, None);
    let keys = |met: &super::WhatAReaderMeetsInTheMenu| {
        met.rows
            .iter()
            .map(|it| it.name.key().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&one), keys(&other));
}
