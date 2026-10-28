//! The line a shell writes, read back by an agent — and what an agent refuses.

use super::*;

/// A conversation with one of every kind of turn.
fn every_kind() -> TheConversationSoFar {
    TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid("list my invoices\nplease".to_owned()),
        AnEarlierTurn::AloAnswered("march.pdf, \"april\".pdf".to_owned()),
        AnEarlierTurn::AloProposed {
            sentence: "Move march.pdf to Paid".to_owned(),
            answer: TheAnswer::Allowed,
        },
        AnEarlierTurn::AloProposed {
            sentence: "Delete april.pdf".to_owned(),
            answer: TheAnswer::Denied,
        },
        AnEarlierTurn::AloProposed {
            sentence: "Rename notes.txt".to_owned(),
            answer: TheAnswer::NotAnswered,
        },
    ])
}

/// **What a shell writes is what an agent reads**, every kind of turn, and on
/// one line whatever the words held.
#[test]
fn a_conversation_written_is_read_back_exactly() {
    let written = every_kind().written();
    assert!(written.starts_with(THE_EARLIER_CONVERSATION));
    assert!(!written.contains('\n'), "{written}");
    assert_eq!(TheConversationSoFar::read(&written), Ok(every_kind()));
    assert_eq!(
        TheConversationSoFar::read(&format!("{written}\n")),
        Ok(every_kind())
    );
}

/// **A first request's line is the marker alone**, and reads as nothing.
#[test]
fn a_first_request_writes_the_marker_alone_and_reads_as_nothing() {
    assert_eq!(
        TheConversationSoFar::nothing().written(),
        THE_EARLIER_CONVERSATION
    );
    assert_eq!(
        TheConversationSoFar::read("alo-earlier:\n"),
        Ok(TheConversationSoFar::nothing())
    );
}

/// **How much was left out travels**, so the agent can tell the model.
#[test]
fn what_was_left_out_travels_with_the_conversation() {
    let long = "y".repeat(5000);
    let shortened = TheConversationSoFar::of(vec![
        AnEarlierTurn::ThePersonSaid(long.clone()),
        AnEarlierTurn::AloAnswered(long),
    ]);
    let read = TheConversationSoFar::read(&shortened.written());
    assert_eq!(read.as_ref().map(TheConversationSoFar::left_out), Ok(1));
    assert_eq!(read, Ok(shortened));
}

/// **A conversation over the bound is refused**, because no shell sends one:
/// it is not shown to a model and it is not cut here.
#[test]
fn a_conversation_over_the_bound_is_refused_and_not_read() {
    let over = "z".repeat(TheConversationSoFar::THE_MOST_SENT + 1);
    let line = format!(
        "{THE_EARLIER_CONVERSATION}{}",
        json!({"turns": [{"alo": over}], "left-out": 0, "cut": false})
    );
    assert!(matches!(
        TheConversationSoFar::read(&line),
        Err(NotTheConversation::TooLong(_))
    ));
    let far_over = format!("{THE_EARLIER_CONVERSATION}{}", "a".repeat(THE_LONGEST_LINE));
    assert!(matches!(
        TheConversationSoFar::read(&far_over),
        Err(NotTheConversation::TooLong(_))
    ));
}

/// **Anything a shell would not write is refused in words**, never guessed at.
#[test]
fn a_line_a_shell_would_not_write_is_refused_in_words() {
    for line in [
        "list my invoices",
        "alo-earlier:{not json",
        "alo-earlier:[]",
        r#"alo-earlier:{"turns":[{"model":"hello"}],"left-out":0,"cut":false}"#,
        r#"alo-earlier:{"turns":[{"proposed":"Move it","answer":"maybe"}],"left-out":0,"cut":false}"#,
        r#"alo-earlier:{"turns":[{"person":"a","alo":"b"}],"left-out":0,"cut":false}"#,
        r#"alo-earlier:{"turns":[],"cut":false}"#,
    ] {
        let refused = TheConversationSoFar::read(line);
        assert!(refused.is_err(), "{line} was read as {refused:?}");
        assert!(
            refused.is_err_and(|why| !why.to_string().is_empty()),
            "{line}"
        );
    }
}
