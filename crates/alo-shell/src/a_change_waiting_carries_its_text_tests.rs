//! **The text alo will type comes with the change, whole, and the person
//! can scroll through every line of it** — and scrolling answers nothing.
//!
//! ADR 0001 as amended 2026-11-13 and the owner's decision of 2026-11-15:
//! the card shows up to twelve lines and then scrolls; nothing is cut.

use super::*;

/// A waiting `accessible.type_text` carrying `text`, read from the wire form
/// the daemon sends.
fn typing(number: u64, text: &str) -> Standing {
    let wire = serde_json::json!({
        "number": number,
        "sentence": {
            "text": "Type the text below into hello.txt in VSCodium",
            "came_from": "the-source",
        },
        "texts": [{ "argument": "text", "text": text }],
    })
    .to_string();
    serde_json::from_str(&wire)
        .unwrap_or_else(|why| panic!("a waiting change did not read from {wire}: {why}"))
}

/// `how_many` numbered lines.
fn lines(how_many: usize) -> String {
    (1..=how_many)
        .map(|it| format!("line {it}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **The text arrives whole and goes when the question is answered**: every
/// character, the hidden one marked, and nothing left after an answer, a
/// refusal or a Stop.
#[test]
fn the_text_arrives_whole_and_goes_with_the_question() {
    let text = "fn main() {\n\tprintln!(\"hé\u{200B}llo\");\n}";
    for answer in 0..3 {
        let daemon = ADaemon::holding(vec![typing(7, text)]);
        let mut task = a_task();
        let mut waiting = AChangeWaiting::nothing_yet();
        assert!(waiting.texts().is_empty(), "the control: nothing asked yet");

        waiting.arrived(&mut task, &daemon);
        let [shown] = waiting.texts() else {
            panic!("one text, and {} arrived", waiting.texts().len());
        };
        assert_eq!(shown.as_given(), text, "the text was changed on its way");
        assert_eq!(shown.how_many_lines(), 3);
        assert_eq!(shown.unseen(), 1, "the U+200B is not marked");

        match answer {
            0 => {
                waiting.allow_once(&mut task, &daemon);
            }
            1 => {
                waiting.deny(&mut task, &daemon);
            }
            _ => {
                waiting.let_go_unanswered(&mut task);
            }
        }
        assert!(waiting.texts().is_empty(), "the text outlived the question");
        assert_eq!(waiting.first_line_shown(), 0);
    }
}

/// **A change that carries no text has none**, and the scroll keys do
/// nothing on it.
#[test]
fn a_change_with_no_text_shows_none_and_does_not_scroll() {
    let daemon = ADaemon::holding(vec![standing(7, "Rename march.pdf")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);
    assert!(waiting.texts().is_empty());
    for key in [
        ApprovalKey::LaterLines,
        ApprovalKey::ALaterPage,
        ApprovalKey::EarlierLines,
        ApprovalKey::AnEarlierPage,
    ] {
        assert_eq!(
            waiting.pressed(key, &mut task, &daemon),
            WhatThePressDid::Nothing
        );
    }
}

/// **Forty lines scroll, a line or a page at a time, and stop at either
/// end**: the last twelve are reachable and nothing past them.
#[test]
fn a_long_text_scrolls_a_line_or_a_page_and_stops_at_both_ends() {
    let daemon = ADaemon::holding(vec![typing(7, &lines(40))]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 0);

    assert_eq!(
        waiting.pressed(ApprovalKey::EarlierLines, &mut task, &daemon),
        WhatThePressDid::Nothing,
        "the top is the top"
    );
    assert_eq!(
        waiting.pressed(ApprovalKey::LaterLines, &mut task, &daemon),
        WhatThePressDid::ItMoved
    );
    assert_eq!(waiting.first_line_shown(), 1);
    waiting.pressed(ApprovalKey::ALaterPage, &mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 13);
    waiting.pressed(ApprovalKey::ALaterPage, &mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 25);
    waiting.pressed(ApprovalKey::ALaterPage, &mut task, &daemon);
    assert_eq!(
        waiting.first_line_shown(),
        28,
        "the last twelve are 29 to 40"
    );
    assert_eq!(
        waiting.pressed(ApprovalKey::LaterLines, &mut task, &daemon),
        WhatThePressDid::Nothing,
        "past the end"
    );
    waiting.pressed(ApprovalKey::AnEarlierPage, &mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 16);
    waiting.pressed(ApprovalKey::EarlierLines, &mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 15);
    for _ in 0..3 {
        waiting.pressed(ApprovalKey::AnEarlierPage, &mut task, &daemon);
    }
    assert_eq!(waiting.first_line_shown(), 0);
}

/// **Twelve lines fit, so they do not scroll**, and thirteen do by one.
#[test]
fn twelve_lines_fit_and_thirteen_scroll_by_one() {
    for (how_many, furthest) in [(1, 0), (12, 0), (13, 1)] {
        let daemon = ADaemon::holding(vec![typing(7, &lines(how_many))]);
        let mut task = a_task();
        let mut waiting = AChangeWaiting::nothing_yet();
        waiting.arrived(&mut task, &daemon);
        for _ in 0..5 {
            waiting.pressed(ApprovalKey::ALaterPage, &mut task, &daemon);
        }
        assert_eq!(waiting.first_line_shown(), furthest, "{how_many} lines");
    }
}

/// **Scrolling answers nothing**: no number is approved or declined, the
/// answer under the keyboard stays where it was, and the question stays.
#[test]
fn scrolling_answers_nothing_and_moves_no_answer() {
    let daemon = ADaemon::holding(vec![typing(7, &lines(40))]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);
    waiting.pressed(ApprovalKey::NextAnswer, &mut task, &daemon);
    for key in [
        ApprovalKey::LaterLines,
        ApprovalKey::ALaterPage,
        ApprovalKey::EarlierLines,
        ApprovalKey::AnEarlierPage,
    ] {
        waiting.pressed(key, &mut task, &daemon);
    }
    assert!(waiting.is_asking());
    assert_eq!(waiting.selected(), Some(WhichAnswer::AllowOnce));
    assert!(daemon.approved.borrow().is_empty());
    assert!(daemon.declined.borrow().is_empty());
    // **The presses did something**: a later page is still in view.
    waiting.pressed(ApprovalKey::ALaterPage, &mut task, &daemon);
    assert_eq!(waiting.first_line_shown(), 12);
    // **And the answer is still the person's to give, once.**
    waiting.pressed(ApprovalKey::Choose, &mut task, &daemon);
    assert_eq!(*daemon.approved.borrow(), [7]);
    assert!(daemon.declined.borrow().is_empty());
}

/// **A new question starts at its top**, however far the last one was
/// scrolled.
#[test]
fn a_new_question_starts_at_its_top() {
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &ADaemon::holding(vec![typing(7, &lines(40))]));
    waiting.pressed(
        ApprovalKey::ALaterPage,
        &mut task,
        &ADaemon::holding(vec![typing(7, &lines(40))]),
    );
    assert_eq!(waiting.first_line_shown(), 12, "the premise");
    let daemon = ADaemon::holding(vec![typing(8, &lines(40))]);
    waiting.deny(&mut task, &daemon);
    waiting.arrived(&mut task, &daemon);
    assert!(waiting.is_asking());
    assert_eq!(waiting.first_line_shown(), 0);
}

/// **What the card draws is what is waiting**, joined end to end: the text
/// that arrived, scrolled where the person scrolled it, reaches the asking
/// task window's card — and is gone from it once the question is answered.
#[test]
fn the_text_waiting_is_the_text_the_task_window_draws() {
    use smithay::utils::Rectangle;

    let strings = crate::settings_testing::settings_words();
    let daemon = ADaemon::holding(vec![typing(7, &lines(40))]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);
    waiting.pressed(ApprovalKey::LaterLines, &mut task, &daemon);
    let drawn = |task: &ATask, waiting: &AChangeWaiting| -> Vec<String> {
        let words = crate::the_tasks_words::TheTasksWords::said(
            std::slice::from_ref(task),
            &[Some(Rectangle::new((0, 0).into(), (900, 700).into()))],
            &strings,
            waiting.selected(),
        )
        .with_the_text_alo_will_type(waiting.texts(), waiting.first_line_shown(), &strings);
        words
            .as_displayed()
            .iter()
            .flat_map(|window| window.said.iter())
            .filter_map(|said| said.answers)
            .flat_map(|answers| answers.texts.iter())
            .map(|text| text.shown.text().to_owned())
            .collect()
    };
    let shown = drawn(&task, &waiting);
    let [only] = shown.as_slice() else {
        panic!("one text on the card, and {shown:?}");
    };
    assert_eq!(only.lines().next(), Some("line 2"), "scrolled by one");
    assert_eq!(only.lines().count(), 12);
    waiting.deny(&mut task, &daemon);
    assert!(
        drawn(&task, &waiting).is_empty(),
        "the text outlived the question"
    );
}
