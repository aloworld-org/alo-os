//! Two screens on one desk: each drawn for its own room, wearing its own
//! background, with its own dock on its own edge, and warmed by its own night
//! light.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_appearance::{Appearance, Background, Token};
use alo_displays::Changes;
use alo_dock::Edge;
use alo_strings::Direction;

use super::*;
use crate::desktop_testing::noon_look;
use crate::screens_testing::{
    a_cold_evening, a_laptop, a_warm_evening, an_office_screen, the_screens,
};

/// A person with a navy desktop everywhere and a cream one on the monitor.
fn two_backgrounds() -> Appearance {
    let mut appearance = Appearance::shipped();
    appearance.set_background(Background::from(Token::Navy.colour()));
    appearance.set_background_on(
        an_office_screen().named_for_the_shell().clone(),
        Background::from(Token::Cream.colour()),
    );
    appearance
}

/// Nowhere a shipped image could be found: every background in these tests is
/// a colour, so nothing here reads a file.
fn no_wallpapers() -> &'static Path {
    Path::new("/nonexistent/alo/wallpapers")
}

/// The two screens drawn, with the dock on `edge` and night light as `tonight`.
fn the_desk(edge: Edge, tonight: &alo_displays::Tonight) -> Vec<ScreenPicture> {
    let appearance = two_backgrounds();
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let screens = the_screens(
        vec![a_laptop(), an_office_screen()],
        &Changes::untouched(),
        &appearance,
        &dock,
        tonight,
    );
    within(
        &screens,
        &dock,
        noon_look(&appearance, Direction::LeftToRight),
        Duration::ZERO,
        no_wallpapers(),
    )
    .unwrap()
}

/// The single flat colour a screen's background is, when it is a colour.
fn ground(picture: &ScreenPicture) -> [u8; 3] {
    assert!(picture.background.inked.is_empty());
    let solid = picture
        .background
        .solids
        .first()
        .expect("a colour background is one shape");
    assert_eq!(picture.background.solids.len(), 1);
    assert_eq!(solid.area.loc.x, 0);
    assert_eq!(solid.area.loc.y, 0);
    assert_eq!((solid.area.size.w, solid.area.size.h), picture.size);
    solid.colour
}

/// **Two screens are two pictures**, each laid out for its own room, each with
/// its own background behind its windows and its own dock along the edge
/// `alo-dock` names — on every edge a person can put the dock on.
///
/// Nothing is stretched across the desk: the monitor's dock is laid out for the
/// monitor and the laptop's for the laptop, which is why their bands are not
/// the same size.
#[test]
fn two_screens_are_two_pictures_each_with_its_own_background_and_dock() {
    let appearance = two_backgrounds();
    let navy = Token::Navy.colour();
    let cream = Token::Cream.colour();

    for edge in Edge::ALL {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let screens = the_screens(
            vec![a_laptop(), an_office_screen()],
            &Changes::untouched(),
            &appearance,
            &dock,
            &a_cold_evening(),
        );
        let drawn = the_desk(edge, &a_cold_evening());
        assert_eq!(drawn.len(), 2, "{edge:?}");

        for picture in &drawn {
            let place = screens
                .each()
                .find(|place| place.name() == &picture.name)
                .expect("every picture is a screen on this desk");
            assert_eq!(picture.size, place.room(), "{edge:?}");
            assert_eq!(picture.at, place.at());
            assert_eq!(picture.dock.size, picture.size, "{edge:?}");
            assert_eq!(picture.dock.layout.edge(), edge);
            assert!(!picture.dock.solids.is_empty());
            let band = picture.dock.band;
            match edge {
                Edge::Top | Edge::Bottom => assert_eq!(band.size.w, picture.size.0),
                Edge::Left | Edge::Right => assert_eq!(band.size.h, picture.size.1),
            }
            assert!(picture.dock.status_area.size.w > 0);
        }

        let laptop = drawn
            .iter()
            .find(|picture| picture.name == *a_laptop().named_for_the_shell())
            .unwrap();
        let office = drawn
            .iter()
            .find(|picture| picture.name == *an_office_screen().named_for_the_shell())
            .unwrap();
        assert_eq!(ground(laptop), [navy.red(), navy.green(), navy.blue()]);
        assert_eq!(ground(office), [cream.red(), cream.green(), cream.blue()]);
        assert_ne!(laptop.size, office.size, "{edge:?}");
        assert_ne!(
            laptop.dock.band.size, office.dock.band.size,
            "{edge:?}: one dock was stretched across two screens"
        );
    }
}

/// **Night light is applied per screen, to everything drawn on it.** With it
/// off, every colour is exactly the one `alo-appearance` and `alo-dock` chose;
/// with it on at 2700 K, the background and the dock on **both** screens have
/// had blue taken out of them — and neither has had light added, because
/// warming a screen never brightens it.
#[test]
fn night_light_warms_the_background_and_the_dock_on_every_screen() {
    let cold = the_desk(Edge::Bottom, &a_cold_evening());
    let warm = the_desk(Edge::Bottom, &a_warm_evening());
    assert_eq!(cold.len(), 2);
    assert_eq!(warm.len(), 2);

    for (before, after) in cold.iter().zip(warm.iter()) {
        assert_eq!(before.name, after.name);
        let [red_before, green_before, blue_before] = ground(before);
        let [red_after, green_after, blue_after] = ground(after);
        assert!(
            blue_after < blue_before,
            "{:?}: the background kept its blue",
            after.name
        );
        assert!(red_after <= red_before && green_after <= green_before);

        let dock_before: Vec<[u8; 3]> = before.dock.solids.iter().map(|s| s.colour).collect();
        let dock_after: Vec<[u8; 3]> = after.dock.solids.iter().map(|s| s.colour).collect();
        assert_eq!(dock_before.len(), dock_after.len());
        assert_ne!(
            dock_before, dock_after,
            "{:?}: the dock was not warmed with the screen",
            after.name
        );
        for (was, now) in dock_before.iter().zip(dock_after.iter()) {
            for (was, now) in was.iter().zip(now.iter()) {
                assert!(now <= was, "warming a screen never adds light");
            }
        }
    }
}

/// **With night light off the dock is drawn exactly as `alo-dock` and
/// `alo-appearance` decided it**, colour for colour, because the neutral
/// warming is the identity and no second palette is kept here.
#[test]
fn with_night_light_off_the_dock_is_drawn_exactly_as_it_was_decided() {
    let appearance = two_backgrounds();
    let look = noon_look(&appearance, Direction::LeftToRight);
    let dock = Dock::shipped();
    let screens = the_screens(
        vec![a_laptop(), an_office_screen()],
        &Changes::untouched(),
        &appearance,
        &dock,
        &a_cold_evening(),
    );

    for picture in within(&screens, &dock, look, Duration::ZERO, no_wallpapers()).unwrap() {
        let undimmed = crate::dock_raster::picture(&dock, look, picture.size).unwrap();
        assert_eq!(picture.dock, undimmed, "{:?}", picture.name);
    }
}
