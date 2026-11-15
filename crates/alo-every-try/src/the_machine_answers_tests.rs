//! Each answer is one the protocol writes and the agent reads, says what a
//! machine at full access says, and carries what the road's next step needs.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_adapters::fallback_verbs::{APPLICATION, NAME, READ_WINDOW, TEXT, TYPE_TEXT};
use alo_applications::OPEN_FILE_IN_APPLICATION;
use alo_capability::{Call, Given};
use alo_protocol::{Done, ToAnAgent};
use alo_publishing::PUBLISH_TO_GITHUB_PAGES;

use super::answering;
use crate::a_road::Answered;
use crate::testing::{the_verbs, the_words};

const EDITOR: &str = "com.vscodium.codium";

fn call(verb: &str, given: &[(&str, &str)]) -> Call {
    let given: Vec<(&str, Given)> = given
        .iter()
        .map(|(named, is)| (*named, Given::text(*is)))
        .collect();
    the_verbs()
        .call(verb, &given)
        .expect("a call the machine accepts")
}

fn written(answer: &ToAnAgent) -> String {
    answer.written().expect("the protocol writes every answer")
}

#[test]
fn a_file_opened_at_full_access_is_done_without_asking_and_names_the_application() {
    let answer = answering(
        &Answered::AFileOpened,
        &call(
            OPEN_FILE_IN_APPLICATION,
            &[
                (APPLICATION, EDITOR),
                (
                    alo_applications::opening_a_file::PATH,
                    "/var/home/alo/hello.txt",
                ),
            ],
        ),
        &the_words(),
    );
    let ToAnAgent::DoneWithoutAsking { did, said } = &answer else {
        panic!("{answer:?}");
    };
    assert_eq!(
        did,
        &Done::of_an_application_act(alo_acting::OPEN_FILE, EDITOR)
    );
    assert!(!said.is_a_bug(), "{}", said.text());
    assert!(written(&answer).contains(EDITOR));
}

#[test]
fn a_window_read_shows_the_field_by_the_name_the_road_types_into() {
    let answer = answering(
        &Answered::AWindowShowing {
            title: "hello.txt - VSCodium".to_owned(),
            field: "hello.txt".to_owned(),
        },
        &call(READ_WINDOW, &[(APPLICATION, EDITOR)]),
        &the_words(),
    );
    assert!(matches!(answer, ToAnAgent::Did(_)), "{answer:?}");
    let line = written(&answer);
    assert!(line.contains("hello.txt - VSCodium"), "{line}");
    assert!(line.contains(r#""hello.txt""#), "{line}");
}

#[test]
fn a_text_typed_says_it_was_saved_and_how_many_characters() {
    let text = "Written by alo in VSCodium";
    let answer = answering(
        &Answered::TypedAndSaved,
        &call(
            TYPE_TEXT,
            &[(APPLICATION, EDITOR), (NAME, "hello.txt"), (TEXT, text)],
        ),
        &the_words(),
    );
    let ToAnAgent::DoneWithoutAsking { did, said } = &answer else {
        panic!("{answer:?}");
    };
    assert_eq!(
        did,
        &Done::of_a_typing(EDITOR, "hello.txt", text.chars().count(), true, true)
    );
    assert!(!said.is_a_bug(), "{}", said.text());
}

#[test]
fn a_folder_published_names_the_address_it_went_to() {
    let answer = answering(
        &Answered::Published {
            account: "alo-demo".to_owned(),
        },
        &call(
            PUBLISH_TO_GITHUB_PAGES,
            &[(
                alo_publishing::words::FOLDER,
                "/var/home/alo/Documents/site",
            )],
        ),
        &the_words(),
    );
    let ToAnAgent::DoneWithoutAsking { said, .. } = &answer else {
        panic!("{answer:?}");
    };
    assert!(!said.is_a_bug(), "{}", said.text());
    assert!(
        written(&answer).contains("https://alo-demo.github.io/"),
        "{}",
        written(&answer)
    );
}

#[test]
fn an_application_opened_is_done_without_asking() {
    let answer = answering(
        &Answered::AnApplicationOpened,
        &call(
            alo_applications::verbs::OPEN_APPLICATION,
            &[(APPLICATION, "org.mozilla.firefox")],
        ),
        &the_words(),
    );
    let ToAnAgent::DoneWithoutAsking { said, .. } = &answer else {
        panic!("{answer:?}");
    };
    assert!(!said.is_a_bug(), "{}", said.text());
    assert!(written(&answer).contains("org.mozilla.firefox"));
}
