//! Unplugging a screen moves what was on it, and plugging it back brings it
//! home.
//!
//! The moving is the shell's. What is held here is the decision — *onto which
//! screen*, and *do they come back* — including the case a single hotplug test
//! never reaches: the screen the windows were moved onto being unplugged in its
//! turn, which has to carry them on rather than lose track of where they came
//! from.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_displays::{
    Attached, Changes, Identity, NotArranged, NotAttached, Panel, Reported, Socket, Support,
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

/// The office screen.
fn a_reported_office_screen() -> Reported {
    Reported::of(
        Socket::named("DP-1").unwrap(),
        Some(Panel::of("Dell", "U2720Q", Some("CN-0ABC")).unwrap()),
        (3840, 2160),
        Some((596, 336)),
    )
    .unwrap()
}

/// The screen at home.
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

/// **What was on a screen that is unplugged goes to the main screen, and goes
/// back when it returns** — to the place the arrangement gives it.
#[test]
fn a_screen_unplugged_hands_its_windows_over_and_takes_them_back() {
    let remembered = Changes::untouched();
    let mut attached = Attached::now(
        vec![a_reported_laptop(), a_reported_office_screen()],
        &remembered,
        Support::Fractional,
    )
    .expect("two screens are an arrangement");
    assert_eq!(attached.main_screen(), &the_laptop());
    let where_it_was = attached
        .on(&the_office_screen())
        .expect("the office screen")
        .placed()
        .position();

    let moved = attached
        .unplugged(&Socket::named("DP-1").unwrap(), &remembered)
        .expect("one screen remains");
    assert_eq!(moved.from(), &the_office_screen());
    assert_eq!(moved.onto(), &the_laptop());
    assert_eq!(moved.carrying().count(), 0);
    assert!(attached.on(&the_office_screen()).is_none());
    assert_eq!(
        attached.windows_away().count(),
        1,
        "the machine knows where they went"
    );

    let back = attached
        .plugged_in(a_reported_office_screen(), &remembered)
        .expect("it fits");
    assert_eq!(back.back(), &the_office_screen());
    assert!(back.anything_goes_back());
    assert_eq!(back.were_on(), Some(&the_laptop()));
    assert_eq!(attached.windows_away().count(), 0);
    assert_eq!(
        attached
            .on(&the_office_screen())
            .expect("the office screen")
            .placed()
            .position(),
        where_it_was,
        "back where the arrangement had it"
    );
}

/// **The chain holds.** Unplug the screen the windows were moved onto, and they
/// move again with the ones already there — and each screen still brings its
/// own home.
#[test]
fn windows_carried_twice_still_find_their_way_home() {
    let remembered = Changes::untouched();
    let mut attached = Attached::now(
        vec![
            a_reported_laptop(),
            a_reported_office_screen(),
            a_reported_home_screen(),
        ],
        &remembered,
        Support::Fractional,
    )
    .expect("three screens are an arrangement");
    assert_eq!(attached.main_screen(), &the_laptop());

    // The office screen goes; its windows go to the laptop.
    let first = attached
        .unplugged(&Socket::named("DP-1").unwrap(), &remembered)
        .expect("two screens remain");
    assert_eq!(first.onto(), &the_laptop());

    // Then the laptop's own panel goes — the lid, a dock. What was on it, and
    // what the office screen had left there, both go to what remains.
    let second = attached
        .unplugged(&Socket::named("eDP-1").unwrap(), &remembered)
        .expect("one screen remains");
    assert_eq!(second.from(), &the_laptop());
    assert_eq!(second.onto(), &the_home_screen());
    assert_eq!(
        second.carrying().collect::<Vec<_>>(),
        vec![&the_office_screen()],
        "the office screen's windows are carried on rather than left behind"
    );

    // Both come back, and each takes its own.
    let laptop_back = attached
        .plugged_in(a_reported_laptop(), &remembered)
        .expect("it fits");
    assert_eq!(laptop_back.back(), &the_laptop());
    assert_eq!(laptop_back.were_on(), Some(&the_home_screen()));

    let office_back = attached
        .plugged_in(a_reported_office_screen(), &remembered)
        .expect("it fits");
    assert_eq!(office_back.back(), &the_office_screen());
    assert_eq!(office_back.were_on(), Some(&the_home_screen()));
    assert_eq!(attached.windows_away().count(), 0);
}

/// **The last screen cannot be unplugged**, because what is open on it would
/// have nowhere to go — and nothing is changed by the refusal.
#[test]
fn the_last_screen_is_refused_and_nothing_changes() {
    let remembered = Changes::untouched();
    let mut attached = Attached::now(vec![a_reported_laptop()], &remembered, Support::Fractional)
        .expect("one screen is an arrangement");
    let before = attached.clone();

    assert_eq!(
        attached.unplugged(&Socket::named("eDP-1").unwrap(), &remembered),
        Err(NotAttached::NothingRemains)
    );
    assert_eq!(attached, before, "the refusal changed nothing at all");
    assert!(attached.on(&the_laptop()).is_some());
}

/// **A socket with no screen in it, and a socket that already has one, are both
/// refused** — and neither changes anything.
#[test]
fn an_empty_socket_and_a_full_one_are_both_refused() {
    let remembered = Changes::untouched();
    let mut attached = Attached::now(
        vec![a_reported_laptop(), a_reported_office_screen()],
        &remembered,
        Support::Fractional,
    )
    .expect("two screens are an arrangement");
    let before = attached.clone();

    assert_eq!(
        attached.unplugged(&Socket::named("DP-9").unwrap(), &remembered),
        Err(NotAttached::NotPluggedIn)
    );
    assert_eq!(
        attached.plugged_in(a_reported_office_screen(), &remembered),
        Err(NotAttached::AlreadyPluggedIn)
    );
    assert_eq!(attached, before, "neither refusal changed anything");
}

/// **A screen plugged in for the first time carries nothing home**, and says
/// nothing about windows — there were none on it to move.
#[test]
fn a_screen_plugged_in_for_the_first_time_carries_nothing() {
    let remembered = Changes::untouched();
    let mut attached = Attached::now(vec![a_reported_laptop()], &remembered, Support::Fractional)
        .expect("one screen is an arrangement");

    let arriving = attached
        .plugged_in(a_reported_office_screen(), &remembered)
        .expect("it fits");
    assert_eq!(arriving.back(), &the_office_screen());
    assert!(!arriving.anything_goes_back());
    assert_eq!(arriving.were_on(), None);
    assert_eq!(arriving.word(), None);
}

/// A machine with nothing plugged into it is refused where it is built, so no
/// caller ever holds an arrangement of no screens.
#[test]
fn nothing_plugged_in_is_not_an_arrangement() {
    assert_eq!(
        Attached::now(Vec::new(), &Changes::untouched(), Support::Fractional),
        Err(NotArranged::NoScreens)
    );
}
