//! **The reader's one line, read**, written before [`found`] and
//! [`the_question`] were (law 6): exactly one gives its point; none, more than
//! one, and anything that is not the one line asked for give none.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

fn in_english() -> Strings {
    Strings::of(crate::words::adapter_words().unwrap())
}

/// **Found once: its centre**, however the one line is punctuated.
#[test]
fn found_once_gives_its_centre() {
    for answer in [
        "1 50 40",
        "1 50 40\n",
        "  1 50 40  ",
        "1, 50, 40",
        "1 (50%, 40%)",
        "1: 50% across, 40% down",
        "1 50 40.",
    ] {
        assert_eq!(found(answer), Ok((50, 40)), "{answer:?}");
    }
    assert_eq!(found("1 0 0"), Ok((0, 0)));
    assert_eq!(found("1 100 100"), Ok((100, 100)));
    assert_eq!(found("1 33 67"), Ok((33, 67)));
}

/// **Found nowhere: no point**, whatever follows the count.
#[test]
fn found_nowhere_gives_no_point() {
    for answer in ["0", "0\n", "0 matches", "0 50 40"] {
        assert_eq!(found(answer), Err(NotFound::Nowhere), "{answer:?}");
    }
}

/// **Found more than once: no point**, even with one given.
#[test]
fn found_more_than_once_gives_no_point() {
    for answer in ["2", "3", "2 50 40", "12 10 10"] {
        assert_eq!(found(answer), Err(NotFound::MoreThanOnce), "{answer:?}");
    }
}

/// **Anything but the one line asked for: no point**, so nothing is clicked
/// at numbers a wandering reader happened to write.
#[test]
fn an_answer_that_is_not_the_one_line_gives_no_point() {
    for answer in [
        "",
        "   ",
        "I can see a Start race button in the middle.",
        "1",
        "1 50",
        "1 50 40 7",
        "1 150 40",
        "1 50 140",
        "1 0.5 0.4",
        "1 50.5 40",
        "-1 50 40",
        "1 -50 40",
        "1 50 40\nIt is the big blue button.",
        "There is one.\n1 50 40",
        "99999999999999999999 1 1",
    ] {
        assert_eq!(found(answer), Err(NotFound::Unreadable), "{answer:?}");
    }
}

/// **The question names the thing and the application**, asks for the one
/// line, and is in the person's language.
#[test]
fn the_question_names_the_thing_and_the_application() {
    let asked = the_question("SuperTuxKart", "the Start race button", &in_english());
    assert!(!asked.is_a_bug(), "{asked}");
    let asked = asked.into_text();
    assert!(asked.contains("the Start race button"), "{asked}");
    assert!(asked.contains("SuperTuxKart"), "{asked}");
    assert!(asked.contains("one line"), "{asked}");
    // **Its own examples read as it says they do**: a question whose examples
    // the reading disagreed with would teach the reader a wrong answer.
    assert_eq!(found("1 50 40"), Ok((50, 40)));
    assert!(asked.contains("1 50 40 for one match"), "{asked}");
    assert!(asked.contains("0 for none"), "{asked}");
    assert!(asked.contains("2 for two"), "{asked}");
}

/// **Each refusal names the thing and the application**, in its own words.
#[test]
fn each_refusal_names_the_thing_and_the_application() {
    for (why, says) in [
        (NotFound::Nowhere, "found nothing like"),
        (NotFound::MoreThanOnce, "more than one thing like"),
        (NotFound::Unreadable, "did not say where"),
    ] {
        let said = why
            .said("SuperTuxKart", "the Start race button", &in_english())
            .into_text();
        assert!(said.contains(says), "{why:?}: {said}");
        assert!(said.contains("“the Start race button”"), "{why:?}: {said}");
        assert!(said.contains("SuperTuxKart"), "{why:?}: {said}");
        assert!(said.contains("nothing was clicked"), "{why:?}: {said}");
    }
}
