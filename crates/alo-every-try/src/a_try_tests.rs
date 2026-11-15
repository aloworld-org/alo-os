//! A try passes only when the model takes every step and then answers; the
//! first step off the road ends it there, and nothing after it is asked.

use alo_adapters::fallback_verbs::{APPLICATION, NAME, READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_applications::opening_a_file::PATH;
use alo_instructing::{Instructions, WhatItMayReach, the_products_words};

use super::{Went, one_try};
use crate::a_road::{ARequest, Answered, Step};
use crate::testing::{Down, Writing, answering, proposing, reading, the_verbs, the_words};
use crate::went_wrong::WentWrong;

const EDITOR: &str = "com.vscodium.codium";
const FILE: &str = "/var/home/alo/hello.txt";
const THE_TEXT: &str = "Written by alo";

fn writing_in_the_editor() -> ARequest {
    ARequest::asking(
        "write in the editor",
        "Open hello.txt in VSCodium and write Written by alo in it",
        vec![
            Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
                .expecting(APPLICATION, EDITOR)
                .expecting(PATH, FILE),
            Step::of(
                READ_WINDOW,
                Answered::AWindowShowing {
                    title: "hello.txt - VSCodium".to_owned(),
                    field: "hello.txt".to_owned(),
                },
            )
            .expecting(APPLICATION, EDITOR),
            Step::of(TYPE_TEXT, Answered::TypedAndSaved)
                .expecting(APPLICATION, EDITOR)
                .expecting(NAME, "hello.txt")
                .expecting(TEXT, THE_TEXT),
        ],
    )
}

fn opening() -> String {
    proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, FILE)],
    )
}

fn looking() -> String {
    reading(READ_WINDOW, &[(APPLICATION, EDITOR)])
}

fn typing() -> String {
    proposing(
        TYPE_TEXT,
        &[(APPLICATION, EDITOR), (NAME, "hello.txt"), (TEXT, THE_TEXT)],
    )
}

fn tried(lines: &[String]) -> (super::Tried, Writing) {
    let mut model = Writing::these(lines);
    let tried = one_try(
        &writing_in_the_editor(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    (tried, model)
}

#[test]
fn every_step_and_then_an_answer_is_the_road() {
    let (tried, _) = tried(&[opening(), looking(), typing(), answering("It is written.")]);
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(
        tried.chose(),
        [
            OPEN_FILE_IN_APPLICATION,
            READ_WINDOW,
            TYPE_TEXT,
            "an answer in words"
        ]
    );
    assert_eq!(tried.asked(), 4);
}

/// **The refusal case**: a model that types before it reads the window has
/// left the road at step 2, the try says so, and the model is not asked again.
#[test]
fn a_step_off_the_road_fails_the_try_there_and_asks_nothing_more() {
    let (tried, model) = tried(&[opening(), typing(), answering("Done.")]);
    assert_eq!(
        tried.went(),
        &Went::WentWrong(WentWrong::AnotherVerb {
            step: 2,
            expected: READ_WINDOW.to_owned(),
            asked: TYPE_TEXT.to_owned(),
        }),
        "{tried:?}"
    );
    assert!(!tried.took_the_road());
    assert_eq!(tried.chose(), [OPEN_FILE_IN_APPLICATION, TYPE_TEXT]);
    assert_eq!(
        tried.said_last(),
        Some(typing().as_str()),
        "the line that left the road is kept"
    );
    assert_eq!(
        model.shown.len(),
        2,
        "nothing is asked after the road is left"
    );
}

#[test]
fn an_answer_before_the_road_is_finished_is_stopping_short() {
    let (tried, _) = tried(&[opening(), answering("I opened it.")]);
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::StoppedShort { done: 1, of: 3, .. })
        ),
        "{tried:?}"
    );
}

#[test]
fn asking_for_more_after_the_whole_road_is_going_on() {
    let (tried, _) = tried(&[opening(), looking(), typing(), looking()]);
    assert_eq!(
        tried.went(),
        &Went::WentWrong(WentWrong::WentOn {
            asked: READ_WINDOW.to_owned()
        }),
        "{tried:?}"
    );
}

#[test]
fn a_line_that_is_never_a_request_runs_out_of_rounds() {
    let broken: Vec<String> = (0..20)
        .map(|_| r#"{"format":1,"asks":{}}"#.to_owned())
        .collect();
    let (tried, _) = tried(&broken);
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::RanOutOfRounds { done: 0, of: 3 })
        ),
        "{tried:?}"
    );
}

#[test]
fn a_model_that_cannot_be_asked_is_not_a_model_that_chose_wrongly() {
    let tried = one_try(
        &writing_in_the_editor(),
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut Down,
    );
    assert!(matches!(tried.went(), Went::NotAnswered(_)), "{tried:?}");
}

/// **The model is shown the real turn's text**: it begins with the product's
/// own words for this registry, which is what makes the daemon pass it on
/// unwrapped, and the request is in it; the second round carries what the
/// machine answered to the first.
#[test]
fn the_model_is_shown_the_text_a_turn_shows() {
    let (_, model) = tried(&[opening(), looking(), typing(), answering("Done.")]);
    let products = the_products_words(Instructions::SHOWN_TO_A_TURN, &the_verbs());
    for shown in &model.shown {
        assert!(shown.starts_with(&products));
        assert_eq!(shown, shown.trim(), "shown as the daemon shows it: trimmed");
        assert!(shown.contains("Open hello.txt in VSCodium and write Written by alo in it"));
    }
    assert!(
        model.shown.get(1).is_some_and(
            |second| second.contains(EDITOR) && second.contains(OPEN_FILE_IN_APPLICATION)
        ),
        "{:?}",
        model.shown.get(1)
    );
    assert!(
        model
            .shown
            .get(2)
            .is_some_and(|third| third.contains("hello.txt - VSCodium")),
        "the third round is shown the window the second read"
    );
}

