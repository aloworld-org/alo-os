//! The follow-up measurements this build carries, and what a reader refuses.

use super::*;
use crate::Catalogue;

/// One measurement, written as the file writes it.
fn one(grade: &str, instructions: Option<&str>, counts: bool) -> String {
    let instructions = instructions.map_or(String::new(), |digest| {
        format!("instructions = \"{digest}\"\n")
    });
    let counts = if counts { "drove = 9\nof = 10\n" } else { "" };
    format!(
        "[[follow_up]]\nmodel = \"gpt-4o\"\nat = \"OpenAI\"\ngrade = \"{grade}\"\n\n\
         [follow_up.measured]\nmachine = \"a test\"\ndate = \"2026-10-28\"\nruntime = \"the \
         provider's API\"\n{counts}{instructions}"
    )
}

/// **The file this build carries reads**, and every measurement in it names
/// the counts and the digest it was earned under.
#[test]
fn the_built_in_follow_ups_read() {
    let measured = FollowUpsMeasured::built_in().unwrap();
    for each in &measured.each {
        assert!(each.measured.instructions.is_some(), "{each:?}");
        assert!(each.measured.drove <= each.measured.of, "{each:?}");
    }
}

/// **No model on this machine has a follow-up grade yet**, and each says so
/// rather than borrowing its first-request grade (ADR 0138).
#[test]
fn every_model_in_the_catalogue_is_not_yet_measured_on_a_follow_up() {
    let measured = FollowUpsMeasured::built_in().unwrap();
    for model in Catalogue::built_in().unwrap().models {
        assert_eq!(
            measured.for_the_turn(&model.id, "this machine"),
            Driving::NotMeasured,
            "{} has a follow-up grade nobody recorded running",
            model.id
        );
    }
}

/// **A grade under the words a turn shows is read; one under any other
/// digest is not**, for ADR 0133's reason.
#[test]
fn only_a_grade_under_the_words_a_turn_shows_is_read() {
    let now =
        FollowUpsMeasured::parse(&one("reliably", Some(THE_WORDS_A_TURN_SHOWS), true)).unwrap();
    assert_eq!(now.for_the_turn("gpt-4o", "OpenAI"), Driving::Reliably);
    assert_eq!(
        now.for_the_turn("gpt-4o", "somewhere else"),
        Driving::NotMeasured
    );
    let before = FollowUpsMeasured::parse(&one(
        "reliably",
        Some(crate::THE_WORDS_A_TURN_SHOWED_BEFORE),
        true,
    ))
    .unwrap();
    assert_eq!(
        before.for_the_turn("gpt-4o", "OpenAI"),
        Driving::NotMeasured
    );
}

/// **A measurement with no digest, no counts, or a grade of not-measured is
/// refused**: each would be a grade about no turn, or no grade at all.
#[test]
fn a_follow_up_grade_about_no_turn_is_refused() {
    assert!(matches!(
        FollowUpsMeasured::parse(&one("reliably", None, true)),
        Err(FollowUpsError::Unanchored(_))
    ));
    assert!(matches!(
        FollowUpsMeasured::parse(&one("reliably", Some(THE_WORDS_A_TURN_SHOWS), false)),
        Err(FollowUpsError::Unanchored(_))
    ));
    assert!(matches!(
        FollowUpsMeasured::parse(&one("not-measured", Some(THE_WORDS_A_TURN_SHOWS), true)),
        Err(FollowUpsError::NotAMeasurement(_))
    ));
}
