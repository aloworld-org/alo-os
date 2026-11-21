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

/// **Every turn is shown every verb offered** (the coordinator, 2026-11-20):
/// a change to one verb's words or arguments, as the model reads them, owns
/// every request — the one that fails is often one whose road never uses that
/// verb, choosing it (#1413: the search box choosing the typing that saves).
#[test]
fn a_changed_verb_owns_every_request_though_its_road_never_uses_it() {
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
    assert_eq!(named(&own), ["one", "two", "three"]);
    assert_eq!(
        own[1].why(),
        &WhyItsOwn::EveryTurnIsShownAVerbOtherwise("a".to_owned()),
        "`two` never asks for `a` and is still shown it"
    );
}

/// A verb that is new is offered to every request, so every request is the
/// change's own, though no request's line changed.
#[test]
fn a_new_verb_owns_every_request() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1"), ("b", "b-1"), ("c", "c-1")],
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
            .all(|one| one.why() == &WhyItsOwn::EveryTurnIsShownAVerbOtherwise("c".to_owned()))
    );
}

/// A verb taken off the list is no longer shown to any turn: every request
/// that remains is the change's own.
#[test]
fn a_verb_taken_away_owns_every_request() {
    let after = Declared::by_hand(
        "turn-1",
        &[("a", "a-1")],
        &[("one", "one-1", &["a"]), ("three", "three-1", &["a"])],
    );
    assert_eq!(named(&its_own(&the_base(), &after)), ["one", "three"]);
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
/// every request, the ones whose road never moves a slider among them.
#[test]
fn on_the_real_set_a_changed_verb_owns_every_request() {
    let set = the_set().expect("the set builds");
    let verbs = the_verbs();
    let before = Declared::of(&set, &verbs).expect("declares");
    let after = before.clone().with_the_verb_changed(SET_VALUE);
    let own = its_own(&before, &after);
    let every: Vec<&str> = set.requests().iter().map(|one| one.named()).collect();
    assert_eq!(named(&own), every);
    assert!(
        before
            .requests()
            .iter()
            .any(|one| !one.asks_for().iter().any(|verb| verb == SET_VALUE)),
        "some request never asks for {SET_VALUE}"
    );
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
    assert_eq!(named(&own), ["one", "two", "three"]);
    assert_eq!(own[0].why(), &WhyItsOwn::ItsRoadChanged);
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

/// **A base declared in the first scheme is read in that scheme** (2026-11-20):
/// the change that moved requests to what they show owns nothing by the move,
/// because the head is fingerprinted the base's way to compare. A change to a
/// request is still its own across the move.
#[test]
fn a_base_in_the_first_scheme_is_compared_in_that_scheme() {
    use crate::the_declaration::Scheme;

    let set = the_set().expect("the set builds");
    let verbs = the_verbs();
    let first = Declared::by_the_scheme(&set, &verbs, Scheme::First)
        .expect("declares")
        .written();
    assert!(
        !first.contains("\nfingerprints\t"),
        "the first scheme's files named no scheme"
    );
    assert!(
        super::its_own_against(&first, &set, &verbs)
            .expect("it reads")
            .is_empty(),
        "moving the scheme owns nothing"
    );
    let changed: String = first
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
    assert_eq!(
        named(&super::its_own_against(&changed, &set, &verbs).expect("it reads")),
        ["list a folder"]
    );
}

/// **A change owns the whole set** when what every turn is told changed or
/// any verb did — and then it is compared with its base
/// (`crate::compared_with_its_base`) — and not when only its own requests'
/// roads did.
#[test]
fn which_changes_own_the_whole_set() {
    let some = [ItsOwn::because("one", WhyItsOwn::ItsRoadChanged)];
    assert!(!super::owns_the_whole_set(&some));
    assert!(!super::owns_the_whole_set(&[]));
    assert!(super::owns_the_whole_set(&[ItsOwn::because(
        "one",
        WhyItsOwn::EveryTurnIsShownAVerbOtherwise("a".to_owned())
    )]));
    assert!(super::owns_the_whole_set(&[ItsOwn::because(
        "one",
        WhyItsOwn::EveryTurnIsToldOtherwise
    )]));
}
