//! How the controls are drawn, held to the measured looks.

use super::*;
use crate::window_edge::{Decorations, edge_of};
use smithay::utils::{Point, Size};

/// The Notes window's band, its controls revealed.
fn notes() -> WindowEdge {
    edge_of(
        Rectangle::new(Point::from((0, 54)), Size::from((1056, 466))),
        Decorations::TheShellDraws,
        true,
        Size::from((4000, 3000)),
    )
}

/// Nothing pointed at, panel closed, fully shown, the typing window's look.
fn at_rest() -> HowTheControlsLook {
    HowTheControlsLook {
        pointing: Pointing::default(),
        open: false,
        how_shown: crate::the_controls_fade::ALL_THE_WAY,
        look: WhichLook::of(true),
    }
}

/// A role's colour as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// The highest alpha in a raster.
fn most_opaque(raster: &Washed) -> u8 {
    raster
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .map(|it| it[3])
        .max()
        .unwrap_or(0)
}

/// **Every mark the design exported reads**, so no control is ever drawn as an
/// empty square.
#[test]
fn every_mark_reads() {
    for does in [
        OnTheEdge::WindowActions,
        OnTheEdge::Minimise,
        OnTheEdge::Close,
    ] {
        assert!(the_mark(does).is_some(), "{does:?} did not read");
    }
}

/// **Window actions wears its square at rest and the navy one while open**,
/// with the mark turning white on navy.
#[test]
fn window_actions_wears_its_open_look_while_the_panel_is_open() {
    assert_eq!(
        its_colours(OnTheEdge::WindowActions, &at_rest()),
        (Some(rgb(Role::BgCool)), rgb(Role::TextPrimary))
    );
    let open = HowTheControlsLook {
        open: true,
        ..at_rest()
    };
    assert_eq!(
        its_colours(OnTheEdge::WindowActions, &open),
        (Some(rgb(Role::TextPrimary)), rgb(Role::BgSurface))
    );
}

/// **Red only on Close, and only while it is pointed at** — never on Minimise,
/// and never on Close at rest.
#[test]
fn red_only_on_close_and_only_while_pointed_at() {
    let danger = rgb(Role::StatusDanger);
    assert_ne!(its_colours(OnTheEdge::Close, &at_rest()).1, danger);
    for hovered in [OnTheEdge::Minimise, OnTheEdge::Close] {
        let how = HowTheControlsLook {
            pointing: Pointing {
                hovered: Some(hovered),
                focused: None,
            },
            ..at_rest()
        };
        let ink = its_colours(hovered, &how).1;
        assert_eq!(ink == danger, hovered == OnTheEdge::Close, "{hovered:?}");
    }
}

/// **Minimise and Close have no square at rest**, as `11:45` and `11:49` draw.
#[test]
fn minimise_and_close_have_no_square_at_rest() {
    assert_eq!(its_colours(OnTheEdge::Minimise, &at_rest()).0, None);
    assert_eq!(its_colours(OnTheEdge::Close, &at_rest()).0, None);
}

/// **Three rasters, each its whole target**, at a doubled display too.
#[test]
fn each_control_is_drawn_into_its_whole_target_at_the_displays_scale() {
    let drawn = the_controls_drawn(&notes(), &at_rest(), 200);
    let areas: Vec<Rectangle<i32, Physical>> = drawn.iter().map(|it| crate::painted::on_the_frame(it.area)).collect();
    assert_eq!(
        areas,
        vec![
            Rectangle::new((1796, 10).into(), (88, 88).into()),
            Rectangle::new((1892, 10).into(), (88, 88).into()),
            Rectangle::new((1988, 10).into(), (88, 88).into()),
        ]
    );
}

/// **Faded right out is nothing drawn**, and half way is half as opaque.
#[test]
fn the_fade_is_how_opaque_they_are_drawn() {
    let gone = HowTheControlsLook {
        how_shown: 0,
        ..at_rest()
    };
    assert!(the_controls_drawn(&notes(), &gone, 100).is_empty());

    let full = the_controls_drawn(&notes(), &at_rest(), 100);
    let half = the_controls_drawn(
        &notes(),
        &HowTheControlsLook {
            how_shown: 500,
            ..at_rest()
        },
        100,
    );
    let (full, half) = (
        full.first().map_or(0, most_opaque),
        half.first().map_or(0, most_opaque),
    );
    assert!(full >= 250, "{full}");
    assert!(
        (i32::from(half) - i32::from(full) / 2).abs() <= 2,
        "{half} of {full}"
    );
}

/// **The focus ring is drawn on the control the keyboard is on and no other**:
/// Minimise has no square at rest, so a ring is the only thing at its corner.
#[test]
fn the_focus_ring_is_on_the_focused_control_only() {
    let corner_alpha = |how: &HowTheControlsLook| -> u8 {
        let drawn = the_controls_drawn(&notes(), how, 100);
        // Minimise is the second; the pixel one down from the middle of its
        // top, which is on the ring and far from the mark.
        drawn
            .get(1)
            .and_then(|it| it.pixels.get(((44 + 22) * 4 + 3)..((44 + 22) * 4 + 4)))
            .and_then(|it| it.first().copied())
            .unwrap_or(0)
    };
    assert_eq!(corner_alpha(&at_rest()), 0, "no ring at rest");
    let focused = HowTheControlsLook {
        pointing: Pointing {
            hovered: None,
            focused: Some(OnTheEdge::Minimise),
        },
        ..at_rest()
    };
    assert!(corner_alpha(&focused) > 200, "the ring is drawn");
}
