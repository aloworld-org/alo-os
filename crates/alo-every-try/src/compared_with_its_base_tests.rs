//! "Compare with main" (the owner, 2026-11-20): a change that owns the whole
//! set is judged against its base, tried the same number of times, and a
//! flagged request is confirmed on twenty tries before it blocks.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected Err or index is the failure being reported"
)]

use super::{Judged, compared, the_flagged, verdict_of};
use crate::counted::Counted;
use crate::what_each_has_reached::{Measured, Reached};

/// What a run measured, written as its file holds it.
fn measured(lines: &[(&str, &str, usize, usize)]) -> Measured {
    let text: String = lines
        .iter()
        .map(|(level, named, on, of)| format!("measured\t{level}\t{named}\t{on}\t{of}\n"))
        .collect();
    Measured::read(&text).expect("it reads")
}

fn judged(change: &[(&str, &str, usize, usize)], base: &[(&str, &str, usize, usize)]) -> Judged {
    judged_with(change, base, &Reached::default())
}

fn judged_with(
    change: &[(&str, &str, usize, usize)],
    base: &[(&str, &str, usize, usize)],
    record: &Reached,
) -> Judged {
    let rows = compared(&measured(change), &measured(base), record);
    assert_eq!(rows.len(), 1);
    rows[0].judged().clone()
}

const L: &str = "full-access";

#[test]
fn no_worse_than_its_base_holds() {
    assert_eq!(
        judged(&[(L, "r", 10, 10)], &[(L, "r", 10, 10)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "r", 9, 10)], &[(L, "r", 10, 10)]),
        Judged::Held
    );
}

/// **A request below the bar on its base may stay below it**, no worse.
#[test]
fn a_request_weak_on_its_base_may_stay_weak_but_no_worse() {
    assert_eq!(
        judged(&[(L, "slider", 4, 10)], &[(L, "slider", 5, 10)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "slider", 0, 10)], &[(L, "slider", 0, 10)]),
        Judged::Held
    );
}

/// **At ten tries, a flag, not a verdict**: a drop of two, falling below
/// nine where the base had nine, and a ratchet broken are each flagged to be
/// confirmed on twenty.
#[test]
fn at_ten_tries_a_fault_is_flagged_to_be_confirmed() {
    assert_eq!(
        judged(&[(L, "slider", 3, 10)], &[(L, "slider", 5, 10)]),
        Judged::ToConfirm,
        "a drop of two"
    );
    assert_eq!(
        judged(&[(L, "close", 8, 10)], &[(L, "close", 9, 10)]),
        Judged::ToConfirm,
        "below nine where the base had nine"
    );
    let record = Reached::read("reached\tfull-access\tclose\n").expect("it reads");
    assert_eq!(
        judged_with(&[(L, "close", 9, 10)], &[(L, "close", 10, 10)], &record),
        Judged::ToConfirm,
        "the ratchet"
    );
}

/// **On twenty tries, the verdict**: a drop of four, below eighteen where the
/// base had eighteen, and a recorded request below twenty each block; less
/// holds.
#[test]
fn on_twenty_tries_the_verdict() {
    assert_eq!(
        judged(&[(L, "slider", 6, 20)], &[(L, "slider", 10, 20)]),
        Judged::ClearlyWorse
    );
    assert_eq!(
        judged(&[(L, "slider", 7, 20)], &[(L, "slider", 10, 20)]),
        Judged::Held
    );
    assert_eq!(
        judged(&[(L, "close", 17, 20)], &[(L, "close", 18, 20)]),
        Judged::BelowTheBarWhereItsBaseMetIt
    );
    assert_eq!(
        judged(&[(L, "close", 18, 20)], &[(L, "close", 20, 20)]),
        Judged::Held
    );
    let record = Reached::read("reached\tfull-access\tclose\n").expect("it reads");
    assert_eq!(
        judged_with(&[(L, "close", 19, 20)], &[(L, "close", 20, 20)], &record),
        Judged::BrokeTheRatchet
    );
    assert_eq!(
        judged_with(&[(L, "close", 20, 20)], &[(L, "close", 17, 20)], &record),
        Judged::Held
    );
}

/// The two runs of one tree measured on 2026-11-20 (CI runs 44779963562 and
/// 44793910551, at full access): "close an application" 10 then 8. Judged on
/// ten alone it would block a change that altered nothing; it is flagged.
#[test]
fn the_measured_wobble_of_an_unchanged_tree_is_flagged_not_blocked() {
    assert_eq!(
        judged(&[(L, "close", 8, 10)], &[(L, "close", 10, 10)]),
        Judged::ToConfirm
    );
}

/// A request new in the change has no base: it is held to the bar.
#[test]
fn a_request_new_in_the_change_is_held_to_the_bar() {
    assert_eq!(judged(&[(L, "new", 9, 10)], &[]), Judged::Held);
    assert_eq!(judged(&[(L, "new", 8, 10)], &[]), Judged::ToConfirm);
    assert_eq!(judged(&[(L, "new", 17, 20)], &[]), Judged::BelowTheBar);
}

/// **Not measured, never a pass**: too few tries, or a base tried a different
/// number of times, cannot be compared.
#[test]
fn what_cannot_be_compared_is_not_measured() {
    assert_eq!(
        judged(&[(L, "r", 5, 5)], &[(L, "r", 5, 5)]),
        Judged::NotMeasured
    );
    assert_eq!(
        judged(&[(L, "r", 10, 10)], &[(L, "r", 18, 20)]),
        Judged::NotMeasured
    );
}

/// **Two rounds add up**: what was measured for one request at one level in
/// the first ten and the confirming ten is judged on the total.
#[test]
fn the_confirming_round_is_added_to_the_first() {
    let rows = compared(
        &measured(&[(L, "close", 8, 10), (L, "close", 10, 10)]),
        &measured(&[(L, "close", 10, 10), (L, "close", 9, 10)]),
        &Reached::default(),
    );
    assert_eq!(rows[0].change(), (18, 20));
    assert_eq!(rows[0].base(), Some((19, 20)));
    assert_eq!(rows[0].judged(), &Judged::Held);
}

#[test]
fn the_verdict_and_what_is_flagged() {
    let rows = compared(
        &measured(&[
            (L, "a", 10, 10),
            (L, "b", 7, 10),
            ("ask-each-time", "a", 10, 10),
        ]),
        &measured(&[
            (L, "a", 10, 10),
            (L, "b", 10, 10),
            ("ask-each-time", "a", 10, 10),
        ]),
        &Reached::default(),
    );
    assert_eq!(
        the_flagged(&rows),
        [(L.to_owned(), "b".to_owned())],
        "only what is flagged is tried again"
    );
    assert_eq!(
        verdict_of(&rows),
        Counted::NotMeasured,
        "a flag is not yet a verdict"
    );
    let held = compared(
        &measured(&[(L, "a", 10, 10)]),
        &measured(&[(L, "a", 10, 10)]),
        &Reached::default(),
    );
    assert_eq!(verdict_of(&held), Counted::ItsOwnHeld);
    let worse = compared(
        &measured(&[(L, "a", 10, 20)]),
        &measured(&[(L, "a", 20, 20)]),
        &Reached::default(),
    );
    assert_eq!(verdict_of(&worse), Counted::ItsOwnSlipped);
}
