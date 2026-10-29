//! What the conversation so far keeps, and tells a model.

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
    assert_eq!(nothing.told(), "");
    assert_eq!(TheConversationSoFar::of(Vec::new()), nothing);
}

/// **Exactly the earlier turns, in order, and nothing else.**
#[test]
fn a_conversation_is_kept_whole_and_in_order() {
    let kept = TheConversationSoFar::of(three_turns());
    assert_eq!(kept.turns(), three_turns().as_slice());
}

/// **A long conversation is kept whole**: alo drops nothing of its own
/// accord, because a follow-up whose earlier parts vanished means something
/// else without anybody seeing it (ADR 0138).
#[test]
fn a_long_conversation_is_never_shortened() {
    let turns: Vec<AnEarlierTurn> = (0..40)
        .map(|at| AnEarlierTurn::AloAnswered(format!("{at}{}", "x".repeat(3000))))
        .collect();
    let kept = TheConversationSoFar::of(turns.clone());
    assert_eq!(kept.turns(), turns.as_slice());
    let told = kept.told();
    for turn in &turns {
        assert!(told.contains(turn.said()));
    }
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
