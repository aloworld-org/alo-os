//! The recovery screen: `alo-keeping-up`'s sentence and nothing else, two
//! moments with neither preselected, and a machine that cannot go back saying
//! why in that crate's words.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keeping_up::{CannotGoBack, GoingBack};

use super::*;
use crate::recovery_testing::{
    a_machine_running_nothing_it_can_name, a_machine_that_can_go_back,
    a_machine_that_no_longer_keeps_it, a_machine_with_an_update_waiting,
    a_machine_with_nothing_before, words,
};

/// The screen for a machine that can go back.
fn a_screen(strings: &Strings) -> RecoveryScreen {
    RecoveryScreen::of(&a_machine_that_can_go_back(), None, strings)
}

/// What the screen is offering, or a panic naming what it is showing instead.
fn offered(screen: &RecoveryScreen) -> (&Said, &[(WhenItApplies, Said)], Option<WhenItApplies>) {
    match screen.shows() {
        RecoveryShows::Offered {
            offer,
            moments,
            selected,
        } => (offer, moments, selected),
        RecoveryShows::CannotGoBack(said) => {
            panic!("going back was not offered: {}", said.text())
        }
    }
}

/// **Every sentence on the screen is `alo-keeping-up`'s.** The offer is that
/// crate's own sentence for this machine, byte for byte, and each moment is the
/// vocabulary's words for that crate's own `when_word` — so a person reads what
/// was decided and never a sentence a compositor wrote.
#[test]
fn every_sentence_on_the_screen_is_alo_keeping_ups() {
    let strings = words();
    let screen = a_screen(&strings);
    let decided = GoingBack::offered(&a_machine_that_can_go_back(), None).unwrap();

    let (offer, moments, selected) = offered(&screen);
    assert_eq!(offer.text(), decided.said(&strings).text());
    assert_eq!(selected, None);
    assert_eq!(moments.len(), 2);
    for (when, said) in moments {
        let word = GoingBack::when_word(*when);
        assert_eq!(
            said.text(),
            strings
                .say(&word.key(), &alo_strings::Filling::nothing())
                .text()
        );
        assert!(!said.text().is_empty());
    }
    assert_eq!(
        moments.iter().map(|(when, _)| *when).collect::<Vec<_>>(),
        THE_TWO_MOMENTS.to_vec()
    );
}

/// **An update already waiting changes the sentence, because going back sets it
/// aside.** A person is told that their own earlier choice is being undone —
/// and it is `alo-keeping-up` that says so, in a different sentence from the
/// ordinary one.
#[test]
fn an_update_already_waiting_changes_the_sentence() {
    let strings = words();
    let plain = RecoveryScreen::of(&a_machine_that_can_go_back(), None, &strings);
    let waiting = RecoveryScreen::of(&a_machine_with_an_update_waiting(), None, &strings);

    let (ordinary, _, _) = offered(&plain);
    let (sets_aside, _, _) = offered(&waiting);
    assert_ne!(ordinary.text(), sets_aside.text());
    assert_eq!(
        sets_aside.text(),
        GoingBack::offered(&a_machine_with_an_update_waiting(), None)
            .unwrap()
            .said(&strings)
            .text()
    );
}

/// **Nothing is preselected, and Enter before moving chooses nothing.** Going
/// back replaces the whole operating system; an Enter still held down from
/// whatever failed a moment ago must not start that.
#[test]
fn nothing_is_preselected_and_enter_before_moving_chooses_nothing() {
    let strings = words();
    let screen = a_screen(&strings);
    assert_eq!(offered(&screen).2, None);

    let after = screen.pressed(RecoveryKey::Choose);
    match after {
        RecoveryChosen::Still(screen) => assert_eq!(offered(&screen).2, None),
        RecoveryChosen::GoBack { .. } => panic!("Enter chose with nothing selected"),
    }
}

