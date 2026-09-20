//! Several screens: laid out as `alo-displays` arranges them, restored as the
//! person left them, each wearing its own background on its own dock edge, and
//! each warmed by the one night light.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Appearance, Background, DisplayId, Token};
use alo_displays::{Arrangement, Note, Placed, Position, Scale};
use alo_dock::{Dock, Edge};

use super::*;
use crate::screens_testing::{
    a_cold_evening, a_desk, a_laptop, a_warm_evening, an_office_screen, the_screens,
};

/// The place of the screen known by `named`.
fn by_name<'a>(screens: &'a Screens, named: &DisplayId) -> &'a ScreenPlace {
    screens
        .each()
        .find(|place| place.name() == named)
        .expect("that screen is plugged in")
}

/// **Every screen is where `alo-displays` put it, at the size it named.** The
/// corner, the size drawn at and the room a surface lays itself out in are all
/// that crate's answers for that screen, and none of them is worked out here —
/// including the room, which is the screen's own pixels at the size it is
/// **drawn** at rather than the one it was asked for.
#[test]
fn every_screen_is_where_alo_displays_put_it_at_the_size_it_named() {
    let remembered = Changes::untouched();
    let appearance = Appearance::shipped();
    let dock = Dock::shipped();
    let reported = vec![a_laptop(), an_office_screen()];

    let attached = a_desk(reported.clone(), &remembered);
    let screens = the_screens(reported, &remembered, &appearance, &dock, &a_cold_evening());

    assert_eq!(screens.how_many(), 2);
    assert_eq!(screens.main_screen(), attached.main_screen());
    for on in attached.each() {
        let place = screens.on(on.identity()).expect("it is plugged in");
        assert_eq!(place.at(), on.placed().position());
        assert_eq!(place.scale(), on.drawn_at());
        assert_eq!(place.pixels(), on.reported().pixels());
        assert_eq!(place.is_main(), on.placed().is_the_main_screen());
        assert_eq!(
            place.room(),
            (
                i32::try_from(on.drawn_at().laid_out(on.reported().pixels().width())).unwrap(),
                i32::try_from(on.drawn_at().laid_out(on.reported().pixels().height())).unwrap(),
            )
        );
    }

    // Neither screen is at 100%: each is sized from its own glass, which is
    // `alo-displays`' third decision and not one made here.
    let office = by_name(&screens, an_office_screen().named_for_the_shell());
    assert_ne!(office.scale(), Scale::a_hundred());
    assert_ne!(
        office.room(),
        (
            i32::try_from(an_office_screen().pixels().width()).unwrap(),
            i32::try_from(an_office_screen().pixels().height()).unwrap()
        )
    );
}

/// **An arrangement is restored when that set of screens is plugged in
/// again.** A person who put the monitor to the left of the laptop and made it
/// the main screen finds it there the next morning — and is told so, in
/// `alo-displays`' own note.
#[test]
fn an_arrangement_is_restored_when_that_set_is_plugged_in_again() {
    let appearance = Appearance::shipped();
    let dock = Dock::shipped();
    let reported = vec![a_laptop(), an_office_screen()];
    let mut remembered = Changes::untouched();

    let first = the_screens(
        reported.clone(),
        &remembered,
        &appearance,
        &dock,
        &a_cold_evening(),
    );
    let laptop = by_name(&first, a_laptop().named_for_the_shell()).clone();
    let office = by_name(&first, an_office_screen().named_for_the_shell()).clone();
    assert!(!first.notes().contains(&Note::AsYouLeftThem));

    remembered.remember(
        Arrangement::of(vec![
            (
                office.identity().clone(),
                Placed::at(Position::at(-3840, 0), office.scale()).as_the_main_screen(),
            ),
            (
                laptop.identity().clone(),
                Placed::at(Position::the_origin(), laptop.scale()),
            ),
        ])
        .unwrap(),
    );

    let again = the_screens(reported, &remembered, &appearance, &dock, &a_cold_evening());
    let office_again = by_name(&again, an_office_screen().named_for_the_shell());
    assert_eq!(office_again.at(), Position::at(-3840, 0));
    assert!(office_again.is_main());
    assert_eq!(again.main_screen(), office.identity());
    assert!(again.notes().contains(&Note::AsYouLeftThem));
}

