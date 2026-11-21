//! A task window as the design draws it — two files shared, three things said,
//! alo working — held to the pixels it drew before the UI kit moved anything
//! (`crate::the_desktops_pictures_pinned`).
//!
//! **Its own words rather than `crate::task_raster`'s test fixtures**, which
//! that module keeps to itself: a pin whose inputs move with somebody else's
//! fixture would fail for a reason that has nothing to do with pixels.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported"
)]

use alo_access::TurnedOn;
use alo_appearance::{Accent, Appearance, TimeOfDay};
use alo_strings::Direction;
use smithay::utils::Rectangle;

use super::{Drawn, every_way, held};
use crate::a_picture_hashed::Hashed;
use crate::desktop_look::DesktopLook;
use crate::task_text::{TheWordsInATaskWindow, WhatWasSaid};

/// What was said, as the design's frame draws it.
fn what_was_said() -> Vec<WhatWasSaid<'static>> {
    [
        (
            alo_tasks::WhoseBlock::ThePersons,
            "Make a six-slide launch presentation from these two files.",
        ),
        (
            alo_tasks::WhoseBlock::Alos,
            "I\u{2019}ll read the two files, shape the story, and build a draft.",
        ),
        (alo_tasks::WhoseBlock::AnAccount, "Reading your documents"),
    ]
    .into_iter()
    .map(|(whose, words)| WhatWasSaid {
        whose,
        words,
        marks: &[],
        answers: None,
    })
    .collect()
}

/// **A task window draws the pixels it drew before**, at every scale and text
/// size.
#[test]
fn a_task_window_draws_the_pixels_it_drew_before() {
    let files = vec!["Launch brief.pdf".to_owned(), "Product notes.md".to_owned()];
    let words = TheWordsInATaskWindow {
        title: "Prepare the launch presentation",
        kind: "alo task \u{b7} Work",
        status: "alo is working",
        status_is: alo_tasks::TheStatus::AloIsWorking,
        model: "SmolLM2 1.7B",
        destination: "On this computer",
        invitation: "Reply to alo or change the direction\u{2026}",
        scope_note: "Permissions and model choices apply to this task.",
        files: &files,
        how_many_files: "2 files shared with this task",
        you: "You",
        alo: "alo",
    };
    let said = what_was_said();
    let now = every_way(|display, text| {
        let mut appearance = Appearance::shipped();
        appearance.set_accent(Accent::Indigo);
        appearance.set_text(text);
        let look = DesktopLook::of(
            &appearance,
            &TurnedOn::nothing(),
            TimeOfDay::checked(10, 0).expect("ten in the morning"),
            Direction::LeftToRight,
            &crate::desktop_testing::a_display(),
        );
        let per_cent = i32::from(display.per_cent());
        let at = Rectangle::new(
            (120 * per_cent / 100, 80 * per_cent / 100).into(),
            (1040 * per_cent / 100, 800 * per_cent / 100).into(),
        );
        let mut fonts = crate::WindowControlLabels::new()
            .map(|labels| labels.fonts)
            .expect("the shell's own fonts load");
        let drawn = crate::task_raster::furniture(
            at,
            look,
            2,
            &words,
            &said,
            alo_tasks::TheScroll::at_the_newest(),
            alo_tasks::TheOneButton::Stop,
            false,
            &mut fonts,
            display,
        )
        .expect("the design's size, at every scale");
        let mut hash = Hashed::begun();
        hash.washed(&drawn.panel);
        hash.inked(&drawn.inked);
        hash.done()
    });
    held("A task window", &now, &BEFORE);
}

/// What a task window drew before. See `crate::the_desktops_pictures_pinned`.
///
/// **Retaken on 2026-11-20, once, from CI's run of the change**, when each
/// message gained its speaker's label (`task_raster_speaker_tests` holds what
/// changed). They were `0xe109_587a_4b8c_7858`, `0xeed9_0760_a7b9_7eed`,
/// `0x3800_b84c_10d5_0178`, `0x19c2_ae70_b293_980f`, `0x7b16_9fc0_1527_7d38` and
/// `0xc640_4fdc_b598_5bb7`.
///
/// **And on 2026-11-21, once, from CI's run of the change**, when the status
/// became the design's pill (`task_raster_status_tests` holds what changed).
/// They were `0x0b43_bb12_7e5b_5e73`, `0x7cef_7565_0e29_8d7c`,
/// `0x7f36_ba0e_d32e_0325`, `0x1389_6157_0b65_3440`, `0x8ef4_708b_d334_3a8e` and
/// `0xe260_b862_37d6_04d3`.
const BEFORE: [Drawn; 6] = [
    Drawn {
        scale: 100,
        text: 100,
        hash: 0xb564_1717_1a62_b5d1,
    },
    Drawn {
        scale: 100,
        text: 125,
        hash: 0x8892_2a2c_c2da_8325,
    },
    Drawn {
        scale: 150,
        text: 100,
        hash: 0xbabb_d3ba_6cd9_731f,
    },
    Drawn {
        scale: 150,
        text: 125,
        hash: 0xfb31_faee_c567_fee8,
    },
    Drawn {
        scale: 200,
        text: 100,
        hash: 0x6e9f_23e8_504d_595f,
    },
    Drawn {
        scale: 200,
        text: 125,
        hash: 0x4099_5737_7ca1_7796,
    },
];
