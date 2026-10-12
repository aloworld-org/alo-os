//! Where the menu opens, what it holds, and the row it will not act on.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{
    A_ROW_IS_TALL, A_ROW_IS_WIDE, BELOW_THE_EDGE, THE_MENU_IS_WIDE, THE_TITLE_BLOCK_IS_TALL,
    WhatARowDoes, WhatTheWindowAllows, the_menu_of,
};
use crate::window_edge::{Decorations, FULL_CONTROLS_NEED, WindowEdge, edge_of};
use smithay::utils::{Logical, Point, Rectangle, Size};

fn an_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// A compact window, low enough on the screen that a menu fits below it.
fn a_compact_edge() -> WindowEdge {
    let window = Rectangle::new(Point::from((400, 200)), Size::from((160, 400)));
    edge_of(window, Decorations::TheShellDraws, true, an_output())
}

fn a_menu() -> super::WindowMenu {
    the_menu_of(
        &a_compact_edge(),
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control")
}

/// **The panel is the measured size**, and its rows are where the design puts
/// them: 264 wide, a 56 title block at 13, then 48-tall rows.
#[test]
fn the_panel_and_its_rows_are_the_measured_ones() {
    let menu = a_menu();

    assert_eq!(menu.panel.size.w, THE_MENU_IS_WIDE);
    assert_eq!(menu.rows.len(), 5, "five rows, as both masters draw");

    // The title block, 9 in from the panel's leading edge and 13 down.
    assert_eq!(menu.title.size.w, A_ROW_IS_WIDE);
    assert_eq!(menu.title.size.h, THE_TITLE_BLOCK_IS_TALL);
    assert_eq!(menu.title.loc.x - menu.panel.loc.x, 9);
    assert_eq!(menu.title.loc.y - menu.panel.loc.y, 13);

    // The first row begins where the title block ends: 13 + 56 = 69, which is
    // the master's own y for *Minimise / action*.
    let first = menu.rows.first().expect("five rows");
    assert_eq!(first.row.loc.y - menu.panel.loc.y, 69);
    assert_eq!(first.row.size.w, A_ROW_IS_WIDE);
    assert_eq!(first.row.size.h, A_ROW_IS_TALL);

    // And each one after it is exactly one row further down: 117, 165, 213,
    // 261 in the master.
    for (which, row) in menu.rows.iter().enumerate() {
        let expected = 69 + i32::try_from(which).unwrap_or(0) * A_ROW_IS_TALL;
        assert_eq!(
            row.row.loc.y - menu.panel.loc.y,
            expected,
            "row {which} ({:?})",
            row.does
        );
    }
}

/// **The order is the design's**, which is also the order a keyboard walks and
/// a reader hears.
#[test]
fn the_rows_are_in_the_order_the_design_draws() {
    let does: Vec<_> = a_menu().rows.iter().map(|it| it.does).collect();

    assert_eq!(
        does,
        vec![
            WhatARowDoes::Minimise,
            WhatARowDoes::Maximise,
            WhatARowDoes::Move,
            WhatARowDoes::Resize,
            WhatARowDoes::Close,
        ]
    );
}

/// **A maximised window says *Restore size* in the same row.**
///
/// `414:27653` differs from `414:27639` in one label and nothing else, so the
/// row count, the order and every position must be identical.
#[test]
fn a_maximised_window_swaps_one_row_and_nothing_else() {
    let normal = a_menu();
    let maximised = the_menu_of(
        &a_compact_edge(),
        WhatTheWindowAllows::everything(),
        true,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu control");

    assert_eq!(normal.panel, maximised.panel, "the panel does not move");
    assert_eq!(normal.rows.len(), maximised.rows.len());
    for (one, two) in normal.rows.iter().zip(maximised.rows.iter()) {
        assert_eq!(one.row, two.row, "a row moved between the two states");
    }

    assert_eq!(normal.rows[1].does, WhatARowDoes::Maximise);
    assert_eq!(maximised.rows[1].does, WhatARowDoes::Restore);
    // And only that one differs.
    for which in [0, 2, 3, 4] {
        assert_eq!(normal.rows[which].does, maximised.rows[which].does);
    }
}

/// **Close is in the menu as well as on the edge**, which the owner asked for
/// in as many words: *the menu also provides Close*.
#[test]
fn the_menu_also_provides_close() {
    assert!(
        a_menu()
            .rows
            .iter()
            .any(|it| it.does == WhatARowDoes::Close && it.available)
    );
}

/// **Nothing the compact edge dropped is missing from the menu.**
///
/// The invariant that makes the compact layout honest: every control the full
/// edge offers is reachable at a compact width, on the edge or in the menu.
/// Without this the layout would silently drop two actions and every other
/// test would pass.
#[test]
fn every_action_the_full_edge_offers_is_reachable_when_compact() {
    let wide = Rectangle::new(Point::from((400, 200)), Size::from((800, 400)));
    let full = edge_of(wide, Decorations::TheShellDraws, true, an_output());
    let on_the_full_edge: Vec<_> = full
        .controls
        .iter()
        .filter_map(|it| crate::window_edge_reading::what_it_does(it.does))
        .collect();
    assert!(
        on_the_full_edge.len() >= 3,
        "the premise: a full edge offers at least Minimise, Maximise and Close"
    );

    let compact = a_compact_edge();
    let menu = a_menu();
    let reachable: Vec<_> = compact
        .controls
        .iter()
        .filter_map(|it| crate::window_edge_reading::what_it_does(it.does))
        .chain(menu.rows.iter().filter_map(|it| it.does.what_it_does()))
        .collect();

    for action in on_the_full_edge {
        assert!(
            reachable.contains(&action),
            "{action:?} is on a wide window's edge and reachable nowhere on a narrow one"
        );
    }
}

/// **A wide window has no menu**, because the full layout draws no menu
/// target to anchor one to.
///
/// Recorded as behaviour rather than hidden: it means Move and Resize are
/// unreachable from a wide window's edge, which is the design's arrangement
/// and the reason a keyboard route to the menu matters.
#[test]
fn a_full_width_edge_has_no_menu_to_open() {
    let wide = Rectangle::new(
        Point::from((400, 200)),
        Size::from((FULL_CONTROLS_NEED, 400)),
    );
    let edge = edge_of(wide, Decorations::TheShellDraws, true, an_output());

    assert!(
        the_menu_of(
            &edge,
            WhatTheWindowAllows::everything(),
            false,
            THE_TITLE_BLOCK_IS_TALL,
            an_output()
        )
        .is_none()
    );
    // The control: the same window one unit narrower is compact and does have
    // one, so the None above is about the layout and not about the function.
    let narrower = Rectangle::new(
        Point::from((400, 200)),
        Size::from((FULL_CONTROLS_NEED - 1, 400)),
    );
    let edge = edge_of(narrower, Decorations::TheShellDraws, true, an_output());
    assert!(
        the_menu_of(
            &edge,
            WhatTheWindowAllows::everything(),
            false,
            THE_TITLE_BLOCK_IS_TALL,
            an_output()
        )
        .is_some()
    );
}

/// **An application that draws its own header still gets a menu**, since its
/// edge carries the Menu control at every width.
#[test]
fn an_application_with_its_own_header_has_a_menu_at_any_width() {
    for wide in [80, 160, 400, 1200] {
        let window = Rectangle::new(Point::from((400, 200)), Size::from((wide, 400)));
        let edge = edge_of(window, Decorations::TheApplicationDraws, true, an_output());
        assert!(
            the_menu_of(
                &edge,
                WhatTheWindowAllows::everything(),
                false,
                THE_TITLE_BLOCK_IS_TALL,
                an_output()
            )
            .is_some(),
            "{wide}: an app-drawn edge has a menu control and so must have a menu"
        );
    }
}

/// **It opens 8 below the edge**, and aligned to the Menu control.
#[test]
fn it_opens_below_the_edge_anchored_to_the_menu_control() {
    let edge = a_compact_edge();
    let menu = a_menu();
    let anchor = edge
        .controls
        .iter()
        .find(|it| it.does == crate::window_edge::OnTheEdge::Menu)
        .expect("a compact edge has a menu control");

    assert_eq!(
        menu.panel.loc.y,
        edge.region.loc.y + edge.region.size.h + BELOW_THE_EDGE
    );
    assert_eq!(menu.panel.loc.x, anchor.target.loc.x);
    assert!(!menu.above, "there was room below");
    assert!(!menu.scrolls, "and room enough not to scroll");
}

/// **Clamped inside the output**, across.
#[test]
fn it_is_clamped_inside_the_output() {
    let output = Size::from((800, 600));
    // Hard against the right, so the anchor would put the panel off the edge.
    let window = Rectangle::new(Point::from((700, 100)), Size::from((160, 300)));
    let edge = edge_of(window, Decorations::TheShellDraws, true, output);
    let menu = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        output,
    )
    .expect("a compact edge has a menu");

    assert_eq!(
        menu.panel.loc.x + menu.panel.size.w,
        800,
        "the panel was allowed off the right of the output"
    );
}

/// **It flips above when there is more room there**, and the edge stays
/// reachable.
#[test]
fn it_flips_above_when_below_is_too_tight() {
    let output = Size::from((1920, 1080));
    // A window near the bottom: below the edge there is almost nothing.
    let window = Rectangle::new(Point::from((400, 1040)), Size::from((160, 40)));
    let edge = edge_of(window, Decorations::TheShellDraws, true, output);
    let menu = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        output,
    )
    .expect("a compact edge has a menu");

    assert!(
        menu.above,
        "there was far more room above and it did not flip"
    );
    assert!(
        menu.panel.loc.y + menu.panel.size.h <= edge.region.loc.y,
        "a flipped panel covered the edge it was opened from"
    );
    assert!(menu.panel.loc.y >= 0, "a flipped panel went off the top");
}

