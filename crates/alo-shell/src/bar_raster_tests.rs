#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
//! The launcher drawn: the panel where it was laid out and wholly on the
//! display, no grey band under the line beneath the field, the chosen row in
//! the accent's tint, the row that asks alo with alo's mark in teal, and the
//! screen behind dimmed.
//!
//! **The first of these is a regression it once had**: measured on a booted
//! machine on 2026-10-23, a panel 604 wide was drawn 276 wide, because the
//! rasteriser was handed the panel's size where it wanted the display's.

use super::*;
use crate::the_bar_finds::{AResult, WhatARowDoes};
use cosmic_text::FontSystem;
use smithay::utils::Size;

/// A palette that does not depend on a person's settings: indigo, light.
fn a_palette() -> crate::desktop_look::DesktopPalette {
    crate::desktop_look::DesktopPalette::of(
        alo_appearance::Scheme::Light,
        alo_appearance::Accent::Indigo.on(alo_appearance::Scheme::Light),
        crate::Contrast::AsDesigned,
    )
    .expect("indigo is one of the accents a person may choose")
}

/// The shell's own faces and nothing else.
fn fonts() -> FontSystem {
    crate::WindowControlLabels::new()
        .map(|labels| labels.fonts)
        .expect("the shell's own fonts load")
}

/// Dolphin, found, as `561:324` draws it.
fn dolphin() -> Vec<AResult> {
    vec![AResult {
        shown: "Dolphin  \u{b7}  Application".to_owned(),
        name: "Dolphin".to_owned(),
        kind: "Application  \u{b7}  Files".to_owned(),
        does: WhatARowDoes::OpensAnApplication("org.kde.dolphin".to_owned()),
    }]
}

/// The launcher as the frame builds it, with the design's own words.
fn a_launcher(
    display: Size<i32, Physical>,
    results: &[AResult],
    chosen: Option<usize>,
    asks: bool,
) -> Option<BarPicture> {
    picture(
        &TheBarOnTheDisplay {
            typed: "files",
            origin: "On this computer  \u{b7}  1 application",
            results,
            chosen,
            display,
            words: TheLaunchersWords {
                heading: "Find, open, or ask",
                open_it: "Open",
                choose: "choose",
                open: "open",
                close: "close",
                esc: "Esc",
            },
            asks: asks.then_some(TheAskRow {
                asking: "Ask alo \u{201c}files\u{201d}",
                what_it_is: "a new task",
                from: "by openai, which has not said where it runs",
            }),
            artwork: None,
        },
        a_palette(),
        [0xFA, 0xF7, 0xF2],
        &mut fonts(),
        DisplayScale::ONE,
    )
}

/// The pixel at `(x, y)` of the panel's raster.
fn pixel(drawn: &BarPicture, x: i32, y: i32) -> [u8; 4] {
    let area = drawn.panel.area;
    let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
        .expect("inside the raster");
    let mut it = [0_u8; 4];
    it.copy_from_slice(&drawn.panel.pixels[at..at + 4]);
    it
}

/// **The texture covers the whole panel, and stays on the display, at every
/// size** — the regression this file was written for.
#[test]
fn the_rasterised_panel_covers_the_panel_and_stays_on_the_display() {
    for (w, h) in [(1280, 800), (1440, 960), (1920, 1080), (2560, 1440)] {
        let display = Size::from((w, h));
        let drawn = a_launcher(display, &dolphin(), None, true).expect("drawn");
        let panel = drawn.places.panel;
        let area = drawn.panel.area;
        assert!(
            area.loc.x <= panel.loc.x && area.loc.y <= panel.loc.y,
            "{w}x{h}"
        );
        assert!(
            area.loc.x + area.size.w >= panel.loc.x + panel.size.w
                && area.loc.y + area.size.h >= panel.loc.y + panel.size.h,
            "{w}x{h}: the panel is cut off"
        );
        assert!(area.loc.x >= 0 && area.loc.y >= 0, "{w}x{h}");
        assert!(
            area.loc.x + area.size.w <= w && area.loc.y + area.size.h <= h,
            "{w}x{h}"
        );
    }
}

/// **No grey band.** The line under the field sits on the panel's own white:
/// every pixel of its box that is not a letter is the panel's ground.
#[test]
fn the_line_under_the_field_has_no_band_behind_it() {
    let drawn = a_launcher(Size::from((1440, 960)), &dolphin(), None, true).expect("drawn");
    let origin = drawn.places.origin;
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let grey = [0xEE, 0xF2, 0xF4, 0xFF];
    for x in origin.loc.x..origin.loc.x + origin.size.w {
        for y in origin.loc.y..origin.loc.y + origin.size.h {
            assert_ne!(pixel(&drawn, x, y), grey, "a grey band at ({x}, {y})");
        }
    }
    // Past the words, the line's box is the panel itself.
    let end = origin.loc.x + origin.size.w - 2;
    assert_eq!(pixel(&drawn, end, origin.loc.y + 9), white);
}

