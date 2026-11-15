//! **A path the model is told to write is one the verbs accept**, read through
//! both real sides: the words `alo-instructing` tells a model about the
//! person's home and folders, and the verbs `alo-turn` offers, validated by
//! `alo-capability` (rule 2 of ADR 0156, a seam test).
//!
//! Each side had passed its own tests while the two disagreed. The model was
//! told *"~ means that folder"*, and every path argument refuses a path that
//! does not start at the root (`alo_capability::path::is_usable`), so a model
//! that believed the words lost a step to a refusal. And asked for *the folder
//! site in my Documents*, it named the Documents folder itself, because nothing
//! told it how a thing inside one of their folders is named.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::Path;

use alo_capability::{Effect, Given, Takes};
use alo_instructing::ThePersonsFolders;

/// The home of the person the walk boots, and their three folders.
const HOME: &str = "/var/home/alo";

fn their_folders() -> ThePersonsFolders {
    let home = Path::new(HOME);
    let documents = home.join("Documents");
    let downloads = home.join("Downloads");
    ThePersonsFolders::of(
        home,
        [
            ("Documents", documents.as_path()),
            ("Downloads", downloads.as_path()),
        ],
    )
    .expect("a home a line may name")
}

/// Whether the machine's own verbs take `path`: asked of every read whose one
/// argument is a path — found by what it takes, so no verb's name is typed
/// here a second time — and true only when every one of them takes it.
fn the_verbs_accept(path: &str) -> bool {
    let verbs = alo_turn::the_verbs_a_whole_machine_offers().expect("the verbs declare");
    let taking_a_path: Vec<(&str, &str)> = verbs
        .all()
        .filter(|verb| verb.effect() == Effect::Read)
        .filter_map(|verb| match verb.args() {
            [only] if matches!(only.takes(), Takes::Path) => Some((verb.name(), only.name())),
            _ => None,
        })
        .collect();
    assert!(!taking_a_path.is_empty(), "some read takes one path");
    taking_a_path
        .iter()
        .all(|(verb, named)| verbs.call(verb, &[(*named, Given::text(path))]).is_ok())
}

/// **A model is never told that `~` names the home unless a path written with
/// it is accepted.** Today it is refused, so the words must say to write the
/// home out in full.
#[test]
fn the_model_is_not_told_a_path_shape_the_verbs_refuse() {
    let told = their_folders().told();
    let with_a_tilde = "~/notes.txt";
    assert!(
        !the_verbs_accept(with_a_tilde),
        "the verbs now take {with_a_tilde}: this test's premise changed"
    );
    assert!(
        !told.contains("~ means"),
        "the model is told ~ means the home, and the verbs refuse {with_a_tilde}:\n{told}"
    );
    assert!(
        the_verbs_accept(&format!("{HOME}/notes.txt")),
        "the home written in full is accepted"
    );
}

/// **The model is told how a thing inside one of their folders is named**: the
/// folder's path, a slash, and its name — and the example it is shown is a
/// path the verbs accept.
#[test]
fn the_model_is_told_a_thing_inside_a_folder_is_its_path_and_its_name() {
    let told = their_folders().told();
    let inside = format!("{HOME}/Documents/");
    let example = told
        .match_indices(&inside)
        .map(|(at, _)| &told[at..])
        .find_map(|from| {
            let end = from
                .find(|c: char| c.is_whitespace() || c == ',')
                .unwrap_or(from.len());
            let path = from[..end].trim_end_matches(['.', ',']);
            (path.len() > inside.len()).then_some(path)
        })
        .unwrap_or_else(|| panic!("no example inside Documents is told:\n{told}"));
    assert!(
        the_verbs_accept(example),
        "the example {example} is accepted"
    );
    // **And it is offered as the path to name**, not as the one to avoid: in
    // its sentence it comes before "never", and the folder itself after it.
    let sentence = told
        .split(". ")
        .find(|sentence| sentence.contains(example))
        .unwrap_or_else(|| panic!("the example is in a sentence:\n{told}"));
    let (to_name, to_avoid) = sentence
        .split_once("never")
        .unwrap_or_else(|| panic!("the sentence says what never to name: {sentence:?}"));
    assert!(
        to_name.contains(example) && !to_avoid.contains(example),
        "the example {example} is told as the path to name: {sentence:?}"
    );
    assert!(
        to_avoid.contains(inside.trim_end_matches('/')),
        "the folder itself is what is never named: {sentence:?}"
    );
}
