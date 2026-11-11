//! The Bar open with four applications found and the keyboard on the second,
//! held to the pixels it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
//!
//! **At three scales and one text size**: the Bar lays itself out at
//! `TextScale::ordinary()` whatever the person chose (`crate::bar_raster`), so
//! a second text size would draw the same picture.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::Size;

use super::{Drawn, LOGICAL, every_scale, held, in_pixels};
use crate::a_picture_hashed::Hashed;
use crate::bar_raster::{TheBarOnTheDisplay, TheBarOpens, TheLaunchersWords, picture};
use crate::the_bar_finds::{AResult, WhatARowDoes};

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
        let drawn = picture(
            &TheBarOnTheDisplay {
                typed: "fil",
                origin: "On this computer",
                results: &results,
                chosen: Some(1),
                display: Size::from(in_pixels(LOGICAL, display)),
                words: TheLaunchersWords {
                    heading: "Find, open, or ask",
                    open_it: "Open",
                    choose: "choose",
                    open: "open",
                    close: "close",
                    esc: "Esc",
                },
                asks: None,
                artwork: None,
                opens: TheBarOpens::InTheMiddle,
            },
            look.palette().unwrap(),
            [0xFA, 0xF7, 0xF2],
            &mut fonts,
            display,
        )
        .unwrap();
        let mut hash = Hashed::begun();
        let (veil, colour, alpha) = drawn.veil;
        hash.area(veil);
        hash.raw(&colour);
        hash.raw(&[alpha]);
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
        hash: 0x4551_f52e_5b51_f5dc,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0x35fd_0ef1_9c0e_f1bb,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0xd862_6021_d055_0a84,
    },
];