/// **A window at the very top does not flip**, because the edge must stay
/// reachable — *flip above when needed if the edge remains reachable*.
#[test]
fn it_does_not_flip_when_there_is_no_room_above() {
    let output = Size::from((1920, 1080));
    let window = Rectangle::new(Point::from((400, 44)), Size::from((160, 900)));
    let edge = edge_of(window, Decorations::TheShellDraws, true, output);
    let menu = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        output,
    )
    .expect("a compact edge has a menu");

    assert_eq!(edge.region.loc.y, 0, "the premise: the edge is at the top");
    assert!(!menu.above);
}

/// **An unavailable row stays and says so**, rather than vanishing.
///
/// *Explain unavailable actions accessibly.* A row that disappeared would move
/// every row below it between one window and the next, and a person who knows
/// the menu would wonder whether they had misremembered it.
#[test]
fn a_row_the_window_does_not_allow_stays_and_is_marked() {
    let cannot_resize = WhatTheWindowAllows {
        resize: false,
        ..WhatTheWindowAllows::everything()
    };
    let menu = the_menu_of(
        &a_compact_edge(),
        cannot_resize,
        false,
        THE_TITLE_BLOCK_IS_TALL,
        an_output(),
    )
    .expect("a compact edge has a menu");

    assert_eq!(menu.rows.len(), 5, "the row is still there");
    let resize = menu
        .rows
        .iter()
        .find(|it| it.does == WhatARowDoes::Resize)
        .expect("the resize row is present");
    assert!(!resize.available);

    // Every other row is unaffected, and the positions did not shift.
    assert_eq!(menu.panel, a_menu().panel);
    for (one, two) in menu.rows.iter().zip(a_menu().rows.iter()) {
        assert_eq!(one.row, two.row, "{:?} moved", one.does);
    }
    assert_eq!(menu.what_can_be_used().len(), 4);
}

