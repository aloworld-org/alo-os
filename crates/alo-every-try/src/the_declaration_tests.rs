//! The declaration is what the code offers the model, line for line, and the
//! committed file is held to it.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_applications::verbs::OPEN_APPLICATION;
use alo_capability::{Arg, Effect, Requires, Takes, Verb, Verbs};
use alo_strings::Word;

use super::{AS_COMMITTED, Declared, a_requests_line, a_verbs_line, the_every_turn_line};
use crate::a_road::{ARequest, Answered, Step};
use crate::testing::the_verbs;
use crate::the_set::the_set;

fn declared_with(verbs: &Verbs) -> Declared {
    Declared::of(&the_set().expect("the set builds"), verbs).expect("the set declares")
}

/// **The committed file is what the code offers.** A change to a verb's
/// words, a request's road or what every turn is told changes a line of it, so
/// a pull request that changes what the model chooses shows it in its diff —
/// and this test fails until the file says so.
#[test]
fn the_committed_declaration_is_what_the_code_offers() {
    let written = declared_with(&the_verbs()).written();
    assert!(
        written == AS_COMMITTED,
        "crates/alo-every-try/what-the-model-is-offered.txt is not what the code offers the \
         model. Write it again with `cargo run -p alo-every-try -- declare` and commit it.\n\n\
         The code declares:\n{written}"
    );
}

#[test]
fn declaring_twice_declares_the_same() {
    assert_eq!(
        declared_with(&the_verbs()).written(),
        declared_with(&the_verbs()).written()
    );
}

#[test]
fn what_is_written_reads_back_as_itself() {
    let declared = declared_with(&the_verbs());
    let read = Declared::read(&declared.written()).expect("its own words read");
    assert_eq!(read, declared);
}

#[test]
fn every_request_and_every_verb_offered_is_declared() {
    let set = the_set().expect("the set builds");
    let verbs = the_verbs();
    let declared = declared_with(&verbs);
    let named: Vec<&str> = declared.requests().iter().map(|one| one.named()).collect();
    let in_the_set: Vec<&str> = set.requests().iter().map(|one| one.named()).collect();
    assert_eq!(named, in_the_set);
    for verb in verbs.all() {
        assert!(declared.verb(verb.name()).is_some(), "{}", verb.name());
    }
}

/// A request is declared with every verb its roads ask for, the reads aside of
/// them included, because a change to any of them can change what it chooses.
#[test]
fn a_request_is_declared_with_every_verb_its_roads_ask_for() {
    let declared = declared_with(&the_verbs());
    let find = |named: &str| {
        declared
            .requests()
            .iter()
            .find(|one| one.named() == named)
            .unwrap_or_else(|| panic!("{named} is declared"))
    };
    assert_eq!(
        find("open an application by name").asks_for(),
        [OPEN_APPLICATION.to_owned()]
    );
    assert!(
        find("write in VSCodium")
            .asks_for()
            .contains(&alo_files::verbs::READ_FILE.to_owned()),
        "a read aside of the road is one of the verbs it asks for"
    );
}

/// **The refusal case**: a request asking for a verb the model is not offered
/// cannot be declared, because a change to that verb could never own it.
#[test]
fn a_request_asking_for_a_verb_not_offered_is_not_declared() {
    let mut fewer = Verbs::default();
    for verb in the_verbs()
        .all()
        .filter(|verb| verb.name() != OPEN_APPLICATION)
    {
        fewer.declare(verb.clone()).expect("each declares once");
    }
    let refused = Declared::of(&the_set().expect("the set builds"), &fewer)
        .expect_err("a request asks for a verb not offered");
    assert!(refused.to_string().contains(OPEN_APPLICATION), "{refused}");
}

/// **The refusal cases**: a line the declaration does not hold, and a request
/// declared twice, are refused rather than read as something.
#[test]
fn a_declaration_that_does_not_read_is_refused() {
    assert!(Declared::read("every-turn\tabc\nsomething else\tx\n").is_err());
    assert!(Declared::read("verb\tonly-a-name\n").is_err());
    assert!(
        Declared::read("every-turn\tabc\nrequest\tone\tf1\ta\nrequest\tone\tf2\ta\n").is_err(),
        "a request declared twice"
    );
    assert!(
        Declared::read("verb\ta\tf1\n").is_err(),
        "what every turn is told is declared"
    );
}

#[test]
fn comments_and_blank_lines_are_not_read() {
    let read = Declared::read("# a comment\n\nevery-turn\tabc\nverb\ta\tf1\nrequest\tone\tf2\ta\n")
        .expect("it reads");
    assert_eq!(read.every_turn(), "abc");
    assert_eq!(read.verb("a"), Some("f1"));
    assert_eq!(read.requests()[0].asks_for(), ["a".to_owned()]);
}

/// A verb of nobody's, with `purpose`, `arg` described as `about`, taking
/// `takes`, of `effect`.
fn a_verb(purpose: &'static str, about: &'static str, takes: Takes, effect: Effect) -> Verb {
    Verb::checked(
        "water_the_plants",
        Word::saying("testing.verb.water-the-plants.purpose", purpose),
        effect,
        vec![Arg::taking(
            "room",
            Word::saying("testing.verb.water-the-plants.room", about),
            takes,
        )],
        Requires::nothing_because("a room is not a path, a file or an application"),
        Word::saying(
            "testing.verb.water-the-plants.sentence",
            "water the plants in {room}",
        ),
    )
    .expect("the verb declares")
}

