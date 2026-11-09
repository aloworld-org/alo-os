//! The window panel, held against `11:51`.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};

/// A display large enough that nothing is clamped.
fn roomy() -> Size<i32, Logical> {
    Size::from((4000, 3000))
}

/// The Notes window of `11:24`, placed where the canvas puts it: at (112, 314)
/// in the frame, so its band is at y 260 and Window actions at x 1010.
fn notes() -> WindowEdge {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    edge_of(window, Decorations::TheShellDraws, true, roomy())
}

/// **Where `11:51` draws it**: the frame puts the panel at (842, 322) — the
/// canvas at 56, 190 plus 786, 132 — 348 wide, 8 below the band and 168 before
/// Window actions.
#[test]
fn the_panel_opens_where_the_design_draws_it() {
    let menu = the_menu_of(&notes(), HowTheWindowIs::default(), roomy()).expect("a panel");
    assert_eq!(menu.panel.loc, Point::from((842, 322)));
    assert_eq!(menu.panel.size.w, 348);
    assert_eq!(menu.heading, Point::from((842 + 22, 322 + 20)));
    assert_eq!(
        menu.divider,
        Rectangle::new(Point::from((864, 482)), Size::from((304, 1)))
    );
}

/// **Only what works is there**: Maximise and Full screen, in keyboard order —
/// no Left or Right until they work, and no Minimise or Close, which are on
/// the band.
#[test]
fn only_what_works_is_in_the_panel_and_nothing_is_on_it_twice() {
    let menu = the_menu_of(&notes(), HowTheWindowIs::default(), roomy()).expect("a panel");
    let does: Vec<WhatARowDoes> = menu.rows.iter().map(|it| it.does).collect();
    assert_eq!(does, vec![WhatARowDoes::Maximise, WhatARowDoes::FullScreen]);
}

/// **The Maximise picture is the first of the three slots**, 94 square at
/// (22, 50), and Full screen a 304 × 40 row at (22, 172).
#[test]
fn the_choices_are_where_the_design_measures_them() {
    let menu = the_menu_of(&notes(), HowTheWindowIs::default(), roomy()).expect("a panel");
    let rows: Vec<Rectangle<i32, Logical>> = menu.rows.iter().map(|it| it.row).collect();
    assert_eq!(
        rows,
        vec![
            Rectangle::new(Point::from((864, 372)), Size::from((94, 94))),
            Rectangle::new(Point::from((864, 494)), Size::from((304, 40))),
        ]
    );
    assert_eq!(menu.panel.size.h, 172 + 40 + 20);
}

/// **The words follow the window**: Restore while maximised, Exit full screen
/// while full screen.
#[test]
fn the_choices_read_restore_and_exit_while_the_window_is_so() {
    let how = HowTheWindowIs {
        maximised: true,
        full_screen: true,
    };
    let menu = the_menu_of(&notes(), how, roomy()).expect("a panel");
    let does: Vec<WhatARowDoes> = menu.rows.iter().map(|it| it.does).collect();
    assert_eq!(
        does,
        vec![WhatARowDoes::Restore, WhatARowDoes::ExitFullScreen]
    );
    assert_eq!(WhatARowDoes::Restore.its_own_word().says(), "Restore");
    assert_eq!(
        WhatARowDoes::ExitFullScreen.its_own_word().says(),
        "Exit full screen"
    );
}

/// **No Window actions, no panel** — a task window's band, which carries Close
/// alone, opens nothing.
#[test]
fn a_band_without_window_actions_opens_no_panel() {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let task = edge_of(window, Decorations::ATaskWindow, true, roomy());
    assert!(the_menu_of(&task, HowTheWindowIs::default(), roomy()).is_none());
    let concealed = edge_of(window, Decorations::TheShellDraws, false, roomy());
    assert!(
        the_menu_of(&concealed, HowTheWindowIs::default(), roomy()).is_none(),
        "a concealed band has no Window actions to anchor to"
    );
}

/// **Kept on the display, never cut**: a window at the right edge and the
/// bottom of a small display still gets its whole panel inside it.
#[test]
fn the_panel_is_moved_onto_the_display_rather_than_cut() {
    let small = Size::from((360, 360));
    let window = Rectangle::new(Point::from((0, 300)), Size::from((360, 60)));
    let edge = edge_of(window, Decorations::TheShellDraws, true, small);
    let menu = the_menu_of(&edge, HowTheWindowIs::default(), small).expect("a panel");
    let display = Rectangle::new(Point::from((0, 0)), small);
    assert!(display.contains_rect(menu.panel), "{:?}", menu.panel);
}

/// **A point is under the choice drawn there, and nowhere else.**
#[test]
fn a_point_finds_the_choice_drawn_under_it() {
    let menu = the_menu_of(&notes(), HowTheWindowIs::default(), roomy()).expect("a panel");
    assert_eq!(
        menu.what_is_under(Point::from((870, 380))).map(|it| it.does),
        Some(WhatARowDoes::Maximise)
    );
    assert_eq!(
        menu.what_is_under(Point::from((900, 500))).map(|it| it.does),
        Some(WhatARowDoes::FullScreen)
    );
    assert_eq!(menu.what_is_under(Point::from((845, 330))), None);
}