/// **The hit test answers rows and nothing else.**
#[test]
fn a_press_lands_on_the_row_it_looks_like() {
    let menu = a_menu();

    for row in &menu.rows {
        let middle = Point::from((
            row.row.loc.x + row.row.size.w / 2,
            row.row.loc.y + row.row.size.h / 2,
        ));
        assert_eq!(
            menu.what_is_under(middle).map(|it| it.does),
            Some(row.does),
            "{:?} did not answer at its own middle",
            row.does
        );
    }

    // The title block is not a row, so a press there does nothing.
    let in_the_title = Point::from((
        menu.title.loc.x + menu.title.size.w / 2,
        menu.title.loc.y + menu.title.size.h / 2,
    ));
    assert_eq!(menu.what_is_under(in_the_title), None);

    // And beyond the panel.
    assert_eq!(
        menu.what_is_under(Point::from((menu.panel.loc.x - 1, menu.panel.loc.y))),
        None
    );
}

/// **Every row has a name a reader can speak, and two have no action.**
///
/// The honest half of this menu, and the thing a reader of the code should
/// find without digging. Move and Resize are named — `alo_access` declares
/// them and `what_a_reader_is_told` announces them — and neither is an
/// action, because `alo_shortcuts` has none and the design forbids inventing
/// a chord for one.
#[test]
fn every_row_is_named_and_two_name_no_action() {
    for does in [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Restore,
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ] {
        assert!(
            !does.its_own_word().says().is_empty(),
            "{does:?} has no sentence a reader could speak"
        );
    }

    assert_eq!(WhatARowDoes::Move.what_it_does(), None);
    assert_eq!(WhatARowDoes::Resize.what_it_does(), None);
    // The control: the other three do have an action, so the Nones above are
    // about those two rows and not about the method.
    for does in [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Restore,
        WhatARowDoes::Close,
    ] {
        assert!(does.what_it_does().is_some(), "{does:?} has no action");
    }
}

