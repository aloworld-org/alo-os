//! That a person can walk the whole way, that going back un-answers what they
//! left, and — the one that matters — that **nothing is erased until the
//! disk's own name is typed exactly.**

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{TheAsking, WhatThePersonDid, WhatTheRingIsOn, WhichScreen};

/// Nothing under the ring, which is what every screen without a list passes.
fn nothing() -> WhatTheRingIsOn<'static> {
    WhatTheRingIsOn::default()
}

/// A named thing under the ring that wants no password.
///
/// The machine's own name is the same string here, which is true of a layout
/// and of a network — and is what makes the disk, where it is **not** the
/// same, the case worth its own helper below.
fn named(what: &str) -> WhatTheRingIsOn<'_> {
    WhatTheRingIsOn {
        named: Some(what),
        as_the_machine_calls_it: Some(what),
        wants_a_password: false,
    }
}

/// A disk under the ring, which has two names: what a person reads and types
/// back, and what the install is given.
fn a_disk<'a>(called: &'a str, path: &'a str) -> WhatTheRingIsOn<'a> {
    WhatTheRingIsOn {
        named: Some(called),
        as_the_machine_calls_it: Some(path),
        wants_a_password: false,
    }
}

/// A network under the ring that does want a password.
fn locked(what: &str) -> WhatTheRingIsOn<'_> {
    WhatTheRingIsOn {
        named: Some(what),
        as_the_machine_calls_it: Some(what),
        wants_a_password: true,
    }
}

/// Type a whole string into the screen's field.
fn type_in(asking: &mut TheAsking, text: &str) {
    for character in text.chars() {
        asking.and_then(WhatThePersonDid::Typed(character), nothing());
    }
}

/// Walk as far as the review screen, on a locked network, and stop there.
fn as_far_as_the_review() -> TheAsking {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);

    asking.there_are_this_many_cards(3);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box 7590"));
    assert_eq!(asking.on(), WhichScreen::ThePassword);

    type_in(&mut asking, "a secret");
    asking.and_then(WhatThePersonDid::Chose, nothing());
    assert_eq!(asking.on(), WhichScreen::TheDisk);

    asking.there_are_this_many_cards(2);
    asking.and_then(WhatThePersonDid::Chose, named("Samsung SSD 870 EVO"));
    assert_eq!(asking.on(), WhichScreen::TheReview);
    asking
}

#[test]
fn a_person_can_walk_the_whole_way_and_what_they_chose_comes_out_at_the_end() {
    let mut asking = as_far_as_the_review();
    assert_eq!(
        asking.finished(),
        None,
        "it finished before the name was typed"
    );

    type_in(&mut asking, "Samsung SSD 870 EVO");
    assert!(asking.and_then(WhatThePersonDid::Chose, nothing()));

    let answers = asking.finished().expect("the walk is finished");
    assert_eq!(answers.layout, "be");
    assert_eq!(answers.network.as_deref(), Some("Fritz!Box 7590"));
    assert_eq!(answers.password, "a secret");
    assert_eq!(answers.disk, "Samsung SSD 870 EVO");
}

#[test]
fn nothing_is_erased_until_the_disks_own_name_is_typed_exactly() {
    // **The promise `words.rs` makes in as many words**, and the test that
    // holds it: *nothing is changed until you type the name of the disk you
    // choose.* Each of these is nearly right and none of them is the name.
    for nearly in [
        "",
        "Samsung",
        "samsung ssd 870 evo",
        " Samsung SSD 870 EVO",
        "Samsung SSD 870 EVO ",
        "Samsung SSD 870 EV",
        "Samsung SSD 870 EVOO",
        "Samsung  SSD 870 EVO",
    ] {
        let mut asking = as_far_as_the_review();
        type_in(&mut asking, nearly);
        assert!(
            !asking.and_then(WhatThePersonDid::Chose, nothing()),
            "{nearly:?} was accepted as the disk's name"
        );
        assert_eq!(
            asking.finished(),
            None,
            "{nearly:?} was accepted and the disk would have been erased"
        );
    }

    // And the control: the exact name is accepted, so the refusals above are
    // about the name and not about the check always refusing.
    let mut asking = as_far_as_the_review();
    type_in(&mut asking, "Samsung SSD 870 EVO");
    assert!(asking.and_then(WhatThePersonDid::Chose, nothing()));
    assert!(asking.finished().is_some());
}