/// **Law 8, on the check itself**: a road that expects the wrong thing fails
/// a model that did exactly what the request asked. Were this green, the
/// check could not tell a right road from a wrong one.
#[test]
fn a_road_that_expects_the_wrong_application_fails_a_model_that_did_right() {
    let wrong = ARequest::asking(
        "write in the wrong editor",
        "Open hello.txt in VSCodium and write Written by alo in it",
        vec![
            Step::of(OPEN_FILE_IN_APPLICATION, Answered::AFileOpened)
                .expecting(APPLICATION, "org.gnome.TextEditor")
                .expecting(PATH, FILE),
        ],
    );
    let mut model = Writing::these(&[opening(), answering("Opened.")]);
    let tried = one_try(
        &wrong,
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    );
    assert!(
        matches!(
            tried.went(),
            Went::WentWrong(WentWrong::AnotherValue { step: 1, .. })
        ),
        "{tried:?}"
    );
}

/// The editor road with its read made **a look**: a step the model may skip
/// when it already knows, or take more than once.
fn writing_with_a_look() -> ARequest {
    let road = writing_in_the_editor()
        .road()
        .iter()
        .map(|step| {
            if step.verb() == READ_WINDOW {
                step.clone().a_look()
            } else {
                step.clone()
            }
        })
        .collect();
    ARequest::asking("write with a look", "the same words", road)
}

fn tried_on(request: &ARequest, lines: &[String]) -> super::Tried {
    let mut model = Writing::these(lines);
    one_try(
        request,
        &WhatItMayReach::nothing().at_full_access(true),
        &the_verbs(),
        &the_words(),
        &mut model,
    )
}

/// **A look may be skipped**: a model that knew what it needed and went
/// straight on took the road.
#[test]
fn a_look_may_be_skipped() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[opening(), typing(), answering("It is written.")],
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
}

/// **A look may be taken twice**, and the road goes on after it.
#[test]
fn a_look_may_be_taken_again() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[
            opening(),
            looking(),
            looking(),
            typing(),
            answering("It is written."),
        ],
    );
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    assert_eq!(
        tried.chose(),
        [
            OPEN_FILE_IN_APPLICATION,
            READ_WINDOW,
            READ_WINDOW,
            TYPE_TEXT,
            "an answer in words"
        ]
    );
}

/// **The refusal case**: skipping a look does not excuse the step after it —
/// a wrong verb there still leaves the road, at that step.
#[test]
fn skipping_a_look_still_holds_the_next_step() {
    let tried = tried_on(
        &writing_with_a_look(),
        &[opening(), opening(), answering("Opened.")],
    );
    assert_eq!(
        tried.went(),
        &Went::WentWrong(WentWrong::AnotherVerb {
            step: 3,
            expected: TYPE_TEXT.to_owned(),
            asked: OPEN_FILE_IN_APPLICATION.to_owned(),
        }),
        "{tried:?}"
    );
}

/// **A file named in the wrong folder is answered as the machine answers it**:
/// refused, with where one of that name is in the home, and a model that then
/// names it there takes the road (the owner's small-by-default answer).
#[test]
fn a_file_named_in_the_wrong_folder_is_told_where_it_is_and_may_be_named_again() {
    let wrong = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[
            (APPLICATION, EDITOR),
            (PATH, "/var/home/alo/Documents/hello.txt"),
        ],
    );
    let (tried, model) = tried(&[wrong, opening(), looking(), typing(), answering("Done.")]);
    assert_eq!(tried.went(), &Went::TheRoad, "{tried:?}");
    let told = model.shown.get(1).cloned().unwrap_or_default();
    assert!(
        told.contains("but one of that name is at /var/home/alo/hello.txt"),
        "{told}"
    );
}

/// **The refusal cases**: a second wrong path at the same step leaves the
/// road — the machine says where the file is once, and a model that ignores
/// it has left — and a file of another name is never answered this way.
#[test]
fn a_file_is_told_where_it_is_once_and_only_for_its_own_name() {
    let wrong = || {
        proposing(
            OPEN_FILE_IN_APPLICATION,
            &[
                (APPLICATION, EDITOR),
                (PATH, "/var/home/alo/Documents/hello.txt"),
            ],
        )
    };
    let (twice, _) = tried(&[wrong(), wrong(), answering("Done.")]);
    assert!(
        matches!(
            twice.went(),
            Went::WentWrong(WentWrong::AnotherValue { step: 1, .. })
        ),
        "{twice:?}"
    );
    let another = proposing(
        OPEN_FILE_IN_APPLICATION,
        &[(APPLICATION, EDITOR), (PATH, "/var/home/alo/notes.txt")],
    );
    let (other, _) = tried(&[another, answering("Done.")]);
    assert!(
        matches!(
            other.went(),
            Went::WentWrong(WentWrong::AnotherValue { step: 1, .. })
        ),
        "{other:?}"
    );
}
