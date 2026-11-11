//! The question about closing a running task, drawn: its three answers in
//! reading order on one row, and the outline on *Keep running* when the
//! keyboard is there (ADR 0139).
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use smithay::utils::Rectangle;

use super::tests::{a_look, the_words, two_files, what_was_said};
use super::{TaskPicture, furniture};
use crate::a_change_waiting_reaches_the_person::WhichAnswer;
use crate::task_text::{TheAnswerBetween, TheTwoAnswers, WhatWasSaid};

/// The design's frame with the closing question at the end, its answers
/// given, at `wide` across.
fn with_the_question(
    wide: i32,
    between: Option<TheAnswerBetween<'static>>,
    selected: Option<WhichAnswer>,
) -> TaskPicture {
    let files = two_files();
    let mut said = what_was_said();
    said.push(WhatWasSaid {
        whose: alo_tasks::WhoseBlock::AnAccount,
        words: "alo is still working on this.",
        marks: &[],
        answers: Some(TheTwoAnswers {
            allow_once: "Stop and close",
            deny: "Cancel",
            selected,
            asking: None,
            once: None,
            touches: &[],
            does: (None, None),
            between,
        }),
    });
    let mut fonts = cosmic_text::FontSystem::new();
    furniture(
        Rectangle::new((120, 80).into(), (wide, 800).into()),
        a_look(),
        2,
        &the_words(&files),
        &said,
        alo_tasks::TheScroll::at_the_newest(),
        alo_tasks::TheOneButton::Stop,
        false,
        &mut fonts,
        crate::display_scale::DisplayScale::ONE,
    )
    .expect("a window this size lays out")
}

/// *Keep running*, not selected.
const KEEP_RUNNING: TheAnswerBetween<'static> = TheAnswerBetween {
    words: "Keep running",
    selected: false,
};

/// **Three answers, one row, in reading order** — *Stop and close*, *Keep
/// running*, *Cancel* — at the design's width and at the least a window may
/// be. The control is the same card with two, so this cannot pass by the
/// third drawing nothing.
#[test]
fn the_three_answers_are_on_one_row_in_reading_order() {
    let least = i32::try_from(alo_tasks::layout::the_least_width()).expect("a small number");
    for wide in [1040, least] {
        let two = with_the_question(wide, None, None);
        let three = with_the_question(wide, Some(KEEP_RUNNING), None);
        assert_eq!(
            three.inked.len(),
            two.inked.len() + 1,
            "at {wide}: Keep running was not written"
        );
        let first_new = two
            .inked
            .iter()
            .zip(&three.inked)
            .position(|(old, new)| old.area != new.area)
            .expect("the labels differ somewhere");
        // The first label is the same in both; the difference starts at the
        // one after it.
        let at = |n: usize| {
            &three
                .inked
                .get(n)
                .expect("a label the order says is there")
                .area
        };
        let (stop, keep, cancel) = (at(first_new - 1), at(first_new), at(first_new + 1));
        assert!(
            stop.loc.x < keep.loc.x && keep.loc.x < cancel.loc.x,
            "at {wide}: the answers are not in reading order"
        );
        assert!(
            (stop.loc.y - keep.loc.y).abs() <= 2 && (keep.loc.y - cancel.loc.y).abs() <= 2,
            "at {wide}: the answers are not on one row"
        );
        let window_ends = 120 + wide;
        assert!(
            cancel.loc.x + cancel.size.w < window_ends,
            "at {wide}: Cancel was drawn off the window"
        );
    }
}

/// **The outline goes to *Keep running* when the keyboard is there**, and to
/// nothing else: the picture differs from none selected and from *Cancel*.
#[test]
fn the_outline_is_on_keep_running_when_the_keyboard_is() {
    let none = with_the_question(1040, Some(KEEP_RUNNING), None);
    let keep = with_the_question(
        1040,
        Some(TheAnswerBetween {
            selected: true,
            ..KEEP_RUNNING
        }),
        None,
    );
    let cancel = with_the_question(1040, Some(KEEP_RUNNING), Some(WhichAnswer::Deny));
    assert_ne!(
        none.panel.pixels, keep.panel.pixels,
        "no outline on Keep running"
    );
    assert_ne!(
        keep.panel.pixels, cancel.panel.pixels,
        "the outline did not move"
    );
}