#[test]
fn a_network_that_wants_no_password_skips_the_field() {
    // Rather than showing an empty field a person has to press Enter past.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(2);
    asking.and_then(WhatThePersonDid::Chose, named("A cafe, open"));
    assert_eq!(asking.on(), WhichScreen::TheDisk);
    assert_eq!(asking.network(), Some("A cafe, open"));
}

#[test]
fn an_empty_password_is_not_accepted_on_a_network_that_asked_for_one() {
    // The router would refuse it, and a person would read that as the
    // password being wrong rather than absent.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box 7590"));
    assert_eq!(asking.on(), WhichScreen::ThePassword);

    assert!(!asking.and_then(WhatThePersonDid::Chose, nothing()));
    assert_eq!(
        asking.on(),
        WhichScreen::ThePassword,
        "an empty password went through"
    );
}

#[test]
fn going_back_un_answers_the_screen_being_left() {
    // The whole point of going back. A person who returns to the networks and
    // picks a different one must not carry the first one's password forward.
    let mut asking = as_far_as_the_review();
    assert_eq!(asking.disk(), Some("Samsung SSD 870 EVO"));

    assert!(asking.and_then(WhatThePersonDid::WentBack, nothing()));
    assert_eq!(asking.on(), WhichScreen::TheDisk);
    assert_eq!(
        asking.disk(),
        None,
        "the disk survived going back from the review"
    );

    assert!(asking.and_then(WhatThePersonDid::WentBack, nothing()));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);
    assert_eq!(
        asking.network(),
        None,
        "the network survived going back from the disk"
    );
    // And the password it had went with it.
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, named("A cafe, open"));
    let mut done = asking.clone();
    done.and_then(WhatThePersonDid::Chose, named("Samsung SSD 870 EVO"));
    type_in(&mut done, "Samsung SSD 870 EVO");
    done.and_then(WhatThePersonDid::Chose, nothing());
    assert_eq!(
        done.finished().expect("finished").password,
        "",
        "the password of the network that was left behind came through"
    );
}

#[test]
fn going_back_from_the_password_returns_to_the_networks_with_nothing_typed() {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(2);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box 7590"));
    type_in(&mut asking, "half a password");

    assert!(asking.and_then(WhatThePersonDid::WentBack, nothing()));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);
    assert_eq!(
        asking.typed(),
        "",
        "what was half typed came back with them"
    );
    assert_eq!(asking.network(), None);
}

#[test]
fn escape_on_the_first_screen_leaves_a_person_somewhere_rather_than_nowhere() {
    let mut asking = TheAsking::begins();
    assert!(!asking.and_then(WhatThePersonDid::WentBack, nothing()));
    assert_eq!(asking.on(), WhichScreen::TheKeyboard);
}

#[test]
fn a_field_takes_typing_and_a_list_does_not() {
    // Typing on a screen of cards must not quietly accumulate somewhere
    // invisible, which would then appear on the next screen that has a field.
    let mut asking = TheAsking::begins();
    assert!(!asking.is_a_field());
    assert!(!asking.and_then(WhatThePersonDid::Typed('x'), nothing()));
    assert_eq!(asking.typed(), "");

    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box"));
    assert!(asking.is_a_field());
    assert!(asking.and_then(WhatThePersonDid::Typed('x'), nothing()));
    assert_eq!(asking.typed(), "x");
}

#[test]
fn rubbing_out_takes_one_character_and_stops_at_nothing() {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box"));

    type_in(&mut asking, "abc");
    assert!(asking.and_then(WhatThePersonDid::RubbedOut, nothing()));
    assert_eq!(asking.typed(), "ab");

    for _ in 0..5 {
        asking.and_then(WhatThePersonDid::RubbedOut, nothing());
    }
    assert_eq!(asking.typed(), "");
    // And rubbing out nothing changes nothing, so the screen is not redrawn.
    assert!(!asking.and_then(WhatThePersonDid::RubbedOut, nothing()));
}

#[test]
fn only_the_password_screen_hides_what_is_typed() {
    // The disk's name is typed to be compared with what is on the screen, so
    // hiding it would make the one thing a person has to check unreadable.
    let asking = as_far_as_the_review();
    assert!(asking.is_a_field());
    assert!(
        !asking.is_hidden(),
        "the disk's name is hidden as it is typed"
    );

    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box"));
    assert!(
        asking.is_hidden(),
        "a password is not hidden as it is typed"
    );
}

