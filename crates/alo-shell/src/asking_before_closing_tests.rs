//! How a key moves on the question about closing a running task.

use super::*;

/// A question nothing is selected on yet.
fn asked() -> ClosingAsked {
    ClosingAsked {
        which: WhichTask::numbered(0),
        selected: None,
    }
}

/// **Nothing is preselected**: Enter alone chooses nothing, and the first Tab
/// lands on *Stop and close*, the second on *Cancel*.
#[test]
fn nothing_is_chosen_until_the_person_moves() {
    let mut question = asked();
    assert_eq!(
        question.pressed(ApprovalKey::Choose),
        WhatTheKeyDid::NotForIt
    );
    assert_eq!(
        question.pressed(ApprovalKey::NextAnswer),
        WhatTheKeyDid::Moved
    );
    assert_eq!(
        question.pressed(ApprovalKey::Choose),
        WhatTheKeyDid::Chose(WhichChoice::StopAndClose)
    );
    assert_eq!(
        question.pressed(ApprovalKey::NextAnswer),
        WhatTheKeyDid::Moved
    );
    assert_eq!(
        question.pressed(ApprovalKey::Choose),
        WhatTheKeyDid::Chose(WhichChoice::Cancel)
    );
}

/// **Escape is *Cancel***, whatever is selected.
#[test]
fn escape_is_cancel_whatever_is_selected() {
    let mut question = asked();
    let _ = question.pressed(ApprovalKey::NextAnswer);
    assert_eq!(
        question.pressed(ApprovalKey::Decline),
        WhatTheKeyDid::Chose(WhichChoice::Cancel)
    );
}