/// **No two rows share a sentence**, except the one pair that shares an action.
///
/// Maximise and Restore are two states of one row and say one thing; any other
/// collision would be two rows a reader cannot tell apart.
#[test]
fn no_two_rows_say_the_same_thing_except_the_pair_that_shares_an_action() {
    let every = [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ];
    let mut keys: Vec<&str> = every.iter().map(|it| it.its_own_word().named()).collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(
        keys.len(),
        before,
        "two rows of one menu say the same sentence"
    );

    // And the pair that does share one, asserted rather than left implicit.
    assert_eq!(
        WhatARowDoes::Maximise.its_own_word().named(),
        WhatARowDoes::Restore.its_own_word().named()
    );
}

/// **An unavailable row is read-only to a reader, not absent and not usable.**
///
/// *Explain unavailable actions accessibly.* `State::ReadOnly` is *read and
/// nothing more*, which is the true thing to say about a row that is there and
/// will not act. What a reader is **not** told is why — that is owed and is
/// recorded on the method rather than invented in English.
#[test]
fn a_reader_finds_an_unavailable_row_read_only() {
    assert_eq!(
        WhatARowDoes::how_a_reader_finds_it(true),
        alo_access::State::CanBeUsed
    );
    assert_eq!(
        WhatARowDoes::how_a_reader_finds_it(false),
        alo_access::State::ReadOnly
    );
}

/// **A reader is told all five things a window can do**, which it was not
/// before the menu existed.
///
/// Three were named by their action and two by nothing: moving and resizing
/// were reachable by dragging a border and by nothing a reader could find.
#[test]
fn a_reader_is_told_about_moving_and_resizing_too() {
    let told = crate::window_edge_reading::what_a_reader_is_told(Decorations::TheShellDraws);
    let names: Vec<&str> = told.iter().map(|it| it.name.named()).collect();

    assert!(
        names.contains(&alo_access::words::MOVE_THE_WINDOW.named()),
        "a reader is not told a window can be moved: {names:?}"
    );
    assert!(
        names.contains(&alo_access::words::RESIZE_THE_WINDOW.named()),
        "a reader is not told a window can be resized: {names:?}"
    );
    assert_eq!(told.len(), 5, "five things a window can do: {names:?}");

    // **Every menu row is among them.** The invariant that makes the menu
    // announced rather than merely drawn: a row a reader is never told about
    // is a row only a sighted person has.
    for does in [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ] {
        assert!(
            names.contains(&does.its_own_word().named()),
            "{does:?} is a menu row a reader is never told about"
        );
    }
}

/// **Maximise and Restore run the same action**, which is why they are one row
/// in two states rather than two rows.
#[test]
fn maximise_and_restore_are_one_action() {
    assert_eq!(
        WhatARowDoes::Maximise.what_it_does(),
        WhatARowDoes::Restore.what_it_does()
    );
    assert_eq!(
        WhatARowDoes::Maximise.its_own_word().named(),
        WhatARowDoes::Restore.its_own_word().named(),
    );
}

/// **A short output shortens the menu and says it scrolls**, rather than
/// drawing past the screen — *the menu grows only within available height;
/// overflow scrolls*.
#[test]
fn a_menu_too_tall_for_the_room_scrolls_instead_of_overflowing() {
    let output = Size::from((1920, 300));
    let window = Rectangle::new(Point::from((400, 100)), Size::from((160, 150)));
    let edge = edge_of(window, Decorations::TheShellDraws, true, output);
    let menu = the_menu_of(
        &edge,
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL,
        output,
    )
    .expect("a compact edge has a menu");

    assert!(menu.scrolls, "it fitted, so this proves nothing");
    assert!(
        menu.panel.loc.y + menu.panel.size.h <= output.h,
        "the panel ran past the bottom of a {}-tall output",
        output.h
    );
}

/// **A long title makes the menu taller, not narrower**, and the rows move
/// down with it.
#[test]
fn a_wrapped_title_grows_the_menu_downward() {
    let two_lines = a_menu();
    let four_lines = the_menu_of(
        &a_compact_edge(),
        WhatTheWindowAllows::everything(),
        false,
        THE_TITLE_BLOCK_IS_TALL * 2,
        an_output(),
    )
    .expect("a compact edge has a menu");

    assert_eq!(
        four_lines.panel.size.w, two_lines.panel.size.w,
        "same width"
    );
    assert_eq!(
        four_lines.panel.size.h - two_lines.panel.size.h,
        THE_TITLE_BLOCK_IS_TALL,
        "the panel grew by exactly what the title took"
    );
    assert_eq!(
        four_lines.rows[0].row.loc.y - two_lines.rows[0].row.loc.y,
        THE_TITLE_BLOCK_IS_TALL,
        "the rows did not move down with the title"
    );
}
