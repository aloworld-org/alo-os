//! **A path where nothing is, refused as the machine refuses it** — and
//! everything else left to the road.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_applications::opening_a_file::PATH;
use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};
use alo_instructing::WhatItMayReach;
use alo_protocol::ToAnAgent;

use super::nothing_is_at;
use crate::as_the_machine_names::AsTheMachineNames;
use crate::testing::{proposing, read, the_verbs, the_words};

const THE_FILE: &str = "/var/home/alo/Downloads/report.pdf";
const ELSEWHERE: &str = "/var/home/alo/Documents/report.pdf";

fn there() -> Vec<PathBuf> {
    vec![PathBuf::from(THE_FILE)]
}

fn asked(path: &str) -> alo_protocol::FromAnAgent {
    read(&proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, "org.kde.dolphin"), (PATH, path)],
    ))
}

fn the_machine() -> AsTheMachineNames {
    AsTheMachineNames::of(&WhatItMayReach::nothing().at_full_access(true))
}

/// **A path where nothing is** is refused in the machine's own words
/// (`alo_files::RealError::Nothing`, as `the_sentence_is_about_something`
/// says it before anybody is asked), naming the path the model gave.
#[test]
fn a_path_where_nothing_is_is_refused_in_the_machines_words() {
    let refused = nothing_is_at(
        &there(),
        &asked(ELSEWHERE),
        &the_verbs(),
        &the_machine(),
        &the_words(),
    )
    .expect("refused");
    let expected = alo_files::RealError::Nothing {
        path: ELSEWHERE.to_owned(),
    }
    .said(&the_words());
    assert_eq!(refused, ToAnAgent::refused(&expected));
}

/// **The refusal cases of the refusal**: a path that is there, a folder that
/// holds what is there, a call with no path at all, and a request that
/// declares nothing about what is there are all left to the road.
#[test]
fn what_is_there_and_what_names_no_path_are_left_to_the_road() {
    let verbs = the_verbs();
    let machine = the_machine();
    let words = the_words();
    assert_eq!(
        nothing_is_at(&there(), &asked(THE_FILE), &verbs, &machine, &words),
        None,
        "the file is there"
    );
    assert_eq!(
        nothing_is_at(
            &there(),
            &asked("/var/home/alo/Downloads"),
            &verbs,
            &machine,
            &words
        ),
        None,
        "the folder that holds it is there"
    );
    let no_path = read(&proposing(
        OPEN_APPLICATION,
        &[(APPLICATION, "org.kde.dolphin")],
    ));
    assert_eq!(
        nothing_is_at(&there(), &no_path, &verbs, &machine, &words),
        None,
        "a call that names no path"
    );
    assert_eq!(
        nothing_is_at(&[], &asked(ELSEWHERE), &verbs, &machine, &words),
        None,
        "a request that says nothing of what is there"
    );
}
