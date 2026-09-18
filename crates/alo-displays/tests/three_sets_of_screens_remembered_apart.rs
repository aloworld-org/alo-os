//! Docking at the office restores the office; docking at home restores home.
//!
//! The one promise in `ROADMAP.md`'s *multi-monitor* line that a person would
//! notice being broken every single morning. Three sets of screens are plugged
//! in in turn, arranged differently, and then plugged in again — through the
//! file in the person's own folder, because an arrangement that only survives
//! until the machine is switched off has not been remembered at all.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_displays::{
    Arrangement, Attached, Changes, Identity, Note, Panel, Placed, Position, Reported, Scale,
    Socket, Support, keeping,
};

/// The laptop's own panel, which says nothing about itself.
fn a_reported_laptop() -> Reported {
    Reported::of(
        Socket::named("eDP-1").unwrap(),
        None,
        (1920, 1080),
        Some((294, 165)),
    )
    .unwrap()
}

/// The office screen: 27 inches at 3840 by 2160, with a serial number.
fn a_reported_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// The screen at home: 24 inches at 1920 by 1080, no serial number.
fn a_reported_home_screen() -> Reported {
    Reported::of(
        Socket::named("HDMI-1").unwrap(),
        Some(Panel::of("Acme", "P24", None).unwrap()),
        (1920, 1080),
        Some((531, 299)),
    )
    .unwrap()
}

/// Which screen the laptop's own panel is.
fn the_laptop() -> Identity {
    Identity::Socket(Socket::named("eDP-1").unwrap())
}

/// Which screen the office monitor is.
fn the_office_screen() -> Identity {
    Identity::Panel(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap())
}

/// Which screen the one at home is.
fn the_home_screen() -> Identity {
    Identity::Panel(Panel::of("Acme", "P24", None).unwrap())
}

/// An arrangement of these screens, the first one the main screen.
fn arranged(places: &[(Identity, i32, u16)]) -> Arrangement {
    Arrangement::of(
        places
            .iter()
            .enumerate()
            .map(|(at, (identity, across, per_cent))| {
                let placed = Placed::at(
                    Position::at(*across, 0),
                    Scale::per_cent(*per_cent).unwrap(),
                );
                (
                    identity.clone(),
                    if at == 0 {
                        placed.as_the_main_screen()
                    } else {
                        placed
                    },
                )
            })
            .collect(),
    )
    .unwrap()
}

