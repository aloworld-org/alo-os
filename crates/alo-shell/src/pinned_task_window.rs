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
        alo_is_working: true,
        model: "SmolLM2 1.7B",
        destination: "On this computer",
        invitation: "Reply to alo or change the direction\u{2026}",
        scope_note: "Permissions and model choices apply to this task.",
        files: &files,
        how_many_files: "2 files shared with this task",
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
const BEFORE: [Drawn; 6] = [
    Drawn { scale: 100, text: 100, hash: 0xe109_587a_4b8c_7858 },
    Drawn { scale: 100, text: 125, hash: 0xeed9_0760_a7b9_7eed },
    Drawn { scale: 150, text: 100, hash: 0x3800_b84c_10d5_0178 },
    Drawn { scale: 150, text: 125, hash: 0x19c2_ae70_b293_980f },
    Drawn { scale: 200, text: 100, hash: 0x7b16_9fc0_1527_7d38 },
    Drawn { scale: 200, text: 125, hash: 0xc640_4fdc_b598_5bb7 },
];
