//! The Bar open in the Dock's field with four applications found and the
//! keyboard on the second: the list rising from a bottom Dock's field, held to
//! the pixels it draws (`crate::the_desktops_pictures_pinned`).
//!
//! **Re-pointed on 2026-11-11.** This file held the centred launcher's picture
//! until the owner's decision of that day put the typing in the Dock's own
//! field; once *Help with this window* had moved into the window's own panel,
//! nothing opened the centred launcher and it was removed. The surface that
//! exists now is this list, so this is the picture held, and its table was
//! taken from this test's own failure on that day.
//!
//! **At three scales and one text size**: the list lays itself out at
//! `TextScale::ordinary()` whatever the person chose
//! (`crate::bar_matches_raster`), so a second text size would draw the same
//! picture.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::{Rectangle, Size};

use super::{Drawn, LOGICAL, every_scale, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::bar_matches_raster::{WhereItRisesFrom, picture};
use crate::the_bar_finds::{AResult, WhatARowDoes};
use crate::what_the_bar_shows::{TheAskRow, TheBarOnTheDisplay, TheLaunchersWords};
use crate::where_the_bars_matches_rise::TheFieldOnTheDock;

/// Four applications found.
fn found() -> Vec<AResult> {
    ["Files", "Terminal", "Settings", "Mail"]
        .iter()
        .map(|name| AResult {
            shown: format!("{name}  \u{b7}  Application"),
            name: (*name).to_owned(),
            kind: "Application".to_owned(),
            does: WhatARowDoes::OpensAnApplication((*name).to_owned()),
        })
        .collect()
}

/// A bottom Dock's band and field on a display of `w` × `h` at `per_cent`:
/// the band 840 × 76 logical, 12 above the bottom edge and centred, the field
/// 344 × 48 logical inside it.
fn a_bottom_dock(w: i32, h: i32, per_cent: i32) -> TheFieldOnTheDock {
    let px = |logical: i32| logical * per_cent / 100;
    let band = Rectangle::new(
        ((w - px(840)) / 2, h - px(88)).into(),
        (px(840), px(76)).into(),
    );
    let field = Rectangle::new(
        (band.loc.x + px(260), band.loc.y + px(14)).into(),
        (px(344), px(48)).into(),
    );
    TheFieldOnTheDock {
        field,
        band,
        edge: alo_dock::Edge::Bottom,
    }
}

/// **The Bar draws the pixels it drew before**, at every scale.
#[test]
fn the_bar_draws_the_pixels_it_drew_before() {
    let results = found();
    let look = crate::desktop_testing::noon_look(
        &crate::desktop_testing::an_appearance(),
        alo_strings::Direction::LeftToRight,
    );
    let now = every_scale(|display| {
        let mut fonts = crate::WindowControlLabels::new().unwrap().fonts;
        let (w, h) = in_pixels(LOGICAL, display);
        let drawn = picture(
            &TheBarOnTheDisplay {
                typed: "fil",
                origin: "On this computer  \u{b7}  4 applications",
                results: &results,
                chosen: Some(1),
                display: Size::from((w, h)),
                words: TheLaunchersWords { open_it: "Open" },
                asks: Some(TheAskRow {
                    asking: "Ask alo \u{201c}fil\u{201d}",
                    what_it_is: "a new task",
                    from: "on this computer",
                }),
                artwork: None,
            },
            look.palette().unwrap(),
            WhereItRisesFrom {
                dock: a_bottom_dock(w, h, i32::from(display.per_cent())),
                clear_of: None,
            },
            &mut fonts,
            display,
        )
        .unwrap();
        let mut hash = Hashed::begun();
        hash.washed(&drawn.panel);
        hash.done()
    });
    held("The Bar", &now, &BEFORE);
}

/// What the Bar drew before. See `crate::the_desktops_pictures_pinned`.
const BEFORE: [Drawn; 3] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0x90f2_3e47_a185_7928,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0xd88f_70a6_5bb5_0be9,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0x0000_0000_0000_0003,
    },
];
