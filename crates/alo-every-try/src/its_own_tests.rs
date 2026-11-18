//! Which requests a change owns: those it adds, those whose road it changes,
//! those that ask for a verb it adds or changes, and every request when it
//! changes what every turn is told — and none when it changes nothing the
//! model is offered.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_adapters::fallback_verbs::SET_VALUE;

use super::{ItsOwn, WhyItsOwn, its_own};
use crate::testing::the_verbs;
use crate::the_declaration::Declared;
use crate::the_set::the_set;

/// A declaration of two verbs and three requests: `one` asks for `a`, `two`
/// for `b`, `three` for both.
fn the_base() -> Declared {
    Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["a", "b"]),
        ],
    )
}

fn named(own: &[ItsOwn]) -> Vec<&str> {
    own.iter().map(ItsOwn::named).collect()
}

/// **A change that touches nothing the model is offered owns nothing**, so
/// nothing is tried and nothing is spent.
#[test]
fn a_change_to_nothing_the_model_is_offered_owns_nothing() {
    assert!(its_own(&the_base(), &the_base()).is_empty());
    let real = Declared::of(&the_set().expect("the set builds"), &the_verbs()).expect("declares");
    assert!(its_own(&real, &real).is_empty());
}

#[test]
fn a_new_request_is_its_own_and_no_other() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["a", "b"]),
            ("four", "four-1", &["a"]),
        ],
    );
    let own = its_own(&the_base(), &after);
    assert_eq!(named(&own), ["four"]);
    assert_eq!(own[0].why(), &WhyItsOwn::ItIsNew);
}

#[test]
fn a_request_whose_road_changed_is_its_own_and_no_other() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-2", &["b"]),
            ("three", "three-1", &["a", "b"]),
        ],
    );
    let own = its_own(&the_base(), &after);
    assert_eq!(named(&own), ["two"]);
    assert_eq!(own[0].why(), &WhyItsOwn::ItsRoadChanged);
}

/// **A verb whose words or arguments changed owns every request that asks for
/// it**, and a request that does not ask for it is not its own.
#[test]
fn a_changed_verb_owns_every_request_that_asks_for_it() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-2"), ("b", "b-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["a", "b"]),
        ],
    );
    let own = its_own(&the_base(), &after);
    assert_eq!(named(&own), ["one", "three"]);
    assert_eq!(own[0].why(), &WhyItsOwn::ItAsksFor("a".to_owned()));
}

/// A verb that is new is a verb that changed: every request asking for it is
/// the change's own, though the request's line says the same.
#[test]
fn a_new_verb_owns_every_request_that_asks_for_it() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1"), ("c", "c-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b", "c"]),
            ("three", "three-1", &["a", "b"]),
        ],
    );
    assert_eq!(named(&its_own(&the_base(), &after)), ["two"]);
}

/// **What every turn is told changed**: the instructions, the machine a turn
/// is told about, the envelope — every request is the change's own.
#[test]
fn a_change_to_what_every_turn_is_told_owns_every_request() {
    let after = Declared::by_hand(
        "turn-2",
        &[("a", "a-1"), ("b", "b-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["a", "b"]),
        ],
    );
    let own = its_own(&the_base(), &after);
    assert_eq!(named(&own), ["one", "two", "three"]);
    assert!(
        own.iter()
            .all(|one| one.why() == &WhyItsOwn::EveryTurnIsToldOtherwise)
    );
}

/// A request taken out of the set is nothing to try.
#[test]
fn a_request_taken_away_owns_nothing() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1")],
        &[("one", "one-1", &["a"]), ("three", "three-1", &["a", "b"])],
    );
    assert!(its_own(&the_base(), &after).is_empty());
}

/// **On the real set**: a change to the words of `accessible.set_value` owns
/// exactly the requests that ask for it — some, and not all.
#[test]
fn on_the_real_set_a_changed_verb_owns_only_its_requests() {
    let set = the_set().expect("the set builds");
    let verbs = the_verbs();
    let before = Declared::of(&set, &verbs).expect("declares");
    let after = before.clone().with_the_verb_changed(SET_VALUE);
    let own = its_own(&before, &after);
    let asking: Vec<&str> = before
        .requests()
        .iter()
        .filter(|one| one.asks_for().iter().any(|verb| verb == SET_VALUE))
        .map(|one| one.named())
        .collect();
    assert!(!asking.is_empty(), "some request asks for {SET_VALUE}");
    assert!(asking.len() < set.requests().len());
    assert_eq!(named(&own), asking);
}

/// A request that is its change's own for two reasons is tried once: listed
/// once, by the first reason.
#[test]
fn a_request_owned_twice_over_is_listed_once() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-2"), ("b", "b-1")],
        &[
            ("one", "one-2", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["a", "b"]),
        ],
    );
    let own = its_own(&the_base(), &after);
    assert_eq!(named(&own), ["one", "three"]);
    assert_eq!(own[0].why(), &WhyItsOwn::ItsRoadChanged);
}

/// Two verbs changed own every request asking for either.
#[test]
fn two_changed_verbs_own_every_request_asking_for_either() {
    let base = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1"), ("c", "c-1")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["c"]),
        ],
    );
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-2"), ("b", "b-1"), ("c", "c-2")],
        &[
            ("one", "one-1", &["a"]),
            ("two", "two-1", &["b"]),
            ("three", "three-1", &["c"]),
        ],
    );
    assert_eq!(named(&its_own(&base, &after)), ["one", "three"]);
}

/// **Against the base's file as the workflow hands it over**: the committed
/// declaration owns nothing, one request's line changed owns that request, and
/// a file that does not read is refused rather than read as owning nothing.
#[test]
fn against_the_bases_file_as_written() {
    let set = the_set().expect("the set builds");
    let verbs = the_verbs();
    let committed = crate::the_declaration::AS_COMMITTED;
    assert!(
        super::its_own_against(committed, &set, &verbs)
            .expect("the committed file reads")
            .is_empty()
    );
    let changed: String = committed
        .lines()
        .map(|line| {
            if line.starts_with("request\tlist a folder\t") {
                line.replacen(
                    "request\tlist a folder\t",
                    "request\tlist a folder\tchanged",
                    1,
                )
            } else {
                line.to_owned()
            }
        })
        .map(|line| line + "\n")
        .collect();
    let own = super::its_own_against(&changed, &set, &verbs).expect("it reads");
    assert_eq!(named(&own), ["list a folder"]);
    assert!(super::its_own_against("nonsense\n", &set, &verbs).is_err());
}