#[test]
fn an_arrow_on_a_field_does_not_move_a_ring_that_is_not_there() {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, locked("Fritz!Box"));
    assert!(!asking.and_then(WhatThePersonDid::WentDown, nothing()));
    assert_eq!(asking.card(), 0);
}

#[test]
fn choosing_with_nothing_under_the_ring_settles_nothing_on_any_list_screen() {
    // The empty-list case, which is real on the network screen: a card with no
    // firmware finds nothing at all.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(0);
    assert!(!asking.and_then(WhatThePersonDid::Chose, nothing()));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);
    assert_eq!(asking.network(), None);
}

#[test]
fn the_ring_cannot_leave_the_cards_at_either_end() {
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(3);

    assert!(!asking.and_then(WhatThePersonDid::WentUp, nothing()));
    assert_eq!(asking.card(), 0);
    for _ in 0..2 {
        asking.and_then(WhatThePersonDid::WentDown, nothing());
    }
    assert_eq!(asking.card(), 2);
    assert!(!asking.and_then(WhatThePersonDid::WentDown, nothing()));
    assert_eq!(asking.card(), 2);
}

#[test]
fn a_shorter_list_pulls_the_ring_back_onto_it() {
    // Real on the network screen: a second scan finds fewer networks than the
    // first, and the ring must not be left pointing past the end.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(6);
    for _ in 0..5 {
        asking.and_then(WhatThePersonDid::WentDown, nothing());
    }
    assert_eq!(asking.card(), 5);
    asking.there_are_this_many_cards(2);
    assert_eq!(asking.card(), 1);
}

#[test]
fn something_else_still_opens_the_list_and_comes_back() {
    let mut asking = TheAsking::begins();
    assert!(asking.and_then(WhatThePersonDid::WentDown, nothing()));
    assert!(asking.and_then(WhatThePersonDid::Chose, nothing()));
    assert_eq!(asking.on(), WhichScreen::EveryLayout);
    assert_eq!(asking.layout(), None, "opening the list settled a layout");

    asking.there_are_this_many_cards(8);
    for _ in 0..5 {
        asking.and_then(WhatThePersonDid::WentDown, nothing());
    }
    assert!(asking.and_then(WhatThePersonDid::Chose, named("nl")));
    assert_eq!(asking.layout(), Some("nl"));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);
}

#[test]
fn the_walk_never_finishes_without_every_answer() {
    // A `finished` carrying an empty disk or no layout would install onto
    // something nobody chose. Swept over every prefix of the walk.
    let mut asking = TheAsking::begins();
    let steps: Vec<(WhatThePersonDid, WhatTheRingIsOn<'_>)> = vec![
        (WhatThePersonDid::Chose, named("be")),
        (WhatThePersonDid::Chose, locked("Fritz!Box")),
    ];
    for (did, ring) in steps {
        assert_eq!(asking.finished(), None);
        asking.and_then(did, ring);
        asking.there_are_this_many_cards(2);
    }
    type_in(&mut asking, "p");
    assert_eq!(asking.finished(), None);
    asking.and_then(WhatThePersonDid::Chose, nothing());
    assert_eq!(asking.finished(), None);
    asking.and_then(WhatThePersonDid::Chose, named("A disk"));
    assert_eq!(asking.finished(), None);
    type_in(&mut asking, "A disk");
    asking.and_then(WhatThePersonDid::Chose, nothing());
    let answers = asking.finished().expect("now it is finished");
    assert!(!answers.layout.is_empty() && !answers.disk.is_empty());
}

#[test]
fn a_computer_with_no_network_to_choose_is_not_asked_about_one() {
    // **A question with no answers is not a question.** A cable, a card with
    // no firmware, or a scan that found nothing — and asking anyway would
    // leave a person on a screen where every key does nothing.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    assert_eq!(asking.on(), WhichScreen::TheNetwork);

    asking.there_are_this_many_cards(0);
    assert!(asking.there_is_no_network_to_choose());
    assert_eq!(asking.on(), WhichScreen::TheDisk);
    assert_eq!(asking.network(), None, "a network was chosen out of none");

    // And the walk still finishes, on a cable, with no network named.
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, named("A disk"));
    type_in(&mut asking, "A disk");
    asking.and_then(WhatThePersonDid::Chose, nothing());
    let answers = asking.finished().expect("the walk finishes on a cable");
    assert_eq!(answers.network, None);
    assert_eq!(answers.password, "");
}