fn its_line(verb: &Verb) -> String {
    let mut verbs = Verbs::default();
    verbs.declare(verb.clone()).expect("it declares");
    let envelope = alo_turn::the_shape_a_provider_is_held_to::the_envelope_naming_the_verbs(&verbs);
    a_verbs_line(verb, &envelope)
}

/// **A verb's line answers to what the model is offered about it**: what it is
/// for, what each argument is for, what each takes, and its door — so a change
/// to any of them makes every request asking for it the change's own.
#[test]
fn a_verbs_line_changes_with_anything_the_model_is_offered_about_it() {
    let base = its_line(&a_verb(
        "water the plants in a room",
        "the room to water",
        Takes::name(64),
        Effect::Change,
    ));
    assert_eq!(
        base,
        its_line(&a_verb(
            "water the plants in a room",
            "the room to water",
            Takes::name(64),
            Effect::Change,
        )),
        "the same verb, the same line"
    );
    for (what, other) in [
        (
            "its purpose",
            a_verb(
                "water every plant",
                "the room to water",
                Takes::name(64),
                Effect::Change,
            ),
        ),
        (
            "an argument's purpose",
            a_verb(
                "water the plants in a room",
                "which room",
                Takes::name(64),
                Effect::Change,
            ),
        ),
        (
            "what an argument takes",
            a_verb(
                "water the plants in a room",
                "the room to water",
                Takes::Text,
                Effect::Change,
            ),
        ),
        (
            "its door",
            a_verb(
                "water the plants in a room",
                "the room to water",
                Takes::name(64),
                Effect::Read,
            ),
        ),
    ] {
        assert_ne!(base, its_line(&other), "a change to {what}");
    }
}

/// **A request's line answers to its whole definition**: its words, its road,
/// what a step expects, a read aside of it and another road.
#[test]
fn a_requests_line_changes_with_anything_in_its_definition() {
    let step =
        || Step::of(OPEN_APPLICATION, Answered::AnApplicationOpened).expecting("application", "a");
    let base = ARequest::asking("open it", "Open Firefox.", vec![step()]);
    assert_eq!(
        a_requests_line(&base),
        a_requests_line(&ARequest::asking("open it", "Open Firefox.", vec![step()]))
    );
    for (what, other) in [
        (
            "its words",
            ARequest::asking("open it", "Start Firefox.", vec![step()]),
        ),
        (
            "what a step expects",
            ARequest::asking(
                "open it",
                "Open Firefox.",
                vec![
                    Step::of(OPEN_APPLICATION, Answered::AnApplicationOpened)
                        .expecting("application", "b"),
                ],
            ),
        ),
        (
            "its road",
            ARequest::asking("open it", "Open Firefox.", vec![step(), step()]),
        ),
        (
            "a read aside of it",
            base.clone()
                .or_looking_at(Step::of(alo_files::verbs::READ_FILE, Answered::AFileOpened)),
        ),
        (
            "another road",
            base.clone().or_by_another_road(vec![step(), step()]),
        ),
    ] {
        assert_ne!(
            a_requests_line(&base),
            a_requests_line(&other),
            "a change to {what}"
        );
    }
}

/// **What every turn is told**: the words before the verbs, the machine a
/// turn is told about and the places it may install from — a change to any
/// of them makes every request the change's own.
#[test]
fn the_every_turn_line_changes_with_anything_every_turn_is_told() {
    let reach = || alo_instructing::WhatItMayReach::nothing().at_full_access(true);
    let places = ["flathub".to_owned()];
    let base = the_every_turn_line("the instructions", &reach(), &places);
    assert_eq!(
        base,
        the_every_turn_line("the instructions", &reach(), &places)
    );
    assert_ne!(
        base,
        the_every_turn_line("other instructions", &reach(), &places)
    );
    assert_ne!(
        base,
        the_every_turn_line(
            "the instructions",
            &alo_instructing::WhatItMayReach::nothing().at_full_access(false),
            &places
        )
    );
    assert_ne!(
        base,
        the_every_turn_line("the instructions", &reach(), &["fedora".to_owned()])
    );
}

/// More refusals: a verb declared twice, and a request asking for a verb the
/// declaration does not hold, which could never be told apart from a change.
#[test]
fn a_verb_twice_or_a_verb_nowhere_is_refused() {
    let refused = Declared::read(
        "# a comment
every-turn	abc
nonsense
",
    )
    .expect_err("a line of no kind is refused");
    assert!(
        refused.to_string().contains("at line 3"),
        "the line refused is named as a person counts it: {refused}"
    );
    assert!(Declared::read("every-turn\tabc\nverb\ta\tf1\nverb\ta\tf2\n").is_err());
    assert!(Declared::read("every-turn\tabc\nverb\ta\tf1\nrequest\tone\tf2\tb\n").is_err());
}
