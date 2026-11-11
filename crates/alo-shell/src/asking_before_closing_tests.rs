//! How a key moves on the question about closing a running task, and that its
//! three answers fit the narrowest task window.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// A question nothing is selected on yet.
fn asked() -> ClosingAsked {
    ClosingAsked {
        which: WhichTask::numbered(0),
        selected: None,
    }
}

/// **Nothing is preselected**: Enter alone chooses nothing, and Tab walks
/// *Stop and close*, *Keep running*, *Cancel*, and round again.
#[test]
fn nothing_is_chosen_until_the_person_moves() {
    let mut question = asked();
    assert_eq!(
        question.pressed(ApprovalKey::Choose),
        WhatTheKeyDid::NotForIt
    );
    for expected in [
        WhichChoice::StopAndClose,
        WhichChoice::KeepRunning,
        WhichChoice::Cancel,
        WhichChoice::StopAndClose,
    ] {
        assert_eq!(
            question.pressed(ApprovalKey::NextAnswer),
            WhatTheKeyDid::Moved
        );
        assert_eq!(
            question.pressed(ApprovalKey::Choose),
            WhatTheKeyDid::Chose(expected)
        );
    }
}

/// **Shift+Tab goes the other way**, entering the row from its far end.
#[test]
fn shift_tab_walks_the_answers_backwards() {
    let mut question = asked();
    for expected in [
        WhichChoice::Cancel,
        WhichChoice::KeepRunning,
        WhichChoice::StopAndClose,
        WhichChoice::Cancel,
    ] {
        assert_eq!(
            question.pressed(ApprovalKey::PreviousAnswer),
            WhatTheKeyDid::Moved
        );
        assert_eq!(question.selected, Some(expected));
    }
}

/// **Escape is *Cancel***, whatever is selected — *Keep running* included, so
/// Escape never sends a window anywhere.
#[test]
fn escape_is_cancel_whatever_is_selected() {
    for steps in 0..4 {
        let mut question = asked();
        for _ in 0..steps {
            let _ = question.pressed(ApprovalKey::NextAnswer);
        }
        assert_eq!(
            question.pressed(ApprovalKey::Decline),
            WhatTheKeyDid::Chose(WhichChoice::Cancel),
            "after {steps} Tabs"
        );
    }
}

/// **The three answers fit a task window at its least width**, on the card
/// the question is asked on — so the third answer is never drawn off the edge
/// of the card or the window.
#[test]
fn the_least_width_leaves_room_for_three_answers() {
    use alo_tasks::measures as m;

    let rows = alo_tasks::layout::TheRows::of(
        alo_tasks::layout::the_least_width(),
        alo_tasks::layout::the_least_height(),
    )
    .expect("a window at its least size lays out");
    let blocks = alo_tasks::TheBlocks::stacked(
        &[(alo_tasks::WhoseBlock::AnAccount, 100)],
        rows.conversation.wide,
    );
    let card = blocks.each().first().expect("the card is placed").wide;
    let the_row = m::INSIDE_A_BLOCK
        + m::ALLOW_ONCE_IS_WIDE
        + m::GAP
        + m::KEEP_RUNNING_IS_WIDE
        + m::GAP
        + m::DENY_IS_WIDE
        + m::INSIDE_A_BLOCK;
    assert!(
        the_row <= card,
        "three answers need {the_row} and the narrowest card is {card}"
    );
}
