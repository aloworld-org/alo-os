//! What the integration's four states offer, and what pressing the row does.
//!
//! **Every sentence comes from this crate's real word list**, built by
//! `words::terminal_words`, not from an empty vocabulary: a `Vocabulary::empty`
//! answers for a word that is declared nowhere, so a test on one passes while a
//! person reads a key.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use alo_strings::Strings;

use super::{WhatThePressDid, WhereItStands};

/// This crate's own words, which is what a person really reads.
fn strings() -> Strings {
    Strings::of(crate::words::terminal_words().expect("this crate's own word list"))
}

/// A temporary directory that goes away with the test.
fn a_folder() -> tempfile::TempDir {
    tempfile::tempdir().expect("a directory of our own")
}

/// **A shell alo ships nothing for gets a sentence and no row.**
///
/// Not silence: a person whose shell we have nothing for is owed the reason,
/// and a row that did nothing when pressed would be the surface lying.
#[test]
fn a_shell_alo_ships_nothing_for_gets_a_sentence_and_no_row() {
    let folder = a_folder();
    let stands = WhereItStands::read(Some("/usr/bin/zsh"), Some(folder.path()));

    assert!(!stands.offers_a_row(), "there is nothing to press");
    let said = stands.said(&strings());
    assert_eq!(said.len(), 1, "one sentence, saying why: {said:?}");
    assert!(
        said.first().expect("the one sentence").contains("zsh"),
        "the sentence names the shell, so a person reads which one: {said:?}"
    );
}

/// **A session with no folder gets a sentence and no row.**
#[test]
fn a_session_with_no_folder_gets_a_sentence_and_no_row() {
    let stands = WhereItStands::read(Some("/bin/bash"), None);

    assert!(!stands.offers_a_row());
    let said = stands.said(&strings());
    assert_eq!(said.len(), 1, "one sentence, saying why: {said:?}");
    assert!(
        said.iter()
            .all(|line| !line.contains('{') && !line.contains('[')),
        "no placeholder reached a person: {said:?}"
    );
}

/// **A session that did not say which shell says nothing and offers nothing.**
///
/// The honest fourth answer. *We have none for your shell* would be a claim
/// about a shell nobody named, and this is the one case where saying less is
/// more truthful than saying something.
#[test]
fn a_session_that_did_not_say_its_shell_says_nothing_and_offers_nothing() {
    let folder = a_folder();
    let stands = WhereItStands::read(None, Some(folder.path()));

    assert!(!stands.offers_a_row());
    assert!(
        stands.said(&strings()).is_empty(),
        "nothing true can be said about a shell nobody named"
    );
}

/// **bash with a folder offers a row, and the row names the file.**
#[test]
fn bash_with_a_folder_offers_a_row_naming_the_file() {
    let folder = a_folder();
    let stands = WhereItStands::read(Some("/bin/bash"), Some(folder.path()));

    assert!(stands.offers_a_row());
    assert!(
        !stands.is_already_there(),
        "nothing is in the folder yet, so it is not already there"
    );
    let row = stands.row(&strings());
    assert_eq!(
        row.len(),
        3,
        "what it gives, what it reveals, where it goes"
    );
    assert!(
        row.get(2).expect("the third part").contains(".bashrc"),
        "the row names the file it would change: {row:?}"
    );
    assert!(
        row.iter()
            .all(|part| !part.contains('{') && !part.contains('[')),
        "no placeholder reached a person: {row:?}"
    );
    // The cost is on the row rather than only in a confirmation afterwards,
    // because a person decides before pressing, not after.
    assert!(
        row.get(1).expect("the second part").contains("machine"),
        "the row says what it reveals before it is pressed: {row:?}"
    );
}

/// **A startup file that already holds it says so, and the row is marked.**
#[test]
fn a_startup_file_that_already_has_it_says_so() {
    let folder = a_folder();
    let at = folder.path().join(".bashrc");
    std::fs::write(
        &at,
        crate::the_snippet_a_person_installs::for_the_shell_called("bash")
            .expect("alo ships integration for bash"),
    )
    .expect("a startup file that already holds it");

    let stands = WhereItStands::read(Some("bash"), Some(folder.path()));
    assert!(stands.offers_a_row());
    assert!(
        stands.is_already_there(),
        "it is in the file, so the row is marked as what their settings are now"
    );
    let row = stands.row(&strings());
    assert!(
        row.get(2).expect("the third part").contains(".bashrc"),
        "the already-there line names the file too: {row:?}"
    );
}

