//! **The confirm round**, with a model that writes chosen roads: a change's
//! own request that slips on the first ten is tried ten more and judged on
//! the twenty, the floor and the ratchet applied there.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};

use super::judging_its_own;
use crate::a_try::TheLevel;
use crate::counted::Counted;
use crate::its_own::{ItsOwn, WhyItsOwn};
use crate::testing::{Writing, answering, proposing, running, the_verbs, the_words};
use crate::the_set::{TheSet, the_set};

const OPENING: &str = "open an application by name";

fn the_one() -> TheSet {
    the_set()
        .expect("the set builds")
        .retaining(|request| request.named() == OPENING)
}

fn mine() -> [ItsOwn; 1] {
    [ItsOwn::because(OPENING, WhyItsOwn::ItsRoadChanged)]
}

/// The lines of `good` tries that take the road and `bad` that run a
/// command, in that order.
fn tries(good: usize, bad: usize) -> Vec<String> {
    let on = || {
        vec![
            proposing(OPEN_APPLICATION, &[(APPLICATION, "org.mozilla.firefox")]),
            answering("Firefox is open."),
        ]
    };
    (0..good)
        .flat_map(|_| on())
        .chain((0..bad).map(|_| running("firefox &")))
        .collect()
}

/// Judge `OPENING` as the change's own, the model writing `rounds` in order.
fn judged(rounds: &[Vec<String>], reached: bool) -> (Counted, usize, String) {
    let lines: Vec<String> = rounds.iter().flatten().cloned().collect();
    let mut model = Writing::these(&lines);
    let judgement = judging_its_own(
        TheLevel::FullAccess,
        &the_one(),
        &mine(),
        0,
        &|_| reached,
        &the_verbs(),
        &the_words(),
        &mut model,
        &mut |_, _, _| {},
        "a scripted model",
    );
    let tried = judgement
        .tallied()
        .iter()
        .find(|one| one.request().named() == OPENING)
        .map_or(0, |one| one.tries().len());
    (judgement.counted(), tried, judgement.said().to_owned())
}

/// Nothing slipped: ten tries, held, and no second round.
#[test]
fn ten_on_the_road_are_held_without_a_second_round() {
    let (counted, tried, said) = judged(&[tries(10, 0)], false);
    assert_eq!(counted, Counted::ItsOwnHeld);
    assert_eq!(tried, 10);
    assert!(!said.contains("more tries of what was flagged"), "{said}");
}

/// **A flag, then a confirm that passes**: 8 of 10, then 10 of 10 — 18 of 20
/// meets the floor.
#[test]
fn a_flag_then_a_confirm_that_passes() {
    let (counted, tried, said) = judged(&[tries(8, 2), tries(10, 0)], false);
    assert_eq!(tried, 20, "the flagged request was tried ten more");
    assert_eq!(counted, Counted::ItsOwnHeld);
    assert!(
        said.contains("After 10 more tries of what was flagged"),
        "{said}"
    );
    assert!(said.contains("18 of 20"), "{said}");
}

/// **A flag, then a confirm that fails**: 8 of 10, then 7 of 10 — 15 of 20 is
/// below the floor of 18, so it is not a pass: it goes to its base, to be
/// judged against it (exit 3).
#[test]
fn a_flag_then_a_confirm_that_fails() {
    let (counted, tried, said) = judged(&[tries(8, 2), tries(7, 3)], false);
    assert_eq!(tried, 20);
    assert_eq!(counted, Counted::ToCompareWithItsBase);
    assert_ne!(counted.exit_code(), 0);
    assert!(said.contains("15 of 20"), "{said}");
}

/// **The floor on twenty**: 17 of 20 is below it and goes to its base; 18
/// holds.
#[test]
fn the_floor_on_twenty_is_eighteen() {
    assert_eq!(
        judged(&[tries(8, 2), tries(9, 1)], false).0,
        Counted::ToCompareWithItsBase
    );
    assert_eq!(
        judged(&[tries(8, 2), tries(10, 0)], false).0,
        Counted::ItsOwnHeld
    );
}

/// **The ratchet on twenty**: a request recorded at every try that slips once
/// on ten is tried ten more, and 19 of 20 is below every try.
#[test]
fn the_ratchet_on_twenty_is_every_try() {
    let (counted, tried, _) = judged(&[tries(9, 1), tries(10, 0)], true);
    assert_eq!(tried, 20);
    assert_eq!(counted, Counted::ItsOwnSlipped);
    let (counted, tried, said) = judged(&[tries(10, 0)], true);
    assert_eq!((counted, tried), (Counted::ItsOwnHeld, 10));
    assert!(
        said.contains("Held to every try at this level"),
        "the summary says why: {said}"
    );
    let (_, _, said) = judged(&[tries(10, 0)], false);
    assert!(!said.contains("Held to every try"), "{said}");
}

/// Judge `OPENING` and say which of its own are to be compared with the base.
fn to_compare(rounds: &[Vec<String>], reached: bool) -> (Counted, Vec<String>) {
    let lines: Vec<String> = rounds.iter().flatten().cloned().collect();
    let mut model = Writing::these(&lines);
    let judgement = judging_its_own(
        TheLevel::FullAccess,
        &the_one(),
        &mine(),
        0,
        &|_| reached,
        &the_verbs(),
        &the_words(),
        &mut model,
        &mut |_, _, _| {},
        "a scripted model",
    );
    (
        judgement.counted(),
        judgement.to_compare_with_its_base().to_vec(),
    )
}

/// **An own request below the floor on twenty is compared with its base**
/// (the coordinator, 2026-11-20): it may be weak on the base too — the search
/// box, for a change that only read windows better — so it is not blocked
/// yet; it is named, to be tried on the base and judged as a whole-set change
/// is.
#[test]
fn an_own_request_below_the_floor_on_twenty_is_compared_with_its_base() {
    let (counted, named) = to_compare(&[tries(8, 2), tries(7, 3)], false);
    assert_eq!(counted, Counted::ToCompareWithItsBase);
    assert_eq!(counted.exit_code(), 3, "neither a pass nor a fail yet");
    assert_eq!(named, [OPENING.to_owned()]);
}

/// **The ratchet still blocks outright**: a request recorded at every try is
/// not rescued by a weak base.
#[test]
fn a_broken_ratchet_is_not_compared_with_its_base() {
    let (counted, named) = to_compare(&[tries(9, 1), tries(9, 1)], true);
    assert_eq!(counted, Counted::ItsOwnSlipped);
    assert!(named.is_empty());
}

/// Held on twenty needs no base.
#[test]
fn held_on_twenty_needs_no_base() {
    let (counted, named) = to_compare(&[tries(8, 2), tries(10, 0)], false);
    assert_eq!(counted, Counted::ItsOwnHeld);
    assert!(named.is_empty());
}
