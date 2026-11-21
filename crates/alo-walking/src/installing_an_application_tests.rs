//! What the installing sections will check, read without a machine: the two
//! applications are ones the catalogue offers and a fresh machine does not
//! ship, every name is read from where the shell and the turn say it, and an
//! installation that was not done can never pass for one that arrived.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use alo_acting::said_on_the_log::{InAnApplication, alo_in_an_application};

use super::*;
use crate::step::Step;

/// **Both are offered by the catalogue, and neither is shipped**: an
/// application already on the machine would be refused as already installed,
/// and the walk would prove nothing about installing.
#[test]
fn both_are_offered_and_neither_is_shipped() {
    let catalogue = alo_software::Catalogue::decided().unwrap();
    let offered: Vec<&str> = catalogue
        .every()
        .iter()
        .map(|one| one.application().identifier())
        .collect();
    let shipped = include_str!("../../alo-software/shipped.toml");
    for application in [KATE, KOOHA] {
        assert!(
            offered.contains(&application),
            "{application} is not offered"
        );
        assert!(
            !shipped.contains(&format!("identifier = \"{application}\"")),
            "{application} is shipped"
        );
    }
}

/// **The verb names are the ones the turn says**, so a check can never wait
/// for a verb nobody carries out.
#[test]
fn the_verb_names_are_the_ones_the_turn_says() {
    assert_eq!(INSTALL_APPLICATION, alo_software::INSTALL_APPLICATION);
    assert_eq!(ACTIVATE_CONTROL, alo_adapters::ACTIVATE_CONTROL);
}

/// **An arrival passes and a refusal fails, in the market's own words**:
/// both begin with the phrase waited for, and only the arrival says what the
/// check reads for.
#[test]
fn an_installation_not_done_never_passes_for_one_that_arrived() {
    let came = market::an_installation_arrived(KATE);
    let not = market::an_installation_was_not_done(KATE, "Refused(DidNotAnswer)");
    let other = market::an_installation_arrived(KOOHA);
    for line in [&came, &not, &other] {
        assert!(line.starts_with(market::AN_INSTALLATION), "{line}");
    }
    assert!(came.contains(&arrived(KATE)), "{came}");
    assert!(!not.contains(&arrived(KATE)), "{not}");
    assert!(!other.contains(&arrived(KATE)), "{other}");
}

/// **alo's installation is read from the turn's own sentence**, and *not
/// done* never satisfies it.
#[test]
fn alos_installation_is_the_turns_own_sentence() {
    let done = alo_in_an_application(INSTALL_APPLICATION, KOOHA, InAnApplication::Done);
    assert!(
        done.starts_with(&acted(INSTALL_APPLICATION, KOOHA)),
        "{done}"
    );
    assert!(done.contains(&done_in(KOOHA)), "{done}");
    let not = alo_in_an_application(INSTALL_APPLICATION, KOOHA, InAnApplication::NotDone);
    assert!(!not.contains(&done_in(KOOHA)), "{not}");
}

/// **The Dock's slot and the market's card are found by name**, with the
/// phrases the shell says them by — never by counting presses.
#[test]
fn the_market_and_kate_are_found_by_name() {
    let steps = kate_from_the_market(Walk::named("w")).steps().to_vec();
    let found: Vec<(&str, &str)> = steps
        .iter()
        .filter_map(|step| match step {
            Step::PressUntilTheKeyboardIsOn { saying, name, .. } => {
                Some((saying.as_str(), name.as_str()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        found,
        vec![
            (
                market::ON_THE_DOCK_THE_KEYBOARD_IS_ON,
                alo_dock::words::APPLICATIONS_ON_THE_DOCK.says()
            ),
            (market::IN_THE_MARKET_THE_KEYBOARD_IS_ON, KATE),
        ]
    );
}

/// **Both sections need the hand-run choice, and only themselves**: each is
/// one gate covering exactly its own steps.
#[test]
fn each_section_is_gated_on_installing_and_covers_itself() {
    for section in [kate_from_the_market, kooha_installed_by_alo] {
        let steps = section(Walk::named("w")).steps().to_vec();
        assert_eq!(
            steps.first(),
            Some(&Step::OnlyWithFor {
                variable: INSTALLING,
                steps: steps.len() - 1,
            })
        );
    }
}

/// **The level counted to is full trust**, from the list Settings draws.
#[test]
fn full_trust_is_the_last_level() {
    assert_eq!(
        alo_capability::HowCodeRuns::ALL.get(FULL_TRUST),
        Some(&alo_capability::HowCodeRuns::AtFullTrust)
    );
}

/// **Kate's check reads the text alo was asked to type**, and the ask names
/// the file the person made.
#[test]
fn the_ask_in_kate_names_the_file_and_the_text() {
    let ask = the_ask_in_kate();
    assert!(
        ask.contains(KATES_FILE) && ask.contains(KATES_TEXT),
        "{ask}"
    );
    assert!(ask.contains("Kate"), "{ask}");
}
