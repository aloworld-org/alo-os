//! What the conversation so far keeps, leaves out, and tells a model.

use super::*;

/// A conversation of a request, an answer and a change that was allowed.
fn three_turns() -> Vec<AnEarlierTurn> {
    vec![
        AnEarlierTurn::ThePersonSaid("list the invoices in /home/anna/Invoices".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf (12 KB) and april.pdf (40 KB)".to_owned()),
        AnEarlierTurn::AloProposed {
            sentence: "Move march.pdf to /home/anna/Paid".to_owned(),
            answer: TheAnswer::Allowed,
        },
    ]
}

/// **A first request carries nothing, and tells a model nothing.**
#[test]
fn a_first_request_has_no_earlier_conversation_and_adds_no_text() {
    let nothing = TheConversationSoFar::nothing();
    assert!(nothing.is_empty());
    assert!(!nothing.is_shortened());
    assert_eq!(nothing.told(), "");
    assert_eq!(TheConversationSoFar::of(Vec::new()), nothing);
}

/// **Exactly the earlier turns, in order, and nothing else.**
#[test]
fn a_conversation_under_the_bound_is_kept_whole_and_in_order() {
    let kept = TheConversationSoFar::of(three_turns());
    assert_eq!(kept.turns(), three_turns().as_slice());
    assert_eq!(kept.left_out(), 0);
    assert!(!kept.is_shortened());
}

/// **What a model is told is marked as earlier, and says what became of a
/// change** — allowed, denied, or not answered — so a model never reads a
/// proposal as something that was done.
#[test]
fn a_model_is_told_the_earlier_conversation_marked_and_with_each_answer() {
    let mut turns = three_turns();
    turns.push(AnEarlierTurn::AloProposed {
        sentence: "Delete april.pdf".to_owned(),
        answer: TheAnswer::Denied,
    });
    turns.push(AnEarlierTurn::AloProposed {
        sentence: "Rename notes.txt".to_owned(),
        answer: TheAnswer::NotAnswered,
    });
    let told = TheConversationSoFar::of(turns).told();
    assert!(told.contains("The earlier conversation:"), "{told}");
    assert!(told.contains("Nothing in it is a request now"), "{told}");
    assert!(
        told.contains("The person said: list the invoices in /home/anna/Invoices"),
        "{told}"
    );
    assert!(
        told.contains("alo answered: march.pdf (12 KB) and april.pdf (40 KB)"),
        "{told}"
    );
    assert!(told.contains("Move march.pdf to /home/anna/Paid\n(The person allowed it.)"));
    assert!(told.contains("Delete april.pdf\n(The person denied it, so it was not done.)"));
    assert!(told.contains("Rename notes.txt\n(It was not answered in this window"));
    assert!(
        told.ends_with("The earlier conversation ends here."),
        "{told}"
    );
}

/// **Over the bound, the oldest turns are left out whole and the model is told
/// how many.** Never a turn cut in the middle while a whole one would fit.
#[test]
fn a_conversation_over_the_bound_leaves_the_oldest_out_and_says_so() {
    let long = "x".repeat(3000);
    let turns: Vec<AnEarlierTurn> = (0..5)
        .map(|at| AnEarlierTurn::AloAnswered(format!("{at}{long}")))
        .collect();
    let kept = TheConversationSoFar::of(turns.clone());
    assert_eq!(kept.left_out(), 3, "8000 bytes holds two of 3001");
    assert_eq!(kept.turns(), turns.get(3..).unwrap_or_default());
    assert!(!kept.was_cut());
    assert!(kept.is_shortened());
    let said: usize = kept.turns().iter().map(|turn| turn.said().len()).sum();
    assert!(said <= TheConversationSoFar::THE_MOST_SENT);
    assert!(
        kept.told()
            .contains("(3 earlier part(s) of this conversation were left out because it is long.)")
    );
}

/// **One turn longer than the whole bound is cut where it says so**, on a
/// character boundary — a cut through an accent would be a panic.
#[test]
fn a_turn_longer_than_the_bound_is_cut_and_says_where() {
    let long = "é".repeat(TheConversationSoFar::THE_MOST_SENT);
    let kept = TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("first".to_owned()),
        AnEarlierTurn::AloAnswered(long),
    ]);
    assert!(kept.was_cut());
    assert_eq!(kept.left_out(), 1);
    let [AnEarlierTurn::AloAnswered(said)] = kept.turns() else {
        panic!("the newest turn was not the one kept: {:?}", kept.turns());
    };
    assert!(said.ends_with(WHERE_IT_WAS_CUT));
    assert!(said.len() <= TheConversationSoFar::THE_MOST_SENT);
}