/// **The chosen row is tinted in the accent and the others are not**; with
/// the keyboard on the row that asks, that row is the tinted one.
#[test]
fn the_row_the_keyboard_is_on_is_tinted() {
    let rows = dolphin();
    let on_dolphin = a_launcher(Size::from((1440, 960)), &rows, None, true).expect("drawn");
    let on_asking = a_launcher(Size::from((1440, 960)), &rows, Some(1), true).expect("drawn");
    let row = on_dolphin.places.rows[0].1;
    let ask = on_dolphin.places.ask.expect("offered");
    let (row_at, ask_at) = (
        (row.loc.x + 300, row.loc.y + 3),
        (ask.loc.x + 300, ask.loc.y + 3),
    );
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    assert_ne!(
        pixel(&on_dolphin, row_at.0, row_at.1),
        white,
        "the chosen row is tinted"
    );
    assert_eq!(
        pixel(&on_dolphin, ask_at.0, ask_at.1),
        white,
        "the other is not"
    );
    assert_eq!(pixel(&on_asking, row_at.0, row_at.1), white);
    assert_ne!(pixel(&on_asking, ask_at.0, ask_at.1), white);
}

/// **The row that asks carries alo's mark in teal, on its own tile** — and
/// teal appears nowhere else on the launcher, which is ADR 0067's reservation.
#[test]
fn only_alos_own_row_is_teal() {
    let teal = {
        let it = alo_appearance::Role::AccentDefault.colour();
        [it.red(), it.green(), it.blue()]
    };
    let with = a_launcher(Size::from((1440, 960)), &dolphin(), None, true).expect("drawn");
    let without = a_launcher(Size::from((1440, 960)), &dolphin(), None, false).expect("drawn");
    let has_teal = |drawn: &BarPicture| {
        drawn
            .panel
            .pixels
            .chunks(4)
            .any(|it| it[..3] == teal[..] && it[3] == 255)
    };
    assert!(has_teal(&with), "alo's mark is drawn");
    assert!(!has_teal(&without), "teal with no row of alo's");
}

/// **The screen behind is dimmed toward the canvas, not covered**: the whole
/// display, at 45 %.
#[test]
fn the_screen_behind_is_dimmed_and_still_seen() {
    let drawn = a_launcher(Size::from((1280, 800)), &dolphin(), None, true).expect("drawn");
    let (area, colour, opacity) = drawn.veil;
    assert_eq!(area, Rectangle::from_size(Size::from((1280, 800))));
    assert_eq!(colour, [0xFA, 0xF7, 0xF2]);
    assert!((opacity - 0.45).abs() < f32::EPSILON);
}

/// **The launcher at both of the approved frames' sizes**, written as a PNG
/// over the dimmed canvas when `ALO_ROUND_2_PICTURES` names a folder — a check
/// somebody runs by hand beside `561:275` and `561:370`. What runs on every
/// build is that both lay out and draw.
#[test]
fn the_launcher_draws_at_both_frames_sizes_and_can_be_looked_at() {
    let folder = std::env::var_os("ALO_ROUND_2_PICTURES");
    for size in [(1440, 960), (1280, 800)] {
        let drawn = a_launcher(Size::from(size), &dolphin(), None, true).expect("drawn");
        let Some(folder) = &folder else {
            continue;
        };
        let (wide, tall) = (
            u32::try_from(size.0).expect("a width"),
            u32::try_from(size.1).expect("a height"),
        );
        let (_, [vr, vg, vb], opacity) = drawn.veil;
        let mix = |over: u8, under: u8, alpha: f32| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a blend of two bytes stays a byte"
            )]
            let it = (f32::from(over) * alpha + f32::from(under) * (1.0 - alpha)).round() as u8;
            it
        };
        let mut image = image::RgbImage::from_pixel(
            wide,
            tall,
            image::Rgb([
                mix(vr, 0xFA, opacity),
                mix(vg, 0xF7, opacity),
                mix(vb, 0xF2, opacity),
            ]),
        );
        let area = drawn.panel.area;
        for (at, it) in drawn.panel.pixels.chunks(4).enumerate() {
            let at = i32::try_from(at).expect("a small picture");
            let (x, y) = (area.loc.x + at % area.size.w, area.loc.y + at / area.size.w);
            let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
                continue;
            };
            let &[red, green, blue, alpha] = it else {
                continue;
            };
            if x >= wide || y >= tall {
                continue;
            }
            let [ur, ug, ub] = image.get_pixel(x, y).0;
            let alpha = f32::from(alpha) / 255.0;
            image.put_pixel(
                x,
                y,
                image::Rgb([
                    mix(red, ur, alpha),
                    mix(green, ug, alpha),
                    mix(blue, ub, alpha),
                ]),
            );
        }
        image
            .save(
                std::path::Path::new(folder)
                    .join(format!("launcher-{}x{}-rendered.png", size.0, size.1)),
            )
            .expect("written");
    }
}