/// **Tab and Shift+Tab reach both moments and nothing else**, wrapping in
/// both directions, and choosing gives the selected moment and ends the screen
/// — there is nothing left to press a second Enter at.
#[test]
fn tab_reaches_both_moments_and_choosing_ends_the_screen() {
    let strings = words();

    // Forwards from nothing reaches the first moment, then the second, then
    // the first again.
    let mut screen = a_screen(&strings);
    for expected in [
        WhenItApplies::AtTheNextRestart,
        WhenItApplies::NowBecauseThePersonAsked,
        WhenItApplies::AtTheNextRestart,
    ] {
        screen = match screen.pressed(RecoveryKey::Next) {
            RecoveryChosen::Still(screen) => *screen,
            RecoveryChosen::GoBack { .. } => panic!("moving chose"),
        };
        assert_eq!(offered(&screen).2, Some(expected));
    }

    // Backwards from nothing reaches the last one.
    let back = match a_screen(&strings).pressed(RecoveryKey::Previous) {
        RecoveryChosen::Still(screen) => *screen,
        RecoveryChosen::GoBack { .. } => panic!("moving chose"),
    };
    assert_eq!(
        offered(&back).2,
        Some(WhenItApplies::NowBecauseThePersonAsked)
    );

    // And choosing hands back what was decided, with the moment chosen.
    match back.pressed(RecoveryKey::Choose) {
        RecoveryChosen::GoBack { going_back, when } => {
            assert_eq!(when, WhenItApplies::NowBecauseThePersonAsked);
            assert_eq!(
                going_back.to(),
                GoingBack::offered(&a_machine_that_can_go_back(), None)
                    .unwrap()
                    .to()
            );
        }
        RecoveryChosen::Still(_) => panic!("Enter on a selected moment chose nothing"),
    }
}

/// **A key that is nothing changes nothing**, including after a moment has been
/// selected.
#[test]
fn a_key_that_is_nothing_changes_nothing() {
    let strings = words();
    let screen = match a_screen(&strings).pressed(RecoveryKey::Next) {
        RecoveryChosen::Still(screen) => *screen,
        RecoveryChosen::GoBack { .. } => panic!("moving chose"),
    };
    let before = offered(&screen).2;
    match screen.pressed(RecoveryKey::Nothing) {
        RecoveryChosen::Still(screen) => assert_eq!(offered(&screen).2, before),
        RecoveryChosen::GoBack { .. } => panic!("a key that means nothing chose"),
    }
}

/// **A machine that cannot go back says why, in `alo-keeping-up`'s words, and
/// no key on it does anything.** Each refusal is that crate's decision made
/// before anything is offered — so nobody is offered a return that would fail
/// halfway — and each is a different sentence.
#[test]
fn a_machine_that_cannot_go_back_says_why_and_takes_no_key() {
    let strings = words();
    let (no_longer_kept, changed) = a_machine_that_no_longer_keeps_it();
    let machines: Vec<(&str, RecoveryScreen, CannotGoBack)> = vec![
        (
            "nothing before",
            RecoveryScreen::of(&a_machine_with_nothing_before(), None, &strings),
            CannotGoBack::NothingBefore,
        ),
        (
            "no longer kept",
            RecoveryScreen::of(&no_longer_kept, Some(&changed), &strings),
            CannotGoBack::NoLongerKept {
                build: crate::recovery_testing::the_build_before(),
            },
        ),
        (
            "already going back",
            RecoveryScreen::of(
                &a_machine_that_can_go_back().going_back_at_the_next_restart(),
                None,
                &strings,
            ),
            CannotGoBack::AlreadyGoingBack,
        ),
        (
            "running nothing it can name",
            RecoveryScreen::of(&a_machine_running_nothing_it_can_name(), None, &strings),
            CannotGoBack::NotRunningABuild,
        ),
    ];

    let mut sentences = Vec::new();
    for (named, screen, why) in machines {
        assert!(!screen.is_offered(), "{named} offered going back");
        let RecoveryShows::CannotGoBack(said) = screen.shows() else {
            panic!("{named} is offering something");
        };
        assert_eq!(said.text(), why.said(&strings).text(), "{named}");
        sentences.push(said.text().to_owned());

        for key in [
            RecoveryKey::Next,
            RecoveryKey::Previous,
            RecoveryKey::Choose,
            RecoveryKey::Nothing,
        ] {
            let screen = RecoveryScreen::of(&a_machine_with_nothing_before(), None, &strings);
            match screen.pressed(key) {
                RecoveryChosen::Still(screen) => assert!(!screen.is_offered()),
                RecoveryChosen::GoBack { .. } => {
                    panic!("{named} went back with {key:?} and nothing to go back to")
                }
            }
        }
    }
    sentences.sort();
    let how_many = sentences.len();
    sentences.dedup();
    assert_eq!(
        sentences.len(),
        how_many,
        "two machines that cannot go back read the same"
    );
}
