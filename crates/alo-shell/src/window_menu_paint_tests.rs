//! What the menu draws, and what it must not.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{PointingAtTheMenu, solids};
use crate::window_edge::{Decorations, WindowEdge, edge_of};
use crate::window_menu::{
    THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, WindowMenu, the_menu_of,
};
use alo_appearance::Role;
use smithay::utils::{Logical, Point, Rectangle, Size};

fn an_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// The fixture the other menu test files use.
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

fn nothing_pointed() -> PointingAtTheMenu {
    PointingAtTheMenu::default()
}

/// **The panel is a surface inside a border**, the way the strip is.
#[test]
fn the_panel_is_a_surface_in_a_border() {
    let menu = everything();
    let drawn = solids(&menu, nothing_pointed());

    assert_eq!(
        drawn.first().copied(),
        Some((menu.panel, Role::BorderDefault.colour())),
        "the border is drawn first and is the whole panel"
    );
    let surface = drawn
        .get(1)
        .copied()
        .expect("a surface is drawn inside the border");
    assert_eq!(surface.1, Role::BgSurface.colour());
    assert!(
        surface.0.size.w < menu.panel.size.w && surface.0.size.h < menu.panel.size.h,
        "the surface must be inside the border, not over it: {surface:?}"
    );
}

/// **Nothing is drawn outside the panel.**
///
/// A menu that painted a row beyond its own edge would put ink over the
/// application, which is the one thing the edge's painter is also held to.
#[test]
fn nothing_is_drawn_outside_the_panel() {
    let menu = everything();
    let drawn = solids(
        &menu,
        PointingAtTheMenu {
            hovered: Some(WhatARowDoes::Move),
            focused: Some(WhatARowDoes::Close),
        },
    );
    assert!(!drawn.is_empty());
    for (rectangle, _) in &drawn {
        assert_eq!(
            menu.panel.intersection(*rectangle),
            Some(*rectangle),
            "{rectangle:?} is not wholly inside the panel {:?}",
            menu.panel
        );
    }
}

/// **Only the hovered row is filled, and only if it can be used.**
#[test]
fn only_the_hovered_row_is_filled() {
    let menu = everything();
    let hovered = WhatARowDoes::Minimise;
    let row = menu
        .rows
        .iter()
        .find(|it| it.does == hovered)
        .copied()
        .expect("the menu has a Minimise row");

    let drawn = solids(
        &menu,
        PointingAtTheMenu {
            hovered: Some(hovered),
            focused: None,
        },
    );
    let fills: Vec<_> = drawn
        .iter()
        .filter(|(_, colour)| *colour == Role::BgCool.colour())
        .collect();
    assert_eq!(fills.len(), 1, "exactly one row is filled: {fills:?}");
    assert_eq!(fills.first().expect("one fill").0, row.row);

    // And with nothing hovered, nothing is filled.
    let quiet = solids(&menu, nothing_pointed());
    assert!(
        !quiet
            .iter()
            .any(|(_, colour)| *colour == Role::BgCool.colour()),
        "a row is filled with nothing pointing at it"
    );
}

/// **A row this window will not allow is not filled on hover**, because a
/// fill says *this one* and then nothing would happen.
#[test]
fn a_row_that_cannot_be_used_is_not_filled_under_the_pointer() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        resize: false,
        ..WhatTheWindowAllows::everything()
    });
    let drawn = solids(
        &menu,
        PointingAtTheMenu {
            hovered: Some(WhatARowDoes::Resize),
            focused: None,
        },
    );
    assert!(
        !drawn
            .iter()
            .any(|(_, colour)| *colour == Role::BgCool.colour()),
        "the unavailable row under the pointer was filled"
    );

    // The control: an available row in the same menu *is* filled, so this is
    // not a menu in which nothing fills.
    let other = solids(
        &menu,
        PointingAtTheMenu {
            hovered: Some(WhatARowDoes::Close),
            focused: None,
        },
    );
    assert!(
        other
            .iter()
            .any(|(_, colour)| *colour == Role::BgCool.colour()),
        "no row fills in this menu at all, so the assertion above proves nothing"
    );
}

/// **Unavailable rows are drawn muted, not hidden** — the drawn half of what
/// a reader is told as `State::ReadOnly`.
#[test]
fn an_unavailable_row_is_drawn_muted_and_still_drawn() {
    let menu = a_menu_allowing(WhatTheWindowAllows {
        resize: false,
        maximise: false,
        ..WhatTheWindowAllows::everything()
    });
    let drawn = solids(&menu, nothing_pointed());

    let muted = drawn
        .iter()
        .filter(|(_, colour)| *colour == Role::TextMuted.colour())
        .count();
    let inked = drawn
        .iter()
        .filter(|(_, colour)| *colour == Role::TextPrimary.colour())
        .count();
    assert_eq!(muted, 2, "two rows cannot be used, so two are muted");
    assert_eq!(inked, 3, "and the other three are not");
}

/// **The ring is the whole row**, as the edge's ring is the whole target.
#[test]
fn the_keyboard_rings_the_whole_row() {
    let menu = everything();
    let row = menu
        .rows
        .iter()
        .find(|it| it.does == WhatARowDoes::Close)
        .copied()
        .expect("the menu has a Close row");

    let ringed = solids(
        &menu,
        PointingAtTheMenu {
            hovered: None,
            focused: Some(WhatARowDoes::Close),
        },
    );
    let quiet = solids(&menu, nothing_pointed());
    let ring: Vec<_> = ringed
        .iter()
        .filter(|solid| !quiet.contains(solid))
        .copied()
        .collect();
    assert_eq!(ring.len(), 4, "a ring is four bars: {ring:?}");

    let left = ring.iter().map(|(r, _)| r.loc.x).min().expect("four bars");
    let top = ring.iter().map(|(r, _)| r.loc.y).min().expect("four bars");
    let right = ring
        .iter()
        .map(|(r, _)| r.loc.x + r.size.w)
        .max()
        .expect("four bars");
    let bottom = ring
        .iter()
        .map(|(r, _)| r.loc.y + r.size.h)
        .max()
        .expect("four bars");
    assert_eq!(
        (left, top, right - left, bottom - top),
        (row.row.loc.x, row.row.loc.y, row.row.size.w, row.row.size.h),
        "the ring is not the row it rings"
    );
}

/// **Every row is drawn**, so a person sees five and not however many this
/// window happens to allow.
#[test]
fn every_row_is_drawn_whatever_the_window_allows() {
    for allows in [
        WhatTheWindowAllows::everything(),
        WhatTheWindowAllows {
            minimise: false,
            maximise: false,
            move_it: false,
            resize: false,
            close: false,
        },
    ] {
        let menu = a_menu_allowing(allows);
        let drawn = solids(&menu, nothing_pointed());
        let marks = drawn
            .iter()
            .filter(|(_, colour)| {
                *colour == Role::TextPrimary.colour() || *colour == Role::TextMuted.colour()
            })
            .count();
        assert_eq!(marks, menu.rows.len(), "{allows:?} drew {marks} marks");
    }
}
