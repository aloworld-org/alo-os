//! The set is the Demo 2 roads, every name in it is one the machine declares,
//! and its VSCodium request is the walk's own.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_adapters::fallback_verbs::{READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_publishing::PUBLISH_TO_GITHUB_PAGES;
use alo_walking::writing_in_vscodium::{THE_FILE, THE_TEXT, VSCODIUM, the_ask};

use super::{THE_HOME, THE_VERSION, the_set};
use crate::testing::the_verbs;

fn the_road_of(named: &str) -> Vec<&'static str> {
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == named)
        .unwrap_or_else(|| panic!("the set has {named}"));
    request
        .road()
        .iter()
        .map(crate::a_road::Step::verb)
        .collect()
}

#[test]
fn the_set_is_versioned_and_holds_the_three_demo_two_roads() {
    assert_eq!(THE_VERSION, 1);
    let set = the_set().expect("the set builds");
    let named: Vec<&str> = set
        .requests()
        .iter()
        .map(|request| request.named())
        .collect();
    assert_eq!(
        named,
        [
            "write in VSCodium",
            "publish to GitHub Pages",
            "open an application by name"
        ]
    );
}

#[test]
fn writing_in_vscodium_opens_reads_and_types_which_also_saves() {
    assert_eq!(
        the_road_of("write in VSCodium"),
        [OPEN_FILE_IN_APPLICATION, READ_WINDOW, TYPE_TEXT]
    );
}

#[test]
fn publishing_is_one_verb() {
    assert_eq!(
        the_road_of("publish to GitHub Pages"),
        [PUBLISH_TO_GITHUB_PAGES]
    );
}

#[test]
fn opening_an_application_is_one_verb() {
    assert_eq!(
        the_road_of("open an application by name"),
        [alo_applications::verbs::OPEN_APPLICATION]
    );
}

/// **The check and the walk ask the same thing**: the request is the walk's
/// own, and the application and the text the road expects are the walk's.
#[test]
fn the_vscodium_request_is_the_walks_own() {
    let set = the_set().expect("the set builds");
    let request = set
        .requests()
        .iter()
        .find(|request| request.named() == "write in VSCodium")
        .expect("the set has it");
    assert_eq!(request.words(), the_ask());
    let expected: Vec<(&str, &str)> = request
        .road()
        .iter()
        .flat_map(|step| step.expected().iter().map(|(arg, is)| (*arg, is.as_str())))
        .collect();
    assert!(expected.contains(&(alo_applications::verbs::APPLICATION, VSCODIUM)));
    assert!(expected.contains(&(TEXT, THE_TEXT)));
    assert!(expected.contains(&(
        alo_applications::opening_a_file::PATH,
        format!("{THE_HOME}/{THE_FILE}").as_str()
    )));
}

/// **Every name the set expects is one the machine offers**: each verb is on
/// the list a model is given, and each argument a step decides is one that
/// verb declares. A rename on either side turns this red.
#[test]
fn every_verb_and_argument_the_set_expects_is_on_the_machines_list() {
    let verbs = the_verbs();
    for request in the_set().expect("the set builds").requests() {
        for step in request.road() {
            let verb = verbs
                .of(step.verb())
                .unwrap_or_else(|| panic!("{} is offered", step.verb()));
            for (argument, _) in step.expected() {
                assert!(
                    verb.args().iter().any(|arg| arg.name() == *argument),
                    "{} takes {argument}",
                    step.verb()
                );
            }
        }
    }
}

/// The machine is the walk's: full access, the person's home, and the
/// applications a fresh machine installs, VSCodium among them.
#[test]
fn the_machine_is_told_full_access_the_home_and_the_applications() {
    let set = the_set().expect("the set builds");
    let told = set.may_reach().told();
    assert!(told.contains(THE_HOME), "{told}");
    assert!(told.contains(VSCODIUM), "{told}");
    assert!(told.contains("org.mozilla.firefox"), "{told}");
    let at_full_access = alo_instructing::WhatItMayReach::nothing().at_full_access(true);
    let full_access_told = at_full_access.told();
    assert!(!full_access_told.is_empty());
    assert!(told.contains(full_access_told.trim()), "{told}");
}

#[test]
fn one_request_can_be_tried_alone_and_an_unknown_one_is_none() {
    let set = the_set().expect("the set builds");
    let only = set
        .clone()
        .only("publish to GitHub Pages")
        .expect("it is in the set");
    assert_eq!(only.requests().len(), 1);
    assert!(set.only("nothing called this").is_none());
}
