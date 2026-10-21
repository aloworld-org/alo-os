//! That every screen says something, that a password is hidden and a disk's
//! name is not, and that no sentence on any screen was invented here.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{how_many_cards, what_this_screen_says};
use crate::the_asking_walks::{TheAsking, WhatThePersonDid, WhatTheRingIsOn, WhichScreen};
use crate::which_disks_may_be_offered::Offered;
use crate::{ANetwork, joining_a_network};

const THE_EIGHT: [(&str, &str); 2] = [("be", "Belgisch"), ("fr", "Français")];

fn strings() -> alo_strings::Strings {
    let vocabulary = alo_saying::everything_this_machine_can_say().expect("alo OS's own words");
    alo_strings::Strings::of(vocabulary)
}

/// Two networks, one locked and one not, as a scan reports them.
fn networks() -> Vec<ANetwork> {
    joining_a_network::every_network("Fritz!Box 7590:82:WPA2\nA cafe, open:51:\n")
}

fn disks() -> Vec<Offered> {
    vec![
        Offered {
            disk: "/dev/disk/by-id/one".to_owned(),
            called: "Samsung SSD 870 EVO  ·  1 TB".to_owned(),
            holding: "Windows and about 340 GB of files".to_owned(),
            erases: true,
        },
        Offered {
            disk: "/dev/disk/by-id/two".to_owned(),
            called: "WD Elements  ·  2 TB".to_owned(),
            holding: "Nothing is on it".to_owned(),
            erases: false,
        },
    ]
}

/// Walk to a screen by taking the obvious road to it.
fn at(screen: WhichScreen) -> TheAsking {
    let mut asking = TheAsking::begins();
    let named = |what: &'static str| WhatTheRingIsOn {
        named: Some(what),
        wants_a_password: false,
    };
    let locked = |what: &'static str| WhatTheRingIsOn {
        named: Some(what),
        wants_a_password: true,
    };
    if screen == WhichScreen::TheKeyboard {
        return asking;
    }
    if screen == WhichScreen::EveryLayout {
        asking.and_then(WhatThePersonDid::WentDown, WhatTheRingIsOn::default());
        asking.and_then(WhatThePersonDid::Chose, WhatTheRingIsOn::default());
        return asking;
    }
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    if screen == WhichScreen::TheNetwork {
        return asking;
    }
    asking.there_are_this_many_cards(2);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box 7590"));
    if screen == WhichScreen::ThePassword {
        return asking;
    }
    for character in "a secret".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    asking.and_then(WhatThePersonDid::Chose, WhatTheRingIsOn::default());
    if screen == WhichScreen::TheDisk {
        return asking;
    }
    asking.there_are_this_many_cards(2);
    asking.and_then(
        WhatThePersonDid::Chose,
        named("Samsung SSD 870 EVO  ·  1 TB"),
    );
    asking
}

#[test]
fn every_screen_says_something_rather_than_showing_a_blank() {
    // **The check that no screen is a quiet nothing.** A variant that fell
    // through to empty strings would draw a grey rectangle, and a person
    // would have no way to know whether the machine had stopped.
    let strings = strings();
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let asking = at(screen);
        assert_eq!(asking.on(), screen, "the walk did not reach {screen:?}");
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        assert!(!words.title.is_empty(), "{screen:?} has no question on it");
        assert!(
            !words.eyebrow.is_empty(),
            "{screen:?} has no label above it"
        );
        assert!(
            !words.cards.is_empty(),
            "{screen:?} offers a person nothing at all"
        );
        assert!(
            !words.consequence.is_empty(),
            "{screen:?} has no quiet line under it"
        );
    }
}

#[test]
fn the_networks_a_scan_found_are_the_cards() {
    let strings = strings();
    let asking = at(WhichScreen::TheNetwork);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let named: Vec<&str> = words.cards.iter().map(|(name, _)| name.as_str()).collect();
    assert!(
        named.contains(&"Fritz!Box 7590"),
        "the network a scan found is not on the screen: {named:?}"
    );
    assert_eq!(words.cards.len(), networks().len());
}

