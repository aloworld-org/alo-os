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

/// **A long conversation travels whole**: far past the 8000 bytes this once
/// cut at, it is written and read back unchanged.
#[test]
fn a_long_conversation_travels_whole() {
    let long = TheConversationSoFar::of(
        (0..60)
            .map(|at| AnEarlierTurn::AloAnswered(format!("{at}{}", "y".repeat(5000))))
            .collect(),
    );
    assert_eq!(TheConversationSoFar::read(&long.written()), Ok(long));
}

/// **Whether a conversation fits in a question is counted as it travels**, so
/// one the daemon would refuse is caught before it is sent — and one well
/// inside is not.
#[test]
fn a_conversation_too_long_for_any_question_is_said_not_to_fit() {
    let fits = every_kind();
    assert!(fits.fits_in_a_question("and the April one too"));
    let too_long = TheConversationSoFar::of(vec![AnEarlierTurn::AloAnswered(
        "z".repeat(THE_MOST_A_QUESTION_IS),
    )]);
    assert!(!too_long.fits_in_a_question("and that one"));
    // Escaping counts: half a question of quotes is a whole question on the wire.
    let quoted = TheConversationSoFar::of(vec![AnEarlierTurn::AloAnswered(
        "\"".repeat(THE_MOST_A_QUESTION_IS / 2),
    )]);
    assert!(!quoted.fits_in_a_question("and that one"));
}

/// **A line longer than any question could be is refused**, and not read.
#[test]
fn a_line_longer_than_a_question_is_refused_and_not_read() {
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
        r#"alo-earlier:{"turns":[{"model":"hello"}]}"#,
        r#"alo-earlier:{"turns":[{"proposed":"Move it","answer":"maybe"}]}"#,
        r#"alo-earlier:{"turns":[{"person":"a","alo":"b"}]}"#,
        r#"alo-earlier:{"turns":[],"left-out":3}"#,
    ] {
        let refused = TheConversationSoFar::read(line);
        assert!(refused.is_err(), "{line} was read as {refused:?}");
        assert!(
            refused.is_err_and(|why| !why.to_string().is_empty()),
            "{line}"
        );
    }
}