/// **What was on a screen that went belongs where `alo-displays` says**, and
/// goes back to it when it is plugged in again. The screen is gone from the
/// desk, what was on it is recorded as away, and nothing about where it went
/// was decided here.
#[test]
fn what_was_on_a_screen_that_went_belongs_where_alo_displays_says() {
    let remembered = Changes::untouched();
    let appearance = Appearance::shipped();
    let dock = Dock::shipped();
    let office = an_office_screen();
    let mut screens = the_screens(
        vec![a_laptop(), office.clone()],
        &remembered,
        &appearance,
        &dock,
        &a_cold_evening(),
    );
    let office_id = by_name(&screens, office.named_for_the_shell())
        .identity()
        .clone();
    let laptop_id = by_name(&screens, a_laptop().named_for_the_shell())
        .identity()
        .clone();

    let moved = screens
        .unplugged(
            office.socket(),
            &remembered,
            &appearance,
            &dock,
            &a_cold_evening(),
        )
        .unwrap();
    assert_eq!(moved.from(), &office_id);
    assert_eq!(moved.onto(), &laptop_id);
    assert_eq!(screens.how_many(), 1);
    assert!(screens.on(&office_id).is_none());
    assert_eq!(
        screens.windows_away().collect::<Vec<_>>(),
        vec![(&office_id, &laptop_id)]
    );

    let back = screens
        .plugged_in(office, &remembered, &appearance, &dock, &a_cold_evening())
        .unwrap();
    assert_eq!(back.back(), &office_id);
    assert!(back.anything_goes_back());
    assert_eq!(back.were_on(), Some(&laptop_id));
    assert_eq!(screens.how_many(), 2);
    assert_eq!(screens.windows_away().count(), 0);
}

/// **Each screen wears its own background, and its dock is on the edge
/// `alo-dock` names for it.** A background chosen for the monitor is on the
/// monitor and not on the laptop; a person who moves the dock moves it on every
/// screen, because that is the one edge `alo-dock` decides today.
#[test]
fn each_screen_wears_its_own_background_on_the_edge_alo_dock_names() {
    let remembered = Changes::untouched();
    let office = an_office_screen();
    let everywhere = Background::from(Token::Navy.colour());
    let only_there = Background::from(Token::Cream.colour());
    let mut appearance = Appearance::shipped();
    appearance.set_background(everywhere.clone());
    appearance.set_background_on(office.named_for_the_shell().clone(), only_there.clone());

    for edge in Edge::ALL {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let screens = the_screens(
            vec![a_laptop(), office.clone()],
            &remembered,
            &appearance,
            &dock,
            &a_cold_evening(),
        );
        assert_eq!(
            by_name(&screens, office.named_for_the_shell())
                .wearing()
                .background(),
            &only_there
        );
        assert_eq!(
            by_name(&screens, a_laptop().named_for_the_shell())
                .wearing()
                .background(),
            &everywhere
        );
        for place in screens.each() {
            assert_eq!(place.edge(), edge, "{edge:?} on {:?}", place.name());
        }
    }
}

/// **Night light reaches every screen, each beside its own background.** One
/// decision for the machine, applied per screen — and with it off, every screen
/// is drawn exactly as it was.
#[test]
fn night_light_reaches_every_screen_beside_its_own_background() {
    let remembered = Changes::untouched();
    let dock = Dock::shipped();
    let office = an_office_screen();
    let mut appearance = Appearance::shipped();
    appearance.set_background(Background::from(Token::Navy.colour()));
    appearance.set_background_on(
        office.named_for_the_shell().clone(),
        Background::from(Token::Cream.colour()),
    );
    let reported = vec![a_laptop(), office];

    let mut screens = the_screens(reported, &remembered, &appearance, &dock, &a_cold_evening());
    for place in screens.each() {
        assert!(place.wearing().warmth().changes_nothing());
        assert_eq!(
            place.warming().applied_to(Token::Terracotta.colour()),
            Token::Terracotta.colour()
        );
    }

    screens.wearing_again(&appearance, &dock, &a_warm_evening());
    let mut backgrounds = Vec::new();
    for place in screens.each() {
        assert_eq!(place.wearing().warmth().as_kelvin(), 2700);
        assert!(place.warming().blue() < place.warming().green());
        backgrounds.push(place.wearing().background().clone());
    }
    assert_eq!(backgrounds.len(), 2);
    assert_ne!(
        backgrounds.first(),
        backgrounds.last(),
        "the same warmth, and still each screen's own background"
    );
}