#[test]
fn the_disks_offered_are_the_cards_and_each_says_what_is_on_it() {
    let strings = strings();
    let asking = at(WhichScreen::TheDisk);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    assert_eq!(words.cards.len(), 2);
    assert_eq!(words.cards[0].0, "Samsung SSD 870 EVO  ·  1 TB");
    assert_eq!(words.cards[0].1, "Windows and about 340 GB of files");
    assert_eq!(words.cards[1].1, "Nothing is on it");
}

#[test]
fn a_password_is_hidden_as_it_is_typed_and_the_dots_are_honest() {
    // One dot per character, because counting that they typed twelve
    // characters is the only honest feedback a hidden field can give.
    let strings = strings();
    let mut asking = at(WhichScreen::ThePassword);
    for character in "secret".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let field = &words.cards[0].0;
    assert!(
        !field.contains("secret"),
        "the password is drawn in the clear: {field:?}"
    );
    assert_eq!(
        field.chars().filter(|one| *one == '•').count(),
        6,
        "six characters did not make six dots: {field:?}"
    );
}

#[test]
fn the_password_screen_names_the_network_being_joined() {
    // So that a person who chose the wrong one finds out here rather than
    // after the router refuses them.
    let strings = strings();
    let asking = at(WhichScreen::ThePassword);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    assert_eq!(words.title, "Fritz!Box 7590");
}

#[test]
fn the_disks_name_is_not_hidden_because_it_has_to_be_read_and_typed_back() {
    let strings = strings();
    let mut asking = at(WhichScreen::TheReview);
    for character in "Samsung".chars() {
        asking.and_then(
            WhatThePersonDid::Typed(character),
            WhatTheRingIsOn::default(),
        );
    }
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
    let field = &words.cards[0].1;
    assert!(
        field.contains("Samsung"),
        "the disk's name is hidden as it is typed: {field:?}"
    );
    assert!(
        !field.contains('•'),
        "the disk's name is drawn as dots: {field:?}"
    );
    // And the screen names the disk at the top, which is what a person reads
    // in order to type it.
    assert_eq!(words.title, "Samsung SSD 870 EVO  ·  1 TB");
}

#[test]
fn a_field_shows_a_caret_so_the_screen_does_not_look_stopped() {
    let strings = strings();
    for screen in [WhichScreen::ThePassword, WhichScreen::TheReview] {
        let asking = at(screen);
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        let field = if screen == WhichScreen::ThePassword {
            &words.cards[0].0
        } else {
            &words.cards[0].1
        };
        assert!(
            field.contains('▌'),
            "{screen:?} shows no caret, so an empty field looks like a stopped machine"
        );
    }
}

#[test]
fn how_many_cards_agrees_with_how_many_the_screen_shows() {
    // **The pair that must agree or the ring points at nothing.** The walk
    // asks `how_many_cards` to bound the ring and the screen draws
    // `words.cards`; if they ever disagree, Enter takes a card a person is not
    // looking at.
    let strings = strings();
    for screen in [
        WhichScreen::TheKeyboard,
        WhichScreen::EveryLayout,
        WhichScreen::TheNetwork,
        WhichScreen::ThePassword,
        WhichScreen::TheDisk,
        WhichScreen::TheReview,
    ] {
        let asking = at(screen);
        let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &networks(), &disks());
        assert_eq!(
            how_many_cards(&asking, &THE_EIGHT, &networks(), &disks()),
            words.cards.len(),
            "{screen:?} bounds the ring at a different number than it draws"
        );
    }
}

#[test]
fn a_scan_that_found_nothing_leaves_the_screen_with_no_cards_rather_than_a_wrong_one() {
    // Real: a card with no firmware finds no networks at all. The screen must
    // still say what it is for.
    let strings = strings();
    let asking = at(WhichScreen::TheNetwork);
    let words = what_this_screen_says(&strings, &asking, &THE_EIGHT, &[], &disks());
    assert!(words.cards.is_empty());
    assert!(
        !words.title.is_empty(),
        "an empty network screen says nothing"
    );
    assert_eq!(how_many_cards(&asking, &THE_EIGHT, &[], &disks()), 0);
}
