//! **Every grade the catalogue ships is what its counts earn, and none owes a
//! second round.**
//!
//! Task 6 of `docs/autonomy/v0-5-the-models-measured-plan.md`. The catalogue
//! writes beside every grade how many attempts drove the verbs and how many were
//! made. This holds those numbers to the bar that made the grade —
//! [`alo_driving::grade_of`], the arithmetic `Measured::grade` is — so a grade
//! cannot be typed that its own counts do not earn, and to the rule that a
//! grade from one round landing within one attempt of a line is not written
//! until a second round has been run: the runtime samples every answer, and ten
//! samples can land either side of a line.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_driving::{THE_SET, grade_of, owes_a_second_round};
use alo_models::{Catalogue, Driving};

/// How many attempts one round of the fixed set is.
fn one_round() -> usize {
    THE_SET.len()
}

/// What is wrong with a grade and its counts, or `None`.
fn what_is_wrong(grade: Driving, drove: usize, of: usize) -> Option<String> {
    let earned = grade_of(drove, of);
    if earned != grade {
        return Some(format!(
            "{drove} of {of} earns {earned:?} and the entry says {grade:?}"
        ));
    }
    if owes_a_second_round(drove, of, one_round()) {
        return Some(format!(
            "{drove} of {of} is one round landing within one attempt of a line: run a second \
             round before writing it"
        ));
    }
    None
}

#[test]
fn every_grade_the_catalogue_ships_is_what_its_counts_earn() {
    let shipped = Catalogue::built_in().unwrap();
    let mut checked = 0;
    for entry in &shipped.models {
        let Some(measured) = &entry.measured else {
            continue;
        };
        let (drove, of) = (
            usize::try_from(measured.drove.unwrap()).unwrap(),
            usize::try_from(measured.of.unwrap()).unwrap(),
        );
        assert_eq!(
            what_is_wrong(entry.drives_verbs, drove, of),
            None,
            "{}",
            entry.id
        );
        checked += 1;
    }
    assert!(checked >= 10, "only {checked} grades were checked");
}

/// **And the check refuses what it exists to refuse.**
#[test]
fn the_check_refuses_a_grade_its_counts_do_not_earn_and_a_round_too_close_to_a_line() {
    // A grade the counts do not earn.
    assert!(what_is_wrong(Driving::Sometimes, 4, 20).is_some());
    assert!(what_is_wrong(Driving::Reliably, 17, 20).is_some());
    // One round, within one of five or of nine.
    for drove in [4, 5, 6, 8, 9, 10] {
        assert!(
            owes_a_second_round(drove, 10, one_round()),
            "{drove} of 10 was not held for a second round"
        );
    }
    // One round, clear of both lines — and any grade out of two rounds.
    for drove in [0, 1, 2, 3, 7] {
        assert!(
            !owes_a_second_round(drove, 10, one_round()),
            "{drove} of 10"
        );
    }
    assert!(!owes_a_second_round(9, 20, one_round()));
    assert_eq!(what_is_wrong(Driving::Rarely, 8, 20), None);
    assert!(what_is_wrong(Driving::Rarely, 4, 10).is_some());
}
