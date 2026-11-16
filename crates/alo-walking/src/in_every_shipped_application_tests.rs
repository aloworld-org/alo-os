//! What the walk of every shipped application will check, read without a
//! machine — each phrase held to the sentence the turn and the desktop really
//! write, and the list held to what a fresh machine ships.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use alo_acting::Opened;
use alo_acting::said_on_the_log::{InAnApplication, a_file_was_opened, alo_in_an_application};

use super::*;
use crate::keys;
use crate::step::Step;

/// **The verb names are the ones the turn says**, so a check can never wait
/// for a verb nobody carries out.
#[test]
fn the_verb_names_are_the_fallbacks_own() {
    assert_eq!(TYPE_TEXT, alo_adapters::TYPE_TEXT);
    assert_eq!(READ_WINDOW, alo_adapters::READ_WINDOW);
}

/// **A phrase and its *done* are the start and end of the turn's own
/// sentence**, and *not done* never satisfies a check.
///
/// What would have to be true for this to pass while broken: the turn writing
/// its line without `alo_acting::said_on_the_log::alo_in_an_application`,
/// which a search for `ALO_IN_AN_APPLICATION` would find.
#[test]
fn each_phrase_is_the_turns_own_and_not_done_never_passes() {
    for application in [DOLPHIN, LOUPE, ARK, PAPERS, FIREFOX] {
        let done = alo_in_an_application(READ_WINDOW, application, InAnApplication::ReadWithText);
        assert!(done.starts_with(&acted(READ_WINDOW, application)), "{done}");
        assert!(done.contains(&done_in(application)), "{done}");
        let not = alo_in_an_application(READ_WINDOW, application, InAnApplication::NotDone);
        assert!(not.starts_with(&acted(READ_WINDOW, application)), "{not}");
        assert!(!not.contains(&done_in(application)), "{not}");
    }
    let opened = a_file_was_opened(LOUPE, Opened::InTheirWindow);
    assert!(opened.starts_with(&opened_a_file_in(LOUPE)), "{opened}");
    assert!(opened.contains(Opened::InTheirWindow.written()), "{opened}");
    let saved = alo_in_an_application(TYPE_TEXT, TEXT_EDITOR, InAnApplication::TypedAndSaved);
    assert!(saved.ends_with(InAnApplication::TypedAndSaved.written()));
}

/// **Every application asked about is one a fresh machine ships**, and every
/// one it ships is asked about except the two walked or left out on purpose:
/// VSCodium, walked just before, and Ptyxis, a terminal (ADR 0111).
#[test]
fn it_asks_in_every_shipped_application_but_the_terminal_and_vscodium() {
    let shipped = include_str!("../../alo-software/shipped.toml");
    let identifiers: Vec<&str> = shipped
        .lines()
        .filter_map(|line| line.strip_prefix("identifier = \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .collect();
    assert!(identifiers.len() >= 8, "{identifiers:?}");
    let asked: Vec<&str> = the_asks().iter().map(|(application, _)| *application).collect();
    for identifier in &identifiers {
        let left_out = *identifier == "app.devsuite.Ptyxis" || *identifier == "com.vscodium.codium";
        assert_eq!(
            asked.contains(identifier),
            !left_out,
            "{identifier} in {asked:?}"
        );
    }
    for application in &asked {
        assert!(identifiers.contains(application), "{application} is not shipped");
    }
}

/// **Every text it types can be typed**, so no run stops halfway through a
/// line for want of a key.
#[test]
fn every_text_it_types_has_keys() {
    let walk = at_full_access(Walk::named("every-application"));
    let mut typed = 0;
    for step in walk.steps() {
        if let Step::Type(text) = step {
            typed += 1;
            if let Err(keys::Untypable { at }) = keys::typing(text) {
                panic!("character {at} of {text:?} has no key");
            }
        }
    }
    assert!(typed >= 8, "only {typed} texts were typed");
}

/// **Nothing runs without the variable**: the section's first step is the
/// gate, so a walk after a merge skips it rather than failing.
#[test]
fn it_runs_only_when_asked_for() {
    let walk = at_full_access(Walk::named("every-application"));
    match walk.steps().first() {
        Some(Step::OnlyWith(variable)) => assert_eq!(*variable, EVERY_APPLICATION),
        other => panic!("the first step is {other:?}"),
    }
}
