//! What the verdict is, for replies chosen here rather than got from a model.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use super::{WhatItWants, the_question_about, what_the_reply_says};

/// **The two words a model is asked for.**
#[test]
fn one_word_is_a_verdict() {
    assert_eq!(what_the_reply_says("ACT"), WhatItWants::Acting);
    assert_eq!(what_the_reply_says("ANSWER"), WhatItWants::Answering);
}

/// **Whitespace, case and the backticks the question itself uses.**
///
/// A model shown `` `ACT` `` in the question writes it back with the backticks
/// often enough that refusing them would be refusing a correct answer for its
/// punctuation.
#[test]
fn a_verdict_survives_how_a_model_dresses_it() {
    for said in ["act", " ACT ", "`ACT`", "\nact\n", "Act"] {
        assert_eq!(what_the_reply_says(said), WhatItWants::Acting, "{said:?}");
    }
    for said in ["answer", " ANSWER\n", "`answer`"] {
        assert_eq!(
            what_the_reply_says(said),
            WhatItWants::Answering,
            "{said:?}"
        );
    }
}

/// **Anything else is *cannot tell*, and is never read as one of the two.**
///
/// This is the test that keeps the file honest. A classifier that dug a verdict
/// out of prose would be guessing on the model's behalf, and the thing it
/// guesses wrong is which of two roads a person's sentence takes.
#[test]
fn a_reply_that_is_not_one_word_decides_nothing() {
    for said in [
        "",
        "   ",
        "I think this is a request, so ACT",
        "ACT or ANSWER, it depends",
        "ANSWER. The command would delete files.",
        "This asks for something to be done.",
        "ACTION",
        "ACTS",
    ] {
        assert_eq!(
            what_the_reply_says(said),
            WhatItWants::CannotTell,
            "{said:?} is not one of the two words and must decide nothing"
        );
    }
}

/// **A reply naming both words decides nothing**, rather than taking the first.
#[test]
fn naming_both_is_not_a_verdict() {
    assert_eq!(what_the_reply_says("ACT ANSWER"), WhatItWants::CannotTell);
    assert_eq!(what_the_reply_says("ANSWER ACT"), WhatItWants::CannotTell);
}

/// **The question carries the sentence and asks for one word.**
#[test]
fn the_question_names_both_roads_and_what_was_typed() {
    let put = the_question_about("open the text editor");
    assert!(put.contains("open the text editor"), "{put}");
    assert!(put.contains("ACT"), "{put}");
    assert!(put.contains("ANSWER"), "{put}");
    assert!(put.contains("one word"), "{put}");
}

/// **A sentence spanning lines is put on one**, so the reply cannot be read as
/// belonging to a second question.
#[test]
fn what_was_typed_is_put_on_one_line() {
    let put = the_question_about("open the\r\neditor");
    let after = put
        .split_once("What they typed is: ")
        .expect("the question names what was typed")
        .1;
    assert!(!after.contains('\n'), "{after:?}");
    assert!(after.contains("open the  editor"), "{after:?}");
}