/// **Pressing it writes the snippet, and says to open a new terminal.**
///
/// The second half is the part a person needs: a shell reads its startup file
/// when it starts, so the terminal already open does not have the integration,
/// and somebody pressing the key there would find nothing happens.
#[test]
fn pressing_it_writes_the_snippet_and_says_to_open_a_new_terminal() {
    let folder = a_folder();
    let at = folder.path().join(".bashrc");
    let mut stands = WhereItStands::read(Some("/bin/bash"), Some(folder.path()));

    assert_eq!(stands.add(&strings()), WhatThePressDid::ItIsThere);

    let held = std::fs::read_to_string(&at).expect("the startup file the press wrote");
    assert!(
        held.contains("alo OS shell integration"),
        "the snippet is in the file"
    );
    assert!(stands.is_already_there(), "it knows it is there now");

    let said = stands.said(&strings());
    assert_eq!(said.len(), 1, "one sentence about the press: {said:?}");
    let sentence = said.first().expect("the one sentence");
    assert!(
        sentence.contains(".bashrc"),
        "it names the file it changed: {said:?}"
    );
    assert!(
        sentence.len() > 30,
        "it says more than `added` — a person has to open a new terminal: {said:?}"
    );
}

/// **Pressing twice leaves one copy and is not reported as a failure.**
#[test]
fn pressing_it_twice_leaves_one_copy() {
    let folder = a_folder();
    let at = folder.path().join(".bashrc");
    let mut stands = WhereItStands::read(Some("bash"), Some(folder.path()));

    assert_eq!(stands.add(&strings()), WhatThePressDid::ItIsThere);
    assert_eq!(
        stands.add(&strings()),
        WhatThePressDid::ItIsThere,
        "the second press is not a failure"
    );

    let held = std::fs::read_to_string(&at).expect("the startup file");
    assert_eq!(
        held.matches("alo OS shell integration, for bash.").count(),
        1,
        "one copy after two presses"
    );
}

/// **Pressing when there is nothing to offer changes nothing.**
#[test]
fn pressing_when_there_is_nothing_to_offer_changes_nothing() {
    let mut nowhere = WhereItStands::read(Some("/bin/bash"), None);
    assert_eq!(
        nowhere.add(&strings()),
        WhatThePressDid::ThereWasNothingToAdd
    );

    let mut unknown = WhereItStands::read(None, None);
    assert_eq!(
        unknown.add(&strings()),
        WhatThePressDid::ThereWasNothingToAdd
    );
}

/// **A file that will not take the write is reported as that, with the file
/// named** — and the invariant holds either way, so this needs no knowledge of
/// whether the test runs as root.
#[test]
fn a_read_only_startup_file_never_reports_a_write_that_did_not_happen() {
    use std::os::unix::fs::PermissionsExt as _;

    let folder = a_folder();
    let at = folder.path().join(".bashrc");
    std::fs::write(&at, "alias ll='ls -l'\n").expect("a startup file of their own");
    std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o444))
        .expect("making it read-only");

    let mut stands = WhereItStands::read(Some("bash"), Some(folder.path()));
    let did = stands.add(&strings());
    let held = std::fs::read_to_string(&at).expect("the file is still readable");

    match did {
        WhatThePressDid::ItIsThere => assert!(
            held.contains("alo OS shell integration"),
            "it said the snippet is there, so it has to be there"
        ),
        WhatThePressDid::ItWouldNotTakeIt => {
            assert!(
                !held.contains("alo OS shell integration"),
                "it refused, so nothing was written"
            );
            let said = stands.said(&strings());
            assert!(
                said.first().expect("a sentence").contains(".bashrc"),
                "the refusal names the file: {said:?}"
            );
        }
        WhatThePressDid::ThereWasNothingToAdd => {
            panic!("bash with a folder has something to add")
        }
    }
    assert!(
        held.starts_with("alias ll='ls -l'"),
        "their own line survived either way, which is what an append protects"
    );
}
