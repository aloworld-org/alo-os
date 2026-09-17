//! A context menu is a closed list of actions the thing under the pointer
//! offers, each of which a person could reach another way.
//!
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 4, second acceptance.
//! ADR 0009 is the rule behind *another way*: a machine has to be whole with the
//! agent switched off, and the same argument holds for a person who never opens
//! a context menu — from the keyboard, through a screen reader, with one hand.
//! **A capability that lives in a right-click is a capability some people do not
//! have.**

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;

use alo_menus::{Action, AlsoBy, Menu, NotChosen, Subject, TheAgent};
use alo_strings::Strings;

/// This crate's own words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(alo_menus::menu_words().expect("this crate's own words"))
}

/// **Every menu is a list this crate wrote**, and every row on it is one of the
/// fourteen actions that exist. Nothing is added at the moment a menu opens,
/// from a subject, from an application or from anywhere else.
#[test]
fn every_menu_offers_only_actions_from_the_closed_list() {
    let everything: BTreeSet<Action> = Action::ALL.iter().copied().collect();
    for subject in Subject::ALL {
        for agent in [TheAgent::OnThisMachine, TheAgent::NotOnThisMachine] {
            let menu = Menu::over(*subject, agent);
            assert!(!menu.entries().is_empty(), "{subject:?} has no menu at all");
            for entry in menu.entries() {
                assert!(everything.contains(entry), "{entry:?} is not on the list");
            }
            // And no row twice: a menu with two *Copy* rows is a menu nobody
            // can read.
            let seen: BTreeSet<Action> = menu.entries().iter().copied().collect();
            assert_eq!(seen.len(), menu.entries().len(), "{subject:?}");
        }
    }
}

/// **Every action a menu offers is reachable another way**, and the other way
/// is a road that exists: a shortcut this machine actually binds, an
/// application's own menus, the files window, or Settings. There is no *and you
/// can also find it somewhere* — a road nobody can name is a road nobody has
/// walked.
#[test]
fn every_action_a_menu_offers_is_reachable_another_way() {
    let bound: BTreeSet<alo_shortcuts::Action> =
        alo_shortcuts::Action::ALL.iter().copied().collect();

    for action in Action::ALL {
        let road = action.also_reached_by();
        if let AlsoBy::AShortcut(shortcut) = road {
            assert!(
                bound.contains(&shortcut),
                "{action:?} points at a shortcut this machine does not have"
            );
            assert_eq!(road.shortcut(), Some(shortcut));
        }
    }

    // And every action that is on some menu has been asked the question — which
    // is every action, because the compiler holds `also_reached_by` to a match
    // over the whole list.
    let offered: BTreeSet<Action> = Subject::ALL
        .iter()
        .flat_map(|subject| {
            Menu::over(*subject, TheAgent::OnThisMachine)
                .entries()
                .to_vec()
        })
        .collect();
    assert_eq!(offered.len(), Action::ALL.len());
}

/// Every row reads as a sentence of its own, in the language the person reads,
/// with nothing left to fill in. A menu is nearly all words; a row that came
/// out as a key would be the whole feature failing in public.
#[test]
fn every_row_a_person_reads_is_a_sentence_of_its_own() {
    let strings = in_english();
    let mut seen = BTreeSet::new();
    for subject in Subject::ALL {
        for entry in Menu::over(*subject, TheAgent::OnThisMachine).entries() {
            let said = entry.said(&strings);
            assert!(!said.is_a_bug(), "{entry:?}");
            assert!(said.unfilled().is_empty(), "{entry:?}: {said}");
            seen.insert(said.text().to_owned());
        }
    }
    assert_eq!(seen.len(), Action::ALL.len());
}

/// **A menu left open over something that has gone does nothing.** The file was
/// deleted, the window closed; choosing a row then carries the action out
/// against nothing rather than against whatever is there now — and says so.
#[test]
fn choosing_something_this_menu_never_offered_does_nothing() {
    let desktop = Menu::over(Subject::TheDesktop, TheAgent::OnThisMachine);
    let refused = desktop.chosen(Action::MoveToTheWastebasket).unwrap_err();
    assert_eq!(
        refused,
        NotChosen::NotOnThisMenu {
            action: Action::MoveToTheWastebasket
        }
    );
    assert_eq!(refused.action(), Action::MoveToTheWastebasket);

    let said = refused.said(&in_english());
    assert!(!said.is_a_bug());
    assert!(said.unfilled().is_empty(), "{said}");
}

/// **Everything this crate says is in the machine's one vocabulary.** A crate
/// whose words nothing collects reaches a real shell as a missing key, in
/// English and in every language somebody has translated.
#[test]
fn everything_this_crate_says_is_collected_into_the_machines_words() {
    let machine = alo_saying::everything_this_machine_can_say().expect("the machine's own words");
    for word in alo_menus::words::EVERY_WORD {
        assert!(
            machine.phrase(&word.key()).is_some(),
            "nothing collects {}",
            word.named()
        );
    }
}
