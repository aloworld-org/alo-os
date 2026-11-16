//! The section's words are the ones the machine writes: the walk waits for the
//! sentences the shell and the turn really say, so a reworded sentence fails
//! here rather than as a walk that waits five minutes for nothing.

use super::*;

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
