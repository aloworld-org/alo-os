//! The section's words are the ones the machine writes: the walk waits for the
//! sentences the shell and the turn really say, so a reworded sentence fails
//! here rather than as a walk that waits five minutes for nothing.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use super::*;
use crate::a_machine_of_pictures::PicturesAndLines;
use crate::report::Verdict;
use crate::step::Step;
use crate::strokes::Pace;
use crate::the_edge_seen::tests::a_kept_picture;

/// No secret is set.
fn nothing_set(_: &str) -> Option<String> {
    None
}

/// **The task closing is read from the journal, not from the second press**:
/// the section's own steps after its last mark, replayed against the walk of
/// 2026-11-17 on #1334's branch (run 43990841766, its pictures 86 and 88).
/// The task had finished, so the first Alt+F4 closed it and Papers was left in
/// front (picture 87); the second changed nothing — pictures 87 and 88 are the
/// same file — and a look that needed a change failed after 60 s, before the
/// journal's line was read. Either order ends with the task closed, as the
/// header says, so the second look may find the screen as it was, and the
/// journal's line decides. The first look still needs a change.
#[test]
fn the_second_close_may_find_nothing_left_to_close() {
    let steps = for_this_task(Walk::named("w")).steps().to_vec();
    let Some(marked) = steps.iter().rposition(|it| *it == Step::Mark) else {
        panic!("the section never marks the log before closing");
    };
    let closing = steps
        .iter()
        .skip(marked)
        .cloned()
        .fold(Walk::named("w").look("86", Settling::UNDER_KVM), Walk::then);
    let opened = a_kept_picture("walk-9459e836-86-papers-opened-for-this-task.png");
    let closed = a_kept_picture("walk-9459e836-88-after-the-second-close.png");
    let mut machine = PicturesAndLines::showing(vec![
        opened.clone(),
        opened.clone(),
        opened.clone(),
        opened.clone(),
        opened,
        closed,
    ])
    .saying(
        Duration::from_secs(30),
        &alo_acting::said_on_the_log::a_tasks_grant_went(),
    );
    let quick = Pace {
        hold: Duration::ZERO,
        between: Duration::ZERO,
    };
    let report = crate::walking::walk(&mut machine, &closing, &nothing_set, quick);
    let verdicts: Vec<(String, Verdict)> = report
        .checks()
        .into_iter()
        .map(|check| (check.name.clone(), check.verdict))
        .collect();
    assert_eq!(
        verdicts,
        vec![
            ("the screen 86".to_owned(), Verdict::Passed),
            (
                "the screen after-the-first-close".to_owned(),
                Verdict::Passed
            ),
            (
                "the screen after-the-second-close".to_owned(),
                Verdict::Passed
            ),
            (
                "closing the task takes back the grant made for it".to_owned(),
                Verdict::Passed
            ),
        ],
        "{}",
        report.as_text()
    );
}

/// **But the first close must still change the screen**: a run where Alt+F4
/// reached nothing at all still fails at its first look.
#[test]
fn the_first_close_must_change_the_screen() {
    let steps = for_this_task(Walk::named("w")).steps().to_vec();
    let first = steps
        .iter()
        .find(|it| matches!(it, Step::Look { name, .. } if name == "after-the-first-close"));
    assert!(
        matches!(
            first,
            Some(Step::Look {
                may_be_as_it_was: false,
                ..
            })
        ),
        "{first:?}"
    );
}

/// **The card the walk waits for is a card that types nothing**, in the
/// card's own sentence.
#[test]
fn the_card_waited_for_types_nothing() {
    let said = alo_acting::said_on_the_log::a_card_is_up(0);
    assert!(said.starts_with(A_CARD_IS_UP), "{said}");
    assert!(said.contains(TYPING_NOTHING), "{said}");
    assert!(
        !alo_acting::said_on_the_log::a_card_is_up(12).contains(TYPING_NOTHING),
        "the check would pass for a card with text"
    );
}

/// **The sheet's sentence names Papers**, in the shell's own words.
#[test]
fn the_sheet_waited_for_names_papers() {
    let said = alo_acting::said_on_the_log::the_sheet_asks_for(PAPERS);
    assert!(said.starts_with(A_SHEET_ASKS_FOR), "{said}");
    assert!(said.contains(PAPERS), "{said}");
}

/// **The grant's sentence is the one the walk waits for.**
#[test]
fn the_grant_taken_back_is_the_line_waited_for() {
    assert!(alo_acting::said_on_the_log::a_tasks_grant_went().starts_with(A_TASKS_GRANT_WENT));
}

/// **Papers is asked for by name**, and is not VSCodium, which the section
/// before gives alo for a day.
#[test]
fn papers_is_asked_for_and_is_not_what_was_given_before() {
    assert!(THE_ASK_FOR_AN_APPLICATION.contains("Papers"));
    assert_ne!(PAPERS, crate::writing_in_vscodium::VSCODIUM);
}
