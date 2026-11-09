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
    let edge = edge_of(
        at_the_very_top,
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    assert!(
        EdgePicture::of(
            &edge,
            Pointing::default(),
            OverTheEdge::default(),
            TheWindow::default(),
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
    let edge = edge_of(
        window(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let ordinary = EdgePicture::of(
        &edge,
        Pointing::default(),
        OverTheEdge::default(),
        TheWindow::default(),
        &mut fonts,
        (2732, 1536),
        100,
    )
    .expect("fits");
    let dense = EdgePicture::of(
        &edge,
        Pointing::default(),
        OverTheEdge::default(),
        TheWindow::default(),
        &mut fonts,
        (2732, 1536),
        200,
    )
    .expect("fits");
    let first = ordinary
        .solids
        .first()
        .expect("a revealed edge has a control");
    let doubled = dense.solids.first().expect("a revealed edge has a control");
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
    let edge = edge_of(
        narrow,
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let picture = EdgePicture::of(
        &edge,
        Pointing::default(),
        OverTheEdge::default(),
        TheWindow {
            title: Some("Launch strategy for the first release"),
            ..TheWindow::default()
        },
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
    let edge = edge_of(
        window(),
        Decorations::TheApplicationDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let picture = EdgePicture::of(
        &edge,
        Pointing::default(),
        OverTheEdge::default(),
        TheWindow {
            title: Some("Calculator"),
            ..TheWindow::default()
        },
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
        let edge = edge_of(
            window,
            Decorations::TheShellDraws,
            true,
            smithay::utils::Size::from((1920, 1080)),
        );
        let picture = EdgePicture::of(
            &edge,
            Pointing {
                hovered: Some(crate::window_edge::OnTheEdge::Close),
                focused: None,
            },
            OverTheEdge::default(),
            TheWindow {
                title: Some("Launch strategy"),
                ..TheWindow::default()
            },
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

/// The title's inked rows for `title` over [`window`] at `scale` per cent:
/// how many rows of text were drawn, and the whole title a reader is told.
fn a_titles_rows(title: &str, scale: u32) -> (usize, String) {
    let mut fonts = fonts();
    let edge = edge_of(
        window(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let picture = EdgePicture::of(
        &edge,
        Pointing::default(),
        OverTheEdge::default(),
        TheWindow {
            title: Some(title),
            ..TheWindow::default()
        },
        &mut fonts,
        (4000, 4000),
        scale,
    )
    .expect("the edge fits");
    let (_, inked) = picture.flat();
    let rows = inked.iter().map(|it| it.area.size.h).sum::<i32>();
    (
        usize::try_from(rows).unwrap_or(0),
        picture.the_whole_title().to_owned(),
    )
}

/// **The title zooms with its strip** — the owner's *"Title bars shrink with
/// zoom"* of 2026-11-08. The edge is drawn at the display's scale times the
/// zoom, so its title is drawn at that scale too: taller at 200 than at 100
/// and shorter at 75, never the same size round a strip that changed.
#[test]
fn the_title_is_drawn_at_the_scale_its_strip_is() {
    let (at_75, _) = a_titles_rows("Home — Dolphin", 75);
    let (at_100, _) = a_titles_rows("Home — Dolphin", 100);
    let (at_200, _) = a_titles_rows("Home — Dolphin", 200);
    assert!(at_100 > 0, "the control: a title was drawn at 100");
    assert!(
        at_200 > at_100,
        "at 200 the title is no taller: {at_200} against {at_100}"
    );
    assert!(
        at_75 < at_100,
        "at 75 the title is no shorter: {at_75} against {at_100}"
    );
}

/// **Below the smallest a letter is drawn at, the title is left out** — and a
/// reader is still told all of it. At 50 per cent (life size zoomed to half)
/// the design's 13 would be 6.5 pixels; at 62 it is just above eight.
#[test]
fn a_title_too_small_to_read_is_left_out_and_still_said() {
    let (at_50, said) = a_titles_rows("Home — Dolphin", 50);
    assert_eq!(at_50, 0, "a title of 6.5 pixels was drawn");
    assert_eq!(said, "Home — Dolphin", "a reader lost the title");
    let (at_62, _) = a_titles_rows("Home — Dolphin", 62);
    assert!(at_62 > 0, "a title of just over eight pixels was left out");
}
