#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or missing pixel is the failure \
              being reported"
)]
//! The matches drawn where they were laid out, wholly on the display, on every
//! edge the Dock can be on — and on a side Dock, with what is typed on the
//! list's first line.

use super::*;
use crate::bar_raster::{TheAskRow, TheBarOpens, TheLaunchersWords};
use crate::the_bar_finds::{AResult, WhatARowDoes};
use smithay::utils::Size;

fn a_palette() -> crate::desktop_look::DesktopPalette {
    crate::desktop_look::DesktopPalette::of(
        alo_appearance::Scheme::Light,
        alo_appearance::Accent::Indigo.on(alo_appearance::Scheme::Light),
        crate::Contrast::AsDesigned,
    )
    .expect("indigo is one of the accents a person may choose")
}

fn fonts() -> FontSystem {
    crate::WindowControlLabels::new()
        .map(|labels| labels.fonts)
        .expect("the shell's own fonts load")
}

fn dolphin() -> Vec<AResult> {
    vec![AResult {
        shown: "Dolphin  \u{b7}  Application".to_owned(),
        name: "Dolphin".to_owned(),
        kind: "Application  \u{b7}  Files".to_owned(),
        does: WhatARowDoes::OpensAnApplication("org.kde.dolphin".to_owned()),
    }]
}

fn a_dock(w: i32, h: i32, edge: alo_dock::Edge) -> TheFieldOnTheDock {
    let (band, field) = match edge {
        alo_dock::Edge::Bottom => (
            Rectangle::new((300, h - 88).into(), (840, 76).into()),
            Rectangle::new((560, h - 74).into(), (344, 48).into()),
        ),
        alo_dock::Edge::Top => (
            Rectangle::new((300, 12).into(), (840, 76).into()),
            Rectangle::new((560, 26).into(), (344, 48).into()),
        ),
        alo_dock::Edge::Left => (
            Rectangle::new((12, 100).into(), (76, 700).into()),
            Rectangle::new((26, 300).into(), (48, 344).into()),
        ),
        alo_dock::Edge::Right => (
            Rectangle::new((w - 88, 100).into(), (76, 700).into()),
            Rectangle::new((w - 74, 300).into(), (48, 344).into()),
        ),
    };
    TheFieldOnTheDock { field, band, edge }
}

fn drawn(edge: alo_dock::Edge, results: &[AResult]) -> Option<MatchesPicture> {
    picture(
        &TheBarOnTheDisplay {
            typed: "files",
            origin: "",
            results,
            chosen: None,
            display: Size::from((1440, 960)),
            words: TheLaunchersWords {
                heading: "Find, open, or ask",
                open_it: "Open",
                choose: "choose",
                open: "open",
                close: "close",
                esc: "Esc",
            },
            asks: Some(TheAskRow {
                asking: "Ask alo \u{201c}files\u{201d}",
                what_it_is: "a new task",
                from: "on this computer",
            }),
            artwork: None,
            opens: TheBarOpens::AboveTheDocksField,
        },
        a_palette(),
        WhereItRisesFrom {
            dock: a_dock(1440, 960, edge),
            clear_of: None,
        },
        &mut fonts(),
        DisplayScale::ONE,
    )
}

#[test]
fn the_raster_covers_the_list_and_stays_on_the_display_on_every_edge() {
    for edge in alo_dock::Edge::EVERY {
        let it = drawn(edge, &dolphin()).expect("drawn");
        let (panel, area) = (it.places.panel, it.panel.area);
        assert!(area.loc.x <= panel.loc.x && area.loc.y <= panel.loc.y, "{edge:?}");
        assert!(
            area.loc.x + area.size.w >= panel.loc.x + panel.size.w
                && area.loc.y + area.size.h >= panel.loc.y + panel.size.h,
            "{edge:?}: the list is cut off"
        );
        assert!(area.loc.x >= 0 && area.loc.y >= 0, "{edge:?}");
        assert!(area.loc.x + area.size.w <= 1440 && area.loc.y + area.size.h <= 960, "{edge:?}");
    }
}

/// **The chosen row is tinted** — the first, until an arrow moves it, which is
/// the row Enter opens.
#[test]
fn the_first_row_is_the_one_tinted() {
    let it = drawn(alo_dock::Edge::Bottom, &dolphin()).expect("drawn");
    let row = it.places.rows[0].1;
    let area = it.panel.area;
    let x = row.loc.x + row.size.w - 4;
    let y = row.loc.y + row.size.h / 2;
    let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
        .expect("inside the raster");
    let pixel = &it.panel.pixels[at..at + 3];
    let white = [0xFF_u8, 0xFF, 0xFF];
    assert_ne!(pixel, white.as_slice(), "the chosen row is not the panel's white");
}

/// **A side Dock's list carries what is typed**, because its field has no
/// room for a line.
#[test]
fn a_side_docks_list_carries_the_typed_line() {
    for edge in [alo_dock::Edge::Left, alo_dock::Edge::Right] {
        let it = drawn(edge, &[]).expect("drawn");
        assert!(it.places.typed.is_some(), "{edge:?}");
    }
    let bottom = drawn(alo_dock::Edge::Bottom, &[]).expect("drawn");
    assert!(bottom.places.typed.is_none());
}