/// A folder of this test's own.
fn a_folder(named: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-displays-{named}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// **Three sets of screens, arranged apart and restored apart** — and each one
/// through the person's own file, so the machine has genuinely been switched
/// off between the arranging and the restoring.
#[test]
fn three_sets_of_screens_are_remembered_apart() {
    let folder = a_folder("three-sets");
    let at = folder.join(keeping::THE_FILE);

    // Nothing has been arranged, so there is no file at all.
    let (mut remembered, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert!(remembered.is_untouched());
    assert!(!at.exists(), "an untouched machine has no file");

    // The laptop on its own, then the office, then home — each arranged the
    // way its owner wants it and kept.
    remembered.remember(arranged(&[(the_laptop(), 0, 175)]));
    remembered.remember(arranged(&[
        (the_laptop(), 0, 175),
        (the_office_screen(), -2560, 150),
    ]));
    remembered.remember(arranged(&[
        (the_laptop(), 0, 175),
        (the_home_screen(), 1920, 100),
    ]));
    keeping::keep(&at, &remembered).unwrap();

    // The machine is switched off and on again: everything below reads the
    // file rather than the value above.
    let (remembered, refused) = keeping::at_sign_in(&at);
    assert_eq!(refused, None);
    assert_eq!(remembered.how_many(), 3);

    // At the office.
    let office = Attached::now(
        vec![a_reported_laptop(), a_reported_office_screen()],
        &remembered,
        Support::Fractional,
    )
    .expect("two screens are an arrangement");
    assert!(office.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(office.main_screen(), &the_laptop());
    assert_eq!(
        office
            .on(&the_office_screen())
            .expect("the office screen")
            .placed()
            .position(),
        Position::at(-2560, 0),
        "on the left, where it was left"
    );
    assert_eq!(
        office
            .on(&the_office_screen())
            .expect("the office screen")
            .placed()
            .scale(),
        Scale::per_cent(150).unwrap()
    );

    // At home, where the second screen is on the right instead.
    let home = Attached::now(
        vec![a_reported_laptop(), a_reported_home_screen()],
        &remembered,
        Support::Fractional,
    )
    .expect("two screens are an arrangement");
    assert!(home.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(
        home.on(&the_home_screen())
            .expect("the home screen")
            .placed()
            .position(),
        Position::at(1920, 0),
        "on the right, where it was left"
    );
    assert!(
        home.on(&the_office_screen()).is_none(),
        "the office is not at home"
    );

    // On the train, with nothing plugged in.
    let alone = Attached::now(vec![a_reported_laptop()], &remembered, Support::Fractional)
        .expect("one screen is an arrangement");
    assert!(alone.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(alone.main_screen(), &the_laptop());
    assert_eq!(
        alone
            .on(&the_laptop())
            .expect("the laptop")
            .placed()
            .position(),
        Position::the_origin()
    );

    // And the three sets are three different keys, none of which answers for
    // another.
    assert_ne!(office.screens(), home.screens());
    assert_ne!(office.screens(), alone.screens());
    assert_ne!(home.screens(), alone.screens());

    let _ = std::fs::remove_dir_all(&folder);
}

/// **A fourth set nobody has arranged does not lose the sizes already chosen.**
/// Plugging a screen in that has never been in this combination makes a set
/// nobody has arranged — and the screens in it that *have* been sized keep
/// their size, while the new one is sized from its own glass.
#[test]
fn a_set_nobody_has_arranged_keeps_the_sizes_its_screens_already_had() {
    let mut remembered = Changes::untouched();
    remembered.remember(arranged(&[
        (the_laptop(), 0, 200),
        (the_office_screen(), -2560, 125),
    ]));

    let all_three = Attached::now(
        vec![
            a_reported_laptop(),
            a_reported_office_screen(),
            a_reported_home_screen(),
        ],
        &remembered,
        Support::Fractional,
    )
    .expect("three screens are an arrangement");

    assert!(!all_three.notes().contains(&Note::AsYouLeftThem));
    assert_eq!(
        all_three
            .on(&the_laptop())
            .expect("the laptop")
            .placed()
            .scale(),
        Scale::per_cent(200).unwrap(),
        "the size this screen was last given, not one worked out again"
    );
    assert_eq!(
        all_three
            .on(&the_office_screen())
            .expect("the office screen")
            .placed()
            .scale(),
        Scale::per_cent(125).unwrap()
    );
    assert_eq!(
        all_three
            .on(&the_home_screen())
            .expect("the home screen")
            .placed()
            .scale(),
        Scale::a_hundred(),
        "a screen nobody has sized is worked out from its own glass"
    );
    assert!(
        all_three
            .notes()
            .contains(&Note::NewHere(the_home_screen())),
        "and the person is told it is new here"
    );
}

/// **A screen that says nothing about itself is remembered by its socket, and
/// the person is told that is weaker.** The laptop's own panel is the screen
/// almost every machine has, and it is the one this applies to.
#[test]
fn a_screen_that_says_nothing_is_remembered_by_its_socket_and_the_person_is_told() {
    let attached = Attached::now(
        vec![a_reported_laptop()],
        &Changes::untouched(),
        Support::Fractional,
    )
    .expect("one screen is an arrangement");

    assert_eq!(attached.main_screen(), &the_laptop());
    assert_eq!(
        attached.main_screen().how_stable(),
        alo_displays::Stability::WhereItIsPluggedIn
    );
    assert!(
        attached
            .notes()
            .contains(&Note::RememberedByItsSocket(the_laptop()))
    );
}

/// **Two identical screens are told apart by where they are plugged in**, and
/// the person is told that too — because swapping their cables swaps them.
#[test]
fn two_screens_that_say_the_same_thing_are_told_apart_by_their_sockets() {
    let one = Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Acme", "P24", None).unwrap()),
        (1920, 1080),
        Some((531, 299)),
    )
    .unwrap();
    let two = Reported::of(
        Socket::named("DP-2").unwrap(),
        Some(Panel::of("Acme", "P24", None).unwrap()),
        (1920, 1080),
        Some((531, 299)),
    )
    .unwrap();

    let attached = Attached::now(vec![one, two], &Changes::untouched(), Support::Fractional)
        .expect("two screens are an arrangement");

    assert!(attached.notes().contains(&Note::ToldApartByTheirSockets));
    assert!(
        attached
            .on(&Identity::Socket(Socket::named("DP-1").unwrap()))
            .is_some()
    );
    assert!(
        attached
            .on(&Identity::Socket(Socket::named("DP-2").unwrap()))
            .is_some()
    );
    assert!(
        attached
            .on(&Identity::Panel(Panel::of("Acme", "P24", None).unwrap()))
            .is_none(),
        "neither of them is remembered by a description that names both"
    );
}
