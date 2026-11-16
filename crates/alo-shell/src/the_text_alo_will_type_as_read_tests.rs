//! **A reader hears the text alo will type, whole, every hidden character
//! named in words where it stands**, under the window of the task asking.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_access::{Surface, TurnedOn};
use alo_models::InferenceSource;
use alo_tasks::ATask;

use super::*;
use crate::the_tasks_a_person_opened::TheTasks;

/// What this machine says, every crate's words declared.
fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A tree with two task windows open, and which is which.
fn two_windows(strings: &Strings) -> (ReadAloudTree, WhichTask, WhichTask) {
    let mut tasks = TheTasks::none_yet();
    let typing = tasks.and_now_this_one(ATask::asked_for(
        "write hello in VSCodium",
        InferenceSource::ThisMachine,
    ));
    let other = tasks.and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let tree = ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_task_windows_open(
            strings,
            &[
                (typing, "write hello in VSCodium".to_owned()),
                (other, "tidy my invoices".to_owned()),
            ],
            &TurnedOn::nothing(),
        );
    (tree, typing, other)
}

/// The sentence the person approves.
const SENTENCE: &str = "Type the text below into hello.txt in VSCodium";

/// **Every line is heard, and each hidden character is named in words in its
/// place** — the text itself is not changed by being read.
#[test]
fn every_line_is_heard_with_each_hidden_character_named() {
    let strings = words();
    let text = TheWholeText::of("one\ntw\u{200B}o\nthree");
    let heard = as_heard(&text, &strings);
    let named = alo_capability::TheWholeText::of("\u{200B}").lines()[0].pieces()[0].clone();
    let alo_capability::Piece::Unseen(unseen) = named else {
        panic!("a zero-width space is not marked");
    };
    let in_words = unseen.said(&strings).into_text();
    assert!(!unseen.said(&strings).is_a_bug(), "the name is not declared");
    assert!(in_words.contains("U+200B"), "{in_words}");
    assert_eq!(heard, format!("one\ntw ({in_words}) o\nthree"));
}

/// **It hangs under the window of the task that is asking**, and only there:
/// the sentence, how long it is, that one character is marked, and the text.
#[test]
fn it_hangs_under_the_asking_tasks_window_and_no_other() {
    let strings = words();
    let (tree, typing, other) = two_windows(&strings);
    let text = TheWholeText::of("fn main() {\n\u{200B}}");
    let before = tree.nodes().len();
    let tree = tree.with_the_text_alo_will_type(&strings, typing, SENTENCE, &[text.clone()]);
    assert_eq!(tree.nodes().len(), before + 4, "the card and its three lines");

    let card = tree.nodes().iter().position(|node| node.name == SENTENCE).unwrap();
    let window = tree.nodes()[card].parent;
    let closes = |which| {
        tree.nodes()
            .iter()
            .any(|node| node.parent == window && node.press == Some(Pressed::CloseTheTask(which)))
    };
    assert!(closes(typing), "the card is not under the asking task's window");
    assert!(!closes(other));
    let under: Vec<&str> = tree.nodes()[card]
        .children
        .iter()
        .map(|at| tree.nodes()[*at].name.as_str())
        .collect();
    assert_eq!(
        under,
        [
            text.how_long(&strings).into_text().as_str(),
            text.what_is_marked(&strings).unwrap().into_text().as_str(),
            as_heard(&text, &strings).as_str(),
        ]
    );
}

/// **No text, or no such window, leaves the tree as it was.**
#[test]
fn no_text_or_no_window_leaves_the_tree_alone() {
    let strings = words();
    let (tree, typing, _) = two_windows(&strings);
    let same = tree.clone().with_the_text_alo_will_type(&strings, typing, SENTENCE, &[]);
    assert_eq!(same, tree);
    let nowhere = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing());
    let still = nowhere.clone().with_the_text_alo_will_type(
        &strings,
        typing,
        SENTENCE,
        &[TheWholeText::of("hello")],
    );
    assert_eq!(still, nowhere);
}

/// **A long line is wrapped and still whole for a reader**, and every kind of
/// hidden character is named.
#[test]
fn every_kind_of_hidden_character_is_named() {
    let strings = words();
    let heard = as_heard(&TheWholeText::of("a\u{202E}b\u{0007}c\u{00A0}d"), &strings);
    for code in ["U+202E", "U+0007", "U+00A0"] {
        assert!(heard.contains(code), "{code} is not named: {heard}");
    }
    assert!(!heard.contains('\u{202E}'), "a direction mark reached the reader");
}

/// **A long text is heard whole**: nothing a reader is given is cut.
#[test]
fn a_long_text_is_heard_whole() {
    let strings = words();
    let long: String = (1..=500).map(|it| format!("line {it}\n")).collect();
    let heard = as_heard(&TheWholeText::of(&long), &strings);
    assert_eq!(heard, long);
}