#[test]
fn a_network_screen_with_networks_on_it_is_not_skipped() {
    // The control. Without it the clause above would pass on code that always
    // skipped the network question, and nobody would ever be asked.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(2);
    assert!(!asking.there_is_no_network_to_choose());
    assert_eq!(asking.on(), WhichScreen::TheNetwork);
}

#[test]
fn no_other_screen_is_ever_skipped_for_being_empty() {
    // Only the network question may have no answers. An empty disk list is a
    // different thing entirely — `NO_DISK_IS_BIG_ENOUGH` is its own screen —
    // and silently passing over it would install onto nothing.
    let mut asking = as_far_as_the_review();
    asking.there_are_this_many_cards(0);
    assert!(!asking.there_is_no_network_to_choose());
    assert_eq!(asking.on(), WhichScreen::TheReview);
}

#[test]
fn a_disk_keeps_both_its_names_and_the_right_one_is_typed_back() {
    // **A disk is the one thing with two names**, and which is which matters:
    // a person reads and types *Samsung SSD 870 EVO · 1 TB*, and the install
    // is given `/dev/disk/by-id/...`. Nobody can be asked to type the second,
    // and the first cannot be installed onto.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, named("A cafe, open"));
    asking.there_are_this_many_cards(1);
    asking.and_then(
        WhatThePersonDid::Chose,
        a_disk(
            "Samsung SSD 870 EVO  ·  1 TB",
            "/dev/disk/by-id/ata-Samsung_SSD_870_EVO",
        ),
    );
    assert_eq!(asking.on(), WhichScreen::TheReview);
    assert_eq!(asking.disk(), Some("Samsung SSD 870 EVO  ·  1 TB"));
    assert_eq!(
        asking.disk_as_the_machine_calls_it(),
        Some("/dev/disk/by-id/ata-Samsung_SSD_870_EVO")
    );

    // Typing the machine's name is **not** typing the disk's name, however
    // exactly it is typed.
    type_in(&mut asking, "/dev/disk/by-id/ata-Samsung_SSD_870_EVO");
    assert!(!asking.and_then(WhatThePersonDid::Chose, nothing()));
    assert_eq!(asking.finished(), None);

    // What a person reads is what they type.
    let mut asking = TheAsking::begins();
    asking.and_then(WhatThePersonDid::Chose, named("be"));
    asking.there_are_this_many_cards(1);
    asking.and_then(WhatThePersonDid::Chose, named("A cafe, open"));
    asking.there_are_this_many_cards(1);
    asking.and_then(
        WhatThePersonDid::Chose,
        a_disk(
            "Samsung SSD 870 EVO  ·  1 TB",
            "/dev/disk/by-id/ata-Samsung_SSD_870_EVO",
        ),
    );
    type_in(&mut asking, "Samsung SSD 870 EVO  ·  1 TB");
    assert!(asking.and_then(WhatThePersonDid::Chose, nothing()));

    let answers = asking.finished().expect("finished");
    assert_eq!(answers.disk_called, "Samsung SSD 870 EVO  ·  1 TB");
    assert_eq!(
        answers.disk, "/dev/disk/by-id/ata-Samsung_SSD_870_EVO",
        "the install would be addressed to the name on the screen"
    );
}

#[test]
fn a_disk_known_by_only_one_of_its_names_is_not_chosen_at_all() {
    // Both names or neither: a disk this walk could show and not install
    // onto, or install onto and not show, is a disk it cannot honestly offer.
    for ring in [
        WhatTheRingIsOn {
            named: Some("A disk"),
            as_the_machine_calls_it: None,
            wants_a_password: false,
        },
        WhatTheRingIsOn {
            named: None,
            as_the_machine_calls_it: Some("/dev/sda"),
            wants_a_password: false,
        },
    ] {
        let mut asking = TheAsking::begins();
        asking.and_then(WhatThePersonDid::Chose, named("be"));
        asking.there_are_this_many_cards(1);
        asking.and_then(WhatThePersonDid::Chose, named("A cafe, open"));
        asking.there_are_this_many_cards(1);
        assert!(
            !asking.and_then(WhatThePersonDid::Chose, ring),
            "a half-named disk was chosen: {ring:?}"
        );
        assert_eq!(asking.on(), WhichScreen::TheDisk);
    }
}
