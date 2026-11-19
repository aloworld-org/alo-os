//! Generic path syntax must not become invented machine or request context.
//!
//! These checks render the actual file registry through the product entry
//! points. Paths explicitly supplied by the person, their machine layout or a
//! read reply remain data, including a name formerly used as a stock example.
//! No model choice, filesystem access or installed acceptance is simulated.

#![expect(
    clippy::expect_used,
    reason = "missing declared arguments or incomplete rendered prompts fail the test"
)]

use std::path::Path;

use alo_capability::Takes;
use alo_instructing::{
    AStep, ATurnInProgress, Instructions, ThePersonsFolders, WhatItMayReach, shown_in_progress,
    shown_to_a_model, the_products_words,
};

/// The declared argument's type clause in the actual rendered catalogue.
fn type_clause<'a>(text: &'a str, argument: &str, purpose: &str) -> &'a str {
    let beginning = format!("  - {argument} (required; ");
    let ending = format!(") — {purpose}");
    let line = text
        .lines()
        .find(|line| line.starts_with(&beginning) && line.ends_with(&ending))
        .expect("the actual required argument and its own purpose remain in the prompt");
    line.strip_prefix(&beginning)
        .and_then(|line| line.strip_suffix(&ending))
        .expect("the real argument's type clause")
}

/// A slash alone describes syntax; a slash followed by a name supplies a value.
fn names_an_absolute_example(clause: &str) -> bool {
    clause.split_whitespace().any(|token| {
        let token = token.trim_matches(|character: char| {
            matches!(
                character,
                '\'' | '"' | '`' | '(' | ')' | '[' | ']' | ',' | ';' | '.'
            )
        });
        token.starts_with('/') && token.len() > 1
    })
}

#[test]
/// A path type explains the shape, not another person's file or an invented value.
fn product_path_hints_do_not_invent_absolute_file_or_folder_values() {
    let verbs = alo_files::file_verbs().expect("the actual shipped file operations");
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        let mut paths = 0;
        for verb in verbs.all() {
            for argument in verb.args() {
                if argument.takes() != &Takes::Path {
                    continue;
                }
                paths += 1;
                let hint = type_clause(&text, argument.name(), argument.purpose_as_written());
                assert!(
                    hint.contains("full path"),
                    "absolute-path form disappeared: {hint}"
                );
                assert!(
                    !names_an_absolute_example(hint),
                    "{}.{}, a generic path hint, supplied a concrete path as context: {hint}",
                    verb.name(),
                    argument.name()
                );
            }
        }
        assert!(
            paths >= 2,
            "the real registry must exercise multiple path arguments"
        );
    }
}

#[test]
/// A folder parameter cannot teach a filename just because files use the same type.
fn folder_parameters_keep_their_declared_meaning_without_a_file_example() {
    let verbs = alo_files::file_verbs().expect("the actual file registry");
    for instructions in Instructions::ALL {
        let text = the_products_words(instructions, &verbs);
        for name in ["list_folder", "find_in_folder", "archive_folder"] {
            let verb = verbs
                .all()
                .find(|verb| verb.name() == name)
                .expect("a real folder operation");
            let folder = verb
                .args()
                .iter()
                .find(|argument| argument.name() == "folder")
                .expect("the operation's actual folder parameter");
            assert_eq!(folder.takes(), &Takes::Path);
            let hint = type_clause(&text, folder.name(), folder.purpose_as_written());
            assert!(folder.purpose_as_written().contains("folder"));
            for file_suffix in [".txt", ".pdf", ".webm", ".zip"] {
                assert!(
                    !hint.contains(file_suffix),
                    "{name}.folder was taught a file example: {hint}"
                );
            }
            assert!(
                text.contains(r#"{"named":"folder","is":VALUE}"#),
                "the folder's actual argument rendering disappeared"
            );
        }
    }
}

#[test]
/// Fixing generic hints must not scrub the person's real explicit path.
fn an_explicit_user_path_remains_exact_even_when_it_matches_the_old_example() {
    let verbs = alo_files::file_verbs().expect("the actual file registry");
    for path in [
        "/home/anna/Documents/notes.txt",
        "/srv/shared/Quarter 4/notes.txt",
        "/opt/資料/.draft.md",
    ] {
        let request = format!("Read the file {path} exactly as named.");
        for instructions in Instructions::ALL {
            let text = shown_to_a_model(instructions, &verbs, &WhatItMayReach::nothing(), &request);
            assert!(
                text.ends_with(&format!("The request: {request}")),
                "the supplied path/request was rewritten: {text}"
            );
        }
    }
}

#[test]
/// Machine-supplied home/folder paths still travel separately from generic hints.
fn the_actual_machine_layout_is_rendered_without_being_replaced_by_an_example_home() {
    let verbs = alo_files::file_verbs().expect("the actual file registry");
    let home = Path::new("/var/home/river");
    let documents = Path::new("/var/home/river/Documents");
    let folders = ThePersonsFolders::of(home, [("Documents", documents)])
        .expect("an explicit test machine's layout");
    let reach = WhatItMayReach::these([documents]).in_this_home(Some(folders));
    for instructions in Instructions::ALL {
        let text = shown_to_a_model(instructions, &verbs, &reach, "List my Documents.");
        let (_, context) = text
            .split_once("This computer's own layout")
            .expect("the real machine's layout is still told");
        assert!(context.contains("/var/home/river"));
        assert!(context.contains("Documents: /var/home/river/Documents"));
        assert!(!context.contains("/home/anna"));
        assert!(text.ends_with("The request: List my Documents."));
    }
}

#[test]
/// A short real-protocol read reply retains exact returned paths in a continued turn.
fn a_live_returned_path_remains_in_the_actual_continued_prompt() {
    let verbs = alo_files::file_verbs().expect("the actual file registry");
    let asked = r#"{"format":1,"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/srv/project","named":"report","most":2}}}}"#;
    let came_back = r#"{"format":1,"tells":{"did":{"found":{"files":["/home/anna/Documents/notes.txt","/srv/project/2026 report.pdf"],"could_not_be_named":0,"cut_short":false}}}}"#;
    let steps = [AStep::of(asked, came_back)];
    let reach = WhatItMayReach::nothing();
    for instructions in Instructions::ALL {
        let text = shown_in_progress(
            instructions,
            &verbs,
            ATurnInProgress::of("Read the returned report.", &steps, &reach),
        );
        let (_, continuation) = text
            .split_once("You have already asked for this, and this came back:")
            .expect("the real continuation renderer includes the observed read");
        assert!(continuation.contains(&format!("You asked: {asked}")));
        assert!(
            continuation.contains(&format!("The answer: {came_back}")),
            "a real returned path was scrubbed or rewritten: {continuation}"
        );
    }
}
