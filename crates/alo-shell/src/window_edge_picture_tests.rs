//! The edge turned into pixels: scaling once, refusing what will not fit, and
//! still saying the whole title.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use smithay::utils::{Point, Size};

/// Both bundled faces, as the shell loads them.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
    fonts
}

/// A window with room above it for its edge.
fn window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((40, 100)), Size::from((600, 400)))
}

/// **A window too near the top has no edge drawn at all.**
///
/// The edge sits above its window, so a window at the very top has one partly
/// or wholly off the output. A half-drawn edge is worse than none: a person
/// reaches for a control that is not where it looks.
#[test]
fn an_edge_that_would_fall_off_the_output_is_not_drawn() {
    let mut fonts = fonts();
    let at_the_very_top = Rectangle::new(Point::from((40, 10)), Size::from((600, 400)));
    let edge = edge_of(at_the_very_top, Decorations::TheShellDraws, true);
    assert!(
        EdgePicture::of(
            &edge,
            Pointing::default(),
            None,
            &mut fonts,
            (1366, 768),
            100
        )
        .is_none(),
        "an edge above the output was drawn anyway"
    );
}

/// **Scaling happens once.** At two hundred per cent every figure doubles and
/// nothing doubles twice.
#[test]
fn the_scale_is_applied_once_and_only_once() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheShellDraws, true);
    let ordinary = EdgePicture::of(
        &edge,
        Pointing::default(),
        None,
        &mut fonts,
        (2732, 1536),
        100,
    )
    .expect("fits");
    let dense = EdgePicture::of(
        &edge,
        Pointing::default(),
        None,
        &mut fonts,
        (2732, 1536),
        200,
    )
    .expect("fits");
    let first = ordinary
        .solids
        .first()
        .expect("a revealed edge has a strip");
    let doubled = dense.solids.first().expect("a revealed edge has a strip");
    assert_eq!(doubled.area.loc.x, first.area.loc.x * 2);
    assert_eq!(doubled.area.size.w, first.area.size.w * 2);
    assert_eq!(doubled.area.size.h, first.area.size.h * 2);
}

/// **The whole title is kept even when none of it is drawn.**
///
/// A window narrow enough that nothing fits still answers with its name, and
/// a reader is what that answer is for.
#[test]
fn the_whole_title_survives_a_window_too_narrow_to_show_it() {
    let mut fonts = fonts();
    let narrow = Rectangle::new(Point::from((0, 100)), Size::from((200, 400)));
    let edge = edge_of(narrow, Decorations::TheShellDraws, true);
    let picture = EdgePicture::of(
        &edge,
        Pointing::default(),
        Some("Launch strategy for the first release"),
        &mut fonts,
        (1366, 768),
        100,
    )
    .expect("fits");
    assert_eq!(
        picture.the_whole_title(),
        "Launch strategy for the first release"
    );
}

/// **An application that draws its own header gets no title from alo**, so
/// nothing of alo's is inked over what the application already wrote.
#[test]
fn no_title_is_inked_for_an_application_that_has_its_own_header() {
    let mut fonts = fonts();
    let edge = edge_of(window(), Decorations::TheApplicationDraws, true);
    let picture = EdgePicture::of(
        &edge,
        Pointing::default(),
        Some("Calculator"),
        &mut fonts,
        (1366, 768),
        100,
    )
    .expect("fits");
    assert!(
        picture.inked.is_empty(),
        "alo inked a title onto an application that draws its own"
    );
}

/// **Nothing is ever drawn below the edge's own region**, at any scale — which
/// is where the application's content begins.
#[test]
fn nothing_is_drawn_over_the_application_at_any_scale() {
    let mut fonts = fonts();
    let window = window();
    for scale in [100, 150, 200] {
        let edge = edge_of(window, Decorations::TheShellDraws, true);
        let picture = EdgePicture::of(
            &edge,
            Pointing {
                hovered: Some(crate::window_edge::OnTheEdge::Close),
                focused: None,
            },
            Some("Launch strategy"),
            &mut fonts,
            (2732, 1536),
            scale,
        )
        .expect("fits");
        let content_starts = scaled(window.loc.y, scale);
        for solid in &picture.solids {
            assert!(
                solid.area.loc.y + solid.area.size.h <= content_starts,
                "at {scale}% a shape reaches {} into content starting at {content_starts}",
                solid.area.loc.y + solid.area.size.h
            );
        }
        for inked in &picture.inked {
            assert!(
                inked.area.loc.y + inked.area.size.h <= content_starts,
                "at {scale}% the title reaches into the application's content"
            );
        }
    }
}
